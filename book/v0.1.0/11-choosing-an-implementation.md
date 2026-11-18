# Choosing an implementation

Back in [Chapter 2](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/02-choosing-between-implementations.md) we wanted `OrderService` to charge cards through Stripe in production and through a fake in tests, and we found that somebody still had to make that choice. Now we have a container, so let's let it choose, in exactly one place.

## Two gateways and two services

The gateways are the ones from Chapter 2, sharing a trait:

Rust, from the program [`ch11_choosing_an_implementation`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/examples/ch11_choosing_an_implementation.rs)

```rust
pub trait PaymentGateway {
    fn charge(&self, cents: u64) -> String;
}

#[derive(Default)]
pub struct StripeGateway;

impl StripeGateway {
    pub fn new() -> Self {
        Self
    }
}

impl PaymentGateway for StripeGateway {
    fn charge(&self, cents: u64) -> String {
        format!("Stripe charged {cents} cents")
    }
}

#[derive(Default)]
pub struct FakeGateway;

impl FakeGateway {
    pub fn new() -> Self {
        Self
    }
}

impl PaymentGateway for FakeGateway {
    fn charge(&self, cents: u64) -> String {
        format!("fake gateway pretended to charge {cents} cents")
    }
}
```

The services take the trait in the two ways Chapter 2 showed. `OrderService` takes a trait object, `Arc<dyn PaymentGateway>`. `RefundService` takes a type parameter:

Rust, from the program [`ch11_choosing_an_implementation`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/examples/ch11_choosing_an_implementation.rs)

```rust
pub struct OrderService {
    payments: Arc<dyn PaymentGateway>,
}

impl OrderService {
    pub fn new(payments: Arc<dyn PaymentGateway>) -> Self {
        Self { payments }
    }

    pub fn place_order(&self, cents: u64) -> String {
        format!("order: {}", self.payments.charge(cents))
    }
}

pub struct RefundService<P: PaymentGateway> {
    payments: Arc<P>,
}

impl<P: PaymentGateway> RefundService<P> {
    pub fn new(payments: Arc<P>) -> Self {
        Self { payments }
    }

    pub fn refund(&self, cents: u64) -> String {
        format!("refund: {}", self.payments.charge(cents))
    }
}
```

## The container makes the choice

Each container makes the choice once, in the type of its `payments` field. `ShopContainer` is the production shop. `TestShopContainer` builds the same services around the fake:

Rust, from the program [`ch11_choosing_an_implementation`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/examples/ch11_choosing_an_implementation.rs)

```rust
#[container]
pub struct ShopContainer {
    #[singleton]
    payments: Arc<StripeGateway>,

    #[singleton(deps = [payments])]
    orders: Arc<OrderService>,

    #[singleton(deps = [payments])]
    refunds: Arc<RefundService<StripeGateway>>,
}

#[container]
pub struct TestShopContainer {
    #[singleton]
    payments: Arc<FakeGateway>,

    #[singleton(deps = [payments])]
    orders: Arc<OrderService>,

    #[singleton(deps = [payments])]
    refunds: Arc<RefundService<FakeGateway>>,
}
```

Let's run both:

Rust, from the program [`ch11_choosing_an_implementation`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/examples/ch11_choosing_an_implementation.rs)

```rust
fn main() {
    let shop = ShopContainer::new();
    println!("production {}", shop.get_orders().place_order(4500));
    println!("production {}", shop.get_refunds().refund(500));

    let test_shop = TestShopContainer::new();
    println!("test {}", test_shop.get_orders().place_order(4500));
    println!("test {}", test_shop.get_refunds().refund(500));
}
```

It prints:

