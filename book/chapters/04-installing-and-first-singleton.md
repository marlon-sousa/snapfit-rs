# Installing Snapfit and your first singleton

## What you want to do

You want to add `snapfit` to a Rust crate, declare a container with one leaf singleton (`ShopConfig`), and verify that calling `get_config()` multiple times returns references to the exact same allocation.

## The code

In `Cargo.toml`, add `snapfit` (`0.1.0`), which re-exports the `#[container]` attribute macro and the `Container` trait.

Define `ShopConfig` with a standard `pub fn new() -> Self` constructor:

```example
name: ch04_first_singleton
region: domain
```

Declare `ShopContainer` with `#[container]` and mark `config` as `#[singleton]`:

```example
name: ch04_first_singleton
region: container
```

Instantiate the container and compare the pointers returned by two calls to `get_config()`:

```example
name: ch04_first_singleton
region: run
```

It prints:

```output
name: ch04_first_singleton
```

## How it works

1. **Automatic `Arc` wrapping:** When a `#[singleton]` field has type `Arc<T>`, `#[container]` strips the outer `Arc<...>` to find the target struct `T`, calls `T::new()`, and wraps the result in `std::sync::Arc::new(...)`.
2. **Generated constructors (`new` and `Default`):** For a synchronous container, `#[container]` generates `pub fn new() -> Self`, implements `std::default::Default`, and implements `snapfit::Container`.
3. **Zero-cost typed getter (`get_config`):** For each field `<name>: <Type>`, `#[container]` emits `pub fn get_<name>(&self) -> <Type>` which returns `self.<name>.clone()`. Because `Arc::clone` only increments an atomic reference count, both calls point to the exact same `ShopConfig` allocation (`Arc::ptr_eq(&first, &second)` is `true`).

## Remember

1. `snapfit` requires Rust `1.85.0` or newer (Rust 2024 Edition).
2. A `#[singleton]` field with no `deps` is a leaf node and is constructed first during `Container::new()`.
3. For `Arc<T>` fields, your constructor `T::new()` returns `T`, not `Arc<T>` — `snapfit` wraps it in `Arc::new(...)` automatically.
