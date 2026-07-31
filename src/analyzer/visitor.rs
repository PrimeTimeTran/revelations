use crate::{analyzer::*, ir::*};
use quote::ToTokens;
use regex_syntax::ast::Ast;
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

pub struct RustVisitor<'a> {
    options: &'a AnalyzerOptions,
    workspace: &'a mut Workspace,
    scope_stack: Vec<SymId>,
    source: &'a str,
    file: SymId,
    current_impl: Option<SymId>,
}
impl<'a> RustVisitor<'a> {
    pub fn new(
        options: &'a AnalyzerOptions,
        workspace: &'a mut Workspace,
        file: SymId,
        source: &'a str,
    ) -> Self {
        Self {
            options,
            workspace,
            file,
            source,
            scope_stack: vec![file],
            current_impl: None,
        }
    }
    fn location(&self, span: proc_macro2::Span) -> SymLocation {
        SymLocation {
            file: self.file,
            start: span.start().line,
            end: span.end().line,
        }
    }
    fn current_scope(&self) -> SymId {
        *self
            .scope_stack
            .last()
            .expect("visitor has no active scope")
    }
    fn add_symbol(&mut self, symbol: Sym) -> SymId {
        let owner = self.current_scope();
        self.workspace.add_symbol(symbol, Some(owner))
    }
    fn push_scope(&mut self, id: SymId) {
        self.scope_stack.push(id);
    }
    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }
}
impl<'ast> Visit<'ast> for RustVisitor<'_> {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.add_symbol(Sym {
            id: 0,
            scope: 0,
            owner: 0,
            name: node.ident.to_string(),
            location: Some(self.location(node.span())),
            kind: SymbolKind::Type(TypeKind::Struct),
            visibility: Visibility::Private,
            // params: None,
            // return_type: None,
            children: Vec::new(),
        });
        visit::visit_item_struct(self, node);
    }
    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.add_symbol(Sym {
            id: 0,
            scope: 0,
            owner: 0,
            name: node.ident.to_string(),
            location: Some(self.location(node.span())),
            kind: SymbolKind::Type(TypeKind::Trait),
            visibility: match &node.vis {
                syn::Visibility::Public(_) => Visibility::Public,
                _ => Visibility::Private,
            },
            // params: None,
            // return_type: None,
            children: Vec::new(),
        });
        visit::visit_item_trait(self, node);
    }
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.add_symbol(Sym {
            id: 0,
            scope: 0,
            owner: 0,
            location: Some(self.location(node.span())),
            name: node.sig.ident.to_string(),
            kind: SymbolKind::Function(FunctionKind::Method),
            visibility: match &node.vis {
                syn::Visibility::Public(_) => Visibility::Public,
                _ => Visibility::Private,
            },
            // params: None,
            // return_type: Some(node.sig.output.to_token_stream().to_string()),
            children: Vec::new(),
        });
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.add_symbol(Sym {
            id: 0,
            scope: 0,
            owner: 0,
            name: node.sig.ident.to_string(),
            location: Some(self.location(node.span())),
            kind: SymbolKind::Function(FunctionKind::Free),
            visibility: match &node.vis {
                syn::Visibility::Public(_) => Visibility::Public,
                _ => Visibility::Private,
            },
            // params: None,
            // return_type: Some(node.sig.output.to_token_stream().to_string()),
            children: Vec::new(),
        });

        visit::visit_item_fn(self, node);
    }
    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.add_symbol(Sym {
            id: 0,
            scope: 0,
            owner: 0,
            location: Some(self.location(node.span())),
            name: node.ident.to_string(),
            kind: SymbolKind::Type(TypeKind::Enum),
            visibility: visibility(&node.vis),
            // params: None,
            // return_type: None,
            children: Vec::new(),
        });
        visit::visit_item_enum(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        let name = node.to_token_stream().to_string();
        self.add_symbol(Sym {
            id: 0,
            scope: 0,
            owner: 0,
            name,
            location: Some(self.location(node.span())),
            kind: SymbolKind::Import(ModuleKind::Dependency),
            visibility: Visibility::Private,
            // params: None,
            // return_type: None,
            children: Vec::new(),
        });
        visit::visit_item_use(self, node);
    }
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let name = node.self_ty.to_token_stream().to_string();
        let impl_id = self.add_symbol(Sym {
            id: 0,
            scope: 0,
            owner: 0,
            name: format!("impl {}", name),
            location: Some(self.location(node.span())),
            kind: SymbolKind::Implementation {
                target_type: name.clone(),
                trait_name: node
                    .trait_
                    .as_ref()
                    .map(|(path, _)| path.to_token_stream().to_string()),
            },
            visibility: Visibility::Private,
            // params: None,
            // return_type: None,
            children: Vec::new(),
        });
        self.current_impl = Some(impl_id);
        self.scope_stack.push(impl_id);
        visit::visit_item_impl(self, node);
        self.scope_stack.pop();
        self.current_impl = None;
    }
}

