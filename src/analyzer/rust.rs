use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use crate::{analyzer::*, ir::*};
use quote::ToTokens;
use syn::{
    File,
    visit::{self, Visit},
};

pub struct RustAnalyzer;

impl Analyzer for RustAnalyzer {
    fn analyze(
        &self,
        request: Analyze,
        options: &AnalyzerOptions,
    ) -> Result<Workspace, AnalysisError> {
        match request.target {
            AnalysisTarget::Workspace(path) => self.analyze_workspace(path, options),
            AnalysisTarget::File(path) => self.analyze_file(path, options),
        }
    }
}
impl RustAnalyzer {
    // pub struct ScopeQuery {
    //     pub root: SymbolId,

    //     // What direction do we walk?
    //     pub direction: Traversal,

    //     // What symbols count?
    //     pub include: SymbolFilter,
    // }

    // pub enum Traversal {
    //     Down,
    //     Up,
    //     Both,
    // }

    // pub enum SymbolFilter {
    //     All,
    //     Declarations,
    //     Code,
    //     Modules,
    // }
    // pub fn metrics(&self, query: ScopeQuery) -> Metrics {
    //     let symbols = self.project(query);
    //     Metrics::from_symbols(symbols)
    // }
    fn analyze_workspace(
        &self,
        path: PathBuf,
        options: &AnalyzerOptions,
    ) -> Result<Workspace, AnalysisError> {
        let mut workspace = Workspace::new();
        for entry in walkdir::WalkDir::new(&path) {
            let entry = entry.map_err(|e| AnalysisError::Parse(e.to_string()))?;
            let file = entry.path();
            if file.extension().and_then(|x| x.to_str()) != Some("rs") {
                continue;
            }
            let source =
                std::fs::read_to_string(file).map_err(|e| AnalysisError::Parse(e.to_string()))?;
            let ast = syn::parse_file(&source).map_err(|e| AnalysisError::Parse(e.to_string()))?;
            let file_id = workspace.add_symbol(
                Symbol::file(
                    file.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                ),
                Some(workspace.root),
            );
            workspace.files.push(file_id);
            let mut visitor = RustVisitor::new(options, &mut workspace, file_id);
            visitor.visit_file(&ast);
        }
        self.workspace_metrics(&workspace);
        self.package_metrics(&workspace);
        // self.module_metrics(&workspace);
        self.file_metrics(&workspace);
        Ok(workspace)
    }
    fn analyze_package(&self, path: PathBuf, options: &AnalyzerOptions) {}
    fn analyze_module(&self, path: PathBuf, options: &AnalyzerOptions) {}
    fn analyze_file(
        &self,
        path: PathBuf,
        options: &AnalyzerOptions,
    ) -> Result<Workspace, AnalysisError> {
        let source =
            std::fs::read_to_string(&path).map_err(|e| AnalysisError::Parse(e.to_string()))?;
        let ast = syn::parse_file(&source).map_err(|e| AnalysisError::Parse(e.to_string()))?;
        let mut workspace = Workspace::new();
        let file_id = workspace.add_symbol(
            Symbol::file(
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            ),
            Some(workspace.root),
        );
        workspace.files.push(file_id);
        let mut visitor = RustVisitor::new(options, &mut workspace, file_id);
        visitor.visit_file(&ast);
        Ok(workspace)
    }
}

impl RustAnalyzer {
    pub fn workspace_metrics(&self, workspace: &Workspace) -> WorkspaceMetrics {
        let metrics = workspace.metrics();
        let mut metrics = WorkspaceMetrics::new(workspace);
        for symbol in &workspace.symbols {
            match &symbol.kind {
                SymbolKind::Function(_) => metrics.functions += 1,
                SymbolKind::Type(_) => metrics.types += 1,
                SymbolKind::Import(_) => metrics.imports += 1,
                _ => {}
            }
        }
        metrics
    }
    pub fn package_metrics(&self, workspace: &Workspace) -> Vec<PackageMetrics> {
        workspace
            .packages
            .iter()
            .map(|package_id| {
                let package = &workspace.symbols[*package_id as usize];
                let mut metrics = PackageMetrics::new(package.name.clone());
                self.collect_package_metrics(workspace, *package_id, &mut metrics);
                metrics
            })
            .collect()
    }
    fn collect_package_metrics(
        &self,
        workspace: &Workspace,
        symbol_id: SymbolId,
        metrics: &mut PackageMetrics,
    ) {
        let symbol = &workspace.symbols[symbol_id as usize];
        match &symbol.kind {
            SymbolKind::Module(ModuleKind::Dependency) => {
                metrics.files += 1;
            }
            SymbolKind::Function(_) => {
                metrics.functions += 1;
                metrics.symbols += 1;
            }
            SymbolKind::Type(_) => {
                metrics.types += 1;
                metrics.symbols += 1;
            }
            _ => {
                metrics.symbols += 1;
            }
        }
        for child in &symbol.children {
            self.collect_package_metrics(workspace, *child, metrics);
        }
    }
    // pub fn module_metrics(&self, workspace: &Workspace) -> Vec<FileMetrics> {
    //     todo!("analyzer module_metrics")
    // }
    pub fn file_metrics(&self, workspace: &Workspace) -> Vec<FileMetrics> {
        workspace
            .files
            .iter()
            .map(|file_id| {
                let file = &workspace.symbols[*file_id as usize];
                let mut metrics = FileMetrics::new(PathBuf::from(&file.name));
                for child in &file.children {
                    let symbol = &workspace.symbols[*child as usize];
                    metrics.symbols += 1;
                    match &symbol.kind {
                        SymbolKind::Function(_) => metrics.functions += 1,
                        SymbolKind::Import(_) => metrics.imports += 1,
                        SymbolKind::Type(_) => metrics.types += 1,
                        _ => {}
                    }
                }
                metrics
            })
            .collect()
    }
}
pub struct ScopeQuery {
    pub root: SymbolId,

