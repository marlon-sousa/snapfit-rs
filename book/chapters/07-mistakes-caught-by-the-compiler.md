# Mistakes caught by the compiler

## What you want to do

When someone mistypes a dependency name, introduces a circular dependency between two services, or configures conflicting lifecycle attributes on a container field, you want `cargo build` to stop immediately with a clear error message naming the problem — rather than failing at runtime when a customer places an order.

## The code

### 1. Referencing a dependency name that does not exist

If `db` lists `deps = [confog]` instead of `config`, `snapfit` rejects the container and lists the valid dependency names the container can wire:

```compile-error
name: ch07_unknown_dependency
```

### 2. Circular dependency between two components

If `orders` depends on `inventory` and `inventory` depends on `orders`, the graph engine detects the cycle at macro expansion time and reports the exact cycle path:

```compile-error
name: ch07_dependency_cycle
```

### 3. A component that depends on itself

If `ledger` lists itself in `deps = [ledger]`, `snapfit` catches the self-dependency cycle:

```compile-error
name: ch07_self_dependency_cycle
```

### 4. Conflicting lifecycle attributes on one field

Declaring both `#[singleton]` and `#[transient]` on the same field is refused:

```compile-error
name: ch07_conflicting_lifecycle
```

### 5. Using an unstable API without opting in

Transient dependencies, asynchronous containers and custom builders work, but their API may still change, so `v0.1.0` keeps them behind the `unstable` feature of the `snapfit` crate. Using one without that feature names the API and how to enable it:

```compile-error
name: ch07_unstable_transient
```

### 6. Using a planned feature

Some attributes in the specification, such as `#[dispatch(...)]`, are designed but not implemented yet. Rather than ignoring them, `snapfit` refuses them:

```compile-error
name: ch07_planned_dispatch
```

### 7. Applying `#[container]` to a tuple struct

Because `snapfit` uses field names to identify nodes in the dependency graph and generate `get_<name>()` methods, tuple structs are refused:

```compile-error
name: ch07_tuple_struct_container
```

## How it works

1. **AST to pure domain IR:** When `rustc` expands `#[container]`, the inbound parser validates the struct's shape and field attributes before constructing a pure intermediate representation (`ContainerIR`).
2. **Topological sort and cycle tracing:** The graph engine checks that every prerequisite in `deps` matches a declared field name and sorts the graph. If the graph has a cycle, it finds the strongly connected component and traces one concrete cycle through it (`orders -> inventory -> orders`).
3. **Diagnostic emission:** Any domain error is converted into a `compile_error!` that underlines the attribute of the field that caused it (or the container's name when no single field is at fault), halting compilation before invalid code can ever be linked or deployed.

## Remember

1. When `snapfit` reports `unknown dependency`, check the list of available field names at the end of the compiler message.
2. Break a dependency cycle by extracting the shared logic into a third pure domain struct that both services depend on.
3. An error ending in "is unstable in this version of snapfit" means the API exists behind `features = ["unstable"]`; one ending in "is planned for a future version" means it is not implemented yet.
