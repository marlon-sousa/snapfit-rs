# When building can fail

So far, every constructor in our shop has returned its value and nothing else. Real services aren't always that lucky. A database can be down when the shop starts, a configuration can name a payment provider that doesn't exist, and a file can be missing. Let's teach the container to hear "no" from a constructor, and to tell us which one said it.

## A database that may not answer

Our database connects when it is built. If nothing answers at its address, there is no database to hand to anyone, so `connect` returns a `Result`, with an error of its own that says what went wrong:

Rust, from the program [`ch15_when_building_can_fail`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/examples/ch15_when_building_can_fail.rs)

```rust
#[derive(Debug)]
pub struct ConnectError {
    pub address: String,
}

impl fmt::Display for ConnectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "nothing answered at {}", self.address)
    }
}

impl Error for ConnectError {}

pub struct Database {
    address: String,
}

impl Database {
    pub fn connect(config: Arc<ShopConfig>) -> Result<Self, ConnectError> {
        let address = config.database_address.clone();
        if DATABASE_ANSWERS.load(Ordering::SeqCst) {
            Ok(Self { address })
        } else {
            Err(ConnectError { address })
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }
}
```

`ConnectError` is ordinary Rust. It implements `Display` and `Error`, as any error type would, and it knows nothing about `snapfit`. To keep the example running the same way every time, a flag stands in for the network, and we can pull the cable whenever we like:

Rust, from the program [`ch15_when_building_can_fail`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/examples/ch15_when_building_can_fail.rs)

```rust
/// Stands in for the network: whether the database answers when the shop starts.
static DATABASE_ANSWERS: AtomicBool = AtomicBool::new(true);
```

Until now, a constructor like this one would have had two bad options. It could unwrap the `Result` and panic, stopping the whole program with a message about a database but nothing about which part of the shop needed it. Or it could hide the failure inside the value, and let every service that uses the database find out later, one request at a time.

## Saying that a field can fail

Here is the container. Two of its fields say `fallible = true`:

Rust, from the program [`ch15_when_building_can_fail`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/examples/ch15_when_building_can_fail.rs)

```rust
#[container]
pub struct ShopContainer {
    #[singleton(build = constructor(load))]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config], fallible = true, build = constructor(connect))]
    db: Arc<Database>,

    #[singleton(deps = [config], fallible = true, build = factory)]
    payments: Arc<dyn PaymentGateway>,

    #[singleton(deps = [db, payments])]
    orders: Arc<OrderService>,
}
```

`fallible = true` tells the container: "what builds this field returns a `Result`, so be ready for an error." That changes what the container gives us. A container with a fallible field has no `new()`, because `new()` promises a container that is always there. Instead it has `try_new()`, which returns a `Result` too: the container, when every field was built, or the first error, when one wasn't.

The payment gateway is built by a factory, as in [Chapter 11](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/11-choosing-an-implementation.md). Because the field is fallible, its factory returns a `Result` as well, and says no when the configuration names a provider we don't have:

Rust, from the program [`ch15_when_building_can_fail`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/examples/ch15_when_building_can_fail.rs)

```rust
impl ShopContainerFactories for ShopContainer {
    fn build_payments(
        inputs: ShopContainerPaymentsInputs,
    ) -> Result<Arc<dyn PaymentGateway>, Box<dyn Error + Send + Sync>> {
        let ShopContainerPaymentsInputs { config } = inputs;
        match config.payment_provider.as_str() {
            "stripe" => Ok(Arc::new(StripeGateway)),
            other => Err(format!("no payment provider is called {other}").into()),
        }
    }
}
```

The error of a factory is a `Box<dyn Error + Send + Sync>`, so it can be any error you like. Here it's a plain sentence, turned into an error with `into()`.

## Opening the shop twice

Let's open the shop once with the database answering, then pull the cable and try again:

Rust, from the program [`ch15_when_building_can_fail`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/examples/ch15_when_building_can_fail.rs)

```rust
fn main() {
    open_the_shop();
    DATABASE_ANSWERS.store(false, Ordering::SeqCst);
    open_the_shop();
}

fn open_the_shop() {
    match ShopContainer::try_new() {
        Ok(shop) => println!("The shop is open: {}.", shop.get_orders().describe()),
        Err(error) => {
            println!("The shop could not open: {error}.");
            println!("The field that failed: {}.", error.field());
            if let Some(cause) = error.source() {
                println!("Its own error: {cause}.");
            }
        }
    }
}
```

It prints:

