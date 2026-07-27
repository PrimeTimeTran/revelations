pub type SymbolId = u32;
#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub visibility: Visibility,
    pub params: Option<Vec<(String, String)>>,
    pub return_type: Option<String>,
    pub children: Vec<SymbolId>,
}
impl Symbol {
    pub fn new(name: impl Into<String>, kind: SymbolKind, visibility: Visibility) -> Self {
        Self {
            id: 0,
            name: name.into(),
            kind,
            visibility,
            params: None,
            return_type: None,
            children: Vec::new(),
        }
    }
}
impl Symbol {
    // 1. Top down init workspace analysis
    pub fn workspace(name: impl Into<String>) -> Self {
        Self {
            id: 0,
            name: name.into(),
            kind: SymbolKind::Root(RootKind::Workspace),
            visibility: Visibility::Public,
            params: None,
            return_type: None,
            children: Vec::new(),
        }
    }
    // 1. Top down workspace init pkg
    pub fn package(name: impl Into<String>) -> Self {
        Self {
            id: 0,
            name: name.into(),
            kind: SymbolKind::Package(PackageKind::Crate),
            visibility: Visibility::Public,
            params: None,
            return_type: None,
            children: Vec::new(),
        }
    }
    // 1. Top down pkgs init module
    pub fn module(name: impl Into<String>) -> Self {
        Self {
            id: 0,
            name: name.into(),
            kind: SymbolKind::Module(ModuleKind::Dependency),
            visibility: Visibility::Public,
            params: None,
            return_type: None,
            children: Vec::new(),
        }
    }
    // 1. Top down workspace/pkg/module init files
    pub fn file(name: impl Into<String>) -> Self {
        Self {
            id: 0,
            name: name.into(),
            kind: SymbolKind::Module(ModuleKind::Dependency),
            visibility: Visibility::Public,
            params: None,
            return_type: None,
            children: Vec::new(),
        }
    }
    pub fn function(name: impl Into<String>, kind: FunctionKind, visibility: Visibility) -> Self {
        // 1. Top down pkgs init files
        Self {
            id: 0,
            name: name.into(),
            kind: SymbolKind::Function(kind),
            visibility,
            params: None,
            return_type: None,
            children: Vec::new(),
        }
    }
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
