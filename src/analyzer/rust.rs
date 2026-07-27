use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use crate::{analyzer::*, ir::*};
use quote::ToTokens;
use regex_syntax::ast::Ast;
use syn::{
    File,
    spanned::Spanned,
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
    // 1. Find every Cargo package.
    // 2. For each package, call analyze_package().
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
            let (source, ast) = self.build_source(path.clone());
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
            let mut visitor = RustVisitor::new(options, &mut workspace, file_id, &source);
            visitor.visit_file(&ast.unwrap());
        }
        self.workspace_metrics(&workspace);
        self.package_metrics(&workspace);
        self.file_metrics(&workspace);
        Ok(workspace)
    }
    // 1. Read Cargo.toml.
    // 2. Discover src/lib.rs, src/main.rs, tests/, examples/, etc.
    // 3. Build the package's module graph.
    // 4. For each file, call analyze_file().
    fn analyze_package(&self, path: PathBuf, options: &AnalyzerOptions) {}
    fn analyze_module(&self, path: PathBuf, options: &AnalyzerOptions) {}
    // 1. Read file.
    // 2. Parse with syn.
    // 3. Visit AST.
    // 4. Populate symbols.
    fn analyze_file(
        &self,
        path: PathBuf,
        options: &AnalyzerOptions,
    ) -> Result<Workspace, AnalysisError> {
        let (source, ast) = self.build_source(path.clone());
        self.build_workspace(options, path.clone());
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
        let mut visitor: RustVisitor<'_> =
            RustVisitor::new(options, &mut workspace, file_id, &source);
        visitor.visit_file(&ast.unwrap());
        Ok(workspace)
    }
}

