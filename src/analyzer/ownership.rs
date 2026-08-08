impl Workspace {
	fn find_node_on_line(nodes: &[AstNode], line: usize) -> Option<&AstNode> {
		nodes
			.iter()
			.filter(|node| node.start_line <= line && line <= node.end_line)
			.min_by_key(|node| node.end_line - node.start_line)
	}
	fn resolve_node_context(syntax_tree: &syn::File, line: usize, column: usize) -> NodeContext {
		let mut resolver = NodeResolver {
			next_id: 0,
			position: pos(line, column),
			nodes: Vec::new(),
			ancestors: Vec::new(),
			best: None,
			line,
			column,
			candidates: Vec::new(),
			current_name: None,
		};
		println!("Candidates:");
		for c in &resolver.candidates {
			println!("  {:?}", c);
		}
		resolver.visit_file(syntax_tree);
		let mut candidates = resolver.candidates;
		candidates.sort_by_key(|node| {
			let start = node.span.start();
			let end = node.span.end();
			(end.line - start.line, end.column - start.column)
		});
		NodeContext {
			subject: candidates.first().cloned(),
			ancestors: candidates,
		}
	}
	fn classify_node(context: &NodeContext) -> NodeClassification {
		let Some(subject) = &context.subject else {
			return NodeClassification::Unknown;
		};
		let parent_kind = context.ancestors.first().map(|a| &a.kind);

		match (&subject.kind, parent_kind) {
			(AstNodeKind::PatternIdentifier, Some(AstNodeKind::Local)) => {
				NodeClassification::VariableDeclaration
			}

			(AstNodeKind::Local, _) => NodeClassification::VariableDeclaration,

			(AstNodeKind::PatternIdentifier, _) => NodeClassification::Variable,

			(AstNodeKind::Path, _) => NodeClassification::Variable,

			(AstNodeKind::Literal, _) => NodeClassification::Literal,
			(AstNodeKind::CallExpr, _) => NodeClassification::FunctionCall,
			(AstNodeKind::BinaryExpr, _) => NodeClassification::BinaryExpression,

			(AstNodeKind::IfExpr, _) => NodeClassification::Condition,
			(AstNodeKind::Block, _) => NodeClassification::Scope,
			_ => NodeClassification::Unknown,
		}
	}
	fn resolve_click(source: &str, line: usize, column: usize) -> NodeContext {
		let syntax_tree = syn::parse_file(source).unwrap();
		let resolver = NodeResolver {
			next_id: 0,
			line,
			column,
			current_name: None,
			position: pos(line, column),
			candidates: Vec::new(),
			best: None,
			nodes: Vec::new(),
			ancestors: Vec::new(),
		};
		resolver.resolve_file(&syntax_tree)
	}
	fn resolve_click_on(source: &str, name: &str, occurrence: Occurrence) -> NodeContext {
		let (line, column) = match occurrence {
			Occurrence::First => position_of(source, name),
			Occurrence::Last => position_of_last(source, name),
		};
		Self::resolve_click(source, line, column)
	}
	fn parse_file(file_path: &PathBuf) -> Result<(String, syn::File), AnalysisError> {
		let source =
			std::fs::read_to_string(file_path).map_err(|e| AnalysisError::Parse(e.to_string()))?;
		match syn::parse_file(&source) {
			Ok(tree) => Ok((source, tree)),
			Err(e) => Err(AnalysisError::Parse(e.to_string())),
		}
	}
	fn add_relation(
		related_lines: &mut Vec<LineRelated>,
		line: usize,
		file_path: &PathBuf,
		relation: OwnershipRelation,
	) {
		if let Some(existing) = related_lines.iter_mut().find(|x| x.line == line) {
			if !existing.relations.contains(&relation) {
				existing.relations.push(relation);
			}
			return;
		}
		related_lines.push(LineRelated {
			line,
			file_path: file_path.clone(),
			relations: vec![relation],
		});
	}
	fn build_downstream(graph: &Graph, subject: &NodeId) -> HashSet<NodeId> {
		let mut related = HashSet::new();
		let mut stack = vec![subject];
		while let Some(current) = stack.pop() {
			for edge in graph.edges_from(&current) {
				if edge.influence != InfluenceKind::Direct {
					continue;
				}
				match edge.relation {
					RelationKind::Downstream => {
						if related.insert(edge.to) {
							stack.push(&edge.to);
						}
					}

					_ => {}
				}
			}
		}
		related
	}
	fn build_direct_relationships(graph: &mut Graph, symbols: &[SymReference]) {
		for symbol in symbols {
			let Some(id) = symbol.resolved_id else {
				continue;
			};

			println!(
				"DIRECT SYMBOL: {} id={:?} role={:?} relation={:?}",
				symbol.name, id, symbol.role, symbol.relation
			);

			// A Reference/Assignment/Argument/etc. points at the
			// resolved symbol. The actual source -> target relationship
			// should already have been created by the visitor when the
			// AST gave us enough context.
		}
		for edge in &graph.edges {
			println!(
				"{:?} -> {:?} ({:?}, {:?})",
				edge.from, edge.to, edge.relation, edge.influence
			);
		}
	}
	fn build_graph(
		tree: &syn::File,
		options: &AnalyzerOptions,
		subject: &ResolvedNode,
	) -> OwnershipGraph {
		let mut graph = Graph::new();
		let (symbols, subject_id, related_spans) = {
			let mut visitor = OwnershipVisitor::new(
				subject.clone(),
				subject.name.clone(),
				options.clone(),
				&mut graph,
			);
			visitor.visit_file(tree);
			(
				visitor.related_symbols.clone(),
				visitor.subject_symbol,
				visitor.related_spans.clone(),
			)
		};
		OwnershipGraph {
			graph,
			symbols,
			subject_id,
			related_spans,
		}
	}
	fn analyze_ownership(
		file_path: &PathBuf,
		source: &str,
		syntax_tree: &syn::File,
		options: &AnalyzerOptions,
	) -> Result<AnalysisReport, AnalysisError> {
		let click = ClickContext::new(file_path, source, options);
		let mut cfg = AnalyzeConfig::new(Some(file_path), options);
		let log = Log::new(options, &mut cfg);
		log.cfg.set_level("1");
		let ctx = Self::resolve_node_context(syntax_tree, click.line, click.column);
		let subject = ctx
			.subject
			.as_ref()
			.ok_or_else(|| AnalysisError::Parse("No subject node found".into()))?;
		let classification = Self::classify_node(&ctx);
		log.print("Subject", Vals::new().subject(subject));
		log.print(
			"Classification",
			Vals::new().classification(&classification),
		);
		for ancestor in &ctx.ancestors {
			log.print("Ancestors", Vals::new().ancestors(&ancestor.kind));
		}
		let ownership = Self::build_graph(syntax_tree, options, subject);

		// let lines = Self::stage_build_line_analysis(&ownership, file_path, source);
		let lines = Self::stage_build_related_lines_from_subject(&ownership, file_path, subject);
		for line in &lines {
			if !line.relations.is_empty() {
				println!("RELATED LINE {} => {:?}", line.line, line.relations);
			}
		}
		Self::stage_report(ctx, click, lines, classification)
	}
	pub fn analyze_ownership_on_click(
		file_path: &PathBuf,
		options: &AnalyzerOptions,
	) -> Result<AnalysisReport, AnalysisError> {
		let (source, syntax_tree) = Self::parse_file(file_path)?;
		Self::analyze_ownership(file_path, &source, &syntax_tree, options)
	}
	pub fn stage_build_line_analysis(
		ownership: &OwnershipGraph,
		file_path: &PathBuf,
		source: &str,
	) -> Vec<LineRelated> {
		let line_count = source.lines().count();

		let mut lines: Vec<LineRelated> = (1..=line_count)
			.map(|line| LineRelated {
				line,
				file_path: file_path.clone(),
				relations: Vec::new(),
			})
			.collect();

		let mut add = |line: usize, relation: OwnershipRelation| {
			if let Some(entry) = lines.get_mut(line.saturating_sub(1)) {
				if !entry.relations.contains(&relation) {
					entry.relations.push(relation);
				}
			}
		};

		let Some(subject_id) = ownership.subject_id else {
			return lines;
		};
		let downstream = Self::build_downstream(&ownership.graph, &subject_id);
		for (span, relation) in &ownership.related_spans {
			add(span.start().line, relation.clone());
		}
		for symbol in &ownership.symbols {
			let Some(id) = symbol.resolved_id else {
				continue;
			};
			if id != subject_id && !downstream.contains(&id) {
				continue;
			}
			match symbol.role {
				SymRole::Reference => {
					add(symbol.span.start().line, symbol.relation.clone());
				}
				SymRole::Declaration => {
					// Deliberately don't add declarations here.
				}

				_ => {}
			}
		}

		lines
	}
	pub fn stage_build_related_lines_from_subject(
		ownership: &OwnershipGraph,
		file_path: &PathBuf,
		_subject: &ResolvedNode,
	) -> Vec<LineRelated> {
		let mut related_lines = Vec::new();

		let Some(subject_id) = ownership.subject_id else {
			return related_lines;
		};

		let downstream = Self::build_downstream(&ownership.graph, &subject_id);

		for symbol in &ownership.symbols {
			let Some(id) = symbol.resolved_id else {
				continue;
			};

			let is_subject = id == subject_id;
			let is_downstream = downstream.contains(&id);

			if !is_subject && !is_downstream {
				continue;
			}

			Self::add_relation(
				&mut related_lines,
				symbol.span.start().line,
				file_path,
				symbol.relation.clone(),
			);
		}

		related_lines
	}
	pub fn stage_report(
		ctx: NodeContext,
		click: ClickContext,
		related_lines: Vec<LineRelated>,
		classification: NodeClassification,
	) -> Result<AnalysisReport, AnalysisError> {
		// let mut stdout = io::stdout();
		// stdout
		//     .write_all(json_output.as_bytes())
		//     .map_err(|e| AnalysisError::IoError(e.to_string()))?;
		// stdout
		//     .write_all(b"\n")
		//     .map_err(|e| AnalysisError::IoError(e.to_string()))?;
		// stdout
		//     .flush()
		//     .map_err(|e| AnalysisError::IoError(e.to_string()))?;
		let analysis = related_lines.clone();
		let analysis = AnalysisData {
			related_lines: analysis,
			node_context: ctx.clone(),
			classification: classification,
			symbols: Vec::new(),
		};
		let formatted_output = build_final_analysis(&click, Some(&related_lines));

		let report = AnalysisReport {
			click,
			analysis,
			formatted_output,
		};
		let json_output = serde_json::to_string(&report)?;
		Ok(report)
	}
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymRelation {
	pub from: SymId,
	pub to: SymId,
	pub kind: RelationKind,
	pub label: Option<String>,
}
#[derive(Debug, Clone)]
pub struct SymReference {
	pub name: String,                // e.g., "bar"
	pub role: SymRole,               // Declaration, Reference, Mutation, etc.
	pub span: Span,                  // Exact line/col of *this specific usage*
	pub resolved_id: Option<NodeId>, // Links back to its SymInfo
	pub relation: OwnershipRelation,
}
struct SymNode {
	pub id: SymId,
	pub name: String,
	pub role: SymRole,
	pub location: SymLocation,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum SymRole {
	Declaration,
	Reference,
	Definition,
	Scope,
	File,
	Module,
	Function,
	Variable,
	Parameter,
	Type,
	Field,
	Expression,
}
struct SymLocation;

struct ResolvedSubject {
	pub name: String,
	pub kind: NodeContext,
	pub span: Span,
}
#[derive(Default)]
struct ScopeVisitor {
	pub target_line: u32,
	pub target_column: u32,
	pub scopes: Vec<proc_macro2::Span>,
}
struct SymbolVisitor {
	pub subject: String,
	pub nodes: Vec<SymNode>,
	pub relations: Vec<SymRelation>,
	pub scopes: Vec<Span>,
}
impl ScopeVisitor {
	fn check_scope(&mut self, span: proc_macro2::Span) {
		let start = span.start();
		let end = span.end();
		let inside = (self.target_line >= start.line as u32) && (self.target_line <= end.line as u32);
		if inside {
			self.scopes.push(span);
		}
	}
}
impl<'ast> syn::visit::Visit<'ast> for ScopeVisitor {
	fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
		self.check_scope(node.span());
		syn::visit::visit_item_fn(self, node);
	}
	fn visit_block(&mut self, node: &'ast syn::Block) {
		self.check_scope(node.span());
		syn::visit::visit_block(self, node);
	}
}

pub struct ScopeLine {
	pub line: usize,
	pub span: Span,
}
pub struct ScopeInfo {
	pub start_line: usize,
	pub end_line: usize,
	pub span: Span,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineRelated {
	pub line: usize,
	pub file_path: PathBuf,
	pub relations: Vec<OwnershipRelation>,
}
#[derive(Serialize)]
pub struct LineSymbol {
	pub line: usize,
	pub name: String,
	pub role: SymRole,
}
pub struct LineAnalysis {
	pub line: usize,
	pub text: String,
	pub flags: LineFlags,
	pub symbols: Vec<SymReference>,
	pub relations: Vec<OwnershipRelation>,
}

pub struct LineAnnotation {
	pub line: usize,
	pub text: String,
}
#[derive(Default, Debug, Clone)]
pub struct LineFlags {
	pub in_scope: bool,
	// Does this line directly mention the subject?
	pub references_subject: bool,
	// Does this line create/change the subject?
	pub influences_subject: bool,
	// Is this line unrelated noise?
	pub unrelated: bool,
	// Is this the declaration site?
	pub is_declaration: bool,
	// Is this where the subject is used?
	pub is_usage: bool,
	// Optional: why it influences
	pub influence_kind: Option<InfluenceKind>,
}

#[derive(Debug)]
pub enum LineKind {
	// Statements
	LetBinding,
	ExpressionStatement,
	Return,
	Break,
	Continue,
	// Control flow
	If,
	Match,
	Loop,
	While,
	For,
	// Items
	Function,
	Struct,
	Enum,
	Trait,
	Impl,
	Module,
	Use,
	// Macro / compiler constructs
	Macro,
	Statement(StatementKind),
	Expression(ExpressionKind),
	Item(ItemKind),
	Unknown,
	Cursor,
}
#[derive(Debug)]
pub enum StatementKind {}
#[derive(Debug)]
pub enum ExpressionKind {}
#[derive(Debug)]
pub enum ItemKind {}
pub struct LineContext {
	pub kind: LineKind,
	pub span: Span,
}
struct LineClassifier {
	line: u32,
	result: Option<LineKind>,
}
impl<'ast> syn::visit::Visit<'ast> for LineClassifier {
	fn visit_local(&mut self, node: &'ast syn::Local) {
		if span_contains_line(node.span(), self.line) {
			self.result = Some(LineKind::LetBinding);
		}
		syn::visit::visit_local(self, node);
	}
	fn visit_expr(&mut self, node: &'ast syn::Expr) {
		if span_contains_line(node.span(), self.line) {
			self.result = Some(LineKind::ExpressionStatement);
		}
		syn::visit::visit_expr(self, node);
	}
	fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
		if span_contains_line(node.span(), self.line) {
			self.result = Some(LineKind::Function);
		}
		syn::visit::visit_item_fn(self, node);
	}
}

fn same_span(a: proc_macro2::Span, b: SerializableSpan) -> bool {
	let a_start = a.start();
	let a_end = a.end();

	let b_start = b.start();
	let b_end = b.end();

	a_start.line == b_start.line
		&& a_start.column == b_start.column
		&& a_end.line == b_end.line
		&& a_end.column == b_end.column
}
fn classify_line(syntax_tree: &syn::File, line: u32) -> LineKind {
	let mut visitor = LineClassifier { line, result: None };
	visitor.visit_file(syntax_tree);
	visitor.result.unwrap_or(LineKind::Unknown)
}
fn span_contains_line(span: proc_macro2::Span, line: u32) -> bool {
	let start = span.start().line as u32;
	let end = span.end().line as u32;
	line >= start && line <= end
}

#[derive(Clone, Serialize)]
pub struct ClickContext {
	pub file: PathBuf,
	pub line: usize,
	pub column: usize,
	pub source: String,
}
pub struct ClickTarget {
	pub node: AstNode,
	pub ancestors: Vec<AstNode>,
}
impl ClickContext {
	pub fn new(file_path: &PathBuf, source: &str, options: &AnalyzerOptions) -> Self {
		Self {
			file: file_path.clone(),
			line: options.line.unwrap_or(1) as usize,
			column: options.column.unwrap_or(0) as usize,
			source: source.to_string(),
		}
	}
	// pub fn print_cursor(&self) {
	//     let line_idx = self.line.saturating_sub(1) as usize;
	//     let Some(source_line) = self.source.lines().nth(line_idx) else {
	//         return;
	//     };
	//     println!("SOURCE:");
	//     println!("{:>4} | {}", self.line, source_line);
	//     let padding = " ".repeat(self.column as usize);
	//     println!(
	//         "     | {}^ (column {}, char {:?})",
	//         padding,
	//         self.column,
	//         source_line.chars().nth(self.column as usize)
	//     );
	// }
}
#[derive(Debug, Clone)]
pub struct AstNode {
	pub span: Span,
	pub name: String,
	pub kind: String,
	pub start_line: usize,
	pub start_col: usize,
	pub end_line: usize,
	pub end_col: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AstNodeKind {
	// Identifiers / symbols
	Identifier,
	Binding,
	// Values
	PatternIdentifier,
	Literal,
	// Expressions
	BinaryExpr,
	UnaryExpr,
	CallExpr,
	MethodCallExpr,
	FieldAccess,
	IndexExpr,
	PathExpr,
	// Statements
	Statement,
	LetStatement,
	ExpressionStatement,
	ReturnStatement,
	// Control flow
	IfExpression,
	MatchExpression,
	MatchArm,
	Loop,
	WhileLoop,
	ForLoop,
	// Containers
	Block,
	Function,
	Closure,
	// Types
	Type,
	Struct,
	Enum,
	Local,
	Expr,
	IfExpr,
	Path,
	// Fallback
	Unknown,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct SerializableSpan {
	pub start_line: usize,
	pub start_col: usize,
	pub end_line: usize,
	pub end_col: usize,
}
impl From<proc_macro2::Span> for SerializableSpan {
	fn from(span: proc_macro2::Span) -> Self {
		let start = span.start();
		let end = span.end();
		Self {
			start_line: start.line,
			start_col: start.column,
			end_line: end.line,
			end_col: end.column,
		}
	}
}
#[derive(Serialize, Debug, Clone)]
pub struct ResolvedNode {
	pub id: NodeId,
	pub kind: AstNodeKind,
	pub span: SerializableSpan,
	pub mutable: Option<bool>,
	pub name: Option<String>,
}

impl From<serde_json::Error> for AnalysisError {
	fn from(err: serde_json::Error) -> Self {
		// AnalysisError::SerializationError(err.to_string())
		// AnalysisError::Json(err.to_string())
		AnalysisError::IoError(err.to_string())
	}
}
impl SerializableSpan {
	pub fn start(&self) -> DummyLineColumn {
		DummyLineColumn {
			line: self.start_line,
			column: self.start_col,
		}
	}
	pub fn end(&self) -> DummyLineColumn {
		DummyLineColumn {
			line: self.end_line,
			column: self.end_col,
		}
	}
	pub fn contains(&self, span: proc_macro2::Span) -> bool {
		let start = span.start();
		let end = span.end();

		start.line >= self.start_line
			&& start.column >= self.start_col
			&& end.line <= self.end_line
			&& end.column <= self.end_col
	}
}
pub struct DummyLineColumn {
	pub line: usize,
	pub column: usize,
}
#[derive(Debug, Clone, Serialize)]
pub struct NodeContext {
	pub subject: Option<ResolvedNode>,
	pub ancestors: Vec<ResolvedNode>,
}
struct NodeResolver {
	pub line: usize,
	pub column: usize,
	current_name: Option<String>,
	position: SourcePosition,
	pub candidates: Vec<ResolvedNode>,
	pub best: Option<ResolvedNode>,
	pub nodes: Vec<AstNodeKind>,
	pub ancestors: Vec<ResolvedNode>,
	next_id: usize,
}
impl NodeResolver {
	// 	fn next_node_id(&mut self) -> NodeId {
	// 	let id = self.next_id;
	// 	self.next_id = NodeId(self.next_id.0 + 1);
	// 	id
	// }
	fn next_node_id(&mut self) -> NodeId {
		let id = NodeId(self.next_id);
		self.next_id += 1;
		id
	}
	pub fn new(line: usize, column: usize, position: SourcePosition) -> Self {
		Self {
			line,
			column,
			current_name: None,
			position,
			candidates: Vec::new(),
			best: None,
			nodes: Vec::new(),
			ancestors: Vec::new(),
			next_id: 0,
		}
	}
	fn resolve(mut self) -> NodeContext {
		self.candidates.sort_by_key(|node| {
			let start = node.span.start();
			let end = node.span.end();
			(end.line - start.line, end.column - start.column)
		});
		let subject = self.candidates.first().cloned();
		NodeContext {
			subject,
			ancestors: self.candidates,
		}
	}
	fn resolve_file(mut self, file: &syn::File) -> NodeContext {
		self.visit_file(file);

		self.candidates.sort_by_key(|node| {
			let start = node.span.start();
			let end = node.span.end();

			(end.line - start.line, end.column - start.column)
		});

		NodeContext {
			subject: self.candidates.first().cloned(),
			ancestors: self.candidates,
		}
	}

	fn contains(&self, span: Span) -> bool {
		let ((start_line, start_column), (end_line, end_column)) = line_col(span);
		let after_start =
			self.line > start_line || (self.line == start_line && self.column >= start_column);
		let before_end = self.line < end_line || (self.line == end_line && self.column <= end_column);
		after_start && before_end
	}
	fn check(&mut self, kind: AstNodeKind, span: proc_macro2::Span) {
		if !self.contains(span) {
			return;
		}

		let id = self.next_node_id();

		self.candidates.push(ResolvedNode {
			id,
			kind,
			name: None,
			span: span.into(),
			mutable: Some(false),
		});
	}

	fn check_with_name(&mut self, kind: AstNodeKind, span: proc_macro2::Span, name: Option<String>) {
		if !self.contains(span) {
			return;
		}

		let id = self.next_node_id();

		self.candidates.push(ResolvedNode {
			id,
			kind,
			name,
			span: span.into(),
			mutable: Some(false),
		});
	}

	fn check_with_metadata(
		&mut self,
		kind: AstNodeKind,
		span: proc_macro2::Span,
		name: Option<String>,
		mutable: Option<bool>,
	) {
		if !self.contains(span) {
			return;
		}

		let id = self.next_node_id();

		self.candidates.push(ResolvedNode {
			id,
			kind,
			span: span.into(),
			name,
			mutable,
		});
	}
	fn visit_macro_tokens(&mut self, tokens: proc_macro2::TokenStream) {
		for token in tokens {
			match token {
				TokenTree::Ident(ident) => {
					self.check_with_name(
						AstNodeKind::PatternIdentifier,
						ident.span(),
						Some(ident.to_string()),
					);
				}
				TokenTree::Group(group) => {
					self.visit_macro_tokens(group.stream());
				}
				TokenTree::Punct(_) | TokenTree::Literal(_) => {}
			}
		}
	}
}
impl<'ast> syn::visit::Visit<'ast> for NodeResolver {
	fn visit_macro(&mut self, node: &'ast syn::Macro) {
		self.visit_macro_tokens(node.tokens.clone());
		syn::visit::visit_macro(self, node);
	}
	fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
		self.check_with_name(
			AstNodeKind::PatternIdentifier,
			node.span(),
			Some(node.ident.to_string()),
		);
		syn::visit::visit_pat_ident(self, node);
	}
	fn visit_local(&mut self, node: &'ast syn::Local) {
		let (name, mutable) = match &node.pat {
			syn::Pat::Ident(pat) => (Some(pat.ident.to_string()), Some(pat.mutability.is_some())),
			_ => (None, None),
		};

		self.check_with_metadata(AstNodeKind::Local, node.span(), name, mutable);

		syn::visit::visit_local(self, node);
	}
	fn visit_block(&mut self, node: &'ast syn::Block) {
		if self.contains(node.span()) {
			self.nodes.push(AstNodeKind::Block);
		}
		syn::visit::visit_block(self, node);
	}
	fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
		if self.contains(node.span()) {
			self.nodes.push(AstNodeKind::CallExpr);
		}
		syn::visit::visit_expr_call(self, node);
	}
	fn visit_item_fn(&mut self, node: &syn::ItemFn) {
		self.check_with_name(
			AstNodeKind::Function,
			node.sig.ident.span(),
			Some(node.sig.ident.to_string()),
		);
		syn::visit::visit_item_fn(self, node);
	}
	fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
		let name = node
			.path
			.segments
			.last()
			.map(|segment| segment.ident.to_string());

		self.check_with_name(AstNodeKind::Path, node.span(), name);

		syn::visit::visit_expr_path(self, node);
	}
	fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
		if self.contains(node.span()) {
			self.nodes.push(AstNodeKind::BinaryExpr);
		}
		syn::visit::visit_expr_binary(self, node);
	}
	fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
		if self.contains(node.span()) {
			self.nodes.push(AstNodeKind::IfExpr);
		}
		syn::visit::visit_expr_if(self, node);
	}

	fn visit_expr_lit(&mut self, node: &'ast syn::ExprLit) {
		if self.contains(node.span()) {
			self.nodes.push(AstNodeKind::Literal);
		}
		syn::visit::visit_expr_lit(self, node);
	}
}

