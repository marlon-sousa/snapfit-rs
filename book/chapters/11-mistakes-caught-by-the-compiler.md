# Mistakes caught by the compiler

## What you want to do

When someone mistypes a dependency name, introduces a circular dependency between two services, or writes an attribute `snapfit` cannot read, you want `cargo build` to stop immediately with a clear error message naming the problem — rather than failing at runtime when a customer places an order.

## The code

### 1. Referencing a dependency name that does not exist

If `db` lists `deps = [confog]` instead of `config`, `snapfit` rejects the container and lists the valid dependency names the container can wire:

```compile-error
name: ch11_unknown_dependency
```

### 2. Circular dependency between two components

If `orders` depends on `inventory` and `inventory` depends on `orders`, `snapfit` refuses the container. The message names the cycle as a sentence and points at the first field of the cycle:

```compile-error
name: ch11_dependency_cycle
```

### 3. A component that depends on itself

If `ledger` lists itself in `deps = [ledger]`, `snapfit` catches the self-dependency cycle:

```compile-error
name: ch11_self_dependency_cycle
```

### 4. A constructor name written in quotes

`constructor = <fn>` takes the function's bare name, written as Rust rather than as a string. Writing it in quotes is refused, and the message shows what to write instead:

```compile-error
name: ch11_quoted_constructor
```

### 5. Several mistakes at once

`snapfit` does not stop at the first problem in the graph. Every unknown dependency and every cycle is its own error, at its own field, in the same `cargo build`. Here `db` names a field that does not exist, and `orders` and `stock` need each other:

```compile-error
name: ch11_several_mistakes
```

### 6. Applying `#[container]` to a tuple struct

Because `snapfit` uses field names to identify nodes in the dependency graph and generate `get_<name>()` methods, tuple structs are refused:

```compile-error
name: ch11_tuple_struct_container
```

## How it works

1. **Reading the struct:** When `rustc` expands `#[container]`, `snapfit` reads every field and its attributes. A field it cannot read, such as one whose constructor name is quoted, is reported at that field.
2. **Checking the graph:** `snapfit` then visits the fields in the order they are declared and orders each one after the fields it needs. On the way, it reports every name in `deps` that is not a field of the container, and every cycle, told from the cycle's first declared field.
3. **Reporting:** Each problem becomes its own compiler error, a sentence that points at the attribute of the field it is about, or at the container's name when no single field is at fault. The fields that cannot be read are reported together, and the graph is checked only once every field reads; then every problem in the graph is reported together. `new()` and the getters are generated only once every problem is fixed.

## Remember

1. When `snapfit` reports `unknown dependency`, check the list of available field names at the end of the compiler message.
2. Break a dependency cycle by extracting the shared logic into a third pure domain struct that both services depend on.
3. Fix the errors in any order: each one is about its own field, or about the container. Once you fix a field `snapfit` could not read, the next `cargo build` may report problems in the graph that it could not check before.
