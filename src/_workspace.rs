use std::collections::HashMap;

use crate::{_scope::*, ir::*};

///-
/// - What is the difference in lifetime of a workspace and a pkg?
/// - A pkg has a different lifetime than a workspace how?
/// - They're both versioned & shared via public registries.
/// - The main characteristic I can think of is that they where a structure in which one wraps the other.
/// - And in so doing makes it easier for the local pkgs to talk to each other more easily.
/// - Although this takes some ceremony as well
/// - We must explicit add the local workspace crate to the workspace pkg management file, Cargo.toml
/// for any cargo command to work. If we init a crate inside of a crate, and do not add it to the ./Cargo.toml
/// running cargo check will throw an error.
pub struct SemanticWorkspace {
    pub symbols: HashMap<SymId, Symbol>,
    pub scopes: HashMap<ScopeId, Scope>,
    pub symbols_by_name: HashMap<String, Vec<SymId>>,
    pub symbols_by_scope: HashMap<ScopeId, Vec<SymId>>,
    pub packages_by_name: HashMap<String, SymId>,
}
