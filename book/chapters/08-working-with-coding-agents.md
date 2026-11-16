# Working with coding agents

## What you want to do

Your team works with AI coding agents to add features to the shop — such as inserting a `FraudGuard` check before checkout — and you want each change to stay localized so an agent never has to read or edit unrelated services to wire a new dependency.

## The code

Adding `FraudGuard` between `ShopConfig` and `CheckoutService` keeps all three structs pure Rust:

```example
name: ch08_agent_workflow
region: domain_step
```

And in `ShopContainer`, wiring `FraudGuard` into the graph is a single field declaration and an updated `deps = [guard]` on `checkout`:

```example
name: ch08_agent_workflow
region: container
```

Running the updated container verifies both an approved order and a held order:

```example
name: ch08_agent_workflow
region: run
```

It prints:

```output
name: ch08_agent_workflow
```

## How it works

1. **Separated responsibilities for agents:**
   - **To change a business rule** (for example, changing how `FraudGuard::allows` evaluates an order): the agent opens only `FraudGuard`. It does not need to read `ShopContainer` or any other service.
   - **To change how components are wired** (for example, routing `CheckoutService` through `FraudGuard`): the agent opens `ShopContainer` and updates `deps = [...]`. The compiler immediately verifies that all prerequisites exist and that no cycle was introduced.
2. **One reviewer rule:** During code review (by a human or an independent reviewer agent), one invariant catches architectural drift immediately: *no domain module may import `snapfit` or construct its own service dependencies internally*.

### Copy-ready rules for your repository's `AGENTS.md`

You can copy the following section directly into your project's `AGENTS.md`:

```markdown
## Dependency Injection (`snapfit`) Rules

1. **Zero Domain Pollution:** Domain structs, services, repositories, and controllers must remain 100% pure Rust with standard `pub fn new(...)` (or named) constructors. Never add `snapfit` attributes or framework traits to domain types.
2. **Centralized Container Manifest:** Declare all lifecycles (`#[singleton]`) and dependency edges (`deps = [...]`) inside the `#[container]` struct.
3. **Named Edges Over Wrapper Types:** When wiring multiple instances of the same type (such as `read_db` and `write_db`), declare separate named fields on the container rather than creating wrapper newtypes.
```

## Remember

1. Keep domain modules and the `#[container]` manifest in distinct modules or crates so agents have clear boundaries.
2. Let `cargo check` and `#[container]` verify every wiring change — if a prerequisite name is wrong or a cycle is created, the compiler stops the agent immediately.