use proc_macro2::TokenTree;

#[derive(Debug, Clone)]
pub enum NodeGoal {
	Expression,
	Statement,
	Declaration,
	Unknown,
}
impl NodeContext {
	pub fn classify(&self) -> NodeGoal {
		for node in &self.ancestors {
			match node.kind {
				AstNodeKind::Path
				| AstNodeKind::Literal
				| AstNodeKind::BinaryExpr
				| AstNodeKind::CallExpr
				| AstNodeKind::IfExpr => {
					return NodeGoal::Expression;
				}
				AstNodeKind::Local => {
					return NodeGoal::Declaration;
				}
				AstNodeKind::Statement => {
					return NodeGoal::Statement;
				}
				_ => {}
			}
		}
		NodeGoal::Unknown
	}
}
pub struct OwnershipNode {
	pub id: SymId,
	pub name: String,
	pub kind: SymbolKind,
	pub location: Span,
	pub ownership: OwnershipRole,
	pub children: Vec<OwnershipRelation>,
	pub parents: Vec<OwnershipRelation>,
}
pub enum OwnershipRole {
	Owner,
	Borrower,
	Moved,
	Clone,
	Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OwnershipRelation {
	Call,
	Declaration,
	Reference,
	Definition,
	Downstream,
	Upstream,
	Assignment,
	Argument,
	Return,
	Mutation,
	Scope,
	ImmutableBorrow,
	MutableBorrow,
	MoveOwnership,
}

// - Subject: Identifier clicked on
// - Candidates: - Potential subjects if one was not clicked on. For example they clicked 'end of line' or a key word like 'impl'.
// - Ancestor: An identfiier which influences the ownership of this subject.
// - Descendents: Identifies who are influenced by this subject
// - Scope:
#[derive(Debug)]
pub struct OwnershipVisitor<'a> {
	// ── Analysis input ─────────────────────────────
	subject: ResolvedNode,
	pub subject_name: Option<String>,
	pub options: AnalyzerOptions,

