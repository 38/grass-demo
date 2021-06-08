use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Error, Ident, Result, Token,
};

mod expr;
mod open_impl;
mod operator;

pub(crate) use expr::QueryExpr;
pub(crate) use operator::Operator;

#[derive(Debug)]
pub(crate) struct VarBinding {
    pub id: Ident,
    pub expr: QueryExpr,
}

impl Parse for VarBinding {
    fn parse(input: ParseStream) -> Result<Self> {
        let _let: Token![let] = input.parse()?;
        let id: Ident = input.parse()?;
        let _eq: Token![=] = input.parse()?;
        let expr: QueryExpr = input.parse()?;
        Ok(Self { id, expr })
    }
}

#[derive(Debug)]
pub(crate) enum QueryStmt {
    Expr(QueryExpr),
    Let(VarBinding),
}

impl Parse for QueryStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let ret = if input.peek(Ident) {
            QueryStmt::Expr(input.parse()?)
        } else if input.peek(Token![let]) {
            QueryStmt::Let(input.parse()?)
        } else {
            return Err(Error::new(
                input.span(),
                "Unexpected token for the Grass DSL",
            ));
        };

        // Just strip the last semicolon if there is one:w
        while input.parse::<Token![;]>().is_ok() {}

        Ok(ret)
    }
}

impl CodeGenerator for QueryStmt {
    fn generate(&self, ctx: &mut CodeGeneratorContext) -> Result<Option<Ident>> {
        match self {
            Self::Expr(query) => {
                return Ok(query.generate(ctx)?);
            }
            Self::Let(VarBinding { id, expr: query }) => {
                let tmp_id = query.generate(ctx)?.unwrap();
                let binding_code = quote! {
                    let mut #id = #tmp_id;
                };
                ctx.append(binding_code);
            }
        }
        Ok(None)
    }
}

#[derive(Debug)]
pub(crate) struct QueryBody {
    pub query_stmts: Vec<(Span, QueryStmt)>,
}

impl Parse for QueryBody {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut stmts = vec![];
        while !input.is_empty() {
            let span = input.span();
            let stmt: QueryStmt = input.parse()?;
            stmts.push((span, stmt));
        }
        Ok(QueryBody { query_stmts: stmts })
    }
}

impl CodeGenerator for QueryBody {
    fn generate(&self, ctx: &mut CodeGeneratorContext) -> Result<Option<Ident>> {
        let mut last = None;
        for (span, stmt) in self.query_stmts.iter() {
            ctx.set_current_span(*span);
            last = stmt.generate(ctx)?;
        }
        Ok(last)
    }
}

pub(crate) struct CodeGeneratorContext {
    code_buf: Vec<TokenStream>,
    current_span: Option<Span>,
    id_prefix: String,
    next_id: usize,
}

impl Default for CodeGeneratorContext {
    fn default() -> Self {
        let stem = uuid::Uuid::new_v4().to_simple();
        Self {
            code_buf: Vec::new(),
            id_prefix: format!("_query_tmp_{}", stem),
            current_span: None,
            next_id: 0,
        }
    }
}

impl CodeGeneratorContext {
    pub fn into_code_vec(self) -> Vec<TokenStream> {
        self.code_buf
    }

    fn get_current_span(&self) -> &Span {
        self.current_span.as_ref().unwrap()
    }

    fn append(&mut self, code: TokenStream) {
        self.code_buf.push(code);
    }

    fn set_current_span(&mut self, span: Span) {
        self.current_span = Some(span);
    }

    fn fresh_id(&mut self) -> Ident {
        let claimed = self.next_id;
        self.next_id += 1;
        let id_text = format!("{}_{}", self.id_prefix, claimed);
        Ident::new(&id_text, self.current_span.unwrap())
    }
}

pub(crate) trait CodeGenerator {
    fn generate(&self, ctx: &mut CodeGeneratorContext) -> Result<Option<Ident>>;
}
