# Static-dispatch spike (throwaway, not for merging)

A spike for #41, the static dispatch of snapfit 0.3.0. Nothing here ships. It
answers whether a statically dispatched field can work without the
`enum_dispatch` crate and without state shared between macro calls.

## The crates

- `oneof`: stands for snapfit's runtime crate. It holds one generic enum per
  number of implementations, `OneOf2<A, B>` and `OneOf3<A, B, C>`.
- `dispatchable`: stands for snapfit's macro crate. `#[dispatchable]` on a trait
  implements the trait for `OneOf2` and `OneOf3`, in the trait's own module,
  so every type its methods name is in scope. Each method is a `match` that
  forwards to the variant.
- `app`: a container, written with today's snapfit, whose generated parts for
  a static field are written by hand in `src/shop.rs`: an alias naming the
  enum, and one `From` per implementation, so that a factory writes `.into()`.
- `probes`: one binary per trait shape the attribute does not handle yet.
  Most of them fail to compile on purpose.

## Running it

From this folder, `cargo run -p app`, and `cargo run -p probes --bin <name>`
for one probe.
