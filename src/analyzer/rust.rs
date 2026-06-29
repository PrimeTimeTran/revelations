use std::collections::HashMap;

use crate::{
    analyzer::{
        AnalysisError,
        r#trait::{Analyzer, AnalyzerOptions},
    },
    ir::{FunctionKind, Symbol, SymbolKind, TypeKind, Visibility},
};
use quote::ToTokens;
use syn::visit::{self, Visit};

pub struct RustAnalyzer;

impl Analyzer for RustAnalyzer {
    fn analyze(
        &self,
        source: &str,
        options: &AnalyzerOptions,
    ) -> Result<Vec<Symbol>, AnalysisError> {
        let ast = syn::parse_file(source).map_err(|e| AnalysisError::Parse(e.to_string()))?;
        let symbols: HashMap<String, Symbol> = HashMap::new();
        let mut visitor = RustVisitor {
            options,
            current_impl: None,
            scope_stack: Vec::new(),
            symbols,
            impl_count: 0,
        };
        visitor.visit_file(&ast);
        let mut result: Vec<Symbol> = visitor.symbols.into_values().collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(result)
    }
}

struct RustVisitor<'a> {
    options: &'a AnalyzerOptions,
    scope_stack: Vec<String>,
    current_impl: Option<Symbol>,
    impl_count: usize,
    symbols: std::collections::HashMap<String, Symbol>,
}

impl<'ast> Visit<'ast> for RustVisitor<'_> {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.add_type(item.ident.to_string(), TypeKind::Struct, &item.vis);
        visit::visit_item_struct(self, item);
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        use std::io::{self, Write};
        self.impl_count += 1;
        println!(
            "Found impl #{}: {}",
            self.impl_count,
            item.self_ty.to_token_stream()
        );
        println!(
            "Found impl #{}: {}",
            self.impl_count,
            item.self_ty.to_token_stream()
        );
        io::stdout().flush().unwrap();
        let target = item.self_ty.to_token_stream().to_string();

        self.symbols.entry(target.clone()).or_insert(Symbol {
            name: target.clone(),
            kind: SymbolKind::Type(TypeKind::Struct),
            visibility: Visibility::Public,
            params: None,
            return_type: None,
            children: Vec::new(),
        });
        self.scope_stack.push(target);
        visit::visit_item_impl(self, item);
        self.scope_stack.pop();
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        self.add_type(item.ident.to_string(), TypeKind::Enum, &item.vis);
        visit::visit_item_enum(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let (method_symbol, _) = create_method_symbol(item.clone());
        if let Some(parent_key) = self.scope_stack.last() {
            if let Some(parent_symbol) = self.symbols.get_mut(parent_key) {
                parent_symbol.children.push(method_symbol);
            }
        } else {
            self.symbols
                .insert(method_symbol.name.clone(), method_symbol);
        }

        visit::visit_impl_item_fn(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        self.add_type(item.ident.to_string(), TypeKind::Trait, &item.vis);
        for trait_item in &item.items {
            match trait_item {
                syn::TraitItem::Fn(m) => {
                    let params = Some(
                        m.sig
                            .inputs
                            .iter()
                            .map(|arg| match arg {
                                syn::FnArg::Typed(pat) => (
                                    pat.pat.to_token_stream().to_string(),
                                    pat.ty.to_token_stream().to_string(),
                                ),
                                syn::FnArg::Receiver(rec) => {
                                    ("self".to_string(), rec.to_token_stream().to_string())
                                }
                            })
                            .collect(),
                    );

                    self.symbols.insert(
                        m.sig.ident.to_string(),
                        Symbol {
                            name: m.sig.ident.to_string(),
                            kind: SymbolKind::Function(FunctionKind::TraitMethod),
                            visibility: visibility(&item.vis),
                            params,
                            children: Vec::new(),
                            return_type: Some(m.sig.output.to_token_stream().to_string()),
                        },
                    );
                }
                syn::TraitItem::Type(t) => {
                    // Handle associated types if needed
                    // self.add_associated_type(t.ident.to_string());
                }
                _ => {}
            }
        }

        visit::visit_item_trait(self, item);
    }
}

impl RustVisitor<'_> {
    fn add_type(&mut self, name: String, kind: TypeKind, vis: &syn::Visibility) {
        let visibility = visibility(vis);

        if self.options.include_private || matches!(visibility, Visibility::Public) {
            self.symbols.insert(
                name.clone(),
                Symbol {
                    name,
                    kind: SymbolKind::Type(kind),
                    visibility,
                    params: None,
                    children: Vec::new(),
                    return_type: None,
                },
            );
        }
    }
    fn add_impl(&mut self, target: String, kind: TypeKind, trait_name: Option<String>) {
        let name = trait_name.as_deref().unwrap_or(&target);
        self.symbols.insert(
            name.to_string(),
            Symbol {
                kind: SymbolKind::Implementation {
                    target_type: target.clone(),
                    trait_name: trait_name.clone(),
                },
                name: trait_name.map_or(target.clone(), |t| format!("{} for {}", t, target)),
                visibility: Visibility::Public,
                params: None,
                return_type: None,
                children: Vec::new(),
            },
        );
    }
}

fn visibility(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility::Public,

        _ => Visibility::Private,
    }
}

fn create_method_symbol(item: syn::ImplItemFn) -> (Symbol, Option<Vec<(String, String)>>) {
    let params = Some(
        item.sig
            .inputs
            .iter()
            .map(|arg| match arg {
                syn::FnArg::Typed(pat) => (
                    pat.pat.to_token_stream().to_string(),
                    pat.ty.to_token_stream().to_string(),
                ),
                syn::FnArg::Receiver(rec) => {
                    ("self".to_string(), rec.to_token_stream().to_string())
                }
            })
            .collect(),
    );
    let method_symbol = Symbol {
        name: item.sig.ident.to_string(),
        kind: SymbolKind::Function(FunctionKind::Method),
        visibility: visibility(&item.vis),
        params: params.clone(),
        return_type: Some(item.sig.output.to_token_stream().to_string()),
        children: Vec::new(),
    };
    (method_symbol, params)
}
