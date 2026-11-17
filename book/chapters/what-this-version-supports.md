# What this version supports

This edition of **The Snapfit Book** covers `snapfit` **`0.1.0`**. It needs Rust **`1.85.0`** or newer; your own crate can use any edition from 2018 on.

## Who 0.1.0 is for

`0.1.0` fits small and medium applications whose components can be built synchronously when the program starts: command-line tools, background workers, and web services whose clients and connection pools are created without waiting, such as an HTTP client or a lazily connecting database pool. It does one thing: it builds a graph of shared components from one struct, checked when the program compiles.

## What it does

| Capability | Details |
| :--- | :--- |
| **Plain domain types** | Services, repositories and controllers need no attribute, trait or import of `snapfit` ([Chapter 6](06-pure-domain-types.md)). |
| **Shared components** | Every field of a `#[container]` is built once, and its getter returns a clone of it, which for an `Arc<T>` field is the same value; fields can be `Arc<T>`, wrapped for you, or any `T: Clone` ([Chapter 5](05-installing-and-first-singleton.md), [Chapter 7](07-wiring-dependencies-and-constructors.md)). |
| **Dependencies by name** | `deps = [...]` lists the fields a component needs, passed to its constructor in that order ([Chapter 7](07-wiring-dependencies-and-constructors.md)). |
| **Named constructors** | `constructor = <fn>` calls a constructor other than `new` ([Chapter 7](07-wiring-dependencies-and-constructors.md)). |
| **Construction order worked out when compiling** | Fields can be declared in any order; each is built after the fields it needs ([Chapter 8](08-read-the-container-know-the-graph.md)). |
| **Several components of one type** | Fields are told apart by name, without wrapper types ([Chapter 9](09-multiple-instances-of-the-same-type.md)). |
| **Choosing an implementation** | A field's concrete type is the choice; services can take a trait object or a type parameter ([Chapter 10](10-choosing-an-implementation.md)). |
| **Mistakes refused when compiling** | Unknown dependencies and cycles, every one in the same build, each at its field ([Chapter 11](11-mistakes-caught-by-the-compiler.md)). |

## What it does not do

These are the questions an experienced reader asks first. `0.1.0` answers each one this way:

1. **A constructor cannot fail.** It returns the value, not a `Result`. A component that cannot start, such as a database that is down, can only panic, so the program stops at startup.
2. **Nothing is awaited.** Constructors are synchronous, so a client that has to be awaited before it is usable cannot be a field of the container.
3. **Nothing is passed in.** `new()` takes no arguments. A configuration component reads its own settings, for example from environment variables, in its constructor.
4. **A field cannot be a trait object.** A field typed `Arc<dyn PaymentGateway>` is refused, because the container has no constructor to call for it. Give the field the concrete type, and let the services take the trait object ([Chapter 10](10-choosing-an-implementation.md)).
5. **Every component is shared.** There is no field that builds a new value each time it is asked for.
6. **A test replaces a component by declaring its own container.** There is no way to swap one field of an existing container ([Chapter 10](10-choosing-an-implementation.md)).
7. **A container cannot be generic.** `struct ShopContainer<T>` is refused.

Some of these may come in later versions. We plan small, frequent releases rather than waiting for one large one, because `0.1.0` is already useful as it is. We do not promise what each release will hold, or when.
