# Mistakes caught by the compiler

## What you want to do

When someone mistypes a dependency name, introduces a circular dependency between two services, or configures conflicting lifecycle attributes on a container field, you want `cargo build` to stop immediately with a clear error message naming the problem — rather than failing at runtime when a customer places an order.

## The code

### 1. Referencing a dependency name that does not exist

If `db` lists `deps = [confog]` instead of `config`, `wireup` rejects the container and lists the valid dependency names the container can wire:

```compile-error
name: ch09_unknown_dependency
```

### 2. Circular dependency between two components

If `orders` depends on `inventory` and `inventory` depends on `orders`, Kahn's algorithm detects the cycle at macro expansion time and reports the exact cycle path:

```compile-error
name: ch09_dependency_cycle
```

### 3. A component that depends on itself

If `ledger` lists itself in `deps = [ledger]`, `wireup` catches the self-dependency cycle:

```compile-error
name: ch09_self_dependency_cycle
```

### 4. Conflicting lifecycle attributes on one field

Declaring both `#[singleton]` and `#[transient]` on the same field is refused:

```compile-error
name: ch09_conflicting_lifecycle
```

### 5. Asynchronous dependency inside a forced synchronous container

If a container sets `#[container(mode = "sync")]` while one of its fields is marked `async`, compilation fails:

```compile-error
name: ch09_sync_mode_with_async_dep
```

### 6. Marking a transient dependency as `async`

Because runtime getters (`get_<name>()`) are always synchronous, a `#[transient]` field cannot be `async`:

```compile-error
name: ch09_async_transient
```

### 7. Applying `#[container]` to a tuple struct

Because `wireup` uses field names to identify nodes in the dependency graph and generate `get_<name>()` methods, tuple structs are refused:

```compile-error
name: ch09_tuple_struct_container
```

## How it works

1. **AST to pure domain IR:** When `rustc` expands `#[container]`, the inbound parser validates the struct's shape and field attributes before constructing a pure intermediate representation (`ContainerIR`).
2. **Kahn's algorithm and DFS cycle tracing:** The graph engine checks that every prerequisite in `deps` matches a declared field name, runs Kahn's topological sort, and — if unvisited nodes remain — runs a depth-first search to extract the minimal cycle path (`orders -> inventory -> orders`).
3. **Diagnostic emission:** Any domain error is converted into a `compile_error!` token stream attached to the container span, halting compilation before invalid code can ever be linked or deployed.

## Remember

1. When `wireup` reports `unknown dependency`, check the list of available field names at the end of the compiler message.
2. Break a dependency cycle by extracting the shared logic into a third pure domain struct that both services depend on.
3. If a dependency requires an `async` constructor, declare it as a `#[singleton(..., async)]` inside an asynchronous container so it resolves once during `init().await`.
