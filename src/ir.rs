use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymId(pub u32);
impl std::fmt::Display for SymId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}
impl Default for SymId {
	fn default() -> Self {
		Self(u32::MAX)
	}
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Sym {
	pub id: SymId,
	pub name: String,
	pub kind: SymbolKind,
	pub owner: Option<SymId>,
	pub location: Option<SymLocation>,
	pub visibility: Visibility,
	pub children: Vec<SymId>,
	pub scope: ScopeId,
	pub relationships: Vec<SymRelationship>,
	pub references: Vec<TypeReference>,
}
impl Sym {
	pub fn new(
		id: SymId,
		name: impl Into<String>,
		kind: SymbolKind,
		owner: Option<SymId>,
		scope: ScopeId,
		visibility: Visibility,
		location: Option<SymLocation>,
	) -> Self {
		Self {
			id,
			name: name.into(),
			kind,
			owner,
			location,
			visibility,
			children: Vec::new(),
			scope,
			relationships: Vec::new(),
			references: Vec::new(),
		}
	}

	/// Initialize the workspace root symbol.
	pub fn workspace(id: SymId, name: impl Into<String>, scope: ScopeId) -> Self {
		Self {
			id,
			name: name.into(),
			owner: None,
			scope,
			location: Some(SymLocation::default()),
			kind: SymbolKind::Root(RootKind::Workspace),
			visibility: Visibility::Public,
			children: Vec::new(),
			relationships: Vec::new(),
			references: Vec::new(),
		}
	}

	/// Initialize a package belonging to `owner`.
	pub fn package(id: SymId, owner: Option<SymId>, name: impl Into<String>, scope: ScopeId) -> Self {
		Self {
			id,
			name: name.into(),
			owner,
			scope,
			location: Some(SymLocation::default()),
			kind: SymbolKind::Package(PackageKind::Crate),
			visibility: Visibility::Public,
			children: Vec::new(),
			relationships: Vec::new(),
			references: Vec::new(),
		}
	}

	/// Initialize a module belonging to `owner`.
	pub fn module(id: SymId, owner: Option<SymId>, name: impl Into<String>, scope: ScopeId) -> Self {
		Self {
			id,
			name: name.into(),
			owner,
			scope,
			location: Some(SymLocation::default()),
			kind: SymbolKind::Module(ModuleKind::Dependency),
			visibility: Visibility::Public,
			children: Vec::new(),
			relationships: Vec::new(),
			references: Vec::new(),
		}
	}

	/// Initialize a file belonging to `owner`.
	pub fn file(id: SymId, owner: Option<SymId>, name: impl Into<String>, scope: ScopeId) -> Self {
		Self {
			id,
			name: name.into(),
			owner,
			scope,
			location: Some(SymLocation::default()),
			kind: SymbolKind::Module(ModuleKind::Dependency),
			visibility: Visibility::Public,
			children: Vec::new(),
			relationships: Vec::new(),
			references: Vec::new(),
		}
	}
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct SymRelationship {
	pub target: SymId,
	pub kind: RelationshipKind,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct TypeReference {
	pub name: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum RelationshipKind {
	Field,
	Reference,
	Contains,
	Uses,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScopeId(pub u32);
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scope {
	pub id: ScopeId,

	/// The symbol that created this scope.
	pub owner: Option<SymId>,

	/// Names directly declared here.
	pub symbols: HashMap<String, SymId>,

	pub parent: Option<ScopeId>,
}
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum SymbolKind {
	Field,
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
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum RootKind {
	Workspace,
	Crate,
	File,
}
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum FileKind {
	Workspace,
	Crate,
	File,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Visibility {
	Public,
	Private,
	Protected,
	Internal,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ImplKind {}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum WorkspaceKind {
	Intrinsic,
	Dependency,
	Internal,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ModuleKind {
	Intrinsic,
	Dependency,
	Internal,
}
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum PackageKind {
	Crate,
	Workspace,
	Dependency,
}
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum FunctionKind {
	Free,
	Method,
	Associated,
	Lambda,
	TraitMethod,
}
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum VariableKind {
	Let,
	Const,
	Var,
	Field,
}
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Signature {
	pub params: Vec<(String, String)>,
	pub return_type: String,
}
