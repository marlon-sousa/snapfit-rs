//! Facade: snapfit's procedural macros, which hand their input to the
//! expansion handler; nothing else of this crate is public.

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

/// Lets a trait be dispatched statically, through a `match` over a closed
/// list of its implementations instead of a trait object.
///
/// Written once, on the trait, where it is defined:
/// `#[snapfit::static_dispatch] pub trait Database { ... }`. It keeps the
/// trait as written and implements it for snapfit's dispatch types, each
/// method forwarding to the implementation the value holds. It takes no
/// arguments, and it refuses, at the item, what a `match` cannot forward,
/// such as a method with no receiver or an associated const.
/// Use it through the `snapfit` crate, as `snapfit::static_dispatch`.
#[proc_macro_attribute]
pub fn static_dispatch(args: TokenStream, input: TokenStream) -> TokenStream {
    expansion::static_dispatch_attribute(args.into(), input.into()).into()
}
