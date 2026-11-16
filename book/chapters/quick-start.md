# Quick start

## What you want to do

The shop needs to load its currency configuration once (`ShopConfig`), open a shared catalog database (`CatalogDb`), and build one `OrderCheckout` handler on top of it — without adding any dependency injection macros to the domain structs.

## The code

First, write the shop's domain types as plain Rust structs with ordinary `pub fn new(...)` constructors:

```example
name: quick_start
region: domain
```

Next, declare `ShopContainer` with `#[container]` to wire the three components together:

```example
name: quick_start
region: container
```

Finally, instantiate the container and resolve `OrderCheckout`:

```example
name: quick_start
region: run
```

When run, the program prints:

```output
name: quick_start
```

## How it works

1. **Pure domain structs:** `ShopConfig`, `CatalogDb`, and `OrderCheckout` do not import `snapfit` or derive any framework traits. They are standard Rust types([Chapter 2](02-pure-domain-types.md)).
2. **Leaf singleton (`#[singleton]`):** `config: Arc<ShopConfig>` has no prerequisites, so `ShopContainer::new()` calls `ShopConfig::new()` once at startup and wraps it in `Arc` ([Chapter 4](04-installing-and-first-singleton.md)).
3. **Dependent singleton (`#[singleton(deps = [config])]`):** `db: Arc<CatalogDb>` declares `config` as its prerequisite, so `ShopContainer::new()` passes `config.clone()` into `CatalogDb::new(...)` ([Chapter 5](05-wiring-dependencies-and-constructors.md)).
4. **A singleton that depends on another (`#[singleton(deps = [db])]`):** `checkout` is built once via `OrderCheckout::new(db)` after `db`, and `container.get_checkout()` returns a clone of the same `Arc<OrderCheckout>` ([Chapter 5](05-wiring-dependencies-and-constructors.md)).

## Remember

1. Domain structs stay 100% pure Rust; only the container struct carries `snapfit` attributes.
2. `#[singleton]` fields are initialized once when the container is built and cloned on `get_<name>()`.
3. Every `#[singleton]` is built once, in dependency order, and each `get_<name>()` returns a clone of it.