	// ── Symbol resolution ─────────────────────────
	scopes: Vec<HashMap<String, NodeId>>,
	next_id: NodeId,

	// ── Traversal state ────────────────────────────
	function_depth: usize,
	current_function: Option<NodeId>,

	// ── Results ────────────────────────────────────
	pub subject_symbol: Option<NodeId>,
	pub related_spans: Vec<(proc_macro2::Span, OwnershipRelation)>,
	pub related_symbols: Vec<SymReference>,
	pub graph: &'a mut Graph,

	// ── AST bookkeeping ────────────────────────────
	pub scope_spans: Vec<proc_macro2::Span>,
	last_call_id: Option<NodeId>,
	call_ids: HashMap<(usize, usize), NodeId>,
}
impl<'a> OwnershipVisitor<'a> {
	pub fn new(
		subject: ResolvedNode,
		subject_name: Option<String>,
		options: AnalyzerOptions,
		graph: &'a mut Graph,
	) -> Self {
		Self {
			graph,
			subject,
			options,
			next_id: NodeId(0),
			subject_name,
			function_depth: 0,
			subject_symbol: None,
			current_function: None,
			scope_spans: Vec::new(),
			related_spans: Vec::new(),
			related_symbols: Vec::new(),
			scopes: vec![HashMap::new()],
			last_call_id: None,
			call_ids: HashMap::new(),
		}
	}

