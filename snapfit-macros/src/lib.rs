//! Facade: snapfit's procedural macro, which hands its input to the expansion
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
