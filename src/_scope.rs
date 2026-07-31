use crate::{_semantic::*, analyzer::*, ir::*};
use std::{collections::HashMap, path::PathBuf};
use syn::{Ident, Result, Token, visit::Visit};

// pub struct Scope {
//     pub id: ScopeId,
//     pub owner: SymId,
//     pub name: String,
//     pub kind: ScopeKind,
//     pub children: Vec<SymId>,
// }

pub type ScopeId = u32;
#[derive(Clone, Debug, Default)]
pub struct Scope {
    pub id: ScopeId,
    // The symbol that created this scope
    pub owner: SymId,
    // Names directly declared here
    pub symbols: HashMap<String, SymId>,
    pub parent: Option<ScopeId>,
}

// impl Scope {
//     pub fn new(name: String, owner: SymId, kind: ScopeKind) -> Self {
//         Self {
//             id: 0,
//             owner,
//             symbols: vec![],
//         }
//     }
// }
pub struct SemanticWorkspace {
    pub symbols: HashMap<SymId, Symbol>,
    pub scopes: HashMap<ScopeId, Scope>,
    // indexes
    pub symbols_by_name: HashMap<String, Vec<SymId>>,
    pub symbols_by_scope: HashMap<ScopeId, Vec<SymId>>,
    pub packages_by_name: HashMap<String, SymId>,
}

#[derive(Clone, Debug)]
pub enum ScopeKind {
    Workspace,
    Package,
    Module,
    File,
    Type,
    Impl,
    Function,
    Block,
}
impl Default for ScopeKind {
    fn default() -> Self {
        ScopeKind::File
    }
}
// #[derive(Clone, Debug, Default)]
// pub struct ScopedWorkspace {
//     pub path: PathBuf,
//     pub pkgs: HashMap<String, SymId>,
//     pub mods: HashMap<String, SymId>,
//     pub symbols: HashMap<String, SymId>,
//     pub scopes: Vec<Scope>,
// }
// #[derive(Clone, Debug, Default)]
// pub struct ScopedPkg {
//     pub path: PathBuf,
//     // Top-level declarations in this file.
//     pub symbols: HashMap<String, SymId>,
//     // All lexical scopes inside the file.
//     pub scopes: Vec<Scope>,
// }
// #[derive(Clone, Debug, Default)]
// pub struct ScopedMod {
//     pub path: PathBuf,
//     // Top-level declarations in this file.
//     pub symbols: HashMap<String, SymId>,
//     // All lexical scopes inside the file.
//     pub scopes: Vec<Scope>,
// }

// #[derive(Clone, Debug, Default)]
// pub struct ScopedFile {
//     pub path: PathBuf,
//     // Top-level declarations in this file.
//     pub symbols: HashMap<String, SymId>,
//     // All lexical scopes inside the file.
//     pub scopes: Vec<Scope>,
// }
pub struct WorkspaceGraph {}

pub struct ScopeVisitor<'a> {
    pub workspace: &'a WorkspaceGraph,
    // Current lexical location.
    pub scope_stack: Vec<ScopeId>,

    // Current file being analyzed.
    pub file: SymId,

    // Current module path.
    pub module: SymId,

    // Current package/crate.
    pub package: SymId,
}

// impl<'ast, 'a> Visit<'ast> for ScopeVisitor<'a> {
//     fn visit_file(&mut self, node: &'ast syn::File) {
//         // TOP LEVEL FILE SCOPE
//         self.push_scope(ScopeKind::File);
//         syn::visit::visit_file(self, node);
//         self.pop_scope();
//     }

//     fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
//         // FUNCTION SCOPE
//         self.push_scope(ScopeKind::Function(node.sig.ident.to_string()));

//         syn::visit::visit_item_fn(self, node);

//         self.pop_scope();
//     }

//     fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
//         // STRUCT TYPE SCOPE
//         self.push_scope(ScopeKind::Struct(node.ident.to_string()));

//         syn::visit::visit_item_struct(self, node);

//         self.pop_scope();
//     }

//     fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
//         // IMPL BLOCK SCOPE
//         self.push_scope(ScopeKind::Impl);

//         syn::visit::visit_item_impl(self, node);

//         self.pop_scope();
//     }

//     fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
//         // TRAIT DEFINITION SCOPE
//         self.push_scope(ScopeKind::Trait(node.ident.to_string()));

//         syn::visit::visit_item_trait(self, node);

//         self.pop_scope();
//     }

//     fn visit_block(&mut self, node: &'ast syn::Block) {
//         // `{ }` BLOCK SCOPE
//         self.push_scope(ScopeKind::Block);

//         syn::visit::visit_block(self, node);

//         self.pop_scope();
//     }
// }

// impl Visit for ScopeVisitor {
//     fn visit_file(&mut self, node: syn::File) {
//         // TOP LEVEL FILE SCOPE
//         self.push_scope(ScopeKind::File);
//         syn::visit::visit_file(self, node);
//         self.pop_scope();
//     }

//     fn visit_item_fn(&mut self, node: syn::ItemFn) {
//         // FUNCTION SCOPE
//         self.push_scope(ScopeKind::Function(node.sig.ident.to_string()));

//         syn::visit::visit_item_fn(self, node);

//         self.pop_scope();
//     }

//     fn visit_item_struct(&mut self, node: syn::ItemStruct) {
//         // STRUCT TYPE SCOPE
//         self.push_scope(ScopeKind::Struct(node.ident.to_string()));

//         syn::visit::visit_item_struct(self, node);

//         self.pop_scope();
//     }

//     fn visit_item_impl(&mut self, node: syn::ItemImpl) {
//         // IMPL BLOCK SCOPE
//         self.push_scope(ScopeKind::Impl);

//         syn::visit::visit_item_impl(self, node);

//         self.pop_scope();
//     }

//     fn visit_item_trait(&mut self, node: syn::ItemTrait) {
//         // TRAIT DEFINITION SCOPE
//         self.push_scope(ScopeKind::Trait(node.ident.to_string()));

//         syn::visit::visit_item_trait(self, node);

//         self.pop_scope();
//     }

//     fn visit_block(&mut self, node: syn::Block) {
//         // `{ }` BLOCK SCOPE
//         self.push_scope(ScopeKind::Block);

//         syn::visit::visit_block(self, node);

//         self.pop_scope();
//     }
// }
