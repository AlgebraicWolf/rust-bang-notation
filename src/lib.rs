use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro2::Span;
use quote::ToTokens;
use syn::parse_quote;
use syn::visit_mut::VisitMut;
use syn::{parse_macro_input, visit_mut, Expr, UnOp};

struct LiftMonadic {
    lifted: Vec<(Expr, Ident)>,
}

impl LiftMonadic {
    pub fn new() -> LiftMonadic {
        LiftMonadic { lifted: Vec::new() }
    }
}

impl VisitMut for LiftMonadic {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        // We are only interested in unary expressions with '!'.
        if let Expr::Unary(un_expr) = i {
            // First traverse recursively. This makes sure that innermost
            // expressions are lifted to the top.
            self.visit_expr_unary_mut(un_expr);

            // If this is a `!`, that is what we were looking for!
            // We shall move the expression to storage for lifting, replacing
            // it with a newly bound identifier here.
            if let UnOp::Not(_) = un_expr.op {
                let id = self.lifted.len();
                let fresh_ident = Ident::new(
                    format!("__bang_inner_bind_{id}").as_str(),
                    Span::call_site(),
                );
                self.lifted
                    .push((*un_expr.expr.clone(), fresh_ident.clone()));
                *i = parse_quote!(#fresh_ident);
            }
        }

        // Perform default visitor routine to recursively traverse everything.
        visit_mut::visit_expr_mut(self, i);
    }
}

#[proc_macro]
pub fn bang(input: TokenStream) -> TokenStream {
    // This is macro entry point.
    // We shall start with parsing the input as an expression.
    let mut ast: Expr = parse_macro_input!(input as Expr);

    // Create a collector and traverse ast tree to replace all monadic
    // values by a bind.
    let mut collector = LiftMonadic::new();
    collector.visit_expr_mut(&mut ast);

    // Now, we need to fold the lifted values and put all the bindings
    // on top of expression.
    let ast = collector
        .lifted
        .iter()
        .rev()
        .fold(ast, |cur_ast, (expr, ident)| {
            parse_quote! {
                (#expr).and_then(|#ident| { #cur_ast })
            }
        });

    ast.to_token_stream().into()
}
