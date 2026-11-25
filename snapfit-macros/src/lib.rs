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
///
/// A field typed as a trait object, `Arc<dyn Trait>`, names the type the
/// container builds with `implementation = <Type>`. A container can also
/// dispatch a trait marked `#[snapfit::static_dispatch]` statically:
/// `#[static_dispatch_for(Trait as Name)]`, written under `#[container]`,
/// declares `Name`, a type alias that holds one of a closed list of the
/// trait's implementations. Fields typed `Name` or `Arc<Name>` add theirs to
/// the list with `implementation = <Type>`, or, beside `build = factory`,
/// with `implementations = [A, B]`, the variants the factory may return;
/// `#[extend_static_dispatch(Name, [Mock])]` adds others, such as a test's
/// under `#[cfg_attr(test, ...)]`. `Name::of(value)` wraps an implementation
/// into its variant, and calling the trait on a `Name` reaches the
/// implementation it holds through a `match`.
///
/// Use it through the `snapfit` crate, as `snapfit::container`.
/// [The Snapfit Book](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/), in the
/// edition of this release, teaches it step by step, with
/// [fields typed as their trait](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/11-choosing-an-implementation/)
/// and [static dispatch](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/13-static-dispatch/).
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
///
/// The container then declares the type that holds the implementations,
/// with `#[static_dispatch_for(Database as Db)]` under `#[container]`, and
/// its fields typed `Db` or `Arc<Db>` name them; `Db::of(value)` wraps an
/// implementation into its variant, such as a mock a test builds by hand.
/// [The Snapfit Book](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/13-static-dispatch/), in the
/// edition of this release, teaches it, and
/// [how to choose it over a trait object](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/14-choosing-static-or-dynamic-dispatch/).
///
/// Use it through the `snapfit` crate, as `snapfit::static_dispatch`.
#[proc_macro_attribute]
pub fn static_dispatch(args: TokenStream, input: TokenStream) -> TokenStream {
    expansion::static_dispatch_attribute(args.into(), input.into()).into()
}
