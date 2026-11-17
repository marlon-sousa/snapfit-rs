# Pure domain types, one container

## What you want to do

The shop's billing module calculates order totals with tax (`PricingConfig` and `InvoiceCalculator`). You want to test `InvoiceCalculator` directly in a unit test with custom values — without spinning up a container — and then wire the exact same struct into `BillingContainer` for production.

## The code

Here are the billing domain structs:

```example
name: ch06_pure_domain
region: domain
```

Here is the container that wires them for the application:

```example
name: ch06_pure_domain
region: container
```

And here is `main`, using `InvoiceCalculator` first as a standalone Rust struct with custom configuration and then through `BillingContainer`:

```example
name: ch06_pure_domain
region: run
```

It prints:

```output
name: ch06_pure_domain
```

## How it works

1. **No framework coupling in domain code:** Neither `PricingConfig` nor `InvoiceCalculator` imports `snapfit`. If your domain types live in a separate `shop-domain` crate, that crate does not even list `snapfit` in its `Cargo.toml`.
2. **Direct construction in tests:** Because `InvoiceCalculator::new` is an ordinary Rust function taking `Arc<PricingConfig>`, a unit test can construct `PricingConfig { tax_basis_points: 500 }` directly and pass it in with zero ceremony.
3. **Zero-boilerplate wiring in production:** When `BillingContainer::new()` runs, it calls `PricingConfig::new()`, wraps the result in `Arc`, and passes a clone into `InvoiceCalculator::new(...)`.

## Which problem this solves

1. **It cannot be tested alone** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): `InvoiceCalculator` is built directly, as a unit test would build it, with its own `PricingConfig`, without a container and without any real infrastructure.
2. **The service picks its own dependencies** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): `InvoiceCalculator::new` receives its configuration. Whether that is the production configuration or a test's is decided by its caller.
3. **Services drift back to choosing** ([Chapter 3](03-who-builds-everything.md)): nothing in `InvoiceCalculator` refers to the container, so it cannot ask the container for anything.

## Remember

1. A domain struct wired by `snapfit` is indistinguishable from a domain struct written without `snapfit`.
2. Unit tests for a single service can call `Service::new(...)` directly without building the full container.
3. By convention, `#[singleton]` without `constructor = <fn>` calls `<Type>::new(...)`.
