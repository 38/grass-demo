use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Ident};

use crate::ql::{CodeGenerator, CodeGeneratorContext};

mod ql;

fn grass_query_impl(query_body : ql::QueryBody) -> (TokenStream2, Option<Ident>) {
    let result_ident;
    let code_fragments = {
        let mut ctx = CodeGeneratorContext::default();
        match query_body.generate(&mut ctx) {
            Ok(ident) => {
                result_ident = ident;
                ctx.into_code_vec()
            },
            Err(e) => return (e.into_compile_error().into(), None),
        }
    };

    let ret = quote! {
        #(#code_fragments)*
    };

    (ret.into(), result_ident)
}

#[proc_macro]
pub fn grass_query_block(input: TokenStream) -> TokenStream {
    let query_body = parse_macro_input!(input as ql::QueryBody);
    grass_query_impl(query_body).0.into()
}

#[proc_macro]
pub fn grass_query(input: TokenStream) -> TokenStream {
    let query_body = parse_macro_input!(input as ql::QueryBody);
    let (query_code, result_id) = grass_query_impl(query_body);

    let display_code = if let Some(result_id) = result_id {
        quote! {
            println!("{:?}", #result_id);
        }
    } else {
        quote! { () }
    };

    (quote! {
        fn main() {
            #query_code

            #display_code
        }
    }).into()
}
