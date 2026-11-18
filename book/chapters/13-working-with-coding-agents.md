# Working with coding agents

More and more code is written together with AI coding agents. An agent works best when it can find what it needs without reading the whole codebase, just like a new colleague on their first week. Let's see how the shape `snapfit` gives your code helps with that, by adding a feature the way an agent would.

## A new feature: a fraud check

The shop wants to hold large orders for review before checkout. That's a new component, `FraudGuard`, which reads the limit from the configuration, and a change to `CheckoutService`, which asks the guard before approving an order. All three stay plain Rust:

```example
name: ch13_agent_workflow
region: domain_step
```

Wiring the guard into the application is one new field in the container, and `guard` in the `deps` of `checkout`:

```example
name: ch13_agent_workflow
region: container
```

Let's try a small order and a large one:

```example
name: ch13_agent_workflow
region: run
```

It prints:

```output
name: ch13_agent_workflow
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

1. **Mixed responsibilities** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): a business rule lives in its service and the wiring lives in the container, so an agent changing one doesn't read or edit the other.
2. **An invisible graph** ([Chapter 3](03-who-builds-everything.md)): an agent learns how the whole application fits together by reading one struct.
3. **Hidden dependencies** ([Chapter 3](03-who-builds-everything.md)): the review rule above, that no domain module imports `snapfit` or builds its own dependencies, keeps every service's needs in its constructor, where anyone can see them.

## Remember

1. Keep domain modules and the `#[container]` in separate modules or crates, so an agent has clear boundaries.
2. Let `cargo check` verify every wiring change: if a name is wrong or a circle appears, the compiler stops the agent right away.