    // What direction do we walk?
    pub direction: Traversal,

    // What symbols count?
    pub include: SymbolFilter,
}

pub enum Traversal {
    Down,
    Up,
    Both,
}

pub enum SymbolFilter {
    All,
    Declarations,
    Code,
    Modules,
}

pub struct RustVisitor<'a> {
    options: &'a AnalyzerOptions,
    workspace: &'a mut Workspace,
    scope_stack: Vec<SymbolId>,
    current_impl: Option<SymbolId>,
}
impl<'a> RustVisitor<'a> {
    pub fn new(
        options: &'a AnalyzerOptions,
        workspace: &'a mut Workspace,
        root_scope: SymbolId,
    ) -> Self {
        Self {
            options,
            workspace,
            scope_stack: vec![root_scope],
            current_impl: None,
        }
    }

    fn current_scope(&self) -> SymbolId {
        *self
            .scope_stack
            .last()
            .expect("visitor has no active scope")
    }
    fn add_symbol(&mut self, symbol: Symbol) -> SymbolId {
        let parent = self.current_scope();
        self.workspace.add_symbol(symbol, Some(parent))
    }
    fn push_scope(&mut self, id: SymbolId) {
        self.scope_stack.push(id);
    }

    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }
}

impl<'ast> Visit<'ast> for RustVisitor<'_> {
    fn visit_file(&mut self, node: &'ast syn::File) {
        println!("VISITING FILE ITEMS={}", node.items.len());
        for item in &node.items {
            println!("ITEM: {:?}", std::mem::discriminant(item));
        }
        visit::visit_file(self, node);
    }
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        let name = node.to_token_stream().to_string();

        self.add_symbol(Symbol {
            id: 0,
            name,
            kind: SymbolKind::Import(ModuleKind::Dependency),
            visibility: Visibility::Private,
            params: None,
            return_type: None,
            children: Vec::new(),
        });

        visit::visit_item_use(self, node);
    }
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.add_symbol(Symbol {
            id: 0,
            name: node.ident.to_string(),
            kind: SymbolKind::Type(TypeKind::Struct),
            visibility: Visibility::Private,
            params: None,
            return_type: None,
            children: Vec::new(),
        });

        visit::visit_item_struct(self, node);
    }
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        println!("FOUND FN {}", node.sig.ident);
        self.add_symbol(Symbol {
            id: 0,
            name: node.sig.ident.to_string(),
            kind: SymbolKind::Function(FunctionKind::Free),
            visibility: match &node.vis {
                syn::Visibility::Public(_) => Visibility::Public,
                _ => Visibility::Private,
            },
            params: None,
            return_type: Some(node.sig.output.to_token_stream().to_string()),
            children: Vec::new(),
        });

        visit::visit_item_fn(self, node);
    }
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let name = node.self_ty.to_token_stream().to_string();

        println!("FOUND IMPL {}", name);

        let impl_id = self.add_symbol(Symbol {
            id: 0,
            name: format!("impl {}", name),
            kind: SymbolKind::Implementation {
                target_type: name.clone(),
                trait_name: node
                    .trait_
                    .as_ref()
                    .map(|(path, _)| path.to_token_stream().to_string()),
            },
            visibility: Visibility::Private,
            params: None,
            return_type: None,
            children: Vec::new(),
        });

        self.current_impl = Some(impl_id);

        self.scope_stack.push(impl_id);

        visit::visit_item_impl(self, node);

        self.scope_stack.pop();

        self.current_impl = None;
    }
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        println!("FOUND METHOD {}", node.sig.ident);

        self.add_symbol(Symbol {
            id: 0,
            name: node.sig.ident.to_string(),
            kind: SymbolKind::Function(FunctionKind::Method),
            visibility: match &node.vis {
                syn::Visibility::Public(_) => Visibility::Public,
                _ => Visibility::Private,
            },
            params: None,
            return_type: Some(node.sig.output.to_token_stream().to_string()),
            children: Vec::new(),
        });

        visit::visit_impl_item_fn(self, node);
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        println!("FOUND ENUM {}", item.ident);
        self.add_symbol(Symbol {
            id: 0,
            name: item.ident.to_string(),
            kind: SymbolKind::Type(TypeKind::Enum),
            visibility: visibility(&item.vis),
            params: None,
            return_type: None,
            children: Vec::new(),
        });
        visit::visit_item_enum(self, item);
    }
    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        println!("FOUND TRAIT {}", item.ident);

        self.add_symbol(Symbol {
            id: 0,
            name: item.ident.to_string(),
            kind: SymbolKind::Type(TypeKind::Trait),
            visibility: match &item.vis {
                syn::Visibility::Public(_) => Visibility::Public,
                _ => Visibility::Private,
            },
            params: None,
            return_type: None,
            children: Vec::new(),
        });

        visit::visit_item_trait(self, item);
    }
}

