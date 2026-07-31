use crate::{analyzer::*, ir::*};
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
    visit::{self, Visit, visit_local},
    visit_mut::{self, VisitMut},
};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OwnershipRelation {
    Scope,
    Declaration,
    ImmutableBorrow,
    Reference,
    MutableBorrow,
    MoveOwnership,
    Assignment,
}
#[derive(Debug, Clone)]
pub struct OwnershipVisitor {
    pub subject: String,
    pub options: AnalyzerOptions,
    pub related_spans: Vec<proc_macro2::Span>,
    pub scope_spans: Vec<proc_macro2::Span>,
    pub related_symbols: Vec<SymReference>,
}
// pub struct ResolvedSymbol {
//     pub name: String,
//     pub declaration: Span,
//     pub references: Vec<Span>,
//     pub scope: Span,
// }
// - Subject: Identifier clicked on
// - Candidates: - Potential subjects if one was not clicked on. For example they clicked 'end of line' or a key word like 'impl'.
// - Ancestor: An identfiier which influences the ownership of this subject.
// - Descendents: Identifies who are influenced by this subject
// - Scope:

impl<'ast> syn::visit::Visit<'ast> for OwnershipVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let name = node.sig.ident.to_string();

        if name == self.subject {
            self.scope_spans.push(node.span());
        }

        syn::visit::visit_item_fn(self, node);
    }
    fn visit_block(&mut self, node: &'ast syn::Block) {
        let span = node.span();

        // If your subject was found inside this block,
        // keep this as a possible parent scope.
        self.scope_spans.push(span);

        syn::visit::visit_block(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        let span = node.span();

        // Phase 1: if we haven't resolved a subject yet,
        // check if this AST node contains the cursor.
        if self.subject.is_empty()
            && span_contains_position(
                span,
                self.options.line.unwrap_or(0),
                self.options.column.unwrap_or(0),
            )
        {
            if let Some(segment) = node.path.segments.last() {
                let name = segment.ident.to_string();

                eprintln!("RESOLVED SUBJECT: {} at {:?}", name, span);

                self.subject = name;
            }
        }

        // Phase 2: once we have a subject, collect references.
        if let Some(segment) = node.path.segments.last() {
            if segment.ident.to_string() == self.subject {
                self.related_spans.push(span);
                self.related_symbols.push(SymReference {
                    name: segment.ident.to_string(),
                    span,
                    role: SymRole::Reference,
                });
            }
        }

        visit::visit_expr_path(self, node);
    }
    fn visit_local(&mut self, node: &'ast syn::Local) {
        if let Some(ident) = extract_ident(&node.pat) {
            self.related_symbols.push(SymReference {
                name: ident.to_string(),
                span: ident.span(),
                role: SymRole::Declaration,
            });
        }

        visit_local(self, node);
    }
}
fn span_contains_position(span: proc_macro2::Span, line: u32, column: u32) -> bool {
    let start = span.start();
    let end = span.end();

    if line < start.line as u32 || line > end.line as u32 {
        return false;
    }

    if line == start.line as u32 && column < start.column as u32 {
        return false;
    }

    if line == end.line as u32 && column > end.column as u32 {
        return false;
    }

    true
}
pub struct OwnershipAnalysisResult {
    pub click: ClickContext,
    pub subject: Option<SymId>,
    pub scope: Option<ScopeInfo>,
    pub lines: Vec<LineAnalysis>,
    pub symbols: Vec<SymReference>,
    pub relations: Vec<SymRelation>,
}
impl Workspace {
    // pub fn analyze_ownership_on_click(
    //     file_path: &PathBuf,
    //     options: &AnalyzerOptions,
    // ) -> Result<OwnershipAnalysisResult, AnalysisError> {

    //     let source = load_source(file_path)?;

    //     let click = ClickContext::new(
    //         file_path,
    //         &source,
    //         options,
    //     );

    //     let ast = parse_source(&source)?;

    //     let subject = resolve_subject(
    //         &ast,
    //         &source,
    //         &click,
    //     )?;

