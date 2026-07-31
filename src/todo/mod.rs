// -----------------------------------------------------------------------------
// syn::ext::IdentExt
//
// Extension methods for `syn::Ident`, primarily for parsing Rust keywords
// as identifiers (e.g. `r#type`) and other identifier-related utilities.
use syn::ext::IdentExt;

// -----------------------------------------------------------------------------
// syn::fold::Fold
//
// Traverses an AST while producing a modified copy.
// Best for functional-style transformations without mutating the original tree.
use syn::fold::Fold;

// -----------------------------------------------------------------------------
// syn::parse::Parse
//
// Implement on your type to define how it is parsed from a token stream.
// Used by `syn::parse_str`, procedural macros, and custom syntax.
use syn::parse::Parse;

// -----------------------------------------------------------------------------
// syn::parse::Parser
//
// A parsing function/object that can parse arbitrary syntax.
// Useful for invoking custom parsers outside of a `Parse` implementation.
use syn::parse::Parser;

// -----------------------------------------------------------------------------
// syn::parse::Peek
//
// Trait used internally to efficiently test whether the next token matches
// a specific syntax element without consuming it.
use syn::parse::Peek;

// -----------------------------------------------------------------------------
// syn::parse::discouraged::AnyDelimiter
//
// Matches any delimiter (`()`, `[]`, `{}`) during parsing.
// Rarely needed; intended for advanced parsing scenarios.
use syn::parse::discouraged::AnyDelimiter;

// -----------------------------------------------------------------------------
// syn::parse::discouraged::Speculative
//
// Allows tentative parsing with rollback if parsing fails.
// Useful for ambiguous grammars where multiple parses are possible.
use syn::parse::discouraged::Speculative;

// -----------------------------------------------------------------------------
// syn::spanned::Spanned
//
// Provides access to the source-code span for an AST node.
// Essential for diagnostics, errors, and source location tracking.
use syn::spanned::Spanned;

// -----------------------------------------------------------------------------
// syn::token::Token
//
// Helper macro/type for matching and parsing Rust punctuation and keywords.
// Commonly used in custom parsers.
use syn::token::Token;

// -----------------------------------------------------------------------------
// syn::visit::Visit
//
// Read-only traversal of an AST.
// Ideal for semantic analysis, indexing, and collecting information.
use syn::visit::Visit;

// -----------------------------------------------------------------------------
// syn::visit_mut::VisitMut
//
// Mutable traversal of an AST.
// Used for refactoring, rewriting, and transforming syntax in place.
use syn::visit_mut::VisitMut;

// use syn::ext::IdentExt;
// use syn::fold::Fold;
// use syn::parse::Parse;
// use syn::parse::Parser;
// use syn::parse::Peek;
// use syn::parse::discouraged::AnyDelimiter;
// use syn::parse::discouraged::Speculative;
// use syn::spanned::Spanned;
// use syn::token::Token;
// use syn::visit::Visit;
// use syn::visit_mut::VisitMut;

use syn::{
    Ident, Result, Token,
    // ext::IdentExt,
    // fold::Fold,
    // parse::discouraged::{AnyDelimiter, Speculative},
    // parse::{Parse, Parser, Peek},
    // spanned::Spanned,
    // token::Token,
};

// ============================================================================
// IdentExt
//
// Context:
// - Used when working with identifiers.
// - Useful for IDE features like rename, symbol indexing, keyword handling.
//
// Setup:
// - Requires a syn::Ident.
//
// Example:
//   `r#type` is a legal Rust identifier even though `type` is a keyword.
//
struct IdentPlayground;

impl IdentPlayground {
    fn inspect_ident(&self, ident: &Ident) {
        println!("identifier = {}", ident.unraw());
    }
}

// ============================================================================
// Fold
//
// Context:
// - AST transformation that returns a NEW tree.
// - Think: "map over the syntax tree".
//
// Useful for:
// - Refactoring
// - Code generation
// - Automated rewrites
//
// Unlike VisitMut:
// - Fold consumes nodes and gives back replacements.
//
struct FoldPlayground;

impl Fold for FoldPlayground {
    fn fold_ident(&mut self, ident: Ident) -> Ident {
        println!("folding ident: {}", ident);
        ident
    }
}

// ============================================================================
// Parse
//
// Context:
// - Lets you define how your own custom syntax is read.
//
// Useful for:
// - Procedural macros
// - Custom DSLs
// - Parsing your own language extensions
//
// Example syntax:
//
//   thing foo bar
//
// becomes:
//   MySyntax { name: foo }
//
struct MySyntax {
    name: Ident,
}

impl Parse for MySyntax {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
        })
    }
}

// ============================================================================
// Parser
//
// Context:
// - A parser is something that can parse tokens.
//
// Useful for:
// - Running custom parsers manually.
// - Parsing fragments instead of entire types.
//
// Usually consumed more than implemented.
//
struct ParserPlayground;

impl ParserPlayground {
    fn parse_ident(&self) -> Result<Ident> {
        let parser = |input: syn::parse::ParseStream| input.parse::<Ident>();

        parser.parse_str("hello")
    }
}

// ============================================================================
// Peek
//
// Context:
// - Look ahead without consuming tokens.
//
// Useful for:
// - Ambiguous grammars.
// - Deciding what parser branch to take.
//
// Usually used indirectly through ParseStream::peek().
//
struct PeekPlayground;

impl PeekPlayground {
    fn check_keyword(&self, input: syn::parse::ParseStream) -> bool {
        input.peek(Token![fn])
    }
}

// ============================================================================
// AnyDelimiter
//
// Context:
// - Advanced parser helper.
//
// Useful for:
// - Parsing arbitrary (), [], {} blocks.
//
// Most projects will rarely touch this.
//
struct DelimiterPlayground;

// ============================================================================
// Speculative
//
// Context:
// - Allows trying a parse and rolling back.
//
// Useful for:
// - Ambiguous syntax.
//
// Example:
//
//   foo(bar)
//
// Could be:
//   function call
//
// or:
//   macro syntax
//
// Try one, rollback, try another.
//
struct SpeculativePlayground;

impl SpeculativePlayground {
    fn try_parse(&self, input: syn::parse::ParseStream) -> Result<()> {
        let fork = input.fork();

        // Try parsing on fork.
        let _maybe_ident: Ident = fork.parse()?;

        // If successful, advance original stream.
        input.advance_to(&fork);

        Ok(())
    }
}

// ============================================================================
// Spanned
//
// Context:
// - Gives source locations.
//
// Extremely useful for:
// - Diagnostics
// - LSP errors
// - Highlighting
//
// Example:
//   "unknown symbol foo"
//   underline exactly `foo`
//
struct SpanPlayground;

impl SpanPlayground {
    fn show_span(&self, node: &impl Spanned) {
        let span = node.span();

        println!("start={:?}, end={:?}", span.start(), span.end());
    }
}

// ============================================================================
// Token
//
// Context:
// - Token types represent Rust punctuation/keywords.
//
// Usually you do not implement Token.
// You use it.
//
// Example:
//
// input.parse::<Token![fn]>()
//
struct TokenPlayground;

impl TokenPlayground {
    fn expects_fn(&self, input: syn::parse::ParseStream) -> Result<()> {
        let _token: Token![fn] = input.parse()?;
        Ok(())
    }
}