impl RustAnalyzer {
    // Todo:
    // - Walk FS for identifying nested vs parent workspace capabilities
    // - Walk FS for multi framework entries
    fn build_workspace(
        &self,
        options: &AnalyzerOptions,
        path: PathBuf,
    ) -> Result<Workspace, AnalysisError> {
        let (source, ast) = self.build_source(path.clone());
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
        let mut visitor = RustVisitor::new(options, &mut workspace, file_id, &source);
        visitor.visit_file(&ast.unwrap());
        Ok(workspace)
    }
    fn build_source(&self, path: PathBuf) -> (String, Result<File, AnalysisError>) {
        let source = std::fs::read_to_string(&path)
            .map_err(|e| AnalysisError::Parse(e.to_string()))
            .unwrap();
        let ast = syn::parse_file(&source).map_err(|e| AnalysisError::Parse(e.to_string()));
        (source, ast)
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
pub struct RustVisitor<'a> {
    options: &'a AnalyzerOptions,
    workspace: &'a mut Workspace,
    scope_stack: Vec<SymbolId>,
    source: &'a str,
    file: SymbolId,
    current_impl: Option<SymbolId>,
    // current_workspace: Option<SymbolId>,
    // current_package: Option<SymbolId>,
    // current_module: Option<SymbolId>,
    // current_file: Option<SymbolId>,
    // current_trait: Option<SymbolId>,
}
impl<'a> RustVisitor<'a> {
    pub fn new(
        options: &'a AnalyzerOptions,
        workspace: &'a mut Workspace,
        file: SymbolId,
        source: &'a str,
    ) -> Self {
        Self {
            options,
            workspace,
            file,
            source,
            scope_stack: vec![file],
            current_impl: None,
        }
    }
    fn location(&self, span: proc_macro2::Span) -> SymbolLocation {
        SymbolLocation {
            file: self.file,
            start: span.start().line,
            end: span.end().line,
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
        visit::visit_file(self, node);
    }
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        let name = node.to_token_stream().to_string();
        self.add_symbol(Symbol {
            id: 0,
            name,
            location: Some(self.location(node.span())),
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
            location: Some(self.location(node.span())),
            kind: SymbolKind::Type(TypeKind::Struct),
            visibility: Visibility::Private,
            params: None,
            return_type: None,
            children: Vec::new(),
        });

        visit::visit_item_struct(self, node);
    }
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        // println!("FOUND FN {}", node.sig.ident);
        self.add_symbol(Symbol {
            id: 0,
            name: node.sig.ident.to_string(),
            location: Some(self.location(node.span())),
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
        // println!("FOUND IMPL {}", name);
        let impl_id = self.add_symbol(Symbol {
            id: 0,
            name: format!("impl {}", name),
            location: Some(self.location(node.span())),
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
        // println!("FOUND METHOD {}", node.sig.ident);
        self.add_symbol(Symbol {
            id: 0,
            location: Some(self.location(node.span())),
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
    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        // println!("FOUND ENUM {}", item.ident);
        self.add_symbol(Symbol {
            id: 0,
            location: Some(self.location(node.span())),
            name: node.ident.to_string(),
            kind: SymbolKind::Type(TypeKind::Enum),
            visibility: visibility(&node.vis),
            params: None,
            return_type: None,
            children: Vec::new(),
        });
        visit::visit_item_enum(self, node);
    }
    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        println!("FOUND TRAIT {}", node.ident);

        self.add_symbol(Symbol {
            id: 0,
            name: node.ident.to_string(),
            location: Some(self.location(node.span())),
            kind: SymbolKind::Type(TypeKind::Trait),
            visibility: match &node.vis {
                syn::Visibility::Public(_) => Visibility::Public,
                _ => Visibility::Private,
            },
            params: None,
            return_type: None,
            children: Vec::new(),
        });

        visit::visit_item_trait(self, node);
    }
}

fn visibility(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility::Public,
        _ => Visibility::Private,
    }
}

#[derive(Clone, Debug)]
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
    pub fn get(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id as usize]
    }
    pub fn get_mut(&mut self, id: SymbolId) -> &mut Symbol {
        &mut self.symbols[id as usize]
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
// pub fn metrics(&self, query: ScopeQuery) -> Metrics {
//     let symbols = self.project(query);
//     Metrics::from_symbols(symbols)
// }
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

pub struct WorkspaceDiscovery {
    pub root: PathBuf,
    pub estate_dir: Option<PathBuf>,
    pub packages: Vec<PackageDiscovery>,
}

pub struct PackageDiscovery {
    pub root: PathBuf,
    pub manifest: Option<PathBuf>,
    pub source_files: Vec<PathBuf>,
}

impl RustAnalyzer {
    fn discover_workspace(&self, path: &Path) -> Result<WorkspaceDiscovery, AnalysisError> {
        todo!()
    }
    fn find_workspace_root(&self, path: &Path) -> Option<PathBuf> {
        todo!()
    }
    fn find_estate_dir(&self, root: &Path) -> Option<PathBuf> {
        todo!()
    }
    fn discover_packages(&self, root: &Path) -> Result<Vec<PackageDiscovery>, AnalysisError> {
        todo!()
    }
    fn discover_sources(&self, package: &Path) -> Result<Vec<PathBuf>, AnalysisError> {
        todo!()
    }
    // pub fn build_workspace(
    //     &self,
    //     options: &AnalyzerOptions,
    //     path: PathBuf,
    // ) -> Result<Workspace, AnalysisError> {
    //     let discovery = self.discover_workspace(&path)?;
    //     let mut workspace = Workspace::new();
    //     for package in &discovery.packages {
    //         self.build_package(options, workspace.clone(), package);
    //     }
    //     Ok(workspace)
    // }
    // pub fn build_package(
    //     &self,
    //     options: &AnalyzerOptions,
    //     workspace: Workspace,
    //     package: &PackageDiscovery,
    // ) -> Result<String, String> {
    //     Ok("".to_string())
    // }
}

// Building up the Estate
// pub struct EstateDiscovery {
//     pub active: PathBuf,
//     pub parents: Vec<PathBuf>,
//     pub children: Vec<PathBuf>,
// }
// impl RustAnalyzer {
//     pub fn build_workspace(
//         &self,
//         options: &AnalyzerOptions,
//         path: PathBuf,
//     ) -> Result<Workspace, AnalysisError> {
//         let discovery = self.discover_workspace(&path)?;
//         let mut workspace = Workspace::new();
//         for package in &discovery.packages {
//             self.build_package(&mut workspace, package, options)?;
//         }
//         Ok(workspace)
//     }
//     fn build_package(
//         &self,
//         workspace: &mut Workspace,
//         package: &PackageDiscovery,
//         options: &AnalyzerOptions,
//     ) -> Result<(), AnalysisError> {
//         let package_id = workspace.add_symbol(
//             Symbol::package(package.root.file_name().unwrap().to_string_lossy()),
//             Some(workspace.root),
//         );
//         workspace.packages.push(package_id);
//         for file in &package.source_files {
//             self.build_file(workspace, package_id, file, options)?;
//         }
//         Ok(())
//     }
//     fn build_file(
//         &self,
//         workspace: &mut Workspace,
//         parent: SymbolId,
//         path: &Path,
//         options: &AnalyzerOptions,
//     ) -> Result<(), AnalysisError> {
//         todo!()
//     }
//     fn find_estate_root(&self, start: &Path) -> Option<PathBuf> {
//         todo!()
//     }
//     fn discover_estates(&self, root: &Path) -> Vec<PathBuf> {
//         todo!()
//     }
// }
