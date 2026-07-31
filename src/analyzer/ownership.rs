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
    visit::{self, Visit, visit_expr_path, visit_local},
    visit_mut::{self, VisitMut},
};

// 1. Click
//    - file
//    - line
//    - column
//    - cursor position
// 2. Find relevant lines
//    RelatedLine[]
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
    pub related_symbols: Vec<SymOccurrence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedLine {
    pub line: usize,
    pub file_path: PathBuf,
    pub relations: Vec<OwnershipRelation>,
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
pub struct SymbolVisitor {
    pub subject: String,
    pub nodes: Vec<SymNode>,
    pub relations: Vec<SymRelation>,
    pub scopes: Vec<Span>,
}
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
        if let Some(segment) = node.path.segments.last() {
            // Compare the Ident's string representation with our target String
            if segment.ident.to_string() == self.subject {
                self.related_spans.push(node.span());
            }
        }
        visit_expr_path(self, node);
    }
    fn visit_local(&mut self, node: &'ast syn::Local) {
        if let Some(ident) = extract_ident(&node.pat) {
            self.related_symbols.push(SymOccurrence {
                name: ident.to_string(),
                span: ident.span(),
                role: SymRole::Declaration,
            });
        }

        visit_local(self, node);
    }
}
pub struct OwnershipAnalysisResult {
    pub click: ClickContext,
    pub subject: Option<SymId>,
    pub scope: Option<ScopeInfo>,
    pub lines: Vec<AnalyzedLine>,
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
    pub fn analyze_ownership_on_click(
        file_path: &PathBuf,
        options: &AnalyzerOptions,
    ) -> Result<Vec<RelatedLine>, AnalysisError> {
        let source =
            std::fs::read_to_string(file_path).map_err(|e| AnalysisError::Parse(e.to_string()))?;
        let click = ClickContext::new(file_path, &source, options);
        if let Some(line) = options.line {
            eprintln!("VS CODE LINE: {}", line);
            let idx = line as usize;
            if let Some(source_line) = source.lines().nth(idx) {
                eprintln!("ZERO BASED SOURCE: {}", source_line);
            }
            if let Some(source_line) = source.lines().nth(idx.saturating_sub(1)) {
                eprintln!("ONE BASED SOURCE: {}", source_line);
            }
        }
        if let Some(column) = options.column {
            eprintln!("Column: {}", column);
        }
        let syntax_tree =
            syn::parse_file(&source).map_err(|e| AnalysisError::Parse(e.to_string()))?;
        let (subject, node_context) = resolve_target_at_position(&syntax_tree, &source, options)?;
        let mut workspace = Workspace::new();
        let file_id = workspace.add_symbol(
            Symbol::file(
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
        // let mut related_lines: Vec<RelatedLine> = ownership_visitor
        //     .related_spans
        //     .iter()
        //     .map(|span| RelatedLine {
        //         line: span.start().line,
        //         file_path: file_path.clone(),
        //         relation_type: OwnershipRelation::ImmutableBorrow,
        //     })
        //     .collect();

        // if let Some(scope_span) = ownership_visitor.scope_spans.first() {
        //     related_lines.push(RelatedLine {
        //         line: scope_span.start().line,
        //         file_path: file_path.clone(),
        //         relation_type: OwnershipRelation::Scope,
        //     });
        // }
        let scope = Self::find_scope_at_position(&syntax_tree, options);
        let mut related_lines: Vec<RelatedLine> = Vec::new();

        fn add_relation(
            lines: &mut Vec<RelatedLine>,
            line: usize,
            file_path: &PathBuf,
            relation: OwnershipRelation,
        ) {
            if let Some(existing) = lines.iter_mut().find(|x| x.line == line) {
                existing.relations.push(relation);
            } else {
                lines.push(RelatedLine {
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
        print_click_analysis(&click, &analysis.symbols);

        Ok(related_lines)
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
        todo!("parse_source")
    }
    fn resolve_subject(
        syntax_tree: &syn::File,
        source: &str,
        click: &ClickContext,
    ) -> Result<Option<ResolvedSubject>, AnalysisError> {
        todo!("resolve_subject")
    }
    fn find_scope(syntax_tree: &syn::File, click: &ClickContext) -> Option<ScopeInfo> {
        todo!("find scope")
    }
    fn collect_lines(source: &str, scope: &ScopeInfo) -> Vec<AnalyzedLine> {
        todo!("collect_lines")
    }
    fn analyze_relationships(
        subject: &ResolvedSubject,
        symbols: &[SymReference],
    ) -> Vec<SymRelation> {
        todo!("analyze_relationships")
    }
    fn classify_lines(
        lines: &mut [AnalyzedLine],
        subject: &ResolvedSubject,
        relations: &[SymRelation],
    ) {
        todo!("classify_lines")
    }

    fn build_analysis_result(
        click: ClickContext,
        subject: Option<ResolvedSubject>,
        scope: Option<ScopeInfo>,
        lines: Vec<AnalyzedLine>,
        symbols: Vec<SymReference>,
        relations: Vec<SymRelation>,
    ) -> OwnershipAnalysisResult {
        todo!("build_analysis_result")
    }
}

#[derive(Default)]
pub struct ScopeVisitor {
    pub target_line: u32,
    pub target_column: u32,
    pub scopes: Vec<proc_macro2::Span>,
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

fn add_related_line(
    lines: &mut Vec<RelatedLine>,
    line: usize,
    file_path: &PathBuf,
    relation: OwnershipRelation,
) {
    if let Some(existing) = lines.iter_mut().find(|x| x.line == line) {
        existing.relations.push(relation);
    } else {
        lines.push(RelatedLine {
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
pub struct LineAnnotation {
    pub line: usize,
    pub text: String,
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
pub struct SymOccurrence {
    pub name: String,
    pub span: proc_macro2::Span,
    pub role: SymRole,
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
    pub location: SourceLocation,
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
struct SourceLocation;
pub struct ClickContext {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub source: String,
}

pub struct ScopeLine {
    pub line: usize,
    pub span: Span,
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

fn print_click_analysis(context: &ClickContext, symbols: &[LineSymbol]) {
    println!("================ CLICK CONTEXT ================");
    println!("FILE   : {}", context.file.display());
    println!("LINE   : {}", context.line);
    println!("COLUMN : {}", context.column);

    println!();
    println!("SOURCE:");

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

        if line_number == context.line {
            println!("     | {}^", " ".repeat(context.column));
        }
    }

    println!("===============================================");
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
}

pub struct ResolvedSubject {
    pub name: String,
    pub kind: NodeContext,
    pub span: Span,
}

pub struct ScopeInfo {
    pub start_line: usize,
    pub end_line: usize,
    pub span: Span,
}

pub struct AnalyzedLine {
    pub line: usize,
    pub text: String,
    pub symbols: Vec<SymReference>,
    pub flags: LineFlags,
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