	fn next_node_id(&mut self) -> NodeId {
		let id = self.next_id;
		self.next_id = NodeId(self.next_id.0 + 1);
		id
	}
	fn push_scope(&mut self) {
		self.scopes.push(HashMap::new());
	}
	fn pop_scope(&mut self) {
		self.scopes.pop();
	}
	fn define_symbol(&mut self, name: String) -> NodeId {
		let id = self.next_node_id();
		self.scopes.last_mut().unwrap().insert(name.clone(), id);
		if self.subject_name.as_deref() == Some(&name) {
			self.subject_symbol = Some(id);
		}
		id
	}
	fn resolve_symbol(&self, name: &str) -> Option<NodeId> {
		self
			.scopes
			.iter()
			.rev()
			.find_map(|scope| scope.get(name).copied())
	}
	fn build_downstream(graph: &Graph, subject: &NodeId) -> HashSet<NodeId> {
		let mut related = HashSet::new();
		let mut stack = vec![subject];
		while let Some(current) = stack.pop() {
			for edge in graph.edges_from(&current) {
				if edge.influence != InfluenceKind::Direct {
					continue;
				}
				match edge.relation {
					RelationKind::Downstream => {
						if related.insert(edge.to) {
							stack.push(&edge.to);
						}
					}
					_ => {}
				}
			}
		}
		related
	}
	fn visit_macro_tokens(&mut self, tokens: proc_macro2::TokenStream) {
		let downstream = self
			.subject_symbol
			.map(|id| Self::build_downstream(self.graph, &id))
			.unwrap_or_default();

		self.visit_macro_tokens_inner(tokens, &downstream);
	}

	fn visit_macro_tokens_inner(
		&mut self,
		tokens: proc_macro2::TokenStream,
		downstream: &HashSet<NodeId>,
	) {
		for token in tokens {
			match token {
				TokenTree::Ident(ident) => {
					let name = ident.to_string();
					let span = ident.span();

					let Some(resolved_id) = self.resolve_symbol(&name) else {
						continue;
					};

					let is_subject = Some(resolved_id) == self.subject_symbol;
					let is_downstream = downstream.contains(&resolved_id);

					if is_subject || is_downstream {
						self
							.related_spans
							.push((span, OwnershipRelation::Reference));

						self.related_symbols.push(SymReference {
							name,
							span,
							role: SymRole::Reference,
							resolved_id: Some(resolved_id),
							relation: OwnershipRelation::Reference,
						});
					}
				}

				TokenTree::Group(group) => {
					self.visit_macro_tokens_inner(group.stream(), downstream);
				}

				TokenTree::Punct(_) | TokenTree::Literal(_) => {}
			}
		}
	}
	fn get_or_create_call_id(&mut self, call: &syn::ExprCall) -> NodeId {
		let key = (call.span().start().line, call.span().start().column);
		if let Some(&id) = self.call_ids.get(&key) {
			return id;
		}
		let id = self.define_symbol(format!("<call:{}:{}>", key.0, key.1));
		self.call_ids.insert(key, id);
		id
	}
	fn call_id_for(&self, node: &syn::ExprCall) -> Option<NodeId> {
		let key = (node.span().start().line, node.span().start().column);
		self.call_ids.get(&key).copied()
	}
}
impl<'ast, 'a> syn::visit::Visit<'ast> for OwnershipVisitor<'a> {
	fn visit_expr(&mut self, node: &'ast syn::Expr) {
		match node {
			syn::Expr::MethodCall(method) => {
				let method_name = method.method.to_string();
				if method_name == "clone" {
					let span = method.span();
				}
			}
			syn::Expr::Call(call) => {
				if let syn::Expr::Path(expr_path) = &*call.func {
					if let Some(segment) = expr_path.path.segments.last() {
						if segment.ident == "Box" {}
					}
				}
			}
			_ => {}
		}
		syn::visit::visit_expr(self, node);
	}
	fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
		let key = (node.span().start().line, node.span().start().column);

		let call_id = self.next_node_id();

		let start = node.span().start();
		self.call_ids.insert((start.line, start.column), call_id);
		self.last_call_id = Some(call_id);
		let mut subject_related = false;

		for arg in &node.args {
			let syn::Expr::Path(path) = arg else {
				continue;
			};

			let Some(segment) = path.path.segments.last() else {
				continue;
			};

			let name = segment.ident.to_string();

			let Some(source_id) = self.resolve_symbol(&name) else {
				continue;
			};

			self.graph.add_relation(
				source_id,
				call_id,
				RelationKind::Downstream,
				InfluenceKind::Direct,
				Some(OwnershipRelation::Argument),
			);

			if Some(source_id) == self.subject_symbol {
				subject_related = true;
				self.related_symbols.push(SymReference {
					name,
					span: arg.span().into(),
					role: SymRole::Reference,
					resolved_id: Some(source_id),
					relation: OwnershipRelation::Argument,
				});
			}
		}

		if subject_related {
			self
				.related_spans
				.push((node.span(), OwnershipRelation::Argument));
		}