fn visibility(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility::Public,
        _ => Visibility::Private,
    }
}

fn create_method_symbol(item: &syn::ImplItemFn) -> (Symbol, Option<Vec<(String, String)>>) {
    let params = Some(
        item.sig
            .inputs
            .iter()
            .map(|arg| match arg {
                syn::FnArg::Typed(pat) => (
                    pat.pat.to_token_stream().to_string(),
                    pat.ty.to_token_stream().to_string(),
                ),
                syn::FnArg::Receiver(rec) => {
                    ("self".to_string(), rec.to_token_stream().to_string())
                }
            })
            .collect(),
    );

    let method_symbol = Symbol {
        id: 0,
        name: item.sig.ident.to_string(),
        kind: SymbolKind::Function(FunctionKind::Method),
        visibility: visibility(&item.vis),
        params: params.clone(),
        return_type: Some(item.sig.output.to_token_stream().to_string()),
        children: Vec::new(),
    };

    (method_symbol, params)
}
pub struct Workspace {
    pub root: SymbolId,
    pub symbols: Vec<Symbol>,
    pub files: Vec<SymbolId>,
    pub packages: Vec<SymbolId>,
    pub modules: Vec<SymbolId>,
    next_sym_id: SymbolId,
}
impl Workspace {
    pub fn new() -> Self {
        let mut workspace = Self {
            root: 0,
            symbols: Vec::new(),
            packages: Vec::new(),
            modules: Vec::new(),
            files: Vec::new(),
            next_sym_id: 0,
        };
        let root = workspace.add_symbol(Symbol::workspace("workspace"), None);
        workspace.root = root;
        workspace
    }
    pub fn add_symbol(&mut self, mut symbol: Symbol, parent: Option<SymbolId>) -> SymbolId {
        let id = self.next_sym_id;
        self.next_sym_id += 1;
        symbol.id = id;
        if let Some(parent_id) = parent {
            self.symbols[parent_id as usize].children.push(id);
        }
        self.symbols.push(symbol);
        id
    }
    pub fn get(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id as usize]
    }
    pub fn get_mut(&mut self, id: SymbolId) -> &mut Symbol {
        &mut self.symbols[id as usize]
    }
}
impl Workspace {
    pub fn metrics(&self) -> AnalysisMetrics {
        AnalysisMetrics {
            workspace: self.workspace_metrics(),
            packages: self
                .packages
                .iter()
                .map(|id| self.package_metrics(*id))
                .collect(),
            modules: self
                .files
                .iter()
                .map(|id| self.module_metrics(*id))
                .collect(),
            files: self.files.iter().map(|id| self.file_metrics(*id)).collect(),
        }
    }
    pub fn workspace_metrics(&self) -> WorkspaceMetrics {
        let mut metrics = WorkspaceMetrics::default();
        metrics.packages = self.packages.len();
        metrics.files = self.files.len();
        self.collect_metrics(self.root, &mut metrics);
        metrics
    }

