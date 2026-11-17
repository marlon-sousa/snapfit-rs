# Quick start

This page is for when you already know what dependency injection is and just want to see `snapfit` working. If you'd rather understand the problem first, start with [Part 1](01-a-service-that-builds-its-own-dependencies.md); this page will still be here.

## Three plain structs

A shop that loads its currency settings once, opens a catalog database that formats prices, and has a checkout that uses the database. Nothing here mentions `snapfit`:

```example
name: quick_start
region: domain
```

## One container

The container lists the three components and what each one needs:

```example
name: quick_start
region: container
```

## Run it

Build the container and ask it for the checkout:

```example
name: quick_start
region: run
```

It prints:

```output
name: quick_start
```

## What you just saw

1. **Plain domain structs.** `ShopConfig`, `CatalogDb` and `OrderCheckout` don't import `snapfit` or implement any of its traits ([Chapter 6](06-pure-domain-types.md)).
2. **A component that needs nothing.** `config: Arc<ShopConfig>` has no `deps`, so `ShopContainer::new()` calls `ShopConfig::new()` once and wraps it in an `Arc` ([Chapter 5](05-installing-and-first-singleton.md)).
3. **Components that need others.** `db` lists `config`, so it receives a clone of it in `CatalogDb::new(...)`; `checkout` lists `db` and is built after it ([Chapter 7](07-wiring-dependencies-and-constructors.md)).
4. **One of each.** Each component is built once, and `get_checkout()` returns a clone of the same `Arc<OrderCheckout>` every time ([Chapter 5](05-installing-and-first-singleton.md)).

## Which problem this solves

[Part 1](01-a-service-that-builds-its-own-dependencies.md) walks through the problems of wiring an application without `snapfit`, and [Chapter 4](04-why-we-built-snapfit.md) shows where each one is answered. On this page you met two of them:

1. **Tight coupling** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): `OrderCheckout` receives its database; only `ShopContainer` decides what it is.
2. **Ordering by hand** ([Chapter 3](03-who-builds-everything.md)): nobody orders the construction. `ShopContainer::new()` builds each field after the fields its `deps` lists.

## Remember

1. Domain structs stay plain Rust; only the container carries `snapfit` attributes.
2. Every `#[singleton]` is built once, after the fields it needs, and its `get_<name>()` returns a clone of it.
