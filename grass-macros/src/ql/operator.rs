use super::CodeGeneratorContext;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use quote::quote;
use syn::{Expr, Ident, LitInt, Result, Token, parenthesized, parse::{Parse, ParseStream}, punctuated::Punctuated, visit_mut::VisitMut};

pub(crate) enum Operator {
    Where(Expr),
    Map(Expr),
    Invoke(Ident, Punctuated<Expr, Token![,]>),
}

impl Debug for Operator {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Operator")
    }
}

struct ClosureRewriter;

fn rewrite_field_access(node: &mut Expr) -> bool {
    let id = match node {
        Expr::Path(path_expr) => {
            if let Some(ident) = path_expr.path.get_ident() {
                let ident_str = ident.to_string();
                if !ident_str.starts_with('_') || !ident_str[1..].chars().all(|x| x.is_digit(10))  || ident_str == "_0" {
                    return false;
                }
                LitInt::new(&format!("{}", ident_str[1..].parse::<usize>().unwrap() - 1), ident.span())
            } else {
                return false;
            }
        }
        _ => { return false; }
    };

    let new_expr : Expr = syn::parse2(quote! {
        _0 . #id
    }).unwrap();

    *node = new_expr;
    true
}

impl VisitMut for ClosureRewriter {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        if !rewrite_field_access(node) {
            syn::visit_mut::visit_expr_mut(self, node);
        }
    }
}

impl Operator {
    pub(crate) fn apply(&self, upstream: Ident, ctx: &mut CodeGeneratorContext) -> Ident {
        match self {
            Operator::Map(code) => {
                let id = ctx.fresh_id();
                let mut code = code.clone();
                ClosureRewriter.visit_expr_mut(&mut code);
                ctx.append(quote! {
                    let #id = {
                        use grass::properties::*;
                        #upstream . map(|mut _0| {
                            #code
                            _0
                        })
                    };
                });
                id
            }
            Operator::Where(expr) => {
                let id = ctx.fresh_id();
                let mut expr = expr.clone();
                ClosureRewriter.visit_expr_mut(&mut expr);
                let code = quote! {
                    let #id = #upstream.filter(|_0| { 
                        use grass::properties::*;
                        #expr 
                    });
                };
                ctx.append(code);
                id
            }
            Operator::Invoke(method, arg) => {
                let id = ctx.fresh_id();
                let code = quote! {
                    let #id = {
                        use grass::high_level_api::*;
                        #upstream . #method ( #arg )
                    };
                };
                ctx.append(code);
                id
            }
        }
    }
}

impl Parse for Operator {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(_) = input.fork().parse::<Token![where]>() {
            let _: Token![where] = input.parse()?;
            let inner;
            let _ = parenthesized!(inner in input);
            return Ok(Operator::Where(inner.parse()?));
        } else if let Ok(id) = input.fork().parse::<Ident>() {
            match id.to_string().as_str() {
                "map" => {
                    let _: Ident = input.parse()?;
                    let inner;
                    let _ = parenthesized!(inner in input);
                    return Ok(Operator::Map(inner.parse()?));
                }
                _ => {
                    let id = input.parse()?;
                    let inner;
                    parenthesized!(inner in input);
                    return Ok(Operator::Invoke(id, Punctuated::parse_terminated(&inner)?));
                }
            }
        } else {
            panic!("Invalid operator");
        }
    }
}