What `ch11_choosing_an_implementation` prints, recorded in [`ch11_choosing_an_implementation.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/expected/ch11_choosing_an_implementation.txt)

```text
production order: Stripe charged 4500 cents
production refund: Stripe charged 500 cents
test order: fake gateway pretended to charge 4500 cents
test refund: fake gateway pretended to charge 500 cents
```

Put the two containers side by side. Apart from their names, they differ only in the gateway's type. That's the ride app from Chapter 2: the services asked for something that charges cards, and the container decided which car to send.

## What happens in each container

1. **The field's type is the choice.** `payments: Arc<StripeGateway>` tells the container to call `StripeGateway::new()`. The container is the only place that says Stripe.
2. **A service that takes a trait object.** `OrderService::new` takes an `Arc<dyn PaymentGateway>`. The generated `new()` passes it a clone of the `Arc<StripeGateway>` field, and Rust converts it, as it would in any function call. That's why `orders` has the same type, `Arc<OrderService>`, in both containers.
3. **A service that takes a type parameter.** `RefundService<P>` works with any gateway, so its field names the concrete one: `Arc<RefundService<StripeGateway>>`. The type parameter shows up in the container, where the choice is made, and nowhere else.
4. **A container for tests.** `TestShopContainer` declares the same fields with `FakeGateway`. A test builds it with `TestShopContainer::new()` and gets the real services around the fake. When a test is about one service only, building that service directly with a fake is often simpler still ([Chapter 6](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/06-pure-domain-types.md)).

## Swapping a component

Now imagine the shop moves from Stripe to another provider. You write a type that implements `PaymentGateway`, change `payments: Arc<StripeGateway>` to that type, and recompile. For services that take a trait object, that's the whole change. A service with a type parameter names the gateway in its own field too, as `Arc<RefundService<StripeGateway>>` does, so that's one more line, still inside the container.

That isn't a special trick for payment gateways. It works for any field, and `TestShopContainer` shows why. Remember that the generated `new()` is ordinary Rust. Let's follow what it does with `FakeGateway`, and see what the new type had to offer:

1. **It has the constructor.** For `payments: Arc<FakeGateway>`, the container calls `FakeGateway::new()` and wraps the result in an `Arc`. So the new type needs to be built the way the attribute says: with `new` here, or with the constructor `build = constructor(...)` names, taking the fields in `deps`, in that order. For an `Arc<T>` field, that constructor lives on `T`.
2. **It can be cloned.** The getter and every field that needs `payments` receive a clone of it. An `Arc` can always be cloned; a field of any other type must implement `Clone`.
3. **The fields that need it accept it.** `orders` and `refunds` receive a clone of `payments`. `OrderService::new` takes an `Arc<dyn PaymentGateway>`, and `FakeGateway` implements `PaymentGateway`, so Rust converts it. `RefundService<P>` is generic over the gateway, so it accepts it too, and its field names the new type: `Arc<RefundService<FakeGateway>>`. A constructor that asked for exactly `Arc<FakeGateway>` would accept it as well.

That's the whole rule: you can change a field's type to any type that has the same constructor, can be cloned, and is accepted by the fields that need it. No service changes, and no attribute does. The only other line that changes is the type of a field that is generic over the new type, like `refunds`. If the new type doesn't fit, the program doesn't compile, so you find out right away.

## Choosing while the program starts

So far the choice was made when we wrote the code: one container says Stripe, the other says fake. But a real shop often decides when it starts. The staging server should use the fake, production should use Stripe, and the configuration says which. Now the container itself can't name a concrete type for `payments`, because it doesn't know which one it will be until the configuration has been read.

What we'd like is a field that is simply "something that charges cards", `Arc<dyn PaymentGateway>`. Let's first see what happens if we just write that, with a plain `#[singleton]`:

Rust, the program [`ch11_trait_object_field`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch11_trait_object_field.rs). **Does not compile**

```rust
use snapfit::container;
use std::sync::Arc;

pub trait PaymentGateway {
    fn charge(&self, cents: u64) -> String;
}

pub struct OrderService;

impl OrderService {
    pub fn new(_payments: Arc<dyn PaymentGateway>) -> Self {
        Self
    }
}

#[container]
pub struct ShopContainer {
    #[singleton]
    payments: Arc<dyn PaymentGateway>,

    #[singleton(deps = [payments])]
    orders: Arc<OrderService>,
}
```

What the compiler says about `ch11_trait_object_field`, recorded in [`ch11_trait_object_field.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch11_trait_object_field.txt)

```text
error: field 'payments' has a trait-object type, which has no constructor; add build = factory to the field and build it in the build_payments method of the container's factories trait
  --> book/compile-errors/ch11_trait_object_field.rs:18:5
   |
18 |     #[singleton]
   |     ^^^^^^^^^^^^
```

The container builds a plain field by calling a constructor of its type, and a trait object has no constructor: there is nothing to call. So the field is refused, and the message tells us what to do instead. Choosing an implementation is exactly the wiring logic that [Chapter 10](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/10-passing-dependencies-by-name.md) said belongs in a factory.

Here is the configuration, which says which provider to use. To keep the example short it derives `Default`, which asks for the fake; a real `ShopConfig` would read the setting in its constructor, from an environment variable for example:

Rust, from the program [`ch11_configured_payments`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/examples/ch11_configured_payments.rs)

```rust
#[derive(Clone, Copy, Default)]
pub enum PaymentProvider {
    Stripe,
    #[default]
    Fake,
}

#[derive(Clone, Default)]
pub struct ShopConfig {
    pub payments: PaymentProvider,
}
```

And the container, whose factory reads the configuration and picks the gateway:

Rust, from the program [`ch11_configured_payments`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/examples/ch11_configured_payments.rs)

```rust
#[container]
pub struct ConfiguredShopContainer {
    #[singleton(build = default)]
    config: ShopConfig,

    #[singleton(deps = [config], build = factory)]
    payments: Arc<dyn PaymentGateway>,

    #[singleton(deps = [payments])]
    orders: Arc<OrderService>,
}

impl ConfiguredShopContainerFactories for ConfiguredShopContainer {
    fn build_payments(inputs: ConfiguredShopContainerPaymentsInputs) -> Arc<dyn PaymentGateway> {
        let ConfiguredShopContainerPaymentsInputs { config } = inputs;
        match config.payments {
            PaymentProvider::Stripe => Arc::new(StripeGateway::new()),
            PaymentProvider::Fake => Arc::new(FakeGateway::new()),
        }
    }
}
```

Because a factory is an ordinary function, we can also call it on its own, with a configuration that asks for Stripe, without building any container. That's handy in a test:

Rust, from the program [`ch11_configured_payments`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/examples/ch11_configured_payments.rs)

```rust
fn factory_alone() {
    let inputs = ConfiguredShopContainerPaymentsInputs {
        config: ShopConfig {
            payments: PaymentProvider::Stripe,
        },
    };
    let gateway = ConfiguredShopContainer::build_payments(inputs);
    println!("factory alone: {}", gateway.charge(100));
}
```

Let's run it:

Rust, from the program [`ch11_configured_payments`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/examples/ch11_configured_payments.rs)

```rust
fn main() {
    let shop = ConfiguredShopContainer::new();
    println!("configured {}", shop.get_orders().place_order(4500));
    println!("configured gateway: {}", shop.get_payments().charge(100));
    factory_alone();
}
```

It prints:

What `ch11_configured_payments` prints, recorded in [`ch11_configured_payments.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/expected/ch11_configured_payments.txt)

```text
configured order: fake gateway pretended to charge 4500 cents
configured gateway: fake gateway pretended to charge 100 cents
factory alone: Stripe charged 100 cents
```

The configuration asks for the fake, so `orders` charges through the fake, and the factory on its own picks Stripe when asked.

You may recognise that `match` from [Chapter 2](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/02-choosing-between-implementations.md), where a closed list of providers made every service change when a provider was added. Here the list lives in one place only, the wiring. Adding a provider means a new type and one more line in this factory; `OrderService` and every other service stay as they are.

Think of a theatre with an understudy. The play is the same every night, and the part is the same, but which actor walks on is decided that evening, by the stage manager, from the evening's call sheet. `OrderService` is the play: it doesn't care who plays the gateway.

## Which way to choose

1. **Name the concrete type in the field when the choice is fixed.** It is the shortest, the container needs no code, and a second container covers tests. That's `ShopContainer` and `TestShopContainer`.
2. **Use a factory when the field should be the trait itself.** Typically, that's because the choice is made while the program starts, from configuration. The field is `Arc<dyn PaymentGateway>`, so its getter returns the trait object too.

## Which problem this solves

1. **Tight coupling** ([Chapter 1](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/01-a-service-that-builds-its-own-dependencies.md)): no service names a gateway. The type of the `payments` field is the only choice.
2. **A closed list** ([Chapter 2](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/02-choosing-between-implementations.md)): adding a provider means writing a type that implements `PaymentGateway` and naming it in a field. No service changes.
3. **Spreading type parameters** ([Chapter 2](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/02-choosing-between-implementations.md)): a service can take a trait object, and its type stays the same whatever the gateway. A service that takes a type parameter still carries it, but only the container names the concrete type.
4. **Hard to test** ([Chapter 1](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/01-a-service-that-builds-its-own-dependencies.md)): `TestShopContainer` builds the real services around a fake gateway.
5. **Someone still has to choose** ([Chapter 2](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/02-choosing-between-implementations.md)): the container chooses, once, in the types of its fields, or in a factory when the choice depends on the configuration.

## Remember

1. Name the implementation once, in the type of a container field, and let services take a trait object or a type parameter.
2. A field can change to any other type that has the same constructor, can be cloned, and is accepted by the fields that need it. No service or attribute changes; a field whose type is generic over it, such as `Arc<RefundService<StripeGateway>>`, names the new type too.
3. A trait object keeps a service's type the same whatever the gateway; a type parameter avoids the indirect call, and its concrete type is named in the container.
4. To build the application around different implementations, such as fakes for tests, declare a second container with the same fields.
5. When the choice is made while the program starts, type the field as the trait object and build it with `build = factory`; the factory reads the configuration and returns the implementation.
