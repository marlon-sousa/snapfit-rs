# Mistakes caught by the compiler

Everybody mistypes a name now and then, and sooner or later someone wires two services that need each other. The question is when you find out. With the service locator of [Chapter 3](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/03-who-builds-everything.md), you found out only when the program asked for the component. Let's make some mistakes on purpose and see when `snapfit` tells us.

Think of a building inspector who checks the plans before anyone moves in, rather than waiting for a wall to fall down.

## A name that doesn't exist

Say `db` lists `deps = [confog]` instead of `config`. The build stops, and the message lists the names the container does know:

Rust, the program [`ch12_unknown_dependency`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch12_unknown_dependency.rs). **Does not compile**

```rust
use std::sync::Arc;
use snapfit::container;

pub struct ShopConfig;
pub struct Database;

#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [confog])]
    db: Arc<Database>,
}
```

What the compiler says about `ch12_unknown_dependency`, recorded in [`ch12_unknown_dependency.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch12_unknown_dependency.txt)

```text
error: unknown dependency 'confog' on field 'db'. You must choose between the dependencies the container can wire: [config]
  --> book/compile-errors/ch12_unknown_dependency.rs:12:5
   |
12 |     #[singleton(deps = [confog])]
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

## Two components that need each other

Now `orders` needs `inventory`, and `inventory` needs `orders`. Neither can be built first. This circle is what the compiler calls a dependency cycle. The message describes it in a sentence, and points at the first field in it:

Rust, the program [`ch12_dependency_cycle`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch12_dependency_cycle.rs). **Does not compile**

```rust
use std::sync::Arc;
use snapfit::container;

pub struct OrderService;
pub struct InventoryService;

#[container]
pub struct CyclicShopContainer {
    #[singleton(deps = [inventory])]
    orders: Arc<OrderService>,

    #[singleton(deps = [orders])]
    inventory: Arc<InventoryService>,
}
```

What the compiler says about `ch12_dependency_cycle`, recorded in [`ch12_dependency_cycle.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch12_dependency_cycle.txt)

```text
error: dependency cycle: orders depends on inventory, which depends on orders
 --> book/compile-errors/ch12_dependency_cycle.rs:9:5
  |
9 |     #[singleton(deps = [inventory])]
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

## A component that needs itself

The smallest circle there is: `ledger` lists itself in `deps = [ledger]`:

Rust, the program [`ch12_self_dependency_cycle`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch12_self_dependency_cycle.rs). **Does not compile**

```rust
use std::sync::Arc;
use snapfit::container;

pub struct LedgerService;

#[container]
pub struct SelfCyclicContainer {
    #[singleton(deps = [ledger])]
    ledger: Arc<LedgerService>,
}
```

What the compiler says about `ch12_self_dependency_cycle`, recorded in [`ch12_self_dependency_cycle.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch12_self_dependency_cycle.txt)

```text
error: dependency cycle: ledger depends on itself
 --> book/compile-errors/ch12_self_dependency_cycle.rs:8:5
  |
8 |     #[singleton(deps = [ledger])]
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

## A constructor name in quotes

`build = constructor(<fn>)` takes the function's name as Rust, not as text. If you write it in quotes, the message shows you what to write instead:

Rust, the program [`ch12_quoted_constructor`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch12_quoted_constructor.rs). **Does not compile**

```rust
use snapfit::container;
use std::sync::Arc;

pub struct ShopConfig;

impl ShopConfig {
    pub fn new() -> Self {
        Self
    }
}

pub struct PaymentGateway;

impl PaymentGateway {
    pub fn connect(_config: Arc<ShopConfig>) -> Self {
        Self
    }
}

#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config], build = constructor("connect"))]
    payments: Arc<PaymentGateway>,
}
```

What the compiler says about `ch12_quoted_constructor`, recorded in [`ch12_quoted_constructor.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch12_quoted_constructor.txt)

```text
error: constructor "connect" on field 'payments' must be written without quotes: build = constructor(connect)
  --> book/compile-errors/ch12_quoted_constructor.rs:25:5
   |
25 |     #[singleton(deps = [config], build = constructor("connect"))]
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

## Several mistakes at once

`snapfit` doesn't stop at the first problem. Every unknown name and every circle is its own error, at its own field, in the same `cargo build`, so you can fix them all in one go. Here `db` names a field that doesn't exist, and `orders` and `stock` need each other:

Rust, the program [`ch12_several_mistakes`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch12_several_mistakes.rs). **Does not compile**

```rust
use snapfit::container;
use std::sync::Arc;

pub struct ShopConfig;
pub struct Database;
pub struct OrderService;
pub struct StockService;

#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [confg])]
    db: Arc<Database>,

    #[singleton(deps = [stock])]
    orders: Arc<OrderService>,

    #[singleton(deps = [orders])]
    stock: Arc<StockService>,
}
```

What the compiler says about `ch12_several_mistakes`, recorded in [`ch12_several_mistakes.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch12_several_mistakes.txt)

```text
error: unknown dependency 'confg' on field 'db'. You must choose between the dependencies the container can wire: [config, orders, stock]
  --> book/compile-errors/ch12_several_mistakes.rs:14:5
   |
14 |     #[singleton(deps = [confg])]
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: dependency cycle: orders depends on stock, which depends on orders
  --> book/compile-errors/ch12_several_mistakes.rs:17:5
   |
17 |     #[singleton(deps = [stock])]
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

## A tuple struct

`snapfit` knows components by their field names, and uses those names for the getters. A tuple struct has no field names, so it is refused:

Rust, the program [`ch12_tuple_struct_container`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch12_tuple_struct_container.rs). **Does not compile**

```rust
use std::sync::Arc;
use snapfit::container;

pub struct ShopConfig;

#[container]
pub struct TupleShopContainer(#[singleton] Arc<ShopConfig>);
```

What the compiler says about `ch12_tuple_struct_container`, recorded in [`ch12_tuple_struct_container.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.0/book/compile-errors/ch12_tuple_struct_container.txt)

```text
error: tuple structs are not supported by #[container]; use named fields
 --> book/compile-errors/ch12_tuple_struct_container.rs:7:12
  |
7 | pub struct TupleShopContainer(#[singleton] Arc<ShopConfig>);
  |            ^^^^^^^^^^^^^^^^^^
```

## How the checks run

1. **First, `snapfit` reads the struct.** When the compiler expands `#[container]`, `snapfit` reads every field and its attribute. A field it can't read, such as one whose constructor name is in quotes, is reported at that field.
2. **Then it checks the graph.** It visits the fields in the order you declared them, and orders each one after the fields it needs. On the way, it reports every name in `deps` that isn't a field, and every circle, told from the first field of that circle you declared.
3. **Then it reports.** Each problem becomes its own error: a sentence that points at the attribute of the field it is about, or at the container's name when no single field is to blame. All the fields it can't read are reported together, and the graph is checked once every field can be read; then all the problems in the graph are reported together. `new()` and the getters are generated only once every problem is fixed.

## Which problem this solves

1. **Run-time failures** ([Chapter 3](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.0/03-who-builds-everything.md)): a dependency that doesn't exist, a circle, or a constructor name that can't be read stops `cargo build`, with an error at the field. The program never starts with a broken graph.

## Remember

1. When `snapfit` reports an unknown dependency, look at the list of field names at the end of the message.
2. To break a circle, move the logic both services share into a third plain struct, and let both depend on it.
3. Fix the errors in any order: each one is about its own field, or about the container. After you fix a field `snapfit` couldn't read, the next build may report problems in the graph that it couldn't check before.
