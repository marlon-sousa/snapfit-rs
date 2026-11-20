# Mistakes caught by the compiler

Everybody mistypes a name now and then, and sooner or later someone wires two services that need each other. The question is when you find out. With the service locator of [Chapter 3](03-who-builds-everything.md), you found out only when the program asked for the component. Let's make some mistakes on purpose and see when `snapfit` tells us.

Think of a building inspector who checks the plans before anyone moves in, rather than waiting for a wall to fall down.

## A name that doesn't exist

Say `db` lists `deps = [confog]` instead of `config`. The build stops, and the message lists the names the container does know:

```compile-error
name: ch13_unknown_dependency
```

## Two components that need each other

Now `orders` needs `inventory`, and `inventory` needs `orders`. Neither can be built first. This circle is what the compiler calls a dependency cycle. The message describes it in a sentence, and points at the first field in it:

```compile-error
name: ch13_dependency_cycle
```

## A component that needs itself

The smallest circle there is: `ledger` lists itself in `deps = [ledger]`:

```compile-error
name: ch13_self_dependency_cycle
```

## A constructor name in quotes

`build = constructor(<fn>)` takes the function's name as Rust, not as text. If you write it in quotes, the message shows you what to write instead:

```compile-error
name: ch13_quoted_constructor
```

## Several mistakes at once

`snapfit` doesn't stop at the first problem. Every unknown name and every circle is its own error, at its own field, in the same `cargo build`, so you can fix them all in one go. Here `db` names a field that doesn't exist, and `orders` and `stock` need each other:

```compile-error
name: ch13_several_mistakes
```

## A tuple struct

`snapfit` knows components by their field names, and uses those names for the getters. A tuple struct has no field names, so it is refused:

```compile-error
name: ch13_tuple_struct_container
```

## How the checks run

1. **First, `snapfit` reads the struct.** When the compiler expands `#[container]`, `snapfit` reads every field and its attribute. A field it can't read, such as one whose constructor name is in quotes, is reported at that field.
2. **Then it checks the graph.** It visits the fields in the order you declared them, and orders each one after the fields it needs. On the way, it reports every name in `deps` that isn't a field, and every circle, told from the first field of that circle you declared.
3. **Then it reports.** Each problem becomes its own error: a sentence that points at the attribute of the field it is about, or at the container's name when no single field is to blame. All the fields it can't read are reported together, and the graph is checked once every field can be read; then all the problems in the graph are reported together. `new()` or `try_new()`, and the getters, are generated only once every problem is fixed.

## Which problem this solves

1. **Run-time failures** ([Chapter 3](03-who-builds-everything.md)): a dependency that doesn't exist, a circle, or a constructor name that can't be read stops `cargo build`, with an error at the field. The program never starts with a broken graph.

## Remember

1. When `snapfit` reports an unknown dependency, look at the list of field names at the end of the message.
2. To break a circle, move the logic both services share into a third plain struct, and let both depend on it.
3. Fix the errors in any order: each one is about its own field, or about the container. After you fix a field `snapfit` couldn't read, the next build may report problems in the graph that it couldn't check before.
