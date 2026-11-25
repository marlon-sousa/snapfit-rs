<!-- Written by `cargo xtask crate-readme` from the README at the repository's root. Edit that one, then run it again. -->

# snapfit (`snapfit-rs`)

> **Zero-Cost, Compile-Time Verified, Pure-Rust Dependency Injection Framework**

Its source, its issues, and how to contribute are in [its repository on GitHub](https://github.com/marlon-sousa/snapfit-rs).


---

## What is `snapfit`?

`snapfit` is a compile-time verified dependency injection framework designed specifically for idiomatic Rust.

Unlike dependency injection frameworks ported from Java (Spring, Dagger) or C# (.NET) that scatter `#[inject]` and `#[component]` attributes across your business logic, or dynamic service locators that rely on `TypeMap` and `dyn Any` downcasting at runtime, `snapfit` inverts the dependency declaration: **the container struct declares the wiring, leaving your domain structs 100% pure Rust.**

---

## Why snapfit?

Most Rust applications eventually face three unsatisfying choices for wiring components:

1. **Invasive component macros (`#[inject]`, `#[component]`):** Every service, repository, and controller has to import DI macros and derive framework traits. Your domain is coupled to the DI framework, and seeing the whole dependency graph requires opening dozens of files.
2. **Dynamic `TypeMap` service locators (`Arc<dyn Any + Send + Sync>`):** Missing dependencies fail at runtime, resolution pays lookup and downcasting overhead, and registering two instances of the same type (`read_db: Database` and `write_db: Database`) silently overwrites the first unless you invent wrapper newtypes.
3. **Hand-rolled `main.rs` initialization chains:** Dozens of manual `Arc::clone` calls obscure which components are shared singletons and what depends on what.

| Guarantee | Traditional Component DI | Dynamic `TypeMap` Locators | `snapfit` |
| :--- | :--- | :--- | :--- |
| **Domain pollution** | Macros and traits on every service | Services or handlers pull from locator | **Zero**: domain structs are plain Rust |
| **Where the graph lives** | Scattered across dozens of files | Hidden inside runtime registration calls | **One central `#[container]` struct** |
| **Multiple instances of one type** | Requires qualifier macros on structs | Overwrites previous entry unless wrapped in newtypes | **Native**: edges name container fields (`deps = [read_db, write_db]`) |
| **Cycle & missing-dep detection** | Build script or runtime failure | Panics or returns `None` at runtime | **Compile-time**, with the error on the offending field |
| **Resolution cost** | Vtable or generated factory | Hash lookup + `dyn Any` downcast | **Zero-cost** direct getters (`get_<name>()`) |

---

## Releases and The Snapfit Book

**The Snapfit Book** walks through the framework step by step with complete, compiled Rust programs for each release.

Each release's edition of the book is on [the snapfit site](https://marlon-sousa.github.io/snapfit-rs/releases/).


---

## Quick Look

Add snapfit to your `Cargo.toml`:

```toml
[dependencies]
snapfit = "0.3"
```

Then declare the container:

```rust
use std::sync::Arc;
use snapfit::container;

// 1. Your domain types are plain Rust: no macros, no framework traits.
pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn new() -> Self {
        Self { database_url: "postgres://localhost/shop".to_string() }
    }
}

pub struct Database {
    pub url: String,
}

impl Database {
    pub fn connect(config: Arc<Config>) -> Self {
        Self { url: config.database_url.clone() }
    }
}

pub struct UserService {
    db: Arc<Database>,
}

impl UserService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

// 2. The container declares the dependency graph in one place:
#[container]
pub struct AppContainer {
    // Leaf singleton: calls Config::new()
    #[singleton]
    config: Arc<Config>,

    // Named constructor: calls Database::connect(config)
    #[singleton(deps = [config], build = constructor(connect))]
    db: Arc<Database>,

    // Calls UserService::new(db)
    #[singleton(deps = [db])]
    users: Arc<UserService>,
}

// 3. Build once, then use the typed getters:
fn main() {
    let container = AppContainer::new();
    let users = container.get_users();
    assert!(Arc::ptr_eq(&container.get_db(), &users.db));
}
```

---

## What Works Today

Each line links to the chapter of The Snapfit Book that teaches it.

* **Plain domain types:** services, repositories and controllers need no attribute, trait or import of snapfit ([chapter 6](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/chapters/06-pure-domain-types.md)).
* **Shared components with typed getters:** each field is built once, and `get_<name>()` returns a clone of it with the type you declared, an `Arc<T>` or any `T: Clone` ([chapter 5](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/chapters/05-installing-and-first-singleton.md)).
* **Dependencies by name:** `deps = [config, db]` passes the named fields to the constructor, so two fields of one type, such as `read_db` and `write_db`, need no wrapper types ([chapter 7](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/chapters/07-wiring-dependencies-and-constructors.md), [chapter 9](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/chapters/09-multiple-instances-of-the-same-type.md)).
* **Ways to build a field:** `new` by default, `build = constructor(connect)`, `build = default`, and `leaf_build` for a whole container ([chapter 7](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/chapters/07-wiring-dependencies-and-constructors.md)); a factory you write, `build = factory`, or the type's builder, `build = builder` ([chapter 10](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/chapters/10-passing-dependencies-by-name.md)).
* **Fields typed as their trait:** `#[singleton(implementation = StripeGateway)] payments: Arc<dyn PaymentGateway>` builds `StripeGateway`, while the field, its getter and every service that depends on it see only the trait object, so a swap changes one word ([chapter 11](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/chapters/11-choosing-an-implementation.md)).
* **Testing:** a test declares its own container with other implementations, or builds one service with stand-ins ([chapter 12](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/chapters/12-testing-a-service-on-its-own.md)).
* **Static dispatch:** `#[snapfit::static_dispatch]` on a trait and `#[static_dispatch_for(PriceRule as Prices)]` on the container generate a type that holds one of the trait's implementations in place, so a field typed `Prices` calls its methods through a `match`, with no pointer to follow; `extend_static_dispatch` adds implementations, such as a test's, and a factory chooses among `implementations = [..]` while the program starts ([chapter 13](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/chapters/13-static-dispatch.md), [chapter 14](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/chapters/14-choosing-static-or-dynamic-dispatch.md)).
* **Components that can fail:** `fallible = true` marks a field whose constructor, builder or factory returns a `Result`, and `try_new()` returns the first failure as a `snapfit::BuildError` naming the field ([chapter 15](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/chapters/15-when-building-can-fail.md)).
* **Mistakes caught when compiling:** unknown dependencies and cycles fail `cargo build`, each at its field and every one in the same compile, and so does a misspelt setting ([chapter 16](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/chapters/16-mistakes-caught-by-the-compiler.md)).

What this version does not do yet is listed in [What this version supports](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/chapters/what-this-version-supports.md).

---

## Minimum Supported Rust Version (MSRV)

This crate requires **Rust 1.85.0** or higher (Rust 2024 edition), which CI checks.


---

## License

Dual-licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/marlon-sousa/snapfit-rs/blob/main/LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](https://github.com/marlon-sousa/snapfit-rs/blob/main/LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.
