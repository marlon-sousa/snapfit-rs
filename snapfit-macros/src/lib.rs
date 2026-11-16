//! Facade: snapfit's procedural macros. Each hands its input to the expansion
//! handler; nothing else of this crate is public.

mod adapters;
mod domain;
mod expansion;
#[cfg(test)]
mod test_support;
mod use_cases;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn container(args: TokenStream, input: TokenStream) -> TokenStream {
    expansion::container_attribute(args.into(), input.into()).into()
}

#[proc_macro_derive(Container, attributes(container, singleton, transient, dispatch))]
pub fn derive_container(input: TokenStream) -> TokenStream {
    expansion::container_derive(input.into()).into()
}