What `ch15_when_building_can_fail` prints, recorded in [`ch15_when_building_can_fail.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/expected/ch15_when_building_can_fail.txt)

```text
The shop is open: orders are stored at orders.db.internal:5432 and paid through Stripe.
The shop could not open: snapfit could not build the field `db` of `ShopContainer`.
The field that failed: db.
Its own error: nothing answered at orders.db.internal:5432.
```

The first time, every field was built, and `try_new()` gave us the shop. The second time, the database didn't answer, and `try_new()` gave us an error instead. Notice what that error tells us:

1. **Which field failed, and in which container.** Its message names `db` and `ShopContainer`, and `error.field()` and `error.container()` return those names, for a log or a metric.
2. **Why it failed, in the field's own words.** `error.source()` is the `ConnectError` that `connect` returned, kept whole. Nothing was turned into a string along the way, so you can still downcast it to `ConnectError` if you want to react to it.
3. **What happened to everything else.** Fields are built in the order they are declared, each after the fields it needs, and the first failure stops the build. `config` was built before `db`, and was dropped when `try_new()` returned. `payments` and `orders` were never built at all.

Think of a pilot's checklist before take-off. Every item is checked in order, and if one fails, the plane stays at the gate, and the pilot says which item failed. Nobody takes off hoping that the engine will sort itself out in the air.

The error is a `snapfit::BuildError`, and `ShopContainer` implements `snapfit::FallibleContainer`, the trait with `try_new()`, the way a container that cannot fail implements `Container`. Our services still name none of them: `Database` returns its own `ConnectError`, and the container does the wrapping, because only the container knows which field it was building.

## What the compiler checks

`fallible = true` is a promise about what the constructor returns, and the compiler holds us to it.

1. **A fallible field's constructor must return a `Result`.** If `connect` returned a plain `Database`, the build would stop at the field's `fallible = true`, with the message "the field is marked `fallible = true`, so what builds it must return a `Result`, not `Database`".
2. **Its error must be an error.** Any error that converts into `Box<dyn Error + Send + Sync>` works: a type that implements `Error`, `Send` and `Sync` and holds no borrowed data, a `String`, or an `anyhow::Error`. Something that isn't an error, such as `()`, is refused.
3. **A `Result` of the wrong value is refused too.** `Result<Settings, E>` for a field of type `Arc<ShopConfig>` is a type mismatch, at the field.
4. **`build = default` can't fail.** `Default::default()` returns the value itself, so a field can't say both `build = default` and `fallible = true`.

The last rule has a cousin. In [Chapter 7](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/07-wiring-dependencies-and-constructors.md), `leaf_build = default` built every field that needs nothing and says nothing about how it is built. A fallible field can't be one of them, because the function `leaf_build` chooses for every leaf can't fail. The one exception is `leaf_build = constructor(new)`, which is what a container does without it. So the container refuses the field, and asks it to say how it is built:

Rust, the program [`ch15_fallible_leaf_build`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/compile-errors/ch15_fallible_leaf_build.rs). **Does not compile**

```rust
//! Chapter 15: a fallible field that the container's `leaf_build` would build.

use snapfit::container;
use std::sync::Arc;

#[derive(Default)]
pub struct DiscountCodes;

pub struct ShopConfig;

impl ShopConfig {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
}

#[container(leaf_build = default)]
pub struct ShopContainer {
    #[singleton]
    discounts: Arc<DiscountCodes>,

    #[singleton(fallible = true)]
    config: Arc<ShopConfig>,
}

fn main() {}
```

What the compiler says about `ch15_fallible_leaf_build`, recorded in [`ch15_fallible_leaf_build.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/compile-errors/ch15_fallible_leaf_build.txt)

```text
error: field 'config' is built with the container's leaf_build, since it has no deps and no build of its own, and leaf_build cannot fail; give the field a build of its own, such as build = constructor(new), or remove fallible = true
  --> book/compile-errors/ch15_fallible_leaf_build.rs:22:5
   |
22 |     #[singleton(fallible = true)]
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

Writing `build = constructor(new)` on `config` fixes it: the field now says, in plain sight, that its own `new` builds it, and that `new` returns a `Result`.

## Which problem this solves

1. **Ordering by hand** ([Chapter 3](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/03-who-builds-everything.md)): a composition root written by hand has to check each step's `Result` in the right order and stop at the first failure; the container does it in the order it works out, drops what it already built, and says which field failed.
2. **An invisible graph** ([Chapter 3](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/03-who-builds-everything.md)): which components can fail is written on the container, next to how each one is built, instead of being hidden in the constructors.

## Remember

1. Mark a field `fallible = true` when what builds it, `new`, a named constructor, a builder or a factory, returns a `Result`; its error can be any error type, a `String` or an `anyhow::Error`.
2. A container with a fallible field is built with `try_new()`, which returns the container or the first `snapfit::BuildError`; it has no `new()`.
3. A `BuildError` names the field and the container, and its `source()` is the field's own error, kept whole.
4. A fallible factory returns `Result<FieldType, Box<dyn Error + Send + Sync>>`.
5. `build = default` can't build a fallible field, and neither can a `leaf_build` other than `constructor(new)`; give the field a `build` of its own, such as `build = constructor(new)`.
