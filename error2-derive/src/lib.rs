mod error_ext;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(ErrorExt, attributes(error2))]
pub fn error_ext(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    error_ext::generate(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
