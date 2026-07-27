use std::path::PathBuf;

use swc_core::ecma::{
    ast::{ClassDecl, FnDecl, Pat, VarDeclarator},
    visit::Visit,
};

use crate::{analyzer::*, ir::*, parser::ParserContext};

pub struct TypeScriptAnalyzer;

// impl Analyzer for TypeScriptAnalyzer {
//     fn analyze(
//         &self,
//         source: &str,
//         options: &AnalyzerOptions,
//     ) -> Result<Vec<Symbol>, AnalysisError> {
//         let ctx = ParserContext {
//             cm: Default::default(),
//         };
//         ctx.with_parser("input.ts", source, |parser| {
//             let module = parser
//                 .parse_module()
//                 .map_err(|e| AnalysisError::Parse(format!("{:?}", e)))?;
//             let mut visitor = TsVisitor {
//                 options,
//                 symbols: vec![],
//             };
//             visitor.visit_module(&module);
//             Ok(visitor.symbols)
//         })
//     }
// }

struct TsVisitor<'a> {
    options: &'a AnalyzerOptions,
    pub symbols: Vec<Symbol>,
}

impl<'a> TsVisitor<'a> {
    fn visibility(&self, is_export: bool) -> Visibility {
        if is_export {
            Visibility::Public
        } else {
            Visibility::Private
        }
    }
}

impl Visit for TsVisitor<'_> {
    fn visit_fn_decl(&mut self, n: &FnDecl) {
        let name = n.ident.sym.to_string();

        let params = Some(
            n.function
                .params
                .iter()
                .map(|param| {
                    let name = match &param.pat {
                        Pat::Ident(i) => i.id.sym.to_string(),
                        _ => "arg".to_string(),
                    };
                    let type_name = "any".to_string();
                    (name, type_name)
                })
                .collect(),
        );
        let return_type = n.function.return_type.as_ref().map(|_| "any".to_string());
        let visibility = self.visibility(false);
        if self.options.include_private || matches!(visibility, Visibility::Public) {
            self.symbols.push(Symbol {
                id: 0,
                name,
                location: Some(SymbolLocation::default()),
                kind: SymbolKind::Function(FunctionKind::Free),
                visibility,
                children: Vec::new(),
                params,
                return_type,
            });
        }
    }
    fn visit_class_decl(&mut self, c: &ClassDecl) {
        self.symbols.push(Symbol {
            id: 0,
            name: c.ident.sym.to_string(),
            location: Some(SymbolLocation::default()),
            kind: SymbolKind::Type(TypeKind::Class),
            visibility: Visibility::Public,
            params: None,
            children: Vec::new(),
            return_type: None,
        });
    }
    fn visit_var_declarator(&mut self, v: &VarDeclarator) {
        let name = match &v.name {
            Pat::Ident(i) => i.id.sym.to_string(),
            _ => return,
        };

        self.symbols.push(Symbol {
            id: 0,
            name,
            location: Some(SymbolLocation::default()),
            kind: SymbolKind::Variable(VariableKind::Let),
            visibility: Visibility::Public,
            params: None,
            children: Vec::new(),
            return_type: None,
        });
    }
}
