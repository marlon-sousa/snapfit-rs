# Working with coding agents

More and more code is written together with AI coding agents. An agent works best when it can find what it needs without reading the whole codebase, just like a new colleague on their first week. Let's see how the shape `snapfit` gives your code helps with that, by adding a feature the way an agent would.

## A new feature: a fraud check

The shop wants to hold large orders for review before checkout. That's a new component, `FraudGuard`, which reads the limit from the configuration, and a change to `CheckoutService`, which asks the guard before approving an order. All three stay plain Rust:

Rust, from the program [`ch13_agent_workflow`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.1/book/examples/ch13_agent_workflow.rs)

```rust
#[derive(Default)]
pub struct ShopConfig {
    pub max_order_cents: u64,
}

impl ShopConfig {
    pub fn new() -> Self {
        Self {
            max_order_cents: 50_000,
        }
    }
}

pub struct FraudGuard {
    config: Arc<ShopConfig>,
}

impl FraudGuard {
    pub fn new(config: Arc<ShopConfig>) -> Self {
        Self { config }
    }

    pub fn allows(&self, cents: u64) -> bool {
        cents <= self.config.max_order_cents
    }
}

pub struct CheckoutService {
    guard: Arc<FraudGuard>,
}

impl CheckoutService {
    pub fn new(guard: Arc<FraudGuard>) -> Self {
        Self { guard }
    }

    pub fn process(&self, order_id: u32, cents: u64) -> String {
        if self.guard.allows(cents) {
            format!("order #{order_id}: approved ({cents} cents)")
        } else {
            format!("order #{order_id}: held for review ({cents} cents)")
        }
    }
}
```

Wiring the guard into the application is one new field in the container, and `guard` in the `deps` of `checkout`:

Rust, from the program [`ch13_agent_workflow`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.1/book/examples/ch13_agent_workflow.rs)

```rust
#[container]
pub struct ShopContainer {
    #[singleton]
    config: Arc<ShopConfig>,

    #[singleton(deps = [config])]
    guard: Arc<FraudGuard>,

    #[singleton(deps = [guard])]
    checkout: Arc<CheckoutService>,
}
```

Let's try a small order and a large one:

Rust, from the program [`ch13_agent_workflow`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.1/book/examples/ch13_agent_workflow.rs)

```rust
fn main() {
    let container = ShopContainer::new();
    let checkout = container.get_checkout();
    println!("{}", checkout.process(201, 12_000));
    println!("{}", checkout.process(202, 75_000));
}
```

It prints:

What `ch13_agent_workflow` prints, recorded in [`ch13_agent_workflow.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.1/book/expected/ch13_agent_workflow.txt)

```text
order #201: approved (12000 cents)
order #202: held for review (75000 cents)
```

## Two kinds of change, two places

Think of a map on the wall of a big building. You don't have to walk every corridor to find the meeting room. The container is that map for your application.

1. **To change a business rule**, such as how `FraudGuard::allows` judges an order, the agent opens `FraudGuard`, and nothing else. It doesn't need the container or any other service.
2. **To change how components are wired**, such as sending checkout through the guard, the agent opens the container and edits a `deps` list. The compiler checks right away that every name exists and that no circle appeared.

Code review gets simpler too. Whether the reviewer is a person or another agent, one rule catches most drift: no domain module imports `snapfit`, and no domain module builds its own dependencies.

## Rules to copy into your `AGENTS.md`

You can copy this section into your project's `AGENTS.md`:

```markdown
## Dependency Injection (`snapfit`) Rules

1. **Zero Domain Pollution:** Domain structs, services, repositories, and controllers must remain 100% pure Rust with standard `pub fn new(...)` (or named) constructors. Never add `snapfit` attributes or framework traits to domain types; a builder crate's attribute the team chose, such as bon's `#[builder]`, is fine.
2. **Centralized Container Manifest:** Declare every singleton (`#[singleton]`) and dependency edge (`deps = [...]`) inside the `#[container]` struct.
3. **Named Edges Over Wrapper Types:** When wiring multiple instances of the same type (such as `read_db` and `write_db`), declare separate named fields on the container rather than creating wrapper newtypes.
4. **Same-Typed Dependencies by Name:** When a constructor takes two or more dependencies of the same type, build the field with `build = builder` or `build = factory`, never with positional `deps`, so the order of `deps` no longer decides who gets what. Prefer a builder, where the compiler checks every setter's name; in a factory, name every input where it is passed.
5. **Logic Where It Belongs:** Logic about what a type is stays in its constructor or builder; logic about how the application is wired, such as choosing an implementation from configuration, goes in a factory next to the container.
```

## Which problem this solves

1. **Mixed responsibilities** ([Chapter 1](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.1/01-a-service-that-builds-its-own-dependencies.md)): a business rule lives in its service and the wiring lives in the container, so an agent changing one doesn't read or edit the other.
2. **An invisible graph** ([Chapter 3](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.1/03-who-builds-everything.md)): an agent learns how the whole application fits together by reading one struct.
3. **Hidden dependencies** ([Chapter 3](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.1/03-who-builds-everything.md)): the review rule above, that no domain module imports `snapfit` or builds its own dependencies, keeps every service's needs in its constructor, where anyone can see them.

## Remember

1. Keep domain modules and the `#[container]` in separate modules or crates, so an agent has clear boundaries.
2. Let `cargo check` verify every wiring change: if a name is wrong or a circle appears, the compiler stops the agent right away.
