# Installing Snapfit and your first singleton

Time to write a container of our own. We'll start as small as possible: one component, the shop's configuration, built once and shared by everyone who asks for it.

## Adding Snapfit

`snapfit` is one crate. Add `snapfit = "0.2"` under `[dependencies]` in your `Cargo.toml`. It gives you the `#[container]` attribute and the `Container` trait. You need Rust `1.85.0` or newer; your own crate can use any edition from 2018 on.

## A configuration to share

Here is the shop's configuration. It is an ordinary struct with an ordinary `new()`:

```example
name: ch05_first_singleton
region: domain
```

Now the container. We mark the struct with `#[container]`, and the `config` field with `#[singleton]`. A singleton is a component the container builds once and then shares:

```example
name: ch05_first_singleton
region: container
```

Let's build the container, ask for the configuration twice, and check whether we got the same one both times:

```example
name: ch05_first_singleton
region: run
```

It prints:

```output
name: ch05_first_singleton
```

The same allocation, both times. Think of the one printer in an office: everyone sends their pages to it, and nobody buys a printer of their own.

## What happened behind the scenes

`#[container]` wrote three things for us.

1. **A way to build the container.** It generated `ShopContainer::new()`, and also implemented `Default` and `snapfit::Container`, so `ShopContainer::default()` builds it too.
2. **The `Arc` around the configuration.** The field's type is `Arc<ShopConfig>`, so the container called `ShopConfig::new()` and wrapped the plain `ShopConfig` it returned in `Arc::new(...)` itself. Your constructor returns the plain type; the `Arc` is the container's job.
3. **A getter.** For each field, it generated a method named after it, here `get_config()`, which returns a clone of the field. Cloning an `Arc` only adds one to a counter, so both calls point to the same `ShopConfig`, which is what `Arc::ptr_eq` confirmed.

## Which problem this solves

1. **Duplicated resources** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): `config` is built once, when the container is built, and every call to `get_config()` returns that same one, as `Arc::ptr_eq` shows.
2. **Sharing by convention** ([Chapter 3](03-who-builds-everything.md)): every field of a container is built once and shared. There is no second kind of field that is quietly built again on each call, so there is nothing to remember.

## Remember

1. `snapfit` needs Rust `1.85.0` or newer, and your crate can use any edition from 2018 on.
2. A `#[singleton]` field with no `deps` needs nothing, so `ShopContainer::new()` builds it when it reaches the field, or earlier if a field declared before it needs it.
3. For an `Arc<T>` field, your constructor returns `T`, and `snapfit` wraps it in `Arc::new(...)` for you.
