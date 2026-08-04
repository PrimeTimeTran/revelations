// ## 1. Symbol/package/module discovery + completion
// **Question:**
// > "User typed `use foo::` — what should we suggest?"
// This should not be solved by walking the AST tree every time.
// Pipeline:
// ```text
// Files
//   |
//   v
// Parser (syn)
//   |
//   v
// Declaration Indexer
//   |
//   v
// Semantic Graph
//   |
//   v
// Completion Indexes
// ```
// The graph stores truth:
// ```text
// Package
//   |
//   Module
//     |
//     Symbol
// ```
// Then build lookup tables:
// ```rust
// packages_by_name
// modules_by_path
// symbols_by_scope
// exports_by_module
// ```
// The completion engine asks:
// ```text
// "What symbols are available from this context?"
// ```
// not:
// ```text
// "Walk everything and find symbols."
// ```
// The stack is for building the graph. Maps are for querying it.
// ---
// ## 2. Metrics / telemetry
// **Question:**
// > "How big is this workspace?"
// This should be a separate analysis pass over the graph.
// Example:
// ```text
// Semantic Graph
//       |
//       v
// Metrics Analyzer
//       |
//       v
// WorkspaceStats
// ```
// Produces:
// ```rust
// WorkspaceStats {
//     files: 400,
//     modules: 120,
//     symbols: 50000,
//     functions: 8000,
//     structs: 1200,
// }
// ```
// Do not make `Symbol` store metrics.
// Metrics are observations, not identity.
// ---
// ## 3. Graph registry
// **Question:**
// > "How do I navigate everything?"
// This is your core database.
// The registry owns:
// ```rust
// HashMap<SymId, Symbol>
// HashMap<ScopeId, Scope>
// HashMap<FileId, File>
// ```
// Everything else references IDs.
// The important thing is stable identity:
// ```text
// File
//  |
// Module
//  |
// Struct
//  |
// Function
//  |
// Reference
// ```
// can all point to each other without copying data.
// This enables:
// * rename
// * navigation
// * refactoring
// * dependency visualization
// * indexing
// ---
// ## 4. Jump to caller
// **Question:**
// > "Who calls this function?"
// This cannot come from declarations.
// You need a reference analysis pass.
// Pipeline:
// ```text
// AST Expressions
//         |
//         v
// Path Resolver
//         |
//         v
// Reference Graph
// ```
// Example:
// ```rust
// foo();
// ```
// becomes:
// ```text
// CallSite(location)
//         |
//         references
//         |
//         v
// Function(foo)
// ```
// Store:
// ```rust
// references_to: HashMap<SymId, Vec<Location>>
// ```
// Then:
// ```text
// Function(foo)
//      |
//      v
// all callers
// ```
// is instant.
// ---
// ## 5. IR interpreter
// **Question:**
// > "Can we execute/analyze code?"
// This is a different consumer of the semantic graph.
// Pipeline:
// ```text
// AST
//  |
//  v
// Lowering
//  |
//  v
// IR
//  |
//  v
// Interpreter
// ```
// The semantic graph answers:
// ```text
// "What does this identifier mean?"
// ```
// The IR answers:
// ```text
// "What does this program do?"
// ```
// Do not mix runtime state into symbols.
// A symbol might point to:
// ```text
// Function add
// ```
// but the interpreter owns:
// ```text
// Stack frame
// Registers
// Heap
// Values
// ```
// ---
// ## 6. Runtime debugger
// **Question:**
// > "What is happening while executing?"
// This sits above the interpreter.
// Pipeline:
// ```text
// IR
//  |
//  v
// Runtime
//  |
//  v
// Debugger
// ```
// The debugger needs mappings:
// ```text
// IR instruction
//         |
//         v
// Source location
// ```
// which means your earlier `location` field becomes extremely valuable.
// Example:
// ```text
// Instruction #42
//       |
//       source:
//       main.rs:15
// ```
// Now breakpoints work.
// ---
// ## The common foundation
// If I were designing this, the layers would look like:
// ```
//                  Source Files
//                       |
//                       v
//               Syntax AST (syn)
//                       |
//                       v
//              Semantic Workspace
//                       |
//         +-------------+-------------+
//         |             |             |
//         v             v             v
//    Resolver      Metrics       Lowering
//         |                         |
//         v                         v
//    LSP Features                 IR
//                                   |
//                                   v
//                              Runtime/Debug
// ```
// The mistake to avoid is making the semantic workspace answer *every possible question directly*.
// Instead:
// * Workspace = source of truth
// * Indexes = fast queries
// * Analysis passes = derived knowledge
// Your current `Symbol + Scope + Container` model is actually the right place to start because it can feed all of these without locking you into one feature.
use crate::{_scope::*, ir::*};
use std::collections::HashMap;

pub struct SemanticWorkspace {
	pub symbols: HashMap<SymId, Sym>,
	pub scopes: HashMap<ScopeId, Scope>,
	// indexes
	pub symbols_by_name: HashMap<String, Vec<SymId>>,
	pub symbols_by_scope: HashMap<ScopeId, Vec<SymId>>,
	pub packages_by_name: HashMap<String, SymId>,
}
