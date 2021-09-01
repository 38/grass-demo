use grass_formats::{FileFormat, FileKind};
use quote::quote;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Error, Ident, LitStr, Result, Token,
};

use super::{open_impl, CodeGenerator, CodeGeneratorContext, Operator};

/// Argument for an open expression
pub(crate) struct OpenArgument {
    path: LitStr,
}

impl Debug for OpenArgument {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "path = {}", self.path.value())
    }
}

impl Parse for OpenArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        let inner;
        let _ = parenthesized!(inner in input);
        Ok(OpenArgument {
            path: inner.parse()?,
        })
    }
}

/// QueryExpr :=
///      open(<StrLit>)
///     intersect(<QueryExpr>, *)
///     <id>
///     <QueryExpr> | <Operator>
#[derive(Debug)]
pub(crate) enum QueryExpr {
    Open(OpenArgument),
    Intersect(Vec<QueryExpr>),
    LeftOutterIntersect(Vec<QueryExpr>),
    VarRef(Ident),
    OpChain((Box<QueryExpr>, Operator)),
}

impl QueryExpr {
    fn parse_left_most(input: ParseStream) -> Result<Self> {
        if let Err(e) = input.fork().parse::<Ident>() {
            return Err(e);
        } else {
            let first_ident = input.parse::<Ident>()?;
            match first_ident.to_string().as_str() {
                "open" => {
                    return Ok(QueryExpr::Open(input.parse()?));
                }
                "intersect" => {
                    let arguments;
                    let _ = parenthesized!(arguments in input);
                    let parsed = Punctuated::<QueryExpr, Token![,]>::parse_terminated(&arguments)?;
                    return Ok(QueryExpr::Intersect(parsed.into_iter().collect()));
                }
                "left_outter_intersect" => {
                    let arguments;
                    let _ = parenthesized!(arguments in input);
                    let parsed = Punctuated::<QueryExpr, Token![,]>::parse_terminated(&arguments)?;
                    return Ok(QueryExpr::LeftOutterIntersect(parsed.into_iter().collect()));
                }
                _ => {
                    return Ok(QueryExpr::VarRef(first_ident));
                }
            }
        }
    }
}

impl Parse for QueryExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut result = Self::parse_left_most(input)?;
        while input.parse::<Token![|]>().is_ok() {
            result = Self::OpChain((Box::new(result), input.parse()?));
        }
        Ok(result)
    }
}

impl CodeGenerator for QueryExpr {
    fn generate(&self, ctx: &mut CodeGeneratorContext) -> Result<Option<Ident>> {
        match self {
            QueryExpr::Open(OpenArgument { path }) => {
                let id = ctx.fresh_id();
                let code = match FileFormat::detect_file(path.value()) {
                    Ok(format) => match format.kind {
                        FileKind::Bed(size) => {
                            open_impl::generate_bed_open_code(&id, path, size, format.deflated)
                        }
                        FileKind::Alignment(_) => open_impl::generate_xam_open_code(&id, path),
                        FileKind::Vcf => open_impl::generate_vcf_open_code(&id, path),
                        _ => panic!("Unsupported file format"),
                    },
                    Err(err) => {
                        return Err(Error::new(
                            path.span(),
                            format!(
                                "Unable to detect file format for {}, reason: {}",
                                path.value(),
                                err
                            ),
                        ));
                    }
                };
                ctx.append(code);
                Ok(Some(id))
            }
            QueryExpr::LeftOutterIntersect(args) => {

                    let first = args[0].generate(ctx)?;
                    let second = args[1].generate(ctx)?;
                    
                    let id = ctx.fresh_id();

                    let code = quote! {
                        let mut #id = {
                            use grass::algorithm::SortedIntersect;
                            #first.sorted_left_outer_intersect(#second)
                        };
                    };
                    ctx.append(code.into());
                    return Ok(Some(id));
            }
            QueryExpr::Intersect(inputs) => {
                let vars: Vec<_> = inputs.iter().map(|x| x.generate(ctx)).collect();
                if vars.len() == 0 {
                    return Err(Error::new(
                        *ctx.get_current_span(),
                        "Intersection with no input",
                    ));
                } else if vars.len() == 1 {
                    return Ok(Some(vars[0].clone()?.unwrap()));
                } else {
                    let first = vars[0].clone()?.unwrap();
                    let mut rem = vec![];
                    for id in vars[1..].iter() {
                        rem.push(id.clone()?.unwrap());
                    }
                    let id = ctx.fresh_id();

                    let mut flatten_closure_arg = quote! { _1 };
                    let mut flatten_closure_body = vec![Ident::new("_1", first.span())];

                    for idx in 2..=vars.len() {
                        let new_ident = Ident::new(
                            &format!("_{}", idx),
                            vars[idx - 1].as_ref().unwrap().as_ref().unwrap().span(),
                        );
                        flatten_closure_arg = quote! {
                            (#flatten_closure_arg, #new_ident)
                        };
                        flatten_closure_body.push(new_ident);
                    }

                    let code = quote! {
                        let mut #id = {
                            use grass::algorithm::SortedIntersect;
                            #first #(.sorted_intersect(#rem))*
                            .map( |#flatten_closure_arg| (#(#flatten_closure_body),*))
                        };
                    };
                    ctx.append(code.into());
                    return Ok(Some(id));
                }
            }
            QueryExpr::OpChain((expr, operator)) => {
                if let Some(upstream_id) = expr.generate(ctx)? {
                    Ok(Some(operator.apply(upstream_id, ctx)))
                } else {
                    Ok(None)
                }
            }
            QueryExpr::VarRef(id) => {
                let fresh_id = ctx.fresh_id();
                let code = quote! {
                    let mut #fresh_id = #id;
                };
                ctx.append(code.into());
                Ok(Some(fresh_id))
            }
        }
    }
}