    //     let scope = find_scope(
    //         &ast,
    //         &click,
    //     );

    //     let mut lines = collect_lines(
    //         &source,
    //         &scope,
    //     );

    //     let symbols = collect_symbols(
    //         &ast,
    //         &lines,
    //     );

    //     let relations = analyze_relationships(
    //         &subject,
    //         &symbols,
    //     );

    //     classify_lines(
    //         &mut lines,
    //         &subject,
    //         &relations,
    //     );

    //     Ok(build_analysis_result(
    //         click,
    //         subject,
    //         scope,
    //         lines,
    //         symbols,
    //         relations,
    //     ))
    // }
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
    fn resolve_node_context(
        syntax_tree: &syn::File,
        options: &AnalyzerOptions,
    ) -> NodeContext {
        let mut resolver = NodeResolver {
            line: options.line.unwrap_or(0),
            column: options.column.unwrap_or(0),
            nodes: Vec::new(),
            candidates: Vec::new(),
        };
    
        resolver.visit_file(syntax_tree);
    
        let mut candidates = resolver.candidates;
    
        candidates.sort_by_key(|node| {
            let start = node.span.start();
            let end = node.span.end();
            (
                end.line - start.line,
                end.column - start.column,
            )
        });
        let subject = candidates.first().cloned();
        NodeContext {
            subject,
            ancestors: candidates,
        }
    }
    pub fn resolve_subject(options: &AnalyzerOptions, syntax_tree: &File) {
        let mut resolver = NodeResolver {
            line: options.line.unwrap(),
            column: options.column.unwrap(),
            candidates: Vec::new(),
            nodes: Vec::new(),
        };
        resolver.visit_file(&syntax_tree);
        let node = resolver.resolve();
    }
    pub fn analyze_ownership_on_click(
        file_path: &PathBuf,
        options: &AnalyzerOptions,
    ) -> Result<Vec<LineRelated>, AnalysisError> {
        let source =
            std::fs::read_to_string(file_path).map_err(|e| AnalysisError::Parse(e.to_string()))?;
        let click = ClickContext::new(file_path, &source, options);
        preflight(options, &source);
        let syntax_tree = Self::parse_source(&source)?;
        let kind = classify_line(
            &syntax_tree,
            options.line.unwrap()
        );
        println!(
            "LINE KIND: {:?}",
            kind
        );
        let context = Self::resolve_node_context(
            &syntax_tree,
            options,
        );
        println!("CLICK AST:");
        let node_context = Self::resolve_node_context(
            &syntax_tree,
            options,
        );
        for node in &context.ancestors {
            println!("  {:?}", node);
        }
        let resolved = resolve_node_at_position(
            &syntax_tree,
            &source,
            options,
        );
        let item = Self::resolve_subject(options, &syntax_tree);
        let (subject, node_context) = match resolved {
            Ok(value) => value,
            Err(err) => {
                eprintln!(
                    "[RESOLVER FAILED] {:?}",
                    err
                );
                print_final_analysis(
                    &click,
                    None,
                );
                return Ok(Vec::new());
            }
        };
        let mut workspace = Workspace::new();
        let file_id = workspace.add_symbol(
            Sym::file(
                0,
                file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            ),
            Some(workspace.root),
        );
        workspace.files.push(file_id);
        let mut ownership_visitor = OwnershipVisitor {
            subject,
            scope_spans: Vec::new(),
            options: options.clone(),
            related_spans: Vec::new(),
            related_symbols: Vec::new(),
        };
        ownership_visitor.visit_file(&syntax_tree);

        // 5. Map the collected spans back to exact line numbers using the source text
        // let mut related_lines: Vec<LineRelated> = ownership_visitor
        //     .related_spans
        //     .iter()
        //     .map(|span| LineRelated {
        //         line: span.start().line,
        //         file_path: file_path.clone(),
        //         relation_type: OwnershipRelation::ImmutableBorrow,
        //     })
        //     .collect();

        // if let Some(scope_span) = ownership_visitor.scope_spans.first() {
        //     related_lines.push(LineRelated {
        //         line: scope_span.start().line,
        //         file_path: file_path.clone(),
        //         relation_type: OwnershipRelation::Scope,
        //     });
        // }
        let scope = Self::find_scope_at_position(&syntax_tree, options);
        let mut related_lines: Vec<LineRelated> = Vec::new();

        fn add_relation(
            lines: &mut Vec<LineRelated>,
            line: usize,
            file_path: &PathBuf,
            relation: OwnershipRelation,
        ) {
            if let Some(existing) = lines.iter_mut().find(|x| x.line == line) {
                existing.relations.push(relation);
            } else {
                lines.push(LineRelated {
                    line,
                    file_path: file_path.clone(),
                    relations: vec![relation],
                });
            }
        }

        // 1. Add scope lines
        if let Some(scope_span) = Self::find_scope_at_position(&syntax_tree, options) {
            for line in scope_span.start().line..=scope_span.end().line {
                add_relation(
                    &mut related_lines,
                    line,
                    file_path,
                    OwnershipRelation::Scope,
                );
            }
        }

        // 2. Add symbol influence/reference lines
        for span in ownership_visitor.related_spans {
            add_relation(
                &mut related_lines,
                span.start().line,
                file_path,
                OwnershipRelation::Reference,
            );
        }
        let analysis = related_lines;
        print_final_analysis(&click, None);
        Ok(analysis)
    }
    pub fn find_scope_at_position(
        syntax_tree: &syn::File,
        options: &AnalyzerOptions,
    ) -> Option<proc_macro2::Span> {
        let mut visitor = ScopeVisitor {
            target_line: options.line.unwrap_or(0),
            target_column: options.column.unwrap_or(0),
            scopes: Vec::new(),
        };

        visitor.visit_file(syntax_tree);

        // Return the smallest scope containing the cursor
        visitor.scopes.into_iter().min_by_key(|span| {
            let size = span.end().line - span.start().line;
            size
        })
    }

