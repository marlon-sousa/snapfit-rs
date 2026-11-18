# Pure domain types, one container

Our shop needs to bill its customers, which means adding tax to every order. Let's write the billing code, and see how the same struct can be tested on its own and wired into a container, without changing a line of it.

## Billing, in plain Rust

Two structs: a pricing configuration with the tax rate, and a calculator that adds the tax to a total. The calculator receives its configuration in `new()`, as every service has since [Chapter 2](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/02-choosing-between-implementations.md):

Rust, from the program [`ch06_pure_domain`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/examples/ch06_pure_domain.rs)

```rust
#[derive(Default)]
pub struct PricingConfig {
    pub tax_basis_points: u64,
}

impl PricingConfig {
    pub fn new() -> Self {
        Self {
            tax_basis_points: 800,
        }
    }
}

pub struct InvoiceCalculator {
    config: Arc<PricingConfig>,
}

impl InvoiceCalculator {
    pub fn new(config: Arc<PricingConfig>) -> Self {
        Self { config }
    }

    pub fn total_with_tax(&self, net_cents: u64) -> u64 {
        let tax = (net_cents * self.config.tax_basis_points) / 10_000;
        net_cents + tax
    }
}
```

Look for `snapfit` in that code. It isn't there: no attribute, no trait, no import. These are domain types, and they know nothing about how they will be assembled.

## The container

The container wires them for the application. The calculator lists `config` in its `deps`, so the container passes the configuration to `InvoiceCalculator::new`:

Rust, from the program [`ch06_pure_domain`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/examples/ch06_pure_domain.rs)

```rust
#[container]
pub struct BillingContainer {
    #[singleton]
    config: Arc<PricingConfig>,

    #[singleton(deps = [config])]
    calculator: Arc<InvoiceCalculator>,
}
```

## The same struct, twice

Now let's use the calculator two ways. First on its own, the way a unit test would, with a configuration we made up for the occasion. Then through the container, with the configuration the application uses:

Rust, from the program [`ch06_pure_domain`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/examples/ch06_pure_domain.rs)

```rust
fn main() {
    let custom_config = Arc::new(PricingConfig {
        tax_basis_points: 500,
    });
    let standalone = InvoiceCalculator::new(custom_config);
    println!(
        "standalone total: {} cents",
        standalone.total_with_tax(10_000)
    );

    let container = BillingContainer::new();
    let wired = container.get_calculator();
    println!("container total: {} cents", wired.total_with_tax(10_000));
}
```

It prints:

What `ch06_pure_domain` prints, recorded in [`ch06_pure_domain.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/expected/ch06_pure_domain.txt)

```text
standalone total: 10500 cents
container total: 10800 cents
```

Five percent on its own, eight percent in the container. The calculator didn't notice the difference, and it shouldn't.

This is the lamp with a plug from [Chapter 1](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/01-a-service-that-builds-its-own-dependencies.md). The same lamp works in the living room socket and on the test bench, because it doesn't care where the electricity comes from.

## Why this matters

1. **No framework in your domain code.** If your domain types live in a separate crate, that crate doesn't even list `snapfit` in its `Cargo.toml`.
2. **Tests build what they need.** `InvoiceCalculator::new` is an ordinary function, so a test passes it any configuration it likes, with no container and no setup.
3. **The container is only for wiring.** When `BillingContainer::new()` runs, it calls `PricingConfig::new()`, wraps the result in an `Arc`, and passes a clone to `InvoiceCalculator::new(...)`. That's exactly what you would write by hand.

## Which problem this solves

1. **Hard to test** ([Chapter 1](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/01-a-service-that-builds-its-own-dependencies.md)): the calculator is built directly, as a unit test builds it, with its own configuration, without a container and without any real infrastructure.
2. **Tight coupling** ([Chapter 1](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/01-a-service-that-builds-its-own-dependencies.md)): `InvoiceCalculator::new` receives its configuration. Whether that is the production configuration or a test's is decided by whoever calls it.
3. **Hidden dependencies** ([Chapter 3](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/03-who-builds-everything.md)): nothing in `InvoiceCalculator` refers to the container, so it can't ask the container for anything. Its constructor says everything it needs.

## Remember

1. A domain struct wired by `snapfit` looks exactly like one written without it.
2. A unit test of one service calls `Service::new(...)` directly, without building the container.
3. Without `build`, `#[singleton]` calls `<Type>::new(...)`.
