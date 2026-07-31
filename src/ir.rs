use crate::_scope::{self, ScopeId};

pub type SymId = u32;
// #[derive(Debug, Clone)]
// pub struct Symbol {
//     pub id: SymId,
//     pub name: String,
//     pub kind: SymbolKind,
//     pub location: Option<SymLocation>,
//     pub visibility: Visibility,
// pub params: Option<Vec<(String, String)>>,
//     pub return_type: Option<String>,
//     pub children: Vec<SymId>,
// }
#[derive(Clone, Debug)]
pub struct Sym {
    pub id: SymId,
    pub name: String,
    pub kind: SymbolKind,
    pub owner: SymId,
    pub location: Option<SymLocation>,
    pub visibility: Visibility,
    pub children: Vec<SymId>,
    pub scope: ScopeId,
}
impl Sym {
    pub fn new(
        name: impl Into<String>,
        kind: SymbolKind,
        owner: SymId,
        // scope,
        visibility: Visibility,
        location: Option<SymLocation>,
    ) -> Self {
        Self {
            id: 0,
            name: name.into(),
            kind,
            scope: 0,
            owner,
            location,
            visibility,
            // params: None,
            // return_type: None,
            children: Vec::new(),
        }
    }
}
impl Sym {
    // 1. Top down init workspace analysis
    pub fn workspace(owner: SymId, name: impl Into<String>) -> Self {
        Self {
            id: 0,
            name: name.into(),
            owner,
            scope: 0,
            location: Some(SymLocation::default()),
            kind: SymbolKind::Root(RootKind::Workspace),
            visibility: Visibility::Public,
            children: Vec::new(),
        }
    }
    // 1. Top down workspace init pkg
    pub fn package(parent: SymId, name: impl Into<String>) -> Self {
        Self {
            id: 0,
            name: name.into(),
            owner: 0,
            scope: 0,
            location: Some(SymLocation::default()),
            kind: SymbolKind::Package(PackageKind::Crate),
            visibility: Visibility::Public,
            children: Vec::new(),
        }
    }
    // 1. Top down pkgs init module
    pub fn module(parent: SymId, name: impl Into<String>) -> Self {
        Self {
            id: 0,
            name: name.into(),
            owner: 0,
            scope: 0,
            location: Some(SymLocation::default()),
            kind: SymbolKind::Module(ModuleKind::Dependency),
            visibility: Visibility::Public,
            children: Vec::new(),
        }
    }
    // 1. Top down workspace/pkg/module init files
    pub fn file(parent: SymId, name: impl Into<String>) -> Self {
        Self {
            id: 0,
            name: name.into(),
            owner: 0,
            scope: 0,
            location: Some(SymLocation::default()),
            kind: SymbolKind::Module(ModuleKind::Dependency),
            visibility: Visibility::Public,
            children: Vec::new(),
        }
    }
    pub fn function(
        owner: SymId,
        name: impl Into<String>,
        kind: FunctionKind,
        visibility: Visibility,
    ) -> Self {
        // 1. Top down pkgs init files
        Self {
            id: 0,
            name: name.into(),
            owner,
            scope: 0,
            location: Some(SymLocation::default()),
            kind: SymbolKind::Function(kind),
            visibility,
            children: Vec::new(),
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct SymLocation {
    pub file: SymId,
    pub start: usize,
    pub end: usize,
}
pub enum SymbolOrigin {
    Internal,   // mine, same workspace/package
    Dependency, // external package manager dependency
    Intrinsic,  // language/runtime/std provided
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Root(RootKind),
    Workspace(WorkspaceKind),
    Package(PackageKind),
    Module(ModuleKind),
    File(FileKind),
    Type(TypeKind),
    Function(FunctionKind),
    Variable(VariableKind),
    Import(ModuleKind),
    Implementation {
        target_type: String,
        trait_name: Option<String>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RootKind {
    Workspace,
    Crate,
    File,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileKind {
    Workspace,
    Crate,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    JavaScript,
    TypeScript,
    JSX,
    TSX,
    Python,
    Go,
    Java,
    CSharp,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Module(ModuleKind),
    Struct,
    Enum,
    Class,
    Trait,
    Interface,
    TypeAlias,
    Impl,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImplKind {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorkspaceKind {
    Intrinsic,
    Dependency,
    Internal,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleKind {
    Intrinsic,
    Dependency,
    Internal,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageKind {
    Crate,
    Workspace,
    Dependency,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FunctionKind {
    Free,
    Method,
    Associated,
    Lambda,
    TraitMethod,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub enum VariableKind {
    Let,
    Const,
    Var,
    Field,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Signature {
    pub params: Vec<(String, String)>,
    pub return_type: String,
}
