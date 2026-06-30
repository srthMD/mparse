mod impls;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(StringifyEnum)]
pub fn stringify(ts: TokenStream) -> TokenStream {
  let input = parse_macro_input!(ts as DeriveInput);
  impls::stringify_impl(input)
    .unwrap_or_else(|err| err.to_compile_error())
    .into()
}