    fn parse_source(source: &str) -> Result<syn::File, AnalysisError> {
        syn::parse_file(source)
            .map_err(|e| AnalysisError::Parse(e.to_string()))
    }
    fn find_scope(syntax_tree: &syn::File, click: &ClickContext) -> Option<ScopeInfo> {
        todo!("find scope")
    }
    fn collect_lines(source: &str, scope: &ScopeInfo) -> Vec<LineAnalysis> {
        todo!("collect_lines")
    }
    fn analyze_relationships(
        subject: &ResolvedSubject,
        symbols: &[SymReference],
    ) -> Vec<SymRelation> {
        todo!("analyze_relationships")
    }
    fn classify_lines(
        lines: &mut [LineAnalysis],
        subject: &ResolvedSubject,
        relations: &[SymRelation],
    ) {
        todo!("classify_lines")
    }
    fn build_analysis_result(
        click: ClickContext,
        subject: Option<ResolvedSubject>,
        scope: Option<ScopeInfo>,
        lines: Vec<LineAnalysis>,
        symbols: Vec<SymReference>,
        relations: Vec<SymRelation>,
    ) -> OwnershipAnalysisResult {
        todo!("build_analysis_result")
    }
}
fn add_related_line(
    lines: &mut Vec<LineRelated>,
    line: usize,
    file_path: &PathBuf,
    relation: OwnershipRelation,
) {
    if let Some(existing) = lines.iter_mut().find(|x| x.line == line) {
        existing.relations.push(relation);
    } else {
        lines.push(LineRelated {
            line,
            file_path: file_path.clone(),
            relations: vec![relation],
        });
    }
}
fn extract_ident(pat: &syn::Pat) -> Option<&syn::Ident> {
    match pat {
        syn::Pat::Ident(pat_ident) => Some(&pat_ident.ident),

        syn::Pat::Type(pat_type) => extract_ident(&pat_type.pat),

        syn::Pat::Reference(pat_ref) => extract_ident(&pat_ref.pat),

        syn::Pat::Paren(pat_paren) => extract_ident(&pat_paren.pat),

        syn::Pat::TupleStruct(tuple_struct) => tuple_struct.elems.iter().find_map(extract_ident),

        syn::Pat::Tuple(tuple) => tuple.elems.iter().find_map(extract_ident),

        _ => None,
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymRelation {
    /// Source symbol
    pub from: SymId,
    /// Target symbol
    pub to: SymId,
    /// Relationship between them
    pub kind: RelationKind,
    /// Optional explanation for debugging/UI
    pub label: Option<String>,
}
#[derive(Debug, Clone)]
pub struct SymReference {
    pub name: String,
    pub role: SymRole,
    pub span: Span,
}
pub struct SymNode {
    pub id: SymId,
    pub name: String,
    pub role: SymRole,
    pub location: SymLocation,
}
#[derive(Debug, Clone)]
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
pub struct SymLocation;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationKind {
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
}

#[derive(Debug, Clone)]
pub enum InfluenceKind {
    Declaration,
    Assignment,
    Borrow,
    Move,
    Mutation,
    Alias,
}
// fn print_final_analysis(
//     context: &ClickContext,
//     node_context: Option<&NodeContext>,
// ) {
//     println!("================ ANALYSIS =================");
//     println!(
//         "FILE: {}",
//         context.file.display()
//     );
//     println!(
//         "CLICK: {}:{}",
//         context.line,
//         context.column
//     );
//     println!();

//     println!("{:<40} NODE", "SOURCE");
//     println!("{:-<60}", "");

//     for (idx, line) in context.source.lines().enumerate() {
//         let line_no = idx + 1;

//         let marker =
//             if line_no == context.line {
//                 " <--"
//             } else {
//                 ""
//             };

//         println!(
//             "{:>3} | {:<40} {}",
//             line_no,
//             line,
//             marker
//         );
//     }

//     println!();

//     println!("SUBJECT:");

//     match &node_context.unwrap().subject {
//         Some(node) => {
//             println!(
//                 "  {:?}",
//                 node.kind
//             );
//         }
//         None => {
//             println!("  None");
//         }
//     }

//     println!();

//     println!("CONTEXT:");

//     for node in &node_context.unwrap().ancestors {
//         println!(
//             "  └─ {:?}",
//             node.kind
//         );
//     }

//     println!("============================================");
// }
fn print_final_analysis(context: &ClickContext, symbols: Option<&[LineSymbol]>) {
    println!("================ CLICK CONTEXT ================");
    println!("FILE   : {}", context.file.display());
    println!("LINE   : {}", context.line);
    println!("COLUMN : {}", context.column);
    println!();

    println!("SOURCE:");

    let symbols = symbols.unwrap_or(&[]);

    for (idx, source_line) in context.source.lines().enumerate() {
        let line_number = idx + 1;

        let labels: Vec<String> = symbols
            .iter()
            .filter(|s| s.line == line_number)
            .map(|s| format!("{}:{:?}", s.name, s.role))
            .collect();

        if labels.is_empty() {
            println!("{:>4} | {}", line_number, source_line);
        } else {
            println!(
                "{:>4} | {:<50} // {}",
                line_number,
                source_line,
                labels.join(", ")
            );
        }

        // Always show cursor position
        if line_number == context.line {
            let prefix = format!("{:>4} | ", line_number);
            println!(
                "{}{}^ (column {})",
                " ".repeat(prefix.len()),
                " ".repeat(context.column as usize),
                context.column
            );
        }
    }

    // In case the click line is outside the source range
    if context.line > context.source.lines().count() {
        println!();
        println!(
            "CLICK POSITION OUT OF RANGE: line {}, column {}",
            context.line, context.column
        );
    }
    println!("===============================================");
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
            i,
            node.kind,
            start.line,
            start.column,
            end.line,
            end.column,
        );
    }

    println!("===============================================");
}