    fn collect_metrics(&self, id: SymbolId, metrics: &mut WorkspaceMetrics) {
        let symbol = &self.symbols[id as usize];
        match &symbol.kind {
            SymbolKind::Function(_) => metrics.functions += 1,
            SymbolKind::Type(_) => metrics.types += 1,
            SymbolKind::Import(_) => metrics.imports += 1,
            // Don't count containers as symbols
            SymbolKind::Package(_) => {}
            SymbolKind::Module(_) => {}
            SymbolKind::File(_) => {}
            SymbolKind::Workspace(_) => {}
            _ => {}
        }

        for child in &symbol.children {
            self.collect_metrics(*child, metrics);
        }
    }
    // fn collect_metrics(&self, id: SymbolId, metrics: &mut WorkspaceMetrics) {
    //     let symbol = &self.symbols[id as usize];
    //     match &symbol.kind {
    //         SymbolKind::Function(_) => metrics.functions += 1,
    //         SymbolKind::Type(_) => metrics.types += 1,
    //         SymbolKind::Import(_) => metrics.imports += 1,
    //         // Don't count containers as symbols
    //         SymbolKind::Package(_) => {}
    //         SymbolKind::Module(_) => {}
    //         SymbolKind::File(_) => {}
    //         SymbolKind::Workspace(_) => {}
    //         _ => {}
    //     }
    //     for child in &symbol.children {
    //         self.collect_metrics(*child, metrics);
    //     }
    // }
    pub fn package_metrics(&self, id: SymbolId) -> PackageMetrics {
        let package = &self.symbols[id as usize];
        let mut metrics = PackageMetrics::new(package.name.clone());
        self.collect_package_metrics(id, &mut metrics);
        metrics
    }
    fn collect_package_metrics(&self, id: SymbolId, metrics: &mut PackageMetrics) {
        let symbol = &self.symbols[id as usize];
        metrics.symbols += 1;
        match &symbol.kind {
            SymbolKind::File(_) => {
                metrics.files += 1;
            }
            SymbolKind::Module(_) => {
                metrics.modules += 1;
            }
            SymbolKind::Package(_) => {
                metrics.packages += 1;
            }
            SymbolKind::Function(_) => {
                metrics.functions += 1;
            }
            SymbolKind::Type(_) => {
                metrics.types += 1;
            }
            SymbolKind::Import(_) => {
                metrics.imports += 1;
            }
            SymbolKind::Implementation { .. } => {
                metrics.implementations += 1;
            }
            _ => {}
        }

        for child in &symbol.children {
            self.collect_package_metrics(*child, metrics);
        }
    }

    pub fn module_metrics(&self, id: SymbolId) -> ModuleMetrics {
        let module = &self.symbols[id as usize];
        let mut metrics = ModuleMetrics::new(module.name.clone());
        self.collect_module_metrics(id, &mut metrics);
        metrics
    }

    fn collect_module_metrics(&self, id: SymbolId, metrics: &mut ModuleMetrics) {
        let symbol = &self.symbols[id as usize];
        metrics.symbols += 1;
        match &symbol.kind {
            SymbolKind::Module(ModuleKind::Internal) => {
                metrics.files += 1;
            }
            SymbolKind::Function(_) => {
                metrics.functions += 1;
            }
            SymbolKind::Type(_) => {
                metrics.types += 1;
            }
            _ => {}
        }
        for child in &symbol.children {
            self.collect_module_metrics(*child, metrics);
        }
    }
    pub fn file_metrics(&self, id: SymbolId) -> FileMetrics {
        let file = &self.symbols[id as usize];
        let path = PathBuf::from(&file.name);
        let mut metrics = FileMetrics::new(path);
        self.collect_file_metrics(id, &mut metrics);
        metrics
    }

    fn collect_file_metrics(&self, id: SymbolId, metrics: &mut FileMetrics) {
        let symbol = &self.symbols[id as usize];

        metrics.symbols += 1;

        match &symbol.kind {
            SymbolKind::Function(_) => {
                metrics.functions += 1;
            }
            SymbolKind::Import(_) => {
                metrics.imports += 1;
            }
            SymbolKind::Type(_) => {
                metrics.types += 1;
            }
            _ => {}
        }

        for child in &symbol.children {
            self.collect_file_metrics(*child, metrics);
        }
    }
}
#[derive(Clone, Debug)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub ast: syn::File,
}
#[derive(Clone, Debug, Default)]
pub struct Scope {
    pub id: SymbolId,
    pub parent: Option<SymbolId>,
    pub children: Vec<SymbolId>,
    pub symbols: HashMap<String, SymbolId>,
}
pub enum ScopeKind {
    Workspace,
    Package,
    Module,
    File,
    Type,
    Impl,
    Function,
}
#[derive(Clone, Debug, Default)]
pub struct FileScope {
    pub path: PathBuf,
    // Every declaration in this file.
    pub symbols: HashMap<String, SymbolId>,

    // Stack of lexical scopes.
    pub scopes: Vec<Scope>,
}