		syn::visit::visit_expr_call(self, node);
	}

	fn visit_type(&mut self, node: &'ast syn::Type) {
		match node {
			syn::Type::Path(type_path) => {
				// Extracts explicit type paths (e.g., String, i32, MyCustomStruct)
				if let Some(segment) = type_path.path.segments.last() {
					let type_name = segment.ident.to_string();
					// Track or log type usage here
				}
			}
			syn::Type::Reference(type_ref) => {
				// Detects explicit references (&T or &mut T)
				// Useful for tracking borrowing vs ownership
			}
			_ => {}
		}
		syn::visit::visit_type(self, node);
	}
	fn visit_block(&mut self, node: &'ast syn::Block) {
		let span = node.span();
		self.scope_spans.push(span); // Track physical code range
		self.push_scope(); // Push semantic symbol map
		syn::visit::visit_block(self, node); // Walk children inside the block
		self.pop_scope(); // Pop symbol map when leaving block
	}
	fn visit_local(&mut self, node: &'ast syn::Local) {
		let syn::Pat::Ident(pat_ident) = &node.pat else {
			syn::visit::visit_local(self, node);
			return;
		};

		let name = pat_ident.ident.to_string();
		let id = self.define_symbol(name.clone());

		self.related_symbols.push(SymReference {
			name,
			span: pat_ident.ident.span().into(),
			role: SymRole::Declaration,
			resolved_id: Some(id),
			relation: OwnershipRelation::Declaration,
		});

		// Visit the initializer first. This creates the call node.
		syn::visit::visit_local(self, node);

		if let Some(init) = &node.init {
			if let syn::Expr::Call(call) = &*init.expr {
				if let Some(call_id) = self.call_id_for(call) {
					self.graph.add_relation(
						call_id,
						id,
						RelationKind::Downstream,
						InfluenceKind::Direct,
						Some(OwnershipRelation::Return),
					);
				}
			}
		}
	}
	fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
		let name = node.sig.ident.to_string();
		let span = node.sig.ident.span();

		// Function itself belongs to the enclosing scope.
		let function_id = self.define_symbol(name.clone());

		println!("DEFINE FUNCTION {} => {:?}", name, function_id);

		self.related_symbols.push(SymReference {
			name: name.clone(),
			span: span.into(),
			role: SymRole::Declaration,
			resolved_id: Some(function_id),
			relation: OwnershipRelation::Definition,
		});

		// Now enter the function's local scope.
		self.push_scope();

		syn::visit::visit_item_fn(self, node);

		self.pop_scope();
	}
	fn visit_expr_return(&mut self, node: &'ast syn::ExprReturn) {
		if let Some(expr) = &node.expr {
			if let syn::Expr::Path(path) = &**expr {
				if let Some(segment) = path.path.segments.last() {
					let name = segment.ident.to_string();

					if let Some(source_id) = self.resolve_symbol(&name) {
						println!("RETURN {} => {:?}", name, source_id);

						self.related_symbols.push(SymReference {
							name,
							span: expr.span().into(),
							role: SymRole::Reference,
							resolved_id: Some(source_id),
							relation: OwnershipRelation::Return,
						});

						if Some(source_id) == self.subject_symbol {
							self
								.related_spans
								.push((expr.span(), OwnershipRelation::Return));
						}
					}
				}
			}
		}

		syn::visit::visit_expr_return(self, node);
	}
	fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
		use syn::BinOp;

		match &node.op {
			BinOp::AddAssign(_)
			| BinOp::SubAssign(_)
			| BinOp::MulAssign(_)
			| BinOp::DivAssign(_)
			| BinOp::RemAssign(_)
			| BinOp::BitXorAssign(_)
			| BinOp::BitAndAssign(_)
			| BinOp::BitOrAssign(_)
			| BinOp::ShlAssign(_)
			| BinOp::ShrAssign(_) => {
				if let syn::Expr::Path(lhs) = &*node.left {
					if let Some(segment) = lhs.path.segments.last() {
						let name = segment.ident.to_string();

						if let Some(id) = self.resolve_symbol(&name) {
							println!(
								"MUTATION {} => {:?} subject={:?}",
								name, id, self.subject_symbol
							);

							if Some(id) == self.subject_symbol {
								self
									.related_spans
									.push((lhs.span(), OwnershipRelation::Mutation));

								self.related_symbols.push(SymReference {
									name,
									span: lhs.span().into(),
									role: SymRole::Reference,
									resolved_id: Some(id),
									relation: OwnershipRelation::Mutation,
								});
							}
						}
					}
				}
			}

			_ => {}
		}
		syn::visit::visit_expr_binary(self, node);
	}
	fn visit_macro(&mut self, node: &'ast syn::Macro) {
		// println!("🔥 VISIT_MACRO");

		// println!("MACRO PATH: {:?}", node.path);
		// println!("MACRO TOKENS: {:?}", node.tokens);

		self.visit_macro_tokens(node.tokens.clone());

		syn::visit::visit_macro(self, node);
	}
	fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
		// println!("🔥 VISIT_EXPR_MACRO HIT");
		// println!("MACRO PATH: {:?}", node.mac.path);
		// println!("MACRO TOKENS: {:?}", node.mac.tokens);

		self.visit_macro_tokens(node.mac.tokens.clone());

		syn::visit::visit_expr_macro(self, node);
	}
	fn visit_expr_assign(&mut self, node: &'ast syn::ExprAssign) {
		if let syn::Expr::Path(lhs) = &*node.left {
			if let Some(segment) = lhs.path.segments.last() {
				let name = segment.ident.to_string();

				if let Some(id) = self.resolve_symbol(&name) {
					if Some(id) == self.subject_symbol {
						self.related_symbols.push(SymReference {
							name,
							span: lhs.span().into(),
							role: SymRole::Reference,
							resolved_id: Some(id),
							relation: OwnershipRelation::Assignment,
						});
					}
				}
			}
		}

		// Important: don't walk the LHS again.
		self.visit_expr(&node.right);
	}
	fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
		let span = node.span();

		let Some(segment) = node.path.segments.last() else {
			return;
		};

		let name = segment.ident.to_string();

		let Some(resolved_id) = self.resolve_symbol(&name) else {
			syn::visit::visit_expr_path(self, node);
			return;
		};

		// println!(
		// 	"REFERENCE {} => {} subject={:?}",
		// 	name, resolved_id, self.subject_symbol
		// );

		// This reference resolves to the node we clicked.
		if Some(resolved_id) == self.subject_symbol {
			self
				.related_spans
				.push((span, OwnershipRelation::Reference));

			self.related_symbols.push(SymReference {
				name: name.clone(),
				span: span.into(),
				role: SymRole::Reference,
				resolved_id: Some(resolved_id),
				relation: OwnershipRelation::Reference,
			});
		}

		syn::visit::visit_expr_path(self, node);
	}
	fn visit_expr_unary(&mut self, node: &'ast syn::ExprUnary) {
		if let syn::UnOp::Deref(_) = &node.op {
			// Handle *x later.
		}

		syn::visit::visit_expr_unary(self, node);
	}
}
// impl<'ast> syn::visit::Visit<'ast> for OwnershipVisitor {
//     fn visit_type(&mut self, node: &'ast syn::Expr) {}
//     fn visit_expr(&mut self, node: &'ast syn::Expr) {}
//     fn visit_block(&mut self, node: &'ast syn::Expr) {}
//     fn visit_local(&mut self, node: &'ast syn::Expr) {}
//     fn visit_item_fn(&mut self, node: &'ast syn::Expr) {}
//     fn visit_expr_path(&mut self, node: &'ast syn::Expr) {}
// }
pub struct ClickReport {
	pub context: ClickContext,
	pub symbols: Vec<LineSymbol>,
	pub node_context: Option<NodeContext>,
}
use crate::{
	_config::{AnalyzeConfig, Logger as Log, Vals},
	_scope::Scope,
	analyzer::*,
	ir::*,
};
use proc_macro2::Span;
use quote::ToTokens;
use regex_syntax::ast::Ast;
use serde::{Deserialize, Serialize};
use std::{
	collections::{HashMap, HashSet, VecDeque},
	fs::read_to_string,
	io::{self},
	path::{Path, PathBuf},
};
use std::{fmt::Write, io::Write as _};
use swc_core::common::LineCol;
use syn::{
	File, Ident, Token,
	spanned::Spanned,
	token::Token,
	visit::{self, Visit, visit_local},
	visit_mut::{self, VisitMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourcePosition {
	pub line: u32,
	pub column: u32,
}
impl From<proc_macro2::LineColumn> for SourcePosition {
	fn from(pos: proc_macro2::LineColumn) -> Self {
		Self {
			line: pos.line as u32,
			column: pos.column as u32,
		}
	}
}

#[inline]
fn line_col(span: Span) -> ((usize, usize), (usize, usize)) {
	let start = span.start();
	let end = span.end();

	((start.line, start.column), (end.line, end.column))
}
#[inline]
fn pos(line: usize, column: usize) -> SourcePosition {
	SourcePosition {
		line: line as u32,
		column: column as u32,
	}
}
fn build_final_analysis(context: &ClickContext, lines: Option<&[LineRelated]>) -> String {
	let mut output = String::new();

	let _ = writeln!(output, "================ CLICK CONTEXT ================");
	let _ = writeln!(output, "FILE   : {}", context.file.display());
	let _ = writeln!(output, "LINE   : {}", context.line);
	let _ = writeln!(output, "COLUMN : {}", context.column);
	let _ = writeln!(output);
	let _ = writeln!(output, "SOURCE:");

	for (idx, source_line) in context.source.lines().enumerate() {
		let line_number = idx + 1;

		let relations = lines
			.and_then(|lines| lines.get(idx))
			.map(|line| &line.relations)
			.map(|relations| relations.as_slice())
			.unwrap_or(&[]);

		let _ = writeln!(
			output,
			"{:>4} | {:<50} // {:?}",
			line_number, source_line, relations,
		);

		if line_number == context.line {
			let prefix = format!("{:>4} | ", line_number);

			let _ = writeln!(
				output,
				"{}{}^ (column {})",
				" ".repeat(prefix.len()),
				" ".repeat(context.column as usize),
				context.column,
			);
		}
	}

	let source_line_count = context.source.lines().count();

	if context.line > source_line_count {
		let _ = writeln!(output);
		let _ = writeln!(
			output,
			"CLICK POSITION OUT OF RANGE: line {}, column {}",
			context.line, context.column,
		);
	}

	let _ = writeln!(output);

	output
}
fn print_node_context(node_context: &NodeContext, kind: LineKind) {
	println!("================ NODE CONTEXT =================");
	if let Some(subject) = &node_context.subject {
		println!(
			"Subject: {:?} {:?} @ {}:{}",
			subject.kind,
			kind,
			subject.span.start().line,
			subject.span.start().column,
		);
	} else {
		println!("SUBJECT: None");
	}
	println!("Line: {:?}", kind);
	println!("Expansion:");
	for (i, node) in node_context.ancestors.iter().enumerate() {
		let start = node.span.start();
		let end = node.span.end();
		println!(
			"{:>2}. {:?} [{}:{} -> {}:{}]",
			i, node.kind, start.line, start.column, end.line, end.column,
		);
	}
	println!("===============================================");
}

struct OwnershipAnalysis {
	pub related_lines: Vec<LineRelated>,
	pub click: ClickContext,
	pub node_context: NodeContext,
	pub classification: NodeClassification,
	pub symbols: Vec<LineSymbol>,
}
#[derive(Serialize)]
pub struct AnalysisReport {
	pub click: ClickContext,
	pub analysis: AnalysisData,
	pub formatted_output: String,
}
#[derive(Serialize)]
pub struct AnalysisData {
	pub related_lines: Vec<LineRelated>,
	pub node_context: NodeContext,
	pub classification: NodeClassification,
	pub symbols: Vec<LineSymbol>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct NodeId(pub usize);
#[derive(Clone, Debug)]
pub struct Graph {
	pub nodes: Vec<GraphNode>,
	pub edges: Vec<Edge>,
}
impl Graph {
	pub fn new() -> Self {
		Self {
			nodes: Vec::new(),
			edges: Vec::new(),
		}
	}

	pub fn add_node(&mut self, node: GraphNode) -> NodeId {
		let id = node.id;
		self.nodes.push(node);
		id
	}

	pub fn add_relation(
		&mut self,
		from: NodeId,
		to: NodeId,
		relation: RelationKind,
		influence: InfluenceKind,
		ownership: Option<OwnershipRelation>,
	) {
		self.edges.push(Edge {
			from,
			to,
			relation,
			influence,
			ownership,
		});
	}

	pub fn edges_from(&self, node: &NodeId) -> Vec<&Edge> {
		self
			.edges
			.iter()
			.filter(|edge| &edge.from == node)
			.collect()
	}

	pub fn edges_to(&self, node: NodeId) -> Vec<&Edge> {
		self.edges.iter().filter(|edge| edge.to == node).collect()
	}

	pub fn filter<F>(&self, predicate: F) -> Vec<&GraphNode>
	where
		F: Fn(&GraphNode) -> bool,
	{
		self.nodes.iter().filter(|node| predicate(node)).collect()
	}
	// fn new_node(&mut self) -> NodeId {
	// 	let id = NodeId(self.next_id);
	// 	self.next_id += 1;
	// 	id
	// }

	pub fn node(&self, id: NodeId) -> Option<&GraphNode> {
		self.nodes.iter().find(|node| node.id == id)
	}

	pub fn has_node(&self, id: NodeId) -> bool {
		self.nodes.iter().any(|node| node.id == id)
	}
}
#[derive(Clone, Debug)]
pub struct GraphNode {
	pub id: NodeId,
	pub name: String,
	pub kind: AstNodeKind,
	pub span: proc_macro2::Span,
}
#[derive(Clone, Debug)]
pub struct Edge {
	pub from: NodeId,
	pub to: NodeId,
	pub relation: RelationKind,
	pub influence: InfluenceKind,
	pub ownership: Option<OwnershipRelation>,
}
// #[derive(Clone, Debug)]
// pub struct GraphEdge {
// 	pub from: NodeId,
// 	pub to: NodeId,
// 	pub relation: RelationKind,
// 	pub influence: InfluenceKind,
// }
pub struct ScopeKind;
pub struct ScopeContext {
	pub kind: ScopeKind,
	pub span: proc_macro2::Span,
	pub owner: Option<ResolvedNode>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationKind {
	Call,
	Downstream,
	Reference,
	// Structural
	Contains,
	DefinedIn,
	DeclaredIn,
	// Ownership
	Owns,
	OwnedBy,
	MovesTo,
	MovedFrom,
	// Borrowing
	BorrowOf,
	BorrowedBy,
	MutableBorrowOf,
	ImmutableBorrowOf,
	// Aliasing
	AliasOf,
	CloneOf,
	// Data flow
	AssignedFrom,
	DerivedFrom,
	DependsOn,
	// Scope / visibility
	VisibleIn,
	CapturedBy,
	// Usage
	References,
	Calls,

	// Add:
	PassedAsArgument,
	ReturnedFrom,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum InfluenceKind {
	Direct,
	Declaration,
	Assignment,
	Borrow,
	Move,
	Mutation,
	Alias,
}
#[derive(Debug)]
pub struct VariableMetadata {
	pub name: String,
	pub mutable: bool,
	pub ty: Option<String>,
}
#[derive(Debug, Serialize)]
pub enum NodeClassification {
	Variable,
	BinaryExpression,
	Scope,
	FunctionCall,
	Literal,
	VariableDeclaration,
	Assignment,
	Borrow,
	Condition,
	Unknown,
}
#[derive(Debug, Clone)]
pub struct SymInfo {
	pub id: usize,        // Unique symbol ID
	pub name: String,     // e.g., "bar"
	pub defined_at: Span, // Where it was born
}

enum Occurrence {
	First,
	Last,
}
fn position_of(source: &str, needle: &str) -> (usize, usize) {
	for (line, text) in source.lines().enumerate() {
		if let Some(column) = text.find(needle) {
			return (line + 1, column);
		}
	}

	panic!("could not find {}", needle);
}
fn positions_of(source: &str, needle: &str) -> Vec<(usize, usize)> {
	let mut result = Vec::new();
	for (line, text) in source.lines().enumerate() {
		let mut offset = 0;
		while let Some(column) = text[offset..].find(needle) {
			let absolute = offset + column;
			result.push((line + 1, absolute));
			offset = absolute + needle.len();
		}
	}
	result
}
fn position_of_last(source: &str, needle: &str) -> (usize, usize) {
	let mut found = None;
	for (line, text) in source.lines().enumerate() {
		if let Some(column) = text.rfind(needle) {
			found = Some((line + 1, column));
		}
	}
	found.expect("could not find occurrence")
}
#[cfg(test)]
mod tests {
	use super::*;
	fn test_source(source: &str) -> String {
		source.strip_prefix('\n').unwrap_or(source).to_string()
	}
	fn parse_test_source(source: &str) -> syn::File {
		syn::parse_file(source.trim_start()).unwrap()
	}
	fn source_line(source: &str, contains: &str) -> usize {
		source
			.lines()
			.position(|line| line.contains(contains))
			.unwrap()
			+ 1
	}
	fn position_of_after(source: &str, needle: &str, start_line: usize) -> (usize, usize) {
		for (line, text) in source.lines().enumerate().skip(start_line - 1) {
			if let Some(column) = text.find(needle) {
				return (line + 1, column);
			}
		}
		panic!("could not find {} after line {}", needle, start_line);
	}
	macro_rules! assert_subject {
		($context:expr, $kind:expr) => {{
			let subject = $context.subject.as_ref().expect("expected subject");

			assert_eq!(subject.kind, $kind);
		}};
	}
	macro_rules! assert_local {
		($context:expr, mutable = $expected:expr) => {{
			let local = $context
				.ancestors
				.iter()
				.find(|node| node.kind == AstNodeKind::Local)
				.expect("expected Local ancestor");
			assert_eq!(local.mutable, $expected);
		}};
	}
	// fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
	// 	println!("MACRO {:?}", node.mac.path);
	// 	syn::visit::visit_expr_macro(self, node);
	// }
	fn analyze_click_on(source: &str, name: &str, occurrence: Occurrence) -> Vec<LineRelated> {
		let (line, column) = match occurrence {
			Occurrence::First => position_of(source, name),
			Occurrence::Last => position_of_last(source, name),
		};

		let syntax_tree = syn::parse_file(source).unwrap();

		let options = AnalyzerOptions {
			line: Some(line as u32),
			column: Some(column as u32),
			..AnalyzerOptions::default()
		};

		Workspace::analyze_ownership(&PathBuf::from("test.rs"), source, &syntax_tree, &options)
			.expect("ownership analysis failed")
			.analysis
			.related_lines
	}
	fn assert_line(related: &[LineRelated], line: usize, relation: OwnershipRelation) {
		let entry = related
			.iter()
			.find(|x| x.line == line)
			.unwrap_or_else(|| panic!("missing line {}", line));
		assert!(
			entry.relations.contains(&relation),
			"line {} missing {:?}, got {:?}",
			line,
			relation,
			entry.relations
		);
	}
	fn assert_lines(related: &[LineRelated], expected: Vec<(usize, OwnershipRelation)>) {
		for (line, relation) in expected {
			let found = related
				.iter()
				.find(|entry| entry.line == line)
				.unwrap_or_else(|| panic!("missing line {}", line));

			assert!(
				found.relations.contains(&relation),
				"line {} missing {:?}, got {:?}",
				line,
				relation,
				found.relations
			);
		}
	}
	fn assert_subject_name(context: &NodeContext, expected: &str) {
		let subject = context.subject.as_ref().expect("expected subject");

		assert_eq!(subject.name.as_deref(), Some(expected));
	}

	#[test]
	fn resolves_variable_usage_name() {
		let source = r#"
	fn main() {
	let item = "1";
	let other = item;
	}
	"#;
		let context = Workspace::resolve_click_on(source, "item", Occurrence::Last);
		assert_subject!(&context, AstNodeKind::Path);
		assert_subject_name(&context, "item");
	}
	#[test]
	fn resolves_variable_declaration() {
		let source = r#"
	fn main() {
    let item = "1";
	}
	"#;

		let (line, column) = position_of(source, "item");

		let context = Workspace::resolve_click(source, line, column);

		let subject = context.subject.expect("expected subject");

		assert_eq!(subject.kind, AstNodeKind::PatternIdentifier);

		let local = context
			.ancestors
			.iter()
			.find(|x| x.kind == AstNodeKind::Local)
			.expect("expected Local ancestor");

		assert_eq!(local.mutable, Some(false));
	}
	#[test]
	fn resolves_variable_usage_as_path() {
		let source = r#"
	fn main() {
    let item = "1";
    let other = item;
	}
	"#;
		let positions = positions_of(source, "item");

		let (line, column) = positions[1];

		let context = Workspace::resolve_click(source, line, column);

		let subject = context.subject.expect("expected subject");

		assert_eq!(subject.kind, AstNodeKind::Path);
	}
	#[test]
	fn resolves_mutable_variable_declaration() {
		let source = r#"
	fn main() {
    let mut item = "1";
	}
	"#;

		let (line, column) = position_of(source, "item");

		let context = Workspace::resolve_click(source, line, column);

		let local = context
			.ancestors
			.iter()
			.find(|x| x.kind == AstNodeKind::Local)
			.expect("expected Local ancestor");

		assert_eq!(local.mutable, Some(true));
	}
	#[test]
	fn does_not_match_other_variables() {
		let source = r#"
	fn main() {
    let item = "1";
    let other = "2";
	}
	"#;

		let (line, column) = position_of(source, "other");

		let context = Workspace::resolve_click(source, line, column);

		let subject = context.subject.expect("expected subject");

		assert_eq!(subject.kind, AstNodeKind::PatternIdentifier);
	}
	#[test]
	fn click_immutable_variable_declaration() {
		let source = r#"
	fn main() {
	let item = "hello";
	}
	"#;
		let context = Workspace::resolve_click_on(source, "item", Occurrence::First);
		assert_subject!(&context, AstNodeKind::PatternIdentifier);
		assert_subject_name(&context, "item");
		assert_local!(context, mutable = Some(false));
	}
	#[test]
	fn click_mutable_variable_declaration() {
		let source = r#"
	fn main() {
    let mut item = 1;
	}
	"#;

		let context = Workspace::resolve_click_on(source, "item", Occurrence::First);

		assert_local!(context, mutable = Some(true));
	}
	#[test]
	fn click_variable_usage() {
		let source = r#"
	fn main() {
    let item = "1";
    let x = item;
	}
	"#;

		let (line, column) = position_of_last(source, "item");

		println!("CLICK {}:{}", line, column);

		let context = Workspace::resolve_click(source, line, column);

		println!("CLICK {}:{} on {}", line, column, "item");

		println!("{:#?}", context);

		let subject = context.subject.expect("expected subject");

		assert_eq!(subject.kind, AstNodeKind::Path);
	}
	#[test]
	fn click_variable_tracks_all_references() {
		let source = r#"fn main() {
	let item = "hello";
	let other = item;
	let third = item;
	}
	"#;

		let analysis = analyze_click_on(&test_source(source), "item", Occurrence::First);

		assert_line(&analysis, 2, OwnershipRelation::Declaration);
		assert_line(&analysis, 3, OwnershipRelation::Reference);
		assert_line(&analysis, 4, OwnershipRelation::Reference);
	}
	#[test]
	fn detects_move() {
		let source = r#"
	fn main() {
	let item = String::new();
	let other = item;
	}
	"#;

		let analysis = analyze_click_on(source, "item", Occurrence::First);
	}
	#[test]
	fn detects_borrow() {
		let source = r#"
	fn main() {
	let item = String::new();
	let other = &item;
	}
	"#;
	}
	#[test]
	fn handles_shadowed_variables() {
		let source = r#"
	fn main() {
	let item = 1;
	{
    let item = 2;
    println!("{}", item);
	}
	println!("{}", item);
	}
	"#;
		let analysis = analyze_click_on(source, "item", Occurrence::First);
	}
	#[test]
	fn detects_explicit_type() {
		let source = r#"
	fn main() {
    let item: i32 = 1;
	}
	"#;
		let node = Workspace::resolve_click_on(source, "item", Occurrence::First);
		// assert_eq!(node.type_name, Some("i32".into()));
	}
	#[test]
	fn does_not_cross_function_boundaries() {
		let source = r#"
	fn foo(item: i32) {
    println!("{}", item);
	}
	fn main() {
    let item = "hello";
    println!("{}", item);
	}
	"#;

		let analysis = analyze_click_on(source, "item", Occurrence::Last);

		assert_line(&analysis, 6, OwnershipRelation::Declaration);
		assert_line(&analysis, 7, OwnershipRelation::Reference);
	}

	#[test]
	fn resolves_local_declaration() {
		let source = r#"
	fn main() {
    let num = 42;
	}
	"#;

		let context = Workspace::resolve_click_on(source, "num", Occurrence::First);

		assert_subject_name(&context, "num");
		assert_subject!(context, AstNodeKind::PatternIdentifier);
	}

	#[test]
	fn resolves_function_call() {
		let source = r#"
	fn foo() {}

	fn main() {
    foo();
	}
	"#;

		let context = Workspace::resolve_click_on(source, "foo", Occurrence::Last);

		assert_subject_name(&context, "foo");
	}

	#[test]
	fn resolves_function_declaration() {
		let source = r#"
	fn foo() {
	}
	"#;

		let context = Workspace::resolve_click_on(source, "foo", Occurrence::First);

		assert_subject_name(&context, "foo");
	}
	#[test]
	fn finds_reference_to_local() {
		let source = r#"
	fn main() {
    let num = 42;
    println!("{}", num);
	}
	"#;

		let related = analyze_click_on(source, "num", Occurrence::First);

		assert_line(&related, 3, OwnershipRelation::Declaration);
		assert_line(&related, 4, OwnershipRelation::Reference);
	}
	#[test]
	fn clicking_reference_finds_declaration() {
		let source = r#"
	fn main() {
    let num = 42;
    println!("{}", num);
	}
	"#;
		let related = analyze_click_on(source, "num", Occurrence::Last);
		assert_line(&related, 3, OwnershipRelation::Declaration);
		assert_line(&related, 4, OwnershipRelation::Reference);
	}
	#[test]
	fn finds_all_references() {
		let source = r#"
	fn main() {
    let num = 42;

    println!("{}", num);
    println!("{}", num);
    println!("{}", num);
	}
	"#;

		let related = analyze_click_on(source, "num", Occurrence::First);

		assert_lines(
			&related,
			vec![
				(3, OwnershipRelation::Declaration),
				(5, OwnershipRelation::Reference),
				(6, OwnershipRelation::Reference),
				(7, OwnershipRelation::Reference),
			],
		);
	}
	#[test]
	fn local_used_as_function_argument() {
		let source = r#"
	fn foo(a: &i32) {
    return a;
	}

	fn main() {
    let num1 = 1;
    let spam1 = foo(num1);

    println!("{}", spam1);
	}
	"#;

		let related = analyze_click_on(source, "num1", Occurrence::First);
		assert_line(&related, 7, OwnershipRelation::Declaration);
		assert_line(&related, 8, OwnershipRelation::Reference);
	}
	#[test]
	fn function_result_flows_into_local() {
		let source = r#"
	fn foo(a: &i32) {
    return a;
	}

	fn main() {
    let num1 = 1;
    let spam1 = foo(num1);

    println!("{}", spam1);
	}
	"#;

		let related = analyze_click_on(source, "spam1", Occurrence::First);

		assert_line(&related, 8, OwnershipRelation::Declaration);
		assert_line(&related, 10, OwnershipRelation::Reference);
	}
	#[test]
	fn separate_locals_do_not_share_references() {
		let source = r#"
	fn foo(a: &i32) {
    return a;
	}

	fn main() {
    let num1 = 1;
    let num2 = 2;

    let spam1 = foo(num1);
    let spam2 = foo(num2);

    println!("{}", spam1);
    println!("{}", spam2);
	}
	"#;

		let related = analyze_click_on(source, "num1", Occurrence::First);

		assert_line(&related, 7, OwnershipRelation::Declaration);
		assert_line(&related, 10, OwnershipRelation::Reference);

		assert!(
			!related.iter().any(|x| x.line == 8),
			"num2 declaration should not be related"
		);

		assert!(
			!related.iter().any(|x| x.line == 11),
			"spam2 usage should not be related"
		);
	}
	#[test]
	fn function_call_is_related_to_function_definition() {
		let source = r#"
	fn foo() {
	}

	fn main() {
    foo();
	}
	"#;

		let related = analyze_click_on(source, "foo", Occurrence::Last);

		assert_line(&related, 2, OwnershipRelation::Definition);
		assert_line(&related, 6, OwnershipRelation::Reference);
	}
	#[test]
	fn argument_is_related_to_parameter() {
		let source = r#"
	fn foo(value: &i32) {
    println!("{}", value);
	}

	fn main() {
    let num = 42;
    foo(&num);
	}
	"#;

		let related = analyze_click_on(source, "num", Occurrence::First);

		assert_line(&related, 7, OwnershipRelation::Declaration);
		assert_line(&related, 8, OwnershipRelation::Reference);
	}
	#[test]
	fn assignment_creates_assignment_relationship() {
		let source = r#"
	fn main() {
    let mut value = 1;
    value = 2;
	}
	"#;

		let related = analyze_click_on(source, "value", Occurrence::First);

		assert_line(&related, 3, OwnershipRelation::Declaration);
		assert_line(&related, 4, OwnershipRelation::Assignment);
	}
	#[test]
	fn mutation_is_distinguished_from_reference() {
		let source = r#"
	fn main() {
    let mut value = 1;
    value += 1;
	}
	"#;

		let related = analyze_click_on(source, "value", Occurrence::First);

		assert_line(&related, 3, OwnershipRelation::Declaration);
		assert_line(&related, 4, OwnershipRelation::Mutation);
	}
	#[test]
	fn follows_downstream_value_flow() {
		let source = r#"
	fn foo(value: &i32) {
	return value;
	}

	fn main() {
	let num1 = 1;
	let spam1 = foo(num1);

	println!("{}", spam1);
	}
	"#;

		let related = analyze_click_on(source, "num1", Occurrence::First);

		assert_lines(
			&related,
			vec![
				(7, OwnershipRelation::Declaration),
				(8, OwnershipRelation::Argument),
				(8, OwnershipRelation::Return),
				(10, OwnershipRelation::Reference),
			],
		);
	}
}
pub struct OwnershipGraph {
	pub graph: Graph,
	pub symbols: Vec<SymReference>,
	pub subject_id: Option<NodeId>,
	pub related_spans: Vec<(proc_macro2::Span, OwnershipRelation)>,
}
impl Workspace {
	// fn resolve_subject(
	// 	options: &AnalyzerOptions,
	// 	syntax_tree: &File,
	// 	line: usize,
	// 	column: usize,
	// ) -> Result<NodeContext, AnalysisError> {
	// 	let mut resolver = NodeResolver {
	// 		next_id: 0,
	// 		line,
	// 		column,
	// 		current_name: None,
	// 		position: pos(line, column),
	// 		candidates: Vec::new(),
	// 		best: None,
	// 		nodes: Vec::new(),
	// 		ancestors: Vec::new(),
	// 	};
	// 	resolver.visit_file(&syntax_tree);
	// 	let node = resolver.resolve();
	// 	Ok(node)
	// }
	// fn resolve_node(
	// 	syntax_tree: &syn::File,
	// 	options: &AnalyzerOptions,
	// ) -> Result<NodeContext, AnalysisError> {
	// 	let line = options
	// 		.line
	// 		.ok_or_else(|| AnalysisError::Parse("Missing line".into()))?;
	// 	let column = options
	// 		.column
	// 		.ok_or_else(|| AnalysisError::Parse("Missing column".into()))?;
	// 	let resolver = NodeResolver {
	// 		next_id: 0,
	// 		line: line as usize,
	// 		column: column as usize,
	// 		current_name: None,
	// 		position: pos(line as usize, column as usize),
	// 		candidates: Vec::new(),
	// 		best: None,
	// 		nodes: Vec::new(),
	// 		ancestors: Vec::new(),
	// 	};
	// 	Ok(resolver.resolve_file(syntax_tree))
	// }
	fn print_click_report(report: &ClickReport) {
		Self::print_click_analysis(
			&report.context,
			&report.symbols,
			report.node_context.as_ref(),
		);
	}
	fn print_click_analysis(
		context: &ClickContext,
		symbols: &[LineSymbol],
		node_context: Option<&NodeContext>,
	) {
		println!("================ CLICK ANALYSIS ================");
		println!("FILE   : {}", context.file.display());
		println!("LINE   : {}", context.line);
		println!("COLUMN : {}", context.column);
		println!();
		println!("SOURCE:");
		for (idx, source_line) in context.source.lines().enumerate() {
			let line_number = idx + 1;
			let labels: Vec<_> = symbols
				.iter()
				.filter(|s| s.line == line_number)
				.map(|s| format!("{}:{:?}", s.name, s.role))
				.collect();
			match labels.is_empty() {
				true => {
					println!("{:>4} | {}", line_number, source_line);
				}
				false => {
					println!(
						"{:>4} | {:<50} // {}",
						line_number,
						source_line,
						labels.join(", ")
					);
				}
			}
			if line_number == context.line {
				println!(
					"{:>width$}^ column {}",
					"",
					context.column,
					width = context.column as usize + 8
				);
			}
		}
		// if let Some(node_context) = &report.node_context {
		//     println!();
		//     print_node_context(
		//         node_context,
		//         LineKind::Cursor,
		//     );
		// }
	}
	fn roadmap() {
		// To nail the compiler boundaries for an interactive click-to-analyze tool (like a language server feature), you want to separate **Phase 1: Query Extraction** (point-in-time lookup) from **Phase 2: Whole-File Semantic Mapping** (line-by-line analysis).

		// Since you are working with `syn::File` (standard Rust AST structures), you can implement `collect_lines` by using a **visitor pattern** or a recursive span-matching pass that scans the syntax tree once and projects the AST nodes onto their respective source lines.

		// Here is how you can structure the boundary and implement `collect_lines`:

		// ### 1. The Compiler Pipeline Boundary

		// * **Pipeline A (Targeted Query):** Click coordinates $\rightarrow$ `resolve_node` $\rightarrow$ Find the exact AST node and its path/ancestors. (Fast, localized, bottom-up).
		// * **Pipeline B (Whole-File Render/Analysis):** Source string + AST $\rightarrow$ `collect_lines` $\rightarrow$ Line-by-line symbol mapping (Top-down AST walk mapping spans to line numbers).
		// * **Pipeline C (The Link):** Compare Pipeline A's resolved subject against Pipeline B's line symbols to answer "does this line affect my click?"

		// 1. Click
		//    - file
		//    - line
		//    - column
		//    - cursor position
		// 2. Find relevant lines
		//    LineRelated[]
		//    Example:
		//    line 2 -> Scope
		//    line 3 -> Scope
		//    line 9 -> Scope
		// 3. Populate symbols inside those lines
		//    Line 3:
		//       symbol: bar
		//       role: Declaration
		//    Line 9:
		//       symbol: bar
		//       role: Reference
		// 4. Compare symbols against clicked subject
		//    Subject:
		//       bar
		//    Line 3:
		//       bar declaration
		//       affects subject: true
		//    Line 2:
		//       foo declaration
		//       affects subject: false
		//
		// 5. Add flags
		//    LineAnalysis {
		//        line: 3,
		//        symbols: [
		//            {
		//              name: "bar",
		//              role: "declaration",
		//              relation_to_subject: "defines"
		//            }
		//        ],
		//        flags: {
		//            in_scope: true,
		//            influences_subject: true,
		//            unrelated: false
		//        }
		//    }
		// 6. Render different views
		//    Decorations:
		//       - highlight influence lines
		//       - grey unrelated lines
		//    Inlay hints:
		//       - show symbol roles
		//    CodeLens:
		//       - show actions
		//    Webview:
		//       - show graph
		// Click
		//  |
		// Resolve AST node
		//  |
		// Collect ancestors
		//  |
		// Classify node
		//  |
		// Build influence edges
		//  |
		// Traverse outward
		//  |
		// Highlight affected lines
	}
	// fn analyze_click(
	// 	ast: &syn::File,
	// 	context: ClickContext,
	// 	symbols: Vec<LineSymbol>,
	// ) -> ClickReport {
	// 	let node_context = Some(Self::resolve_node_context(
	// 		ast,
	// 		context.line,
	// 		context.column,
	// 	));
	// 	ClickReport {
	// 		context,
	// 		symbols,
	// 		node_context,
	// 	}
	// }
	// fn find_scope_at_position(
	// 	syntax_tree: &syn::File,
	// 	options: &AnalyzerOptions,
	// ) -> Option<proc_macro2::Span> {
	// 	let mut visitor = ScopeVisitor {
	// 		target_line: options.line.unwrap_or(0),
	// 		target_column: options.column.unwrap_or(0),
	// 		scopes: Vec::new(),
	// 	};
	// 	visitor.visit_file(syntax_tree);
	// 	// Return the smallest scope containing the cursor
	// 	visitor.scopes.into_iter().min_by_key(|span| {
	// 		let size = span.end().line - span.start().line;
	// 		size
	// 	})
	// }
	// fn is_value_flow(relation: &OwnershipRelation) -> bool {
	// 	matches!(
	// 		relation,
	// 		OwnershipRelation::Reference
	// 			| OwnershipRelation::Assignment
	// 			| OwnershipRelation::Argument
	// 			| OwnershipRelation::Return
	// 			| OwnershipRelation::Mutation
	// 			| OwnershipRelation::MoveOwnership
	// 	)
	// }
	// fn build_upstream(graph: &Graph, subject: &ResolvedNode) {
	// 	let subject_id = subject.id;

	// 	for edge in graph.edges_to(subject_id) {
	// 		println!(
	// 			"UPSTREAM: {:?} -> {:?} ({:?})",
	// 			edge.from, edge.to, edge.relation
	// 		);
	// 	}
	// }
}