// fn span_contains_position(
//     span: Span,
//     line: u32,
//     column: u32,
// ) -> bool {
//     let start = span.start();
//     let end = span.end();

//     if line < start.line as u32 || line > end.line as u32 {
//         return false;
//     }

//     if line == start.line as u32 && column < start.column as u32 {
//         return false;
//     }

//     if line == end.line as u32 && column > end.column as u32 {
//         return false;
//     }

//     true
// }



pub struct ResolvedSubject {
    pub name: String,
    pub kind: NodeContext,
    pub span: Span,
}
#[derive(Default)]
pub struct ScopeVisitor {
    pub target_line: u32,
    pub target_column: u32,
    pub scopes: Vec<proc_macro2::Span>,
}
pub struct SymbolVisitor {
    pub subject: String,
    pub nodes: Vec<SymNode>,
    pub relations: Vec<SymRelation>,
    pub scopes: Vec<Span>,
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
impl ScopeVisitor {
    fn check_scope(&mut self, span: proc_macro2::Span) {
        let start = span.start();
        let end = span.end();

        let inside =
            (self.target_line >= start.line as u32) && (self.target_line <= end.line as u32);

        if inside {
            self.scopes.push(span);
        }
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
pub struct LineAnalysis {
    pub line: usize,
    pub text: String,
    pub symbols: Vec<SymReference>,
    pub flags: LineFlags,
}
pub struct LineAnnotation {
    pub line: usize,
    pub text: String,
}
pub struct LineSymbol {
    pub line: usize,
    pub name: String,
    pub role: SymRole,
}
#[derive(Debug, Clone)]
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
}
#[derive(Debug)]
pub enum StatementKind {
    
}
#[derive(Debug)]
pub enum ExpressionKind {
    
}
#[derive(Debug)]
pub enum ItemKind {
    
}
pub struct LineContext {
    pub kind: LineKind,
    pub span: Span,
}
pub fn classify_line(
    syntax_tree: &syn::File,
    line: u32,
) -> LineKind {
    let mut visitor = LineClassifier {
        line,
        result: None,
    };

    visitor.visit_file(syntax_tree);

    visitor.result.unwrap_or(LineKind::Unknown)
}


struct LineClassifier {
    line: u32,
    result: Option<LineKind>,
}


impl<'ast> syn::visit::Visit<'ast> for LineClassifier {