// impl<'ast> Visit<'ast> for RustVisitor<'_> {
//     fn visit_bound_lifetimes(&mut self, i: &'ast syn::BoundLifetimes) {}
//     fn visit_impl_item(&mut self, i: &'ast syn::ImplItem) {}
//     fn visit_impl_item_const(&mut self, i: &'ast syn::ImplItemConst) {}
//     fn visit_impl_item_macro(&mut self, i: &'ast syn::ImplItemMacro) {}
//     fn visit_impl_item_type(&mut self, i: &'ast syn::ImplItemType) {}
//     fn visit_type_impl_trait(&mut self, i: &'ast syn::TypeImplTrait) {}
//     fn visit_local(&mut self, i: &'ast syn::Local) {}
//     fn visit_local_init(&mut self, i: &'ast syn::LocalInit) {}
//     fn visit_where_clause(&mut self, i: &'ast syn::WhereClause) {}
//     fn visit_where_clause_placement(&mut self, i: &'ast syn::WhereClausePlacement) {}
//     fn visit_where_predicate(&mut self, i: &'ast syn::WherePredicate) {}
//     fn visit_file(&mut self, node: &'ast syn::File) {
//         visit::visit_file(self, node);
//     }
//     fn visit_use_name(&mut self, i: &'ast syn::UseName) {}
//     fn visit_use_tree(&mut self, i: &'ast syn::UseTree) {}
//     fn visit_use_glob(&mut self, i: &'ast syn::UseGlob) {}
//     fn visit_use_group(&mut self, i: &'ast syn::UseGroup) {}
//     fn visit_use_path(&mut self, i: &'ast syn::UsePath) {}
//     fn visit_use_rename(&mut self, i: &'ast syn::UseRename) {}
//     fn visit_signature(&mut self, i: &'ast syn::Signature) {}
//     fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
//         todo!("type")
//     }

//     fn visit_item_extern_crate(&mut self, node: &'ast syn::ItemExternCrate) {
//         todo!("type")
//     }
//     fn visit_item_foreign_mod(&mut self, node: &'ast syn::ItemForeignMod) {
//         todo!("type")
//     }

//     fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
//         todo!("type")
//     }
//     fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
//         todo!("mod")
//     }
//     fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
//         todo!("type")
//     }
//     fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
//         todo!("type")
//     }
//     fn visit_item_union(&mut self, node: &'ast syn::ItemUnion) {
//         todo!("visit_item_union")
//     }

//     fn visit_lifetime_param(&mut self, i: &'ast syn::LifetimeParam) {}
//     fn visit_lifetime(&mut self, i: &'ast syn::Lifetime) {}
//     fn visit_macro(&mut self, node: &'ast syn::Macro) {
//         todo!("visit_macro")
//     }
//     fn visit_abi(&mut self, i: &'ast syn::Abi) {}
//     fn visit_block(&mut self, i: &'ast syn::Block) {}
//     fn visit_ident(&mut self, i: &'ast proc_macro2::Ident) {}
//     fn visit_path(&mut self, i: &'ast syn::Path) {}
//     fn visit_trait_bound(&mut self, i: &'ast syn::TraitBound) {
//         todo!("visit_trait_bound")
//     }
//     fn visit_trait_item_const(&mut self, i: &'ast syn::TraitItemConst) {
//         todo!("visit_trait_item_const")
//     }
//     fn visit_trait_item_fn(&mut self, i: &'ast syn::TraitItemFn) {
//         todo!("visit_trait_item_fn")
//     }
//     fn visit_trait_item_macro(&mut self, i: &'ast syn::TraitItemMacro) {
//         todo!("visit_trait_item_macro")
//     }
//     fn visit_trait_item_type(&mut self, i: &'ast syn::TraitItemType) {
//         todo!("visit_trait_item_type")
//     }
//     fn visit_trait_item(&mut self, i: &'ast syn::TraitItem) {
//         todo!("visit_trait_item")
//     }
//     fn visit_visibility(&mut self, i: &'ast syn::Visibility) {}
//     fn visit_data_enum(&mut self, i: &'ast syn::DataEnum) {}
//     fn visit_data_struct(&mut self, i: &'ast syn::DataStruct) {}
//     fn visit_data_union(&mut self, i: &'ast syn::DataUnion) {}
//     fn visit_data(&mut self, i: &'ast syn::Data) {}
// }

