use crate::{_config::AnalyzeConfig, analyzer::*, ir::*};
use proc_macro2::Span;
use quote::ToTokens;
use regex_syntax::ast::Ast;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	fs::read_to_string,
	path::{Path, PathBuf},
};
use syn::{
	File, Ident,
	spanned::Spanned,
	visit::{self, Visit},
	visit_mut::{self, VisitMut},
};

///--------------------------------------------------------------------------------
///      Pipelines:
///      - Estate(namespace definition): Are we in a workspace and how many packages do we have?
///      - Semantic(validation): Given fs, modules, pkgs, workspaces, is my syntax correct? My imports?
///      - Metrics(telemetry): Given a project composed of 1 or more files above, below, and sibling, what are the numbers?
///--------------------------------------------------------------------------------
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
				Sym::file(
					0,
					file
						.file_name()
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
			Sym::file(
				0,
				path
					.file_name()
					.unwrap_or_default()
					.to_string_lossy()
					.to_string(),
			),
			Some(workspace.root),
		);
		workspace.files.push(file_id);
		let mut visitor: RustVisitor<'_> = RustVisitor::new(options, &mut workspace, file_id, &source);
		visitor.visit_file(&ast.unwrap());
		Ok(workspace)
	}
	// fn analyze_file(
	//     &self,
	//     path: PathBuf,
	//     options: &AnalyzerOptions,
	//     subject: Option<&AnalyzeSubject>,
	// ) -> Result<Workspace, AnalysisError> {
	//     let (source, ast) = self.build_source(path.clone());
	//     let syntax_tree = ast.map_err(|e| AnalysisError::Parse(e.to_string()))?;

	//     let mut workspace = Workspace::new();
	//     let file_id = workspace.add_symbol(
	//         Symbol::file(
	//             0,
	//             path.file_name()
	//                 .unwrap_or_default()
	//                 .to_string_lossy()
	//                 .to_string(),
	//         ),
	//         Some(workspace.root),
	//     );
	//     workspace.files.push(file_id);

	//     // 1. Run your standard AST visitor for workspace symbols/metrics
	//     let mut visitor = RustVisitor::new(options, &mut workspace, file_id, &source);
	//     visitor.visit_file(&syntax_tree);

	//     // 2. Conditionally run your OwnershipVisitor if a target subject was clicked/provided
	//     if let Some(sub) = subject {
	//         let mut ownership_visitor = OwnershipVisitor {
	//             target_ident: &sub.identifier,
	//             target_offset: sub.offset,
	//             related_spans: Vec::new(),
	//         };

	//         // Walk the AST specifically for ownership/reference tracing
	//         use syn::visit::Visit;
	//         ownership_visitor.visit_file(&syntax_tree);

	//         // TODO: Map ownership_visitor.related_spans to line numbers
	//         // and attach them to your workspace or return results!
	//     }

	//     Ok(workspace)
	// }
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
			Sym::file(
				0,
				path
					.file_name()
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
		symbol_id: SymId,
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

#[derive(Clone, Debug)]
pub struct Workspace {
	pub root: SymId,
	pub symbols: Vec<Sym>,
	pub files: Vec<SymId>,
	pub packages: Vec<SymId>,
	pub modules: Vec<SymId>,
	pub config: AnalyzeConfig,
	next_sym_id: SymId,
}
impl Workspace {
	pub fn new() -> Self {
		let config = AnalyzeConfig::default();
		let mut workspace = Self {
			config,
			root: 0,
			symbols: Vec::new(),
			packages: Vec::new(),
			modules: Vec::new(),
			files: Vec::new(),
			next_sym_id: 0,
		};
		let root = workspace.add_symbol(Sym::workspace(1, "workspace"), None);
		workspace.root = root;
		workspace
	}
	pub fn get(&self, id: SymId) -> &Sym {
		&self.symbols[id as usize]
	}
	pub fn get_mut(&mut self, id: SymId) -> &mut Sym {
		&mut self.symbols[id as usize]
	}
	pub fn add_symbol(&mut self, mut symbol: Sym, parent: Option<SymId>) -> SymId {
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
	fn collect_metrics(&self, id: SymId, metrics: &mut WorkspaceMetrics) {
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
	pub fn package_metrics(&self, id: SymId) -> PackageMetrics {
		let package = &self.symbols[id as usize];
		let mut metrics = PackageMetrics::new(package.name.clone());
		self.collect_package_metrics(id, &mut metrics);
		metrics
	}
	fn collect_package_metrics(&self, id: SymId, metrics: &mut PackageMetrics) {
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

	pub fn module_metrics(&self, id: SymId) -> ModuleMetrics {
		let module = &self.symbols[id as usize];
		let mut metrics = ModuleMetrics::new(module.name.clone());
		self.collect_module_metrics(id, &mut metrics);
		metrics
	}
	fn collect_module_metrics(&self, id: SymId, metrics: &mut ModuleMetrics) {
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

	pub fn file_metrics(&self, id: SymId) -> FileMetrics {
		let file = &self.symbols[id as usize];
		let path = PathBuf::from(&file.name);
		let mut metrics = FileMetrics::new(path);
		self.collect_file_metrics(id, &mut metrics);
		metrics
	}
	fn collect_file_metrics(&self, id: SymId, metrics: &mut FileMetrics) {
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
	pub root: SymId,

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

// pub fn resolve_subject_at_offset(
//     path: &Path,
//     offset: usize,
// ) -> Result<ParsedSubject, Box<dyn std::error::Error>> {
//     let source_code = std::fs::read_to_string(path)?;

//     // 1. Safety check bounds
//     if offset >= source_code.len() {
//         return Err("Offset out of bounds".into());
//     }

//     // 2. Extract character / token / word boundaries around the offset
//     // (Or use your compiler's lexer/parser cursor lookup here)
//     let slice = &source_code[offset..];
//     let identifier = slice
//         .split(|c: char| !c.is_alphanumeric() && c != '_')
//         .next()
//         .unwrap_or("")
//         .to_string();

//     // 3. Determine syntactic context (Statement vs Expression vs Identifier)
//     // Here you can query your parser's AST nodes that enclose this offset.
//     let kind = classify_node_at_offset(&source_code, offset);

//     Ok(ParsedSubject { identifier, kind })
// }

#[derive(Debug)]
pub enum NodeKind {
	Statement,
	Expression,
	Identifier,
	Unknown,
}

// fn classify_node_at_offset(source: &str, offset: usize) -> NodeKind {
//     // Inspect surrounding syntax (e.g., trailing semicolons for statements,
//     // operator context for expressions, etc.)
//     NodeKind::Expression // stub for your AST check
// }

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
//         parent: SymId,
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
// };

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeContext {
	Expression,
	Statement,
	Identifier,
	Unknown,
}
pub fn resolve_node_at_position(
	syntax_tree: &syn::File,
	source: &str,
	options: &AnalyzerOptions,
) -> Result<(String, NodeContext), AnalysisError> {
	let line = options
		.line
		.ok_or_else(|| AnalysisError::Parse("Missing line position".into()))?;

	let column = options
		.column
		.ok_or_else(|| AnalysisError::Parse("Missing column position".into()))?;

	let target_offset = source
		.lines()
		.take(line.saturating_sub(1) as usize)
		.map(|line| line.len() + 1)
		.sum::<usize>()
		+ column as usize;

	struct OffsetFinder {
		target: usize,
		found_ident: Option<String>,
		found_context: NodeContext,
	}

	impl<'ast> Visit<'ast> for OffsetFinder {
		fn visit_ident(&mut self, i: &'ast syn::Ident) {
			let range = i.span().byte_range();

			if self.target >= range.start && self.target <= range.end {
				self.found_ident = Some(i.to_string());
				self.found_context = NodeContext::Identifier;
			}
		}

		fn visit_expr(&mut self, i: &'ast syn::Expr) {
			let range = i.span().byte_range();

			if self.target >= range.start
				&& self.target <= range.end
				&& matches!(self.found_context, NodeContext::Unknown)
			{
				self.found_context = NodeContext::Expression;
			}

			syn::visit::visit_expr(self, i);
		}

		fn visit_stmt(&mut self, i: &'ast syn::Stmt) {
			let range = i.span().byte_range();

			if self.target >= range.start
				&& self.target <= range.end
				&& matches!(self.found_context, NodeContext::Unknown)
			{
				self.found_context = NodeContext::Statement;
			}

			syn::visit::visit_stmt(self, i);
		}
	}

	let mut finder = OffsetFinder {
		target: target_offset,
		found_ident: None,
		found_context: NodeContext::Unknown,
	};

	finder.visit_file(syntax_tree);

	let ident = finder.found_ident.ok_or_else(|| {
		AnalysisError::Parse(format!(
			"No valid identifier found at line {}, column {}",
			options.line.unwrap(),
			options.column.unwrap()
		))
	})?;

	Ok((ident, finder.found_context))
}

pub enum SymbolKindOutline {
	File = 1,
	Module = 2,
	Namespace = 3,
	Package = 4,

	Class = 5,
	Method = 6,
	Property = 7,
	Field = 8,
	Constructor = 9,

	Enum = 10,
	Interface = 11,
	Function = 12,
	Variable = 13,
	Constant = 14,

	String = 15,
	Number = 16,
	Boolean = 17,
	Array = 18,

	Object = 19,
	Key = 20,
	Null = 21,

	EnumMember = 22,
	Struct = 23,
	Event = 24,

	Operator = 25,
	TypeParameter = 26,
}
