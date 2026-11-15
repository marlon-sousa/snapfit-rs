#![deny(warnings)]
#![allow(dead_code, unused_imports)]

mod domain;
mod ports;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn container(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_derive(Container, attributes(container, singleton, transient, dispatch))]
pub fn derive_container(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