    fn visit_local(&mut self, node: &'ast syn::Local) {

        if span_contains_line(
            node.span(),
            self.line,
        ) {
            self.result = Some(LineKind::LetBinding);
        }

        syn::visit::visit_local(self, node);
    }


    fn visit_expr(&mut self, node: &'ast syn::Expr) {

        if span_contains_line(
            node.span(),
            self.line,
        ) {
            self.result = Some(
                LineKind::ExpressionStatement
            );
        }

        syn::visit::visit_expr(self, node);
    }


    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if span_contains_line(
            node.span(),
            self.line,
        ) {
            self.result = Some(LineKind::Function);
        }

        syn::visit::visit_item_fn(self, node);
    }
}
fn span_contains_line(
    span: proc_macro2::Span,
    line: u32,
) -> bool {
    let start = span.start().line as u32;
    let end = span.end().line as u32;

    line >= start && line <= end
}
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
    pub fn print_cursor(&self) {
        let line_idx = self.line.saturating_sub(1) as usize;
        let Some(source_line) = self.source.lines().nth(line_idx) else {
            return;
        };
        println!("SOURCE:");
        println!("{:>4} | {}", self.line, source_line);
        let padding = " ".repeat(self.column as usize);
        println!(
            "     | {}^ (column {}, char {:?})",
            padding,
            self.column,
            source_line.chars().nth(self.column as usize)
        );
    }
}
#[derive(Debug, Clone)]
pub struct AstNode {
    pub kind: AstNodeKind,
    pub span: proc_macro2::Span,
    pub name: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstNodeKind {
    // Identifiers / symbols
    Identifier,
    Binding,

    // Values
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
#[derive(Debug, Clone)]
pub struct ResolvedNode {
    pub kind: AstNodeKind,
    pub span: proc_macro2::Span,
}
pub struct NodeResolver {
    pub line: u32,
    pub column: u32,
    pub candidates: Vec<ResolvedNode>,
    pub nodes: Vec<AstNodeKind>,
}
impl NodeResolver {
    fn contains(&self, span: proc_macro2::Span) -> bool {
        let start = span.start();
        let end = span.end();

        self.line >= start.line as u32
            && self.line <= end.line as u32
    }
}
impl NodeResolver {
    fn check(&mut self, kind: AstNodeKind, span: proc_macro2::Span) {
        if span_contains_position(span, self.line, self.column) {
            self.candidates.push(ResolvedNode {
                kind,
                span,
            });
        }
    }
    pub fn resolve(mut self) -> NodeContext {
        self.candidates.sort_by_key(|node| {
            let start = node.span.start();
            let end = node.span.end();
            (
                end.line - start.line,
                end.column - start.column
            )
        });
        let subject = self.candidates.first().cloned();
        NodeContext {
            subject,
            ancestors: self.candidates,
        }
    }
}
impl<'ast> syn::visit::Visit<'ast> for NodeResolver {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if self.contains(node.span()) {
            self.nodes.push(AstNodeKind::Function);
        }

        syn::visit::visit_item_fn(self, node);
    }
    fn visit_block(&mut self, node: &'ast syn::Block) {
        if self.contains(node.span()) {
            self.nodes.push(AstNodeKind::Block);
        }

        syn::visit::visit_block(self, node);
    }
    fn visit_local(&mut self, node: &'ast syn::Local) {
        if self.contains(node.span()) {
            self.nodes.push(AstNodeKind::Local);
        }

        syn::visit::visit_local(self, node);
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
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if self.contains(node.span()) {
            self.nodes.push(AstNodeKind::CallExpr);
        }

        syn::visit::visit_expr_call(self, node);
    }
    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        self.check(
            AstNodeKind::Path,
            node.span(),
        );
    
        syn::visit::visit_expr_path(self, node);
    }
    fn visit_expr_lit(&mut self, node: &'ast syn::ExprLit) {
        if self.contains(node.span()) {
            self.nodes.push(AstNodeKind::Literal);
        }

        syn::visit::visit_expr_lit(self, node);
    }
    // fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
    //     self.check(
    //         AstNodeKind::PathExpr,
    //         node.span(),
    //     );

