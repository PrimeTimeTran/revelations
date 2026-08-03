use crate::{_scope::Scope, analyzer::ownership::ownership::SymReference, ir::Sym};


pub mod ownership;

struct CodeMetrics;
struct CodeAnalyzer {
    // Symbol table & scope stack (from your second snippet)
    symbols: Vec<Sym>,
    scopes: Vec<Scope>,
    
    // Ownership & tracking state (from your first snippet)
    related_symbols: Vec<SymReference>,
    
    // Metrics collection state
    metrics: CodeMetrics,
    
    // Feature toggles if you want to run specific passes optionally
    // config: AnalyzerConfig,
}

impl<'ast> syn::visit::Visit<'ast> for CodeAnalyzer {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        // 1. Symbol Table logic
        self.add_symbol(Sym {
            name: node.ident.to_string(),
            kind: SymbolKind::Type(TypeKind::Struct),
            // ...
        });
        
        // 2. Metrics logic (e.g., count fields)
        self.metrics.struct_count += 1;
        
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        // Ownership / reference tracking logic
        if let Some(segment) = node.path.segments.last() {
            let name = segment.ident.to_string();
            let resolved_id = self.resolve_symbol(&name);
            self.related_symbols.push(SymReference {
                name,
                span: node.span(),
                role: SymRole::Reference,
                resolved_id: Some(resolved_id),
            });
        }
        syn::visit::visit_expr_path(self, node);
    }
}