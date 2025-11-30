mod error2;
mod generics;
mod messages;
mod parser;
mod types;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Error2, attributes(error2))]
pub fn error2(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    error2::generate(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