    //     syn::visit::visit_expr_path(self, node);
    // }
    // fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
    //     self.check(
    //         AstNodeKind::BinaryExpr,
    //         node.span(),
    //     );

    //     syn::visit::visit_expr_binary(self, node);
    // }
    // fn visit_expr_lit(&mut self, node: &'ast syn::ExprLit) {
    //     self.check(
    //         AstNodeKind::Literal,
    //         node.span(),
    //     );

    //     syn::visit::visit_expr_lit(self, node);
    // }
    // fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
    //     self.check(
    //         AstNodeKind::IfExpression,
    //         node.span(),
    //     );

    //     syn::visit::visit_expr_if(self, node);
    // }
    // fn visit_local(&mut self, node: &'ast syn::Local) {
    //     self.check(
    //         AstNodeKind::LetStatement,
    //         node.span(),
    //     );

    //     syn::visit::visit_local(self, node);
    // }
}
#[derive(Debug, Clone)]
pub struct NodeContext {
    pub subject: Option<ResolvedNode>,
    pub ancestors: Vec<ResolvedNode>,
}
impl NodeContext {
    pub fn print_tree(&self) {
        for node in &self.ancestors {
            let start = node.span.start();

            println!(
                "{:?} @ {}:{}",
                node.kind,
                start.line,
                start.column
            );
        }
    }
}
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


pub fn preflight(options: &AnalyzerOptions, source: &str) {
    if let Some(line) = options.line {
        let idx = line.saturating_sub(1) as usize;
        eprintln!("CLICK:");
        eprintln!("  line   : {}", line);
        eprintln!("  column : {:?}", options.column);
        if let Some(source_line) = source.lines().nth(idx) {
            eprintln!("  source : {}", source_line);
        }
        if let Some(column) = options.column {
            if let Some(source_line) = source.lines().nth(idx) {
                let ch = source_line.chars().nth(column as usize);
                eprintln!("  char   : {:?}", ch);
            }
        }
    }
}