pub struct RustVisitMutator {}
impl RustVisitMutator {
    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        if ident == "foo" {
            *ident = Ident::new("bar", ident.span());
        }
    }
}
impl VisitMut for RustVisitMutator {
    fn visit_pat_mut(&mut self, i: &mut syn::Pat) {}
    fn visit_attributes_mut(&mut self, i: &mut Vec<syn::Attribute>) {}
    fn visit_attribute_mut(&mut self, i: &mut syn::Attribute) {}
    fn visit_return_type_mut(&mut self, i: &mut syn::ReturnType) {}
    fn visit_expr_closure_mut(&mut self, i: &mut syn::ExprClosure) {}
    fn visit_expr_call_mut(&mut self, i: &mut syn::ExprCall) {}
    fn visit_named_arg_mut(&mut self, i: &mut syn::NamedArg) {}
    fn visit_block_mut(&mut self, i: &mut syn::Block) {}
    fn visit_bound_lifetimes_mut(&mut self, i: &mut syn::BoundLifetimes) {}
    fn visit_data_enum_mut(&mut self, i: &mut syn::DataEnum) {}
    fn visit_data_mut(&mut self, i: &mut syn::Data) {}
    fn visit_data_struct_mut(&mut self, i: &mut syn::DataStruct) {}
    fn visit_data_union_mut(&mut self, i: &mut syn::DataUnion) {}
    fn visit_expr_assign_mut(&mut self, i: &mut syn::ExprAssign) {}
    fn visit_expr_block_mut(&mut self, i: &mut syn::ExprBlock) {}
    fn visit_expr_field_mut(&mut self, i: &mut syn::ExprField) {}
    fn visit_expr_group_mut(&mut self, i: &mut syn::ExprGroup) {}
    fn visit_expr_if_mut(&mut self, i: &mut syn::ExprIf) {}
    fn visit_expr_index_mut(&mut self, i: &mut syn::ExprIndex) {}
    fn visit_expr_mut(&mut self, i: &mut syn::Expr) {}
    fn visit_field_mut(&mut self, i: &mut syn::Field) {}
    fn visit_field_pat_mut(&mut self, i: &mut syn::FieldPat) {}
    fn visit_field_value_mut(&mut self, i: &mut syn::FieldValue) {}
    fn visit_fields_mut(&mut self, i: &mut syn::Fields) {}
    fn visit_fields_named_mut(&mut self, i: &mut syn::FieldsNamed) {}
    fn visit_fields_unnamed_mut(&mut self, i: &mut syn::FieldsUnnamed) {}
    fn visit_file_mut(&mut self, node: &mut syn::File) {}
    fn visit_generic_argument_mut(&mut self, i: &mut syn::GenericArgument) {}
    fn visit_generic_param_mut(&mut self, i: &mut syn::GenericParam) {}
    fn visit_generics_mut(&mut self, i: &mut syn::Generics) {}
    fn visit_ident_mut(&mut self, i: &mut proc_macro2::Ident) {}
    fn visit_impl_item_const_mut(&mut self, i: &mut syn::ImplItemConst) {}
    fn visit_impl_item_fn_mut(&mut self, i: &mut syn::ImplItemFn) {}
    fn visit_impl_item_macro_mut(&mut self, i: &mut syn::ImplItemMacro) {}
    fn visit_impl_item_mut(&mut self, i: &mut syn::ImplItem) {}
    fn visit_impl_item_type_mut(&mut self, i: &mut syn::ImplItemType) {}
    fn visit_index_mut(&mut self, i: &mut syn::Index) {}
    fn visit_item_enum_mut(&mut self, i: &mut syn::ItemEnum) {}
    fn visit_item_fn_mut(&mut self, node: &mut syn::ItemFn) {}
    fn visit_item_impl_mut(&mut self, i: &mut syn::ItemImpl) {}
    fn visit_item_macro_mut(&mut self, i: &mut syn::ItemMacro) {}
    fn visit_item_mod_mut(&mut self, i: &mut syn::ItemMod) {}
    fn visit_item_mut(&mut self, i: &mut syn::Item) {}
    fn visit_item_static_mut(&mut self, i: &mut syn::ItemStatic) {}
    fn visit_item_struct_mut(&mut self, node: &mut syn::ItemStruct) {}
    fn visit_item_trait_alias_mut(&mut self, i: &mut syn::ItemTraitAlias) {}
    fn visit_item_trait_mut(&mut self, i: &mut syn::ItemTrait) {}
    fn visit_item_type_mut(&mut self, i: &mut syn::ItemType) {}
    fn visit_item_use_mut(&mut self, i: &mut syn::ItemUse) {}
    fn visit_lifetime_mut(&mut self, i: &mut syn::Lifetime) {}
    fn visit_lifetime_param_mut(&mut self, i: &mut syn::LifetimeParam) {}
    fn visit_local_init_mut(&mut self, i: &mut syn::LocalInit) {}
    fn visit_local_mut(&mut self, i: &mut syn::Local) {}
    fn visit_parenthesized_generic_arguments_mut(
        &mut self,
        i: &mut syn::ParenthesizedGenericArguments,
    ) {
    }
    fn visit_label_mut(&mut self, i: &mut syn::Label) {}
    fn visit_type_mut(&mut self, i: &mut syn::Type) {}
    fn visit_trait_item_type_mut(&mut self, i: &mut syn::TraitItemType) {}
    fn visit_trait_item_fn_mut(&mut self, i: &mut syn::TraitItemFn) {}
    fn visit_trait_bound_mut(&mut self, i: &mut syn::TraitBound) {}
    fn visit_trait_item_const_mut(&mut self, i: &mut syn::TraitItemConst) {}
    fn visit_trait_item_macro_mut(&mut self, i: &mut syn::TraitItemMacro) {}
    fn visit_predicate_lifetime_mut(&mut self, i: &mut syn::PredicateLifetime) {}
    fn visit_signature_mut(&mut self, i: &mut syn::Signature) {}
    fn visit_stmt_mut(&mut self, i: &mut syn::Stmt) {}
    fn visit_trait_item_mut(&mut self, i: &mut syn::TraitItem) {}
    fn visit_type_impl_trait_mut(&mut self, i: &mut syn::TypeImplTrait) {}
    fn visit_use_path_mut(&mut self, i: &mut syn::UsePath) {}
    fn visit_visibility_mut(&mut self, i: &mut syn::Visibility) {}
    fn visit_abi_mut(&mut self, i: &mut syn::Abi) {
        visit_mut::visit_abi_mut(self, i);
    }
    fn visit_angle_bracketed_generic_arguments_mut(
        &mut self,
        i: &mut syn::AngleBracketedGenericArguments,
    ) {
        visit_mut::visit_angle_bracketed_generic_arguments_mut(self, i);
    }
    fn visit_arm_mut(&mut self, i: &mut syn::Arm) {
        visit_mut::visit_arm_mut(self, i);
    }
    fn visit_assoc_const_mut(&mut self, i: &mut syn::AssocConst) {
        visit_mut::visit_assoc_const_mut(self, i);
    }
    fn visit_assoc_type_mut(&mut self, i: &mut syn::AssocType) {
        visit_mut::visit_assoc_type_mut(self, i);
    }
    fn visit_attr_style_mut(&mut self, i: &mut syn::AttrStyle) {
        visit_mut::visit_attr_style_mut(self, i);
    }
    fn visit_bin_op_mut(&mut self, i: &mut syn::BinOp) {
        visit_mut::visit_bin_op_mut(self, i);
    }
    fn visit_captured_param_mut(&mut self, i: &mut syn::CapturedParam) {
        visit_mut::visit_captured_param_mut(self, i);
    }
    fn visit_const_param_mut(&mut self, i: &mut syn::ConstParam) {
        visit_mut::visit_const_param_mut(self, i);
    }
    fn visit_constraint_mut(&mut self, i: &mut syn::Constraint) {
        visit_mut::visit_constraint_mut(self, i);
    }
    fn visit_derive_input_mut(&mut self, i: &mut syn::DeriveInput) {
        visit_mut::visit_derive_input_mut(self, i);
    }
    fn visit_expr_array_mut(&mut self, i: &mut syn::ExprArray) {
        visit_mut::visit_expr_array_mut(self, i);
    }
    fn visit_expr_async_mut(&mut self, i: &mut syn::ExprAsync) {
        visit_mut::visit_expr_async_mut(self, i);
    }
    fn visit_expr_await_mut(&mut self, i: &mut syn::ExprAwait) {
        visit_mut::visit_expr_await_mut(self, i);
    }
    fn visit_expr_binary_mut(&mut self, i: &mut syn::ExprBinary) {
        visit_mut::visit_expr_binary_mut(self, i);
    }
    fn visit_expr_break_mut(&mut self, i: &mut syn::ExprBreak) {
        visit_mut::visit_expr_break_mut(self, i);
    }
    fn visit_expr_cast_mut(&mut self, i: &mut syn::ExprCast) {
        visit_mut::visit_expr_cast_mut(self, i);
    }
    fn visit_expr_const_mut(&mut self, i: &mut syn::ExprConst) {
        visit_mut::visit_expr_const_mut(self, i);
    }
    fn visit_expr_continue_mut(&mut self, i: &mut syn::ExprContinue) {
        visit_mut::visit_expr_continue_mut(self, i);
    }
    fn visit_expr_for_loop_mut(&mut self, i: &mut syn::ExprForLoop) {
        visit_mut::visit_expr_for_loop_mut(self, i);
    }
    fn visit_expr_infer_mut(&mut self, i: &mut syn::ExprInfer) {
        visit_mut::visit_expr_infer_mut(self, i);
    }
    fn visit_expr_let_mut(&mut self, i: &mut syn::ExprLet) {
        visit_mut::visit_expr_let_mut(self, i);
    }
    fn visit_expr_lit_mut(&mut self, i: &mut syn::ExprLit) {
        visit_mut::visit_expr_lit_mut(self, i);
    }
    fn visit_expr_loop_mut(&mut self, i: &mut syn::ExprLoop) {
        visit_mut::visit_expr_loop_mut(self, i);
    }
    fn visit_expr_macro_mut(&mut self, i: &mut syn::ExprMacro) {
        visit_mut::visit_expr_macro_mut(self, i);
    }
    fn visit_expr_match_mut(&mut self, i: &mut syn::ExprMatch) {
        visit_mut::visit_expr_match_mut(self, i);
    }
    fn visit_expr_method_call_mut(&mut self, i: &mut syn::ExprMethodCall) {
        visit_mut::visit_expr_method_call_mut(self, i);
    }
    fn visit_expr_paren_mut(&mut self, i: &mut syn::ExprParen) {
        visit_mut::visit_expr_paren_mut(self, i);
    }
    fn visit_expr_path_mut(&mut self, i: &mut syn::ExprPath) {
        visit_mut::visit_expr_path_mut(self, i);
    }
    fn visit_expr_range_mut(&mut self, i: &mut syn::ExprRange) {
        visit_mut::visit_expr_range_mut(self, i);
    }
    fn visit_expr_raw_addr_mut(&mut self, i: &mut syn::ExprRawAddr) {
        visit_mut::visit_expr_raw_addr_mut(self, i);
    }
    fn visit_expr_reference_mut(&mut self, i: &mut syn::ExprReference) {
        visit_mut::visit_expr_reference_mut(self, i);
    }
    fn visit_expr_repeat_mut(&mut self, i: &mut syn::ExprRepeat) {
        visit_mut::visit_expr_repeat_mut(self, i);
    }
    fn visit_expr_return_mut(&mut self, i: &mut syn::ExprReturn) {
        visit_mut::visit_expr_return_mut(self, i);
    }
    fn visit_expr_struct_mut(&mut self, i: &mut syn::ExprStruct) {
        visit_mut::visit_expr_struct_mut(self, i);
    }
    fn visit_expr_try_mut(&mut self, i: &mut syn::ExprTry) {
        visit_mut::visit_expr_try_mut(self, i);
    }
    fn visit_expr_try_block_mut(&mut self, i: &mut syn::ExprTryBlock) {
        visit_mut::visit_expr_try_block_mut(self, i);
    }
    fn visit_expr_tuple_mut(&mut self, i: &mut syn::ExprTuple) {
        visit_mut::visit_expr_tuple_mut(self, i);
    }
    fn visit_expr_unary_mut(&mut self, i: &mut syn::ExprUnary) {
        visit_mut::visit_expr_unary_mut(self, i);
    }
    fn visit_expr_unsafe_mut(&mut self, i: &mut syn::ExprUnsafe) {
        visit_mut::visit_expr_unsafe_mut(self, i);
    }
    fn visit_expr_while_mut(&mut self, i: &mut syn::ExprWhile) {
        visit_mut::visit_expr_while_mut(self, i);
    }
    fn visit_expr_yield_mut(&mut self, i: &mut syn::ExprYield) {
        visit_mut::visit_expr_yield_mut(self, i);
    }
    fn visit_fn_arg_mut(&mut self, i: &mut syn::FnArg) {
        visit_mut::visit_fn_arg_mut(self, i);
    }
    fn visit_fn_ptr_variadic_mut(&mut self, i: &mut syn::FnPtrVariadic) {
        visit_mut::visit_fn_ptr_variadic_mut(self, i);
    }
    fn visit_foreign_item_mut(&mut self, i: &mut syn::ForeignItem) {
        visit_mut::visit_foreign_item_mut(self, i);
    }
    fn visit_foreign_item_fn_mut(&mut self, i: &mut syn::ForeignItemFn) {
        visit_mut::visit_foreign_item_fn_mut(self, i);
    }
    fn visit_foreign_item_macro_mut(&mut self, i: &mut syn::ForeignItemMacro) {
        visit_mut::visit_foreign_item_macro_mut(self, i);
    }
    fn visit_foreign_item_static_mut(&mut self, i: &mut syn::ForeignItemStatic) {
        visit_mut::visit_foreign_item_static_mut(self, i);
    }
    fn visit_foreign_item_type_mut(&mut self, i: &mut syn::ForeignItemType) {
        visit_mut::visit_foreign_item_type_mut(self, i);
    }
    fn visit_frontmatter_mut(&mut self, i: &mut syn::Frontmatter) {
        visit_mut::visit_frontmatter_mut(self, i);
    }
    fn visit_item_const_mut(&mut self, i: &mut syn::ItemConst) {
        visit_mut::visit_item_const_mut(self, i);
    }
    fn visit_item_extern_crate_mut(&mut self, i: &mut syn::ItemExternCrate) {
        visit_mut::visit_item_extern_crate_mut(self, i);
    }
    fn visit_item_foreign_mod_mut(&mut self, i: &mut syn::ItemForeignMod) {
        visit_mut::visit_item_foreign_mod_mut(self, i);
    }
    fn visit_item_union_mut(&mut self, i: &mut syn::ItemUnion) {
        visit_mut::visit_item_union_mut(self, i);
    }
    fn visit_lit_mut(&mut self, i: &mut syn::Lit) {
        visit_mut::visit_lit_mut(self, i);
    }
    fn visit_lit_bool_mut(&mut self, i: &mut syn::LitBool) {
        visit_mut::visit_lit_bool_mut(self, i);
    }
    fn visit_lit_byte_mut(&mut self, i: &mut syn::LitByte) {
        visit_mut::visit_lit_byte_mut(self, i);
    }
    fn visit_lit_byte_str_mut(&mut self, i: &mut syn::LitByteStr) {
        visit_mut::visit_lit_byte_str_mut(self, i);
    }
    fn visit_lit_cstr_mut(&mut self, i: &mut syn::LitCStr) {
        visit_mut::visit_lit_cstr_mut(self, i);
    }
    fn visit_lit_char_mut(&mut self, i: &mut syn::LitChar) {
        visit_mut::visit_lit_char_mut(self, i);
    }
    fn visit_lit_float_mut(&mut self, i: &mut syn::LitFloat) {
        visit_mut::visit_lit_float_mut(self, i);
    }
    fn visit_lit_int_mut(&mut self, i: &mut syn::LitInt) {
        visit_mut::visit_lit_int_mut(self, i);
    }
    fn visit_lit_str_mut(&mut self, i: &mut syn::LitStr) {
        visit_mut::visit_lit_str_mut(self, i);
    }
    fn visit_macro_mut(&mut self, i: &mut syn::Macro) {
        visit_mut::visit_macro_mut(self, i);
    }
    fn visit_macro_delimiter_mut(&mut self, i: &mut syn::MacroDelimiter) {
        visit_mut::visit_macro_delimiter_mut(self, i);
    }
    fn visit_member_mut(&mut self, i: &mut syn::Member) {
        visit_mut::visit_member_mut(self, i);
    }
    fn visit_meta_mut(&mut self, i: &mut syn::Meta) {
        visit_mut::visit_meta_mut(self, i);
    }
    fn visit_meta_list_mut(&mut self, i: &mut syn::MetaList) {
        visit_mut::visit_meta_list_mut(self, i);
    }
    fn visit_meta_name_value_mut(&mut self, i: &mut syn::MetaNameValue) {
        visit_mut::visit_meta_name_value_mut(self, i);
    }
    fn visit_pat_guard_mut(&mut self, i: &mut syn::PatGuard) {
        visit_mut::visit_pat_guard_mut(self, i);
    }
    fn visit_pat_ident_mut(&mut self, i: &mut syn::PatIdent) {
        visit_mut::visit_pat_ident_mut(self, i);
    }
    fn visit_pat_or_mut(&mut self, i: &mut syn::PatOr) {
        visit_mut::visit_pat_or_mut(self, i);
    }
    fn visit_pat_paren_mut(&mut self, i: &mut syn::PatParen) {
        visit_mut::visit_pat_paren_mut(self, i);
    }
    fn visit_pat_reference_mut(&mut self, i: &mut syn::PatReference) {
        visit_mut::visit_pat_reference_mut(self, i);
    }
    fn visit_pat_rest_mut(&mut self, i: &mut syn::PatRest) {
        visit_mut::visit_pat_rest_mut(self, i);
    }
    fn visit_pat_slice_mut(&mut self, i: &mut syn::PatSlice) {
        visit_mut::visit_pat_slice_mut(self, i);
    }
    fn visit_pat_struct_mut(&mut self, i: &mut syn::PatStruct) {
        visit_mut::visit_pat_struct_mut(self, i);
    }
    fn visit_pat_tuple_mut(&mut self, i: &mut syn::PatTuple) {
        visit_mut::visit_pat_tuple_mut(self, i);
    }
    fn visit_pat_tuple_struct_mut(&mut self, i: &mut syn::PatTupleStruct) {
        visit_mut::visit_pat_tuple_struct_mut(self, i);
    }
    fn visit_pat_type_mut(&mut self, i: &mut syn::PatType) {
        visit_mut::visit_pat_type_mut(self, i);
    }
    fn visit_pat_wild_mut(&mut self, i: &mut syn::PatWild) {
        visit_mut::visit_pat_wild_mut(self, i);
    }
    fn visit_path_mut(&mut self, i: &mut syn::Path) {
        visit_mut::visit_path_mut(self, i);
    }
    fn visit_path_arguments_mut(&mut self, i: &mut syn::PathArguments) {
        visit_mut::visit_path_arguments_mut(self, i);
    }
    fn visit_path_segment_mut(&mut self, i: &mut syn::PathSegment) {
        visit_mut::visit_path_segment_mut(self, i);
    }
    fn visit_pointer_mutability_mut(&mut self, i: &mut syn::PointerMutability) {
        visit_mut::visit_pointer_mutability_mut(self, i);
    }
    fn visit_precise_capture_mut(&mut self, i: &mut syn::PreciseCapture) {
        visit_mut::visit_precise_capture_mut(self, i);
    }
    fn visit_predicate_type_mut(&mut self, i: &mut syn::PredicateType) {
        visit_mut::visit_predicate_type_mut(self, i);
    }
    fn visit_qself_mut(&mut self, i: &mut syn::QSelf) {
        visit_mut::visit_qself_mut(self, i);
    }
    fn visit_range_limits_mut(&mut self, i: &mut syn::RangeLimits) {
        visit_mut::visit_range_limits_mut(self, i);
    }
    fn visit_receiver_mut(&mut self, i: &mut syn::Receiver) {
        visit_mut::visit_receiver_mut(self, i);
    }
    fn visit_receiver_kind_mut(&mut self, i: &mut syn::ReceiverKind) {
        visit_mut::visit_receiver_kind_mut(self, i);
    }
    fn visit_safety_mut(&mut self, i: &mut syn::Safety) {
        visit_mut::visit_safety_mut(self, i);
    }
    fn visit_static_mutability_mut(&mut self, i: &mut syn::StaticMutability) {
        visit_mut::visit_static_mutability_mut(self, i);
    }
    fn visit_stmt_macro_mut(&mut self, i: &mut syn::StmtMacro) {
        visit_mut::visit_stmt_macro_mut(self, i);
    }
    fn visit_token_stream_mut(&mut self, i: &mut proc_macro2::TokenStream) {}
    fn visit_type_array_mut(&mut self, i: &mut syn::TypeArray) {
        visit_mut::visit_type_array_mut(self, i);
    }
    fn visit_type_fn_ptr_mut(&mut self, i: &mut syn::TypeFnPtr) {
        visit_mut::visit_type_fn_ptr_mut(self, i);
    }
    fn visit_type_group_mut(&mut self, i: &mut syn::TypeGroup) {
        visit_mut::visit_type_group_mut(self, i);
    }
    fn visit_type_infer_mut(&mut self, i: &mut syn::TypeInfer) {
        visit_mut::visit_type_infer_mut(self, i);
    }
    fn visit_type_macro_mut(&mut self, i: &mut syn::TypeMacro) {
        visit_mut::visit_type_macro_mut(self, i);
    }
    fn visit_type_never_mut(&mut self, i: &mut syn::TypeNever) {
        visit_mut::visit_type_never_mut(self, i);
    }
    fn visit_type_param_mut(&mut self, i: &mut syn::TypeParam) {
        visit_mut::visit_type_param_mut(self, i);
    }
    fn visit_type_param_bound_mut(&mut self, i: &mut syn::TypeParamBound) {
        visit_mut::visit_type_param_bound_mut(self, i);
    }
    fn visit_type_paren_mut(&mut self, i: &mut syn::TypeParen) {
        visit_mut::visit_type_paren_mut(self, i);
    }
    fn visit_type_path_mut(&mut self, i: &mut syn::TypePath) {
        visit_mut::visit_type_path_mut(self, i);
    }
    fn visit_type_ptr_mut(&mut self, i: &mut syn::TypePtr) {
        visit_mut::visit_type_ptr_mut(self, i);
    }
    fn visit_type_reference_mut(&mut self, i: &mut syn::TypeReference) {
        visit_mut::visit_type_reference_mut(self, i);
    }
    fn visit_type_slice_mut(&mut self, i: &mut syn::TypeSlice) {
        visit_mut::visit_type_slice_mut(self, i);
    }
    fn visit_type_trait_object_mut(&mut self, i: &mut syn::TypeTraitObject) {
        visit_mut::visit_type_trait_object_mut(self, i);
    }
    fn visit_type_tuple_mut(&mut self, i: &mut syn::TypeTuple) {
        visit_mut::visit_type_tuple_mut(self, i);
    }
    fn visit_un_op_mut(&mut self, i: &mut syn::UnOp) {
        visit_mut::visit_un_op_mut(self, i);
    }
    fn visit_use_glob_mut(&mut self, i: &mut syn::UseGlob) {
        visit_mut::visit_use_glob_mut(self, i);
    }
    fn visit_use_group_mut(&mut self, i: &mut syn::UseGroup) {
        visit_mut::visit_use_group_mut(self, i);
    }
    fn visit_use_name_mut(&mut self, i: &mut syn::UseName) {
        visit_mut::visit_use_name_mut(self, i);
    }
    fn visit_use_rename_mut(&mut self, i: &mut syn::UseRename) {
        visit_mut::visit_use_rename_mut(self, i);
    }
    fn visit_use_tree_mut(&mut self, i: &mut syn::UseTree) {
        visit_mut::visit_use_tree_mut(self, i);
    }
    fn visit_variadic_mut(&mut self, i: &mut syn::Variadic) {
        visit_mut::visit_variadic_mut(self, i);
    }
    fn visit_variant_mut(&mut self, i: &mut syn::Variant) {
        visit_mut::visit_variant_mut(self, i);
    }
    fn visit_vis_restricted_mut(&mut self, i: &mut syn::VisRestricted) {
        visit_mut::visit_vis_restricted_mut(self, i);
    }
    fn visit_where_clause_mut(&mut self, i: &mut syn::WhereClause) {
        visit_mut::visit_where_clause_mut(self, i);
    }
    fn visit_where_clause_placement_mut(&mut self, i: &mut syn::WhereClausePlacement) {
        visit_mut::visit_where_clause_placement_mut(self, i);
    }
    fn visit_where_predicate_mut(&mut self, i: &mut syn::WherePredicate) {
        visit_mut::visit_where_predicate_mut(self, i);
    }
}

fn visibility(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility::Public,
        _ => Visibility::Private,
    }
}
