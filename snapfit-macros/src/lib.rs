//! Facade: snapfit's procedural macro, which hands its input to the expansion
//! handler; nothing else of this crate is public.

mod adapters;
mod domain;
mod expansion;
#[cfg(test)]
mod test_support;
mod use_cases;

use proc_macro::TokenStream;

/// Turns a struct into a dependency injection container.
///
/// Each field is marked `#[singleton]`, whose settings say how it is built
/// and which other fields it depends on, by name, such as
/// `#[singleton(deps = [config])]`. The macro checks the whole graph at
/// compile time, makes a synchronous container implement
/// `snapfit::Container`, or `snapfit::FallibleContainer` when a field says
/// `fallible = true`, and gives each field a getter, `get_<field>()`.
/// Use it through the `snapfit` crate, as `snapfit::container`.
/// [The Snapfit Book](https://marlon-sousa.github.io/snapfit-rs/book/)
/// teaches it step by step.
#[proc_macro_attribute]
pub fn container(args: TokenStream, input: TokenStream) -> TokenStream {
    expansion::container_attribute(args.into(), input.into()).into()
}
