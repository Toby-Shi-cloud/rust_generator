mod decorator;

use decorator::recreate_function;
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn generator(_attrs: TokenStream, code: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(code as ItemFn);
    recreate_function(&mut ast);
    quote!(#ast).into()
}
