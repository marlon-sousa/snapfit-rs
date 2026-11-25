# `snapfit` — Specification & Architecture Document

> **Zero-Cost, Compile-Time Verified, Pure-Rust Dependency Injection Framework**

---

## 0. What ships in 0.1.0

This document describes the full design. Only part of it ships as a stable API in 0.1.0; the rest is either behind the `unstable` Cargo feature or not implemented yet. Using a planned feature, or an unstable one without the feature, is a compile error that names the feature.

| Capability | Status | Where it is specified |
| :--- | :--- | :--- |
| `#[container]` with `#[singleton]` fields | Stable | §2, §3, §4 |
| `deps = [...]` named edges, multiple instances of one type | Stable | §2, §5.A |
| `build = default` and `build = constructor(<fn>)` (a bare function name, `build = constructor(connect)`) | Stable | §5.A |
| `#[container(leaf_build = ...)]` for fields with no `deps` | Stable | §2, §5 |
| Factories: `build = factory` and the generated `<ContainerName>Factories` trait | Stable | §5.B |
| Builders: `build = builder`, dependency labels and `builder(start = ..., finish = ...)` | Stable | §5.C |
| Compile-time missing-dependency and cycle detection | Stable | §6 |
| `Container` trait, `new()`, `Default` | Stable | §2 |
| Fallible fields: `fallible = true`, `try_new()`, the `FallibleContainer` trait and `BuildError` | Not in 0.1.0; stable from 0.2.0 | §5.D |
| Fields typed as their trait: `implementation = <Type>` on an `Arc<dyn Trait>` field | Not in 0.1.0; stable from 0.2.0 | §5.E |
| Async containers: `mode = "async"`, `#[singleton(async)]`, `init().await` (sequential) | Unstable | §8 |
| `#[transient]` dependencies | Unstable | §4 |
| Concurrent async boot (`join!` over independent async singletons) | Planned | §8 |
| Static dispatch of a trait of another crate (`remote = <path>`) | Planned | §7 |
| Static dispatch: `#[snapfit::static_dispatch]` on a trait, `static_dispatch_for` and `extend_static_dispatch` on a container | Not in 0.2.0; stable from 0.3.0 | §7 |
| Test overrides (`builder().with_*()` / `fork_with()`) | Planned | §9 |
| Child containers (`parent = ...`, `parent(dep)`) | Planned | §10 |
| Introspection (`introspection = true`, `dependency_graph()`) | Planned | §12 |
| `#[external]` inputs and `before_init` computed fields | Planned | §2 |

Each unstable and planned capability has an [open issue labelled `feature`](https://github.com/marlon-sousa/snapfit-rs/issues?q=is%3Aissue%20is%3Aopen%20label%3Afeature), which holds what is decided about it and what it must not break.

Enable the unstable APIs with:

```toml
[dependencies]
snapfit = { version = "0.3", features = ["unstable"] }
```

### Removed from the 0.1.0 runtime crate (to restore later)

These items existed before 0.1.0 but nothing generated or used them, so they were removed rather than shipped as unused public API. Each one comes back with the feature that needs it:

* **`AsyncContainer` trait**: the removed trait, whose `init()` returned `impl Future<Output = Result<Self, Self::Error>>`, is to return with stabilised async containers, in a new shape. The generated `init()` currently returns `Box<dyn Error + Send + Sync>`, which cannot satisfy the old `Error: std::error::Error` bound. The error type is now designed: `init()` is to return `BuildError` (§5.D), which names the field that failed, and `async` is to stop implying that a field can fail, which `fallible = true` says instead ([#56](https://github.com/marlon-sousa/snapfit-rs/issues/56)). The traits are then to mirror the synchronous pair `Container` and `FallibleContainer`: `AsyncContainer`, whose `init()` resolves to `Self`, for an async container with no fallible field, and `AsyncFallibleContainer`, whose `init()` resolves to `Result<Self, BuildError>`, for one with a fallible field.
* **`introspection` module** (`GraphDescriptor`, `NodeDescriptor`, `EdgeDescriptor`): return with `introspection = true`.
* **`SnapfitError` and the `Error` alias** (`ResolutionFailed`, `MissingDependency`, `InitializationFailed`): return when async initialization errors get dependency context.
* **`ConstructionPlan::parallel_batches`** (groups of independent async singletons): the planner computed them but the generator never used them. They return with concurrent async boot.
* **The `snapfit::container` module**, public in the alphas: it held only the `Container` trait, which the crate re-exports, so the trait is reached as `snapfit::Container` alone ([#88](https://github.com/marlon-sousa/snapfit-rs/issues/88)). Nothing is to be restored.

### Removed from the 0.1.0 macros

* **`#[derive(Container)]`**: removed before 0.1.0 ([#32](https://github.com/marlon-sousa/snapfit-rs/issues/32)), so that a container has one way to be declared. A derive can only add code next to the struct, never rewrite it, so it could never take a transient or a parent, each of which leaves the struct changed; on the stable build it generated the same code as `#[container]`. `#[derive(Container)]` is now the compiler's own "cannot find derive macro" error. The `Container` trait stays. Adding a derive form back later would break no one, but it needs a reason `#[container]` cannot meet.

* **`constructor = <fn>`**, stable in 0.1.0-alpha.1: replaced by `build = constructor(<fn>)` (§5), so that every way of building a field is a value of one setting and two of them cannot be written on one field. `constructor` is now a setting `#[singleton]` does not know. Nothing is to be restored.
* **`builder = <fn>`**, which was unstable: replaced by factories ([#40](https://github.com/marlon-sousa/snapfit-rs/issues/40)), which build the same dependencies from inputs passed by name instead of by position, and which ship as stable. `builder` is now a setting `#[singleton]` does not know, and the `<ContainerName>Builders` companion trait once planned for it is dropped. Nothing is to be restored.

### Held back from The Snapfit Book (to restore later)

The book's `v0.1.0` edition teaches only the stable API. Its chapters on the unstable APIs are kept, with their examples and expected output, in `book/unreleased/`, and return to Part 3 when those APIs are stabilised:

* **Transient dependencies**: `book/unreleased/chapters/07-transient-dependencies.md`, example `ch07_transients`.
* **Asynchronous containers**: `book/unreleased/chapters/08-asynchronous-containers.md`, example `ch08_async_container`.

### Decided syntax for later versions

* **Async containers get their own macro** (decided 2026-11-16, not implemented yet, recorded on [#37](https://github.com/marlon-sousa/snapfit-rs/issues/37)): `#[async_container]` replaces `#[container(mode = "async")]`, and `mode` is removed. Inside it, async dependencies are marked with the bare `async` flag (`#[singleton(deps = [config], async, build = constructor(connect))]`); everything else stays sync. `async` inside a plain `#[container]` becomes an error pointing to `#[async_container]`. Until then the unstable `mode = "async"` syntax keeps working.

### Container shapes refused in 0.1.0

These are compile errors that point at the offending field or container, instead of generating code that does not compile:

* **Generic containers** (`struct Repo<T>`): planned; the generated `impl` blocks and getters need a design for bounds first.
* **`#[cfg]` / `#[cfg_attr]` on a dependency field**: conditional dependencies would change the graph per build, so they are refused.
* **Trait-object fields without a way to build them**: a trait object has no constructor or builder to call. An `Arc<dyn PaymentGateway>` field that names no `implementation` (§5.E) and has no `build = factory` is refused, and the error offers both. Any other type whose value is a trait object, which is a type that has `dyn Trait` itself as a type argument once the field's own `Arc` is set aside, such as `Box<dyn PaymentGateway>`, `Arc<Box<dyn PaymentGateway>>` or `Arc<Archive<dyn Store>>`, needs `build = factory` (§5.B), and its error says so. Both errors are the same on both builds. A type that keeps a trait object only deeper in its type arguments, such as behind a pointer of its own in `Arc<Controller<Arc<dyn PaymentGateway>>>` or `Controller<Box<dyn PaymentGateway>>`, is not a trait object: it is built like any other field, by its `new`, its `build` or `leaf_build`, and the trait object is one of the values its constructor takes ([#136](https://github.com/marlon-sousa/snapfit-rs/issues/136)). Holding the concrete type (`Arc<StripeGateway>`) still works, and a constructor that takes `Arc<dyn PaymentGateway>` accepts that field, because Rust converts the `Arc` where the generated code passes it. Static dispatch (§7) is another way, for a closed list of implementations.
* **`#[dispatch(...)]` on a field**, the syntax planned for static dispatch before 0.3.0: refused at the field, naming its replacement, `static_dispatch_for` on the container (§7).
* **Static dispatch that cannot be followed** (§7): a trait holding what a `match` cannot forward, such as a method with no receiver, an associated const or a macro called among its items, refused at that item; an `unsafe trait`; `#[snapfit::static_dispatch]` on anything but a trait, or with any argument; a `static_dispatch_for` or `extend_static_dispatch` that cannot be read, that names a type the container does not declare, or that leaves a dispatched type with no implementation; a field typed with a declared name that has neither `implementation` nor `build = factory`, or names it through a path; `implementations` on a field without `build = factory`, or `implementation` beside it; and, inside a function, an implementation written with `self::` in one place and without it in another that names two different types, refused at the second spelling.
* **`#[snapfit::static_dispatch(remote = <path>)]`**, static dispatch of a trait of another crate through a copy of it that snapfit would generate: planned, refused as not implemented yet ([#140](https://github.com/marlon-sousa/snapfit-rs/issues/140)).
* **An `implementation` that cannot be followed** (§5.E): a bare `implementation`, one in parentheses, a quoted one, one that is no type the container can build (`impl Reader`, `_`, `dyn Reader`, `3`), and a second one, are refused at the field. So are an `implementation` on a field whose type holds no trait object and is neither `Name` nor `Arc<Name>` for a name the container's `static_dispatch_for` declares (§7), on a trait object held in anything but an `Arc`, on a type that keeps its trait object deeper in its type arguments, and beside `build = factory`, where a dispatched field is asked for `implementations` instead. `implementations = [A, B]`, the variants a factory returns, is refused on every field but one of a dispatched type built by `build = factory` (§7), and so are an empty list, a value that is not a list, a list given to `implementation`, and both settings on one field.
* **`build` values that are not a way of building** (§5), and **`constructor(...)` values that are not a plain function name** (`"connect()"`, `Db::connect`).
* **`build = default` with `deps`**: `Default::default()` takes nothing, so the field's `deps` would never be passed.
* **`build = default` on an async dependency** (unstable): `Default::default()` cannot be awaited.
* **`build = factory` on an async dependency** (unstable): planned, refused as not implemented yet, because a factory that awaits needs an async trait method, designed with `#[async_container]` ([#37](https://github.com/marlon-sousa/snapfit-rs/issues/37)).
* **A dependency label without `build = builder`** (`deps = [disk = folder_files]` on a field built another way): only a builder has setters for a label to name.
* **A `deps` entry that is not another field's name**, a label written `setter = field` or `parent(<name>)`, such as `config + 1` or `a::b = config`, and a `deps` that is not a list in square brackets, such as `deps = config`, or that writes its list in parentheses, such as `deps(config)`, which is told to write the same entries in square brackets, `deps = [config]` ([#69](https://github.com/marlon-sousa/snapfit-rs/issues/69)). Each bad entry is its own error at the field, and every one is reported in the same compile, beside any other problem of the same attribute ([#65](https://github.com/marlon-sousa/snapfit-rs/issues/65)).
* **A repeated `deps` entry**, the same field passed again to the same setter, such as `deps = [config, config]` or `deps = [disk = folder_files, disk = folder_files]`: it is refused at the field, once for each entry repeated ([#69](https://github.com/marlon-sousa/snapfit-rs/issues/69)). A builder may still pass one field to two setters, such as `deps = [reader = pool, writer = pool]`.
* **Two `deps` entries that pass different fields to the same builder setter**, and **`builder(...)` settings other than `start` and `finish` with a bare function name each.**
* **`build = builder` on an async dependency** (unstable): planned, refused as not implemented yet, like an async factory.
* **A bare `fallible`, and a `fallible` that is not `true` or `false`** (`fallible = "true"`, `fallible(true)`): refused at the field, asking for `fallible = true` (§5.D).
* **`build = default` with `fallible = true`**: `Default::default()` cannot fail.
* **A fallible field that `leaf_build` would build**: a synchronous field with no `deps` and no `build` of its own, in a container whose `leaf_build` is not `constructor(new)`, is refused at the field, asking for a `build` of its own (§5.D).
* **`fallible = true` on a transient** (unstable): planned, refused as not implemented yet, because a getter promises to return the value itself (§3).

---

## 1. Vision & Core Philosophy

Most dependency injection frameworks in Rust fall into two undesirable extremes:
1. **Invasive Java/C# ports** (`lockjaw`, `waiter`): Pollute business logic with intrusive macros (`#[inject]`, `#[component]`), require complex DSLs, and rely heavily on dynamic runtime downcasting (`dyn Any`).
2. **Dynamic Service Locators** (`axum::State`, `actix_web::web::Data`, `dptree`): Lack compile-time dependency graph verification, cannot detect cycles, and enforce `Arc<dyn Trait>` runtime indirection.

**`snapfit`** solves this by inverting the dependency declaration: **the container struct declares the wiring, leaving domain structs 100% pure Rust.**

### Key Design Pillars:
* **Zero Domain Pollution:** Your services, repositories, and controllers require **zero framework macros or traits**. They are standard Rust structs with plain `pub fn new(...)` constructors. The one exception is a trait dispatched statically, which carries `#[snapfit::static_dispatch]` where it is defined (§7).
* **Compile-Time Graph Verification:** The container macro builds the Directed Acyclic Graph (DAG) during `cargo build` and rejects dependency cycles with compile errors.
* **Zero-Cost Direct Getters:** Direct, compile-time checked getters (`get_<dep>()`) for fast production code — singletons are cloned on read, and transients are freshly constructed on every call with zero `dyn Any` overhead.
* **First-Class Static Polymorphism:** One generated type stands for a closed list of a trait's implementations, called through a `match` with no vtable, so backends swap and tests build a mock variant without `Arc<dyn Trait>` (§7).
* **Eager Async Boot with Synchronous Getters:** All async connections (databases, external APIs) are resolved during `Container::init().await`, while runtime getters remain **100% synchronous, non-blocking, and instant** for web request handlers.
* **Child Containers & Testing Overrides:** Fluent `.builder().with_*(mock)` overrides for testing, and macro-driven child containers that copy parent singletons for request-scoped workflows.

---

## 2. Declarative Container Syntax & Settings

A container in `snapfit` is declared using the `#[container]` procedural attribute macro.

```rust
use std::sync::Arc;
use snapfit::container;

#[container]
#[static_dispatch_for(OrderStore as ShopStore)]
pub struct AppContainer {
    // 1. Leaf Singleton: Instantiated via Config::new()
    #[singleton]
    config: Arc<Config>,

    // 2. Singleton with dependencies: calls Database::new(config)
    #[singleton(deps = [config])]
    db: Arc<Database>,

    // 3. Singleton with multiple dependencies of the same type:
    #[singleton(deps = [read_db, write_db])]
    repo: Arc<UserRepo>,

    // 4. Transient dependency: Stripped from struct memory; created on every get
    #[transient(deps = [db])]
    user_controller: UserController,

    // 5. Factory (built by a function the user writes, §5.B)
    #[singleton(deps = [config, db], build = factory)]
    auth: Arc<AuthService>,

    // 6. Static dispatch (§7): ShopStore is declared by #[static_dispatch_for(OrderStore as ShopStore)]
    // under #[container], and this field holds its PostgresStore variant
    #[singleton(implementation = PostgresStore)]
    orders_store: Arc<ShopStore>,
}
```

### Container Settings Reference (`#[container(...)]`):

| Setting | Options | Default | Description |
| :--- | :--- | :--- | :--- |
| **`mode`** | `"async"`, `"sync"` | `"sync"` *(or auto `"async"` if any field is `async`)* | Controls whether the container produces `pub async fn init() -> Result<Self, ...>` or synchronous `pub fn new() -> Self`, or `pub fn try_new() -> Result<Self, snapfit::BuildError>` when a field is fallible (§5.D). |
| **`parent`** | Type Identifier | None | Marks this container as a Child Scope copying/inheriting singletons from a parent container. |
| **`leaf_build`** | `default`, `constructor(<fn>)` | `constructor(new)` | How every field with no `deps` and no `build` of its own is built (§5). |
| **`introspection`** | `bool` | `false` | When `true`, generates `.dependency_graph()` and runtime inspection methods for BDD testing or runtime visualization. |

The settings are the arguments of the one `#[container(...)]` that declares the struct. A second `#[container]` on the same struct, in either order, is refused with `multiple #[container] attributes are not allowed`, pointing at the container's name.

### Field-Level Attribute Vocabulary:

| Attribute | Role | Structural & Memory Impact |
| :--- | :--- | :--- |
| **`#[singleton]`** | **Singleton** | Stored in container struct. Initialized once at boot. Cloned on read. |
| **`#[singleton(deps = [a, b])]`** | **Singleton (With Dependencies)** | Stored in container struct. Created at boot passing dependencies in order. |
| **`#[transient]`** | **Transient** | **Stripped from struct memory storage.** Reconstructed on each `get_<name>()`. |
| **`#[transient(deps = [a])]`** | **Transient (With Dependencies)** | **Stripped from struct memory storage.** Reconstructed on each `get_<name>()` passing `a`. |
| **`#[singleton(deps = [...], build = ...)]`** | **Singleton (Built Another Way)** | Stored in container struct. Built once at boot by `Default::default()`, a named constructor, a factory, or a builder (§5). |
| **`#[singleton(deps = [...], build = factory)]`** | **Factory** | Stored in container struct. Built once at boot by the user's `build_<name>` method of the generated `<ContainerName>Factories` trait. |
| **`#[singleton(implementation = StripeGateway)]`** | **Singleton Typed as its Trait** | An `Arc<dyn Trait>` field, stored as declared. Built once at boot by the named type's `new`, `Default`, named constructor or builder, then wrapped in `Arc::new` (§5.E). |
| **`#[singleton(fallible = true, ...)]`** | **Fallible Singleton** | Stored in container struct. Built once at boot by its `build`, or its type's `new`, which returns a `Result`; the container is then built by `try_new()` (§5.D). |
| **`#[singleton(implementation = MySql)] db: Arc<Db>`** | **Statically Dispatched Singleton** | A `Db` or `Arc<Db>` field, where `Db` is declared by the container's `static_dispatch_for`, stored as declared. Built once at boot like an `implementation` field, then wrapped in its variant, and in `Arc::new` when the field is an `Arc` (§7). |
| **`#[singleton(build = factory, implementations = [Postgres, Sqlite])] db: Arc<Db>`** | **Statically Dispatched Factory** | A `Db` or `Arc<Db>` field built by its factory, which returns one of the variants the list names; each joins the dispatched type's list (§7). |
| **`#[external]`** | **External Input (Planned)** | Passed as a parameter to `new(...)` / `init(...)` and exposed as a wireable leaf node in the DAG. |
| *(Unannotated field)* | **Pre-Init Computed (Planned)** | Populated via an optional `before_init` hook prior to DAG wiring and exposed as a wireable node in the DAG. |

A container written by a `macro_rules!` is read as if written by hand: a setting's value, a `deps` entry or any part of one that the macro passes as `$e:expr` means what the same words would mean typed into the attribute ([#64](https://github.com/marlon-sousa/snapfit-rs/issues/64)), and a field's type that the macro passes as `$t:ty`, such as `Arc<Mailer>` or `Arc<dyn Reader>`, is built as the same type typed into the struct ([#114](https://github.com/marlon-sousa/snapfit-rs/issues/114)).



---

## 3. Zero-Cost Direct Getters & Flexible Return Types

For each dependency field `foo: T`, the macro emits a strongly-typed, zero-cost getter returning `T`:

```rust
impl AppContainer {
    // 1. Singletons: Return the declared type directly via .clone()
    pub fn get_config(&self) -> Arc<Config> { self.config.clone() }
    pub fn get_db_pool(&self) -> sqlx::PgPool { self.db_pool.clone() } // Zero redundant Arc!

    // 2. Transient: Freshly evaluates constructor on every call; returns a new owned T
    pub fn get_user_controller(&self) -> UserController {
        let config = self.get_config();
        let db = self.get_db();
        UserController::new(config.clone(), db.clone())
    }
}
```

* **Zero `dyn Any` Indirection:** Resolution is 100% static and compile-time type-checked, with zero runtime downcasting or string lookup overhead.
* **No Forced `Arc<T>`:** Singletons require only `T: Clone`. If a type is already internally ref-counted (`sqlx::PgPool`, `reqwest::Client`) or a lightweight config struct, you declare it bare without a wasteful `Arc<Pool>` double indirection.
* **Fresh Transient Instantiation:** Calling `get_<transient>()` resolves its prerequisites via their getters and invokes the transient's constructor to return a brand-new instance on every call.
* **Documented:** every getter, and the container's `new()`, `try_new()` or `init()`, carries a one-line doc comment, such as "Returns a clone of `config`, built once with the container.", so that a crate that warns on or denies `missing_docs` accepts what snapfit writes; the factories trait, its inputs structs and a dispatched type's alias carry theirs too (§5.B, §7) ([#156](https://github.com/marlon-sousa/snapfit-rs/issues/156)).

---

## 4. Dependency Lifecycles & Struct Storage Rewriting

The `#[container]` attribute macro actively shapes the memory layout of the generated container struct:

| Lifecycle | Declaration | Storage in Container Struct | Resolution Behavior |
| :--- | :--- | :--- | :--- |
| **Singleton** | `#[singleton]` or `#[singleton(deps = [...])]` | **Stored in struct.** | Constructed once during `init()`, `new()` or `try_new()`. Returned by `.clone()`. |
| **Transient** | `#[transient]` or `#[transient(deps = [...])]` | **Stripped from struct memory.** | **Zero struct bytes allocated.** Reconstructed on every `get_<name>()` call. |
| **Static Dispatch** | `#[static_dispatch_for(Trait as Name)]` on the container, fields typed `Name` or `Arc<Name>` | **Stored in struct as declared.** | Calls go through a `match` over the implementations, with no vtable (§7). |
| **Scoped** | `#[container(parent = ...)]` | Inherited / Scope-local. | Clones/borrows parent singletons and instantiates request-local instances. |

---

## 5. Dependency Wiring & Factories

`snapfit` eliminates invasive domain macros entirely. Container wiring is declared centrally in the container definition, keeping all service structs 100% pure Rust.

Each field is built one way, which its `build` setting chooses. The value is a bare word, never a string, so that an editor can complete it:

| The field says | snapfit calls | How `deps` is passed | Where |
| :--- | :--- | :--- | :--- |
| no `build` | `Type::new(...)` | by position, in the order `deps` lists them | §5.A |
| `build = default` | `<Type as Default>::default()` | no `deps` are allowed | §5.A |
| `build = constructor(connect)` | `Type::connect(...)` | by position | §5.A |
| `build = factory` | `<Container as <ContainerName>Factories>::build_<field>(inputs)` | by name, through the inputs struct | §5.B |
| `build = builder` | `Type::builder().<dep>(...)....build()` | by name, one setter each | §5.C |

`Type` is the field's type, or `T` when the field holds `Arc<T>`, in which case the value is wrapped in `Arc::new`. A word that is none of these, such as `build = factroy`, is refused at the field, naming the ways the build accepts and suggesting `build = constructor(factroy)`. A quoted value, such as `build = "factory"`, is refused with the same value unquoted.

A container can say once how its leaves are built: `#[container(leaf_build = default)]` builds every field that has no `deps` and no `build` of its own as if it said `build = default`, which suits a project whose leaf types all derive `Default`. `leaf_build` applies to synchronous fields only, because `Default::default()` and a constructor chosen for the whole container cannot be awaited; an async field keeps its own constructor. A fallible leaf that `leaf_build` would build is refused (§5.D). `leaf_build` takes the values a leaf can take, `default` or `constructor(<fn>)`; leaving it out is `constructor(new)`, the behaviour above. `leaf_build` only changes the default for leaves, such as from `new` to `default`: a field's own `build` always wins, so a leaf that says `build = constructor(sandbox)` is still built by `sandbox`; a field with `deps` is never affected; and every field still needs its `#[singleton]`. Any other value, such as `leaf_build = factory`, is refused at the container's name.

A field and a `deps` entry, label included, are named as Rust reads them: a raw identifier names the same field as the plain word, so `r#config` and `config` are one field, and messages name it without the `r#` ([#83](https://github.com/marlon-sousa/snapfit-rs/issues/83)).

Where the user names a function, as in `build = constructor(connect)` or `leaf_build = constructor(open)`, the generated code calls it with the user's own token, keeping its position, so that an editor completing or hovering inside the attribute sees the type's real functions.

### A. Standard Constructors (`::new`, `default`, `constructor(...)`)
For standard structs implementing `new(...)`, prerequisites are declared via `deps = [...]` and mapped in sequence:

```rust
#[container]
pub struct AppContainer {
    // Leaf singleton: calls Config::new()
    #[singleton]
    config: Arc<Config>,

    // Singleton with dependency: calls Database::new(config.clone())
    #[singleton(deps = [config])]
    db: Arc<Database>,

    // Transient: calls UserController::new(config.clone(), db.clone()) on each get
    #[transient(deps = [config, db])]
    controller: UserController,
}
```

A type whose constructor has another name is built with `build = constructor(<fn>)`, which takes the same `deps` in the same order, and a leaf whose type only implements `Default` with `build = default`:

```rust
#[container]
pub struct AppContainer {
    // Calls TaxRateTable::default()
    #[singleton(build = default)]
    tax_rates: Arc<TaxRateTable>,

    // Calls Database::connect(config.clone())
    #[singleton(deps = [config], build = constructor(connect))]
    db: Arc<Database>,
}
```

### B. Factories via the Generated Trait (`<ContainerName>Factories`)
Some dependencies need a hand-written way of being built: a constructor whose arguments are not the dependencies themselves, two dependencies of the same type that must not be swapped, or a field that holds a trait object (`Arc<dyn OrderStore>`), which has no constructor at all. A factory is that hand-written way. The container calls it instead of `Type::new(...)`.

1. Mark the field with `build = factory`:
   ```rust
   #[container]
   pub struct Shop {
       #[singleton]
       config: Arc<Config>,

       #[singleton(deps = [config])]
       read_db: Arc<Database>,

       #[singleton(deps = [config])]
       write_db: Arc<Database>,

       #[singleton(deps = [read_db, write_db], build = factory)]
       order_store: Arc<dyn OrderStore>,
   }
   ```

2. `#[container]` generates one trait for the container, `<ContainerName>Factories`, with one method for each factory field, and one inputs struct for each of those methods:
   ```rust
   pub trait ShopFactories {
       fn build_order_store(inputs: ShopOrderStoreInputs) -> Arc<dyn OrderStore>;
   }

   pub struct ShopOrderStoreInputs {
       pub read_db: Arc<Database>,
       pub write_db: Arc<Database>,
   }
   ```

3. Implement the trait on the container, taking the inputs by name:
   ```rust
   impl ShopFactories for Shop {
       fn build_order_store(inputs: ShopOrderStoreInputs) -> Arc<dyn OrderStore> {
           let ShopOrderStoreInputs { read_db, write_db } = inputs;
           Arc::new(SqlOrderStore::new(read_db, write_db))
       }
   }
   ```

* **Names:** the trait is `<ContainerName>Factories`. Each method is `build_<field>`, an associated function with no `self`, because the container does not exist yet while it is being built, and it returns the field's type exactly as declared, or a `Result` of it for a fallible field (§5.D). Each inputs struct is `<ContainerName><Field>Inputs`, the field's name in PascalCase (`order_store` gives `ShopOrderStoreInputs`).
* **Inputs by name:** the inputs struct has one `pub` field for each `deps` entry, with that dependency's name and the type of its container field, holding a clone of the singleton, or a fresh value of a transient (unstable). The method always takes the struct, even when the field has no `deps`, so that adding a dependency never changes its signature. The inputs are passed by name, not by position, because Rust does not check parameter names in a trait impl: a method whose two `Arc<Database>` parameters were swapped would compile. With a struct, a misspelt, missing or swapped name is a compile error.
* **The graph stays in `deps`:** a factory receives only what its `deps` names, so the construction order, the unknown-dependency and the cycle checks of §6 apply to it as to any field.
* **Visibility:** the trait and the inputs structs have the container's visibility.
* **One name per inputs struct:** two factory fields whose names give the same PascalCase, such as `db2` and `db_2`, generate the same inputs struct name, which rustc reports as defined twice; rename one of the fields.
* **Conditional Generation:** if no field of the container says `build = factory`, neither the trait nor any inputs struct is generated.
* **The compiler checks the rest:** forgetting the `impl`, leaving out a method, or returning the wrong type is rustc's own error about the trait. A missing `impl` is reported at the factory field.
* **Transients (unstable):** a `#[transient(build = factory)]` field is built by its factory on every `get_<name>()`, which receives `&self`'s singletons as its inputs.
* **IDE Auto-Scaffolding:** in `rust-analyzer` or any LSP-enabled editor, typing `impl ShopFactories for Shop {}` offers an "Implement missing members" code action that writes each method's signature.
* **Absolute Zero Domain Pollution:** services, repositories and controllers remain plain Rust with no framework dependencies.

### C. Builders (`build = builder`)
Many types are built through a builder. No builder crate is required: any type works whose start function returns a value with the named setters and a finish method that returns the type, whether the builder is written by hand or generated by a crate such as `bon` or `typed-builder`. `build = builder` calls the type's builder, with one setter for each dependency, so the dependencies are passed by name and their order never matters:

```rust
#[container]
pub struct Shop {
    #[singleton]
    folder_files: Arc<FolderFiles>,

    #[singleton]
    memory_files: Arc<MemoryFiles>,

    // Calls Exporter::builder().disk(folder_files.clone()).memory(memory_files.clone()).build()
    #[singleton(deps = [disk = folder_files, memory = memory_files], build = builder)]
    exporter: Arc<Exporter>,
}
```

* **One setter per dependency:** `deps = [read_db, write_db]` calls `.read_db(...)` and `.write_db(...)`, each with a clone of the singleton, or a fresh transient (unstable).
* **Labels rename a setter:** `deps = [disk = folder_files]` calls `.disk(...)` with `folder_files`. A label is accepted only with `build = builder`; on any other field it is refused at the field, because a constructor or a factory has no setters to name.
* **Start and finish:** the chain starts with `Type::builder()` and ends with `.build()`, which must return the value itself. `build = builder(start = new_builder, finish = done)` names other functions for a builder that uses them; either can be given alone. Each takes a bare function name, like `constructor(...)`, once; any other setting or value inside `builder(...)`, and a setting given twice, is refused at the field.
* **Every name is the user's own token:** the setters, the start and the finish are written with the tokens of the attribute (the label, or the `deps` entry when there is none, and the `start`, `finish` or `builder` word), so that rustc's errors about them point inside the attribute and an editor completing there sees the builder's real methods.
* **The compiler checks the rest:** an unknown or misspelt setter is rustc's own error. A missing or repeated setter is a compile error only with a typestate builder, such as those `bon` and `typed-builder` generate; a hand-written builder decides for itself what a missing setter means.
* **One setter, one entry:** two `deps` entries that pass different fields to the same setter, such as `[disk = folder_files, disk]`, are refused at the field, naming the setter. The same field passed twice to the same setter is refused as a repeated entry instead (§0), and one field passed to two setters is accepted.
* **Trait objects:** a trait-object field has no type to call `builder()` on. An `Arc<dyn Trait>` field that names its `implementation` (§5.E) calls the implementation's `builder()`; any other trait-object field with `build = builder` is refused with the trait-object error of §0.
* **Async (unstable):** `build = builder` on an async dependency is planned, refused as not implemented yet, for the same reason as an async factory.
* **No dependency on a builder crate:** snapfit only writes the chain of calls. The builder crate, if any, is the user's own dependency.

### D. Fallible fields (`fallible = true`)
Reading a configuration file, opening a socket or checking a setting can fail without being async. A field whose constructor, builder or factory can fail says so with `fallible = true`, and the container is then built by `try_new()`, which returns the first failure instead of `Self`:

```rust
#[container]
pub struct AppContainer {
    // Calls ShopConfig::load(), which returns Result<ShopConfig, ConfigError>
    #[singleton(fallible = true, build = constructor(load))]
    config: Arc<ShopConfig>,

    // Calls Database::connect(config.clone()), which returns Result<Database, io::Error>
    #[singleton(deps = [config], fallible = true, build = constructor(connect))]
    db: Arc<Database>,

    // Infallible, as before: calls OrderService::new(db.clone())
    #[singleton(deps = [db])]
    orders: Arc<OrderService>,
}

use std::error::Error;

match AppContainer::try_new() {
    Ok(app) => serve(app.get_orders()),
    Err(error) => match error.source() {
        Some(cause) => eprintln!("{error}: {cause}"),
        None => eprintln!("{error}"),
    },
}
```

* **The setting:** `fallible = true` or `fallible = false`, a boolean value as the other settings take; leaving it out is `fallible = false`. A bare `fallible`, and any value other than `true` or `false`, is refused at the field, asking for `fallible = true`.
* **What builds the field returns a `Result`:** `Type::new(...)`, `build = constructor(<fn>)`, the builder's finish method, or the factory, returns `Result<T, E>`, where `T` is what an infallible field's would return (`T` for an `Arc<T>` field, which is wrapped in `Arc::new` after it succeeds) and `E` is the user's own error, anything that converts into `Box<dyn Error + Send + Sync>`, such as any `Error + Send + Sync + 'static` type, `String` or `anyhow::Error`. A fallible field whose builder returns a plain value is a compile error at its `fallible` setting, which says that the field is marked `fallible = true`, so what builds it must return a `Result`. A `Result` whose error does not convert, such as `Result<T, ()>`, is rustc's own error about that conversion, at the same place. A `Result` whose value is not `T`, such as `Result<Settings, E>` for an `Arc<Config>` field, is rustc's own mismatched types error, which names both types. The services stay plain Rust: none of them names a snapfit type.
* **Factories:** the `build_<field>` method of a fallible field returns `Result<FieldType, Box<dyn Error + Send + Sync>>`, where `FieldType` is the field's type exactly as declared; `?` turns any error into that box inside the method.
* **`try_new()` instead of `new()`:** a synchronous container with at least one fallible field generates `pub fn try_new() -> Result<Self, snapfit::BuildError>`, implements `snapfit::FallibleContainer`, and has no `new()`, no `Default` and no `Container`, which promise a container that cannot fail. A container with no fallible field generates exactly what it did before. The getters are the same in both: `get_<field>()` returns the value itself, because a container that exists has built every field.
* **The first failure stops the build:** fields are built in the order of §6, each after the fields it needs. When one fails, `try_new()` returns at once; the fields already built are dropped, and the ones after it are never built.
* **`snapfit::BuildError`:** the generated code wraps the field's own error into a `BuildError`, because only it knows which field failed. Its message is a sentence, such as "snapfit could not build the field `db` of `AppContainer`"; `container()` and `field()` name them, without the `r#` of a raw identifier; and `source()` is the field's error, kept whole, so that it can be logged, or downcast to its own type when it is an error type. It is `Send + Sync + 'static`, and `#[non_exhaustive]`, so that later versions can add to it.
* **`leaf_build` cannot build a fallible field:** a synchronous fallible field with no `deps` and no `build` of its own, in a container whose `leaf_build` is not `constructor(new)`, is refused at the field, because the function chosen for every leaf returns the value itself, and building the field another way than `leaf_build` says would mislead the reader of the container. The error asks for a `build` of the field's own, such as `build = constructor(new)`, or for `fallible = true` to go.
* **Refused:** `build = default` with `fallible = true`, because `Default::default()` cannot fail; a fallible leaf that `leaf_build` would build, as above; and `fallible = true` on a transient (unstable), planned and refused as not implemented yet, because a getter promises to return the value itself (§3).
* **Async containers (unstable):** a fallible field of an async container is wrapped into a `BuildError` the same way, after its `.await` when it is async, and `init()` returns it boxed, as it returns every error today.

### E. Fields typed as their trait (`implementation = <Type>`)
A field can be the trait itself, `Arc<dyn PaymentGateway>`, and name the type the container builds for it. Swapping the implementation is then that one word, and the field, its getter, and what every dependent receives keep their types:

```rust
#[container]
pub struct ShopContainer {
    // Calls Arc::new(StripeGateway::new()), seen as Arc<dyn PaymentGateway>
    #[singleton(implementation = StripeGateway)]
    payments: Arc<dyn PaymentGateway>,

    // OrderService::new takes Arc<dyn PaymentGateway>
    #[singleton(deps = [payments])]
    orders: Arc<OrderService>,
}
```

* **The setting:** `implementation = <Type>`, a type written in full, such as `StripeGateway` or `Cache<Order>`, also when a `macro_rules!` passes it as `$t:ty`. It names one type: a value in square brackets is refused, asking for `implementations`, with an s.
* **`implementations = [<Type>, ...]`:** the variants a factory may return, on a field of a dispatched type built by `build = factory` (§7). It is always a list, even of one, of types written in full as `implementation` takes them, also when a `macro_rules!` passes the whole list or any of its types. A value that is not one pair of square brackets with nothing after them is refused, asking for `implementation` for a single type the container builds, and so are an empty list, each entry that is not a type, as for `implementation`, a second `implementations`, and `implementations` beside `implementation`. On a field of no dispatched type, such as `Arc<dyn Trait>`, and on a dispatched field without `build = factory`, it is refused at the field, asking for `implementation`. A bare `implementation`, an empty one, one in parentheses or in quotes, a type that is no type the container can name or call a constructor of (`impl Trait` or `_` anywhere in it, or a trait object), any other value, and a second `implementation`, are refused at the field, each with every other problem the field has.
* **Which fields take it:** a field typed `Name` or `Arc<Name>`, for a name its container's `static_dispatch_for` declares, which holds the implementation as a variant (§7), and otherwise only a field typed `Arc<dyn Trait>`, through any of the paths of `Arc` that §3 reads and with any bounds on the trait object (`Arc<dyn PaymentGateway + Send + Sync>`). Any other field whose type holds no trait object, one that holds its trait object in anything but an `Arc` (`Box<dyn Trait>`, `Arc<Archive<dyn Trait>>`), and one that keeps its trait object deeper in its type (`Arc<Controller<Arc<dyn Trait>>>`), are refused at the field: the first is built as it is, the second has no single place to put the implementation, and the third is built by its own constructor, which takes the trait object from another field.
* **How it is built:** everything else about the field stays as it is. `build` is read as for an `Arc<T>` field whose `T` is the implementation: its `new` taking `deps` in order, `build = default`, `build = constructor(<fn>)`, or `build = builder`, and `leaf_build` applies to it as a leaf. The value is then wrapped in `Arc::new` and converted to the field's trait object, or, for a dispatched field, wrapped in its variant and then in `Arc::new` when the field is an `Arc` (§7), where an implementation that does not implement the trait is rustc's own error, at the type `implementation` names. The `Arc::new` the macro writes is not linted by clippy as the user's code, so an implementation that is not both `Send` and `Sync` meets no `arc_with_non_send_sync`, just as a field typed `Arc<T>` meets none. `fallible = true` works as §5.D says, and so does `#[transient]` (unstable), which builds a new `Arc` for every get. `build = factory` beside `implementation` is refused, because a factory returns the field itself and has no implementation to call; on a dispatched field the message asks for `implementations` instead (§7).
* **What does not change:** the getter returns the field's type exactly as declared, `Arc<dyn Trait>`, and every dependent, factory inputs included, receives a clone of that `Arc`. A dispatched field does the same with its own type (§7).

---

## 6. Compile-Time Graph Checks & Diagnostics

Because all dependencies and their prerequisites are declared within the struct, the procedural macro checks the whole graph at compile time:

1. **Construction order:** Each field's `deps = [...]` names the fields it needs. The macro visits the fields in the order they are declared and builds each one after the fields it needs, so the construction order follows the declaration wherever the graph allows it. Declaring a field before the fields it needs is allowed.
2. **Unknown dependencies:** A `deps` entry that names no field of the container is refused. Each field gets one error, which names each unknown entry once, even one the field lists twice, says "unknown dependency" for one and "unknown dependencies" for several, and lists every field the container can wire.
3. **Cycles:** A field that needs itself, directly or through other fields, is refused. The message reads as a sentence, without arrows, so that a screen reader speaks it naturally, and it points at the first field of the cycle in declaration order:

```text
error: dependency cycle: service_a depends on service_b, which depends on service_a
  --> src/main.rs:12:5
   |
12 |     #[singleton(deps = [service_b])]
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

A field that needs only itself reads `dependency cycle: service_a depends on itself`.

4. **Every problem at once:** The macro reports every problem it finds in one compile, each as its own error pointing at the field it is about (its snapfit attribute, or its name when it has none), or at the container's name when no field is at fault. Problems found while reading the fields, such as a missing `#[singleton]` or a quoted function name, are reported together; once every field reads, the unknown dependencies, cycles, refused features and other rules of the declaration are reported together. The struct is kept as written, without snapfit's attributes, so that the compiler adds no follow-on errors about a missing type. A problem of a dispatch attribute points at that attribute, and each type whose `static_dispatch_for` reads as far as its trait and its name is still written, holding the implementations that the fields and extensions that read give it, or standing for `()` when they give it none, so that the fields that name it raise no error of their own.
5. **Only what the build accepts:** snapfit's own advice on what to write offers only what the build in use accepts, and never a planned feature. On the stable build it offers no unstable feature: a field with no snapfit attribute reads `field 'config' has no snapfit attribute; add #[singleton] to have the container build it`, and an unknown mode reads `invalid container mode 'threaded': write mode = "sync", or leave mode out`, the same on both builds. A feature the user wrote is still named when it is refused, such as `parent` being planned. One exception is not snapfit's own advice: for a misspelt or unknown setting, darling, which reads the attributes, either suggests the closest name it knows, such as "Did you mean `build`?", or lists the names it knows, such as "Available values: `async`, `build`, `deps`", which could in principle include an unstable or planned setting.

---

## 7. Static dispatch (`static_dispatch_for`)

A trait object, `Arc<dyn Database>`, reaches its implementation through a vtable. Static dispatch reaches it through a `match` over a closed list: one type stands for every implementation on the list, and each call matches the variant it holds and calls that implementation directly, a call the compiler can inline. Every dependent receives that one type and calls the trait on it, without a `match` of its own, and a test can build any variant itself, including one no field injects ([#41](https://github.com/marlon-sousa/snapfit-rs/issues/41)).

### User code

```rust
// db.rs: the trait, marked once where it is defined
#[snapfit::static_dispatch]
pub trait Database {
    fn find(&self, id: u32) -> Option<Order>;
}

pub struct MySql { /* ... */ }
pub struct Postgres { /* ... */ }
pub struct Mock { /* ... */ }
// impl Database for MySql, for Postgres and for Mock, as usual

// container.rs
#[container]
#[static_dispatch_for(Database as Db)]
#[cfg_attr(test, extend_static_dispatch(Db, [Mock]))]
pub struct Shop {
    #[singleton]
    config: Arc<Config>,

    #[singleton(implementation = MySql)]
    orders_db: Arc<Db>,

    #[singleton(implementation = Postgres, deps = [config], build = constructor(connect))]
    reports_db: Arc<Db>,

    // Orders::new takes Arc<Db>, and calls self.db.find(id)
    #[singleton(deps = [orders_db])]
    orders: Arc<Orders>,
}

// a test builds the variant it needs, with no container
#[test]
fn total_reads_the_order() {
    let orders = Orders::new(Arc::new(Db::of(Mock::with_order(7, 30))));
    assert_eq!(orders.total(7), 30);
}
```

### The trait: `#[snapfit::static_dispatch]`

The container's macro sees only the container, never a trait's methods, so the trait itself carries `#[snapfit::static_dispatch]`, written once where it is defined. It implements the trait for snapfit's dispatch types, in the trait's own module, so that every type its methods name is in scope. The implementations stay plain types with a plain `impl`.

* **Which trait:** the one `static_dispatch_for` names, and every supertrait it has, because Rust asks the dispatched type to implement each of them, even one whose methods are never called. A supertrait of the same crate carries the attribute too; without it, rustc's own error says that the dispatched type does not implement it. `Send`, `Sync` and `Sized` need nothing, and neither do `Clone` and `Debug`, which the dispatch types implement whenever every implementation on the list does. Any other supertrait of another crate, such as `Display` or `PartialEq`, cannot be implemented for the dispatched type, because of Rust's orphan rule, so a trait that has one cannot be dispatched statically. It can be dispatched dynamically (§5.E) only when a trait object can hold it, as one with `Display` can and one with `PartialEq` cannot. A trait of another crate cannot carry the attribute, and a local subtrait of it does not help, because the dispatched type would also have to implement the other crate's trait, which Rust's orphan rule forbids. Such a trait is dispatched dynamically (§5.E), or through a local trait that copies the methods it needs, which each implementation implements by calling the other crate's trait.
* **What a trait may hold:** methods taking `self`, `&self` or `&mut self`, also written `mut self`, `self: Self`, `self: &Self` or `self: &mut Self`, with any arguments, generic methods, `impl Trait` arguments, `async fn`, `where Self: Sized` and lifetime bounds on `Self` such as `where Self: 'a`, default methods, which are forwarded so that an implementation that overrides one is the one called, a method whose only return is `Self`, which is built back into the variant it came from, associated types, and type parameters and lifetimes of the trait itself (`Store<T>`). A method under `#[cfg]`, or under a `#[cfg_attr]` that can write a `cfg`, such as `#[cfg_attr(test, cfg(feature = "mocks"))]`, is forwarded under the same `cfg`, or under a `cfg_attr` with the same condition that writes only its `cfg` and `cfg_attr`, so that the forwarding method exists exactly when the trait's does, and carries no attribute an impl's method may not, such as `deprecated`; this holds too when a `macro_rules!` passes the condition or the `cfg` ([#156](https://github.com/marlon-sousa/snapfit-rs/issues/156)).
* **Associated types:** every implementation on a list gives each associated type the same type. One that differs is rustc's own error, where the container checks the list.
* **What it refuses:** each at the item it is about, with every other problem of the trait:
  * a method with no receiver, such as `fn create() -> Self`, because no value says which implementation to call;
  * an associated const, because the implementations could disagree;
  * a receiver other than `self`, `&self` or `&mut self`, such as `self: Box<Self>` or `self: Pin<&mut Self>`;
  * a generic associated type, such as `type Rows<'a>`, which 0.3.0 does not forward;
  * `Self` anywhere in a method's signature other than its receiver, its whole return type, `Self::` paths and `where Self: Sized`, such as an argument `other: Self` or a return `Option<Self>`, because two values could hold different variants;
  * a method that bounds `Self` by a trait other than `Sized`, such as `where Self: Clone`, because snapfit cannot promise the bound of every implementation on the list; the message asks for the bound to go, or to become a supertrait;
  * an associated type that names `Self` in its bounds, such as `type Twin: PartialEq<Self>`, because the bound would be about the dispatched type rather than about each implementation, and one whose `where` clause bounds `Self` by anything but `Sized`, such as `type Pair where Self: Clone` or `type Kept where Self: 'static`, because the type each impl gives it is the first implementation's, which the bound does not reach; a lifetime bound on `Self` is forwarded on a method only, and a generic associated type is refused as generic alone;
  * an associated type under `#[cfg]`, or under a `#[cfg_attr]` that can write a `cfg`, because the bound that makes every implementation agree on it is written for every build;
  * a return type that is `impl Trait`, because each implementation returns its own type; the message asks for a named type, such as `Box<dyn Iterator<Item = Order>>`;
  * a macro called among the trait's items, such as `sources!();`, because snapfit cannot see the items it writes; each call is refused at its own line, also when the same macro is called twice;
  * an `unsafe trait`, refused at its `unsafe`, because snapfit cannot promise that the trait's contract holds for its dispatch types. An `unsafe fn` of a safe trait is forwarded, its call written in an `unsafe` block, since its caller keeps the promise the implementation's own `unsafe fn` asks for.
* **Where it goes:** on a trait, with no arguments. On any other item it is refused, and so is any argument; `remote = <path>` is refused as planned ([#140](https://github.com/marlon-sousa/snapfit-rs/issues/140), §0).

### The container: `static_dispatch_for` and `extend_static_dispatch`

```rust
#[container]
#[static_dispatch_for(Database as Db)]
#[extend_static_dispatch(Db, [Sqlite])]
#[cfg_attr(test, extend_static_dispatch(Db, [Mock1, Mock2]))]
pub struct Shop { /* ... */ }
```

* **`static_dispatch_for(<Trait> as <Name>)`** declares the dispatched type: the trait, as a path with any type arguments (`Store<Order> as Orders`), and the name the fields and the dependents write, a bare identifier. It is required: the container never deduces a dispatched type, so that a misspelt type or one that already exists is not taken for one. A container may declare several, each with a name of its own. It takes nothing after the name: any setting there, such as `extra = [...]`, is refused at the attribute, offering `implementation` on a field and `extend_static_dispatch`. A missing `as <Name>`, a name written as a path, and a name declared twice in one container are refused at the attribute. Two containers in one module that declare the same name generate two items of that name, which rustc reports as defined twice; rename one of them. `static_dispatch_for` must hold in every build. Under a `cfg_attr` whose condition is false, the container never sees it: a field with `implementation` is then refused as §5.E says, and every use of the name, the field's own type included, is rustc's own error for a type it cannot find.
* **What the type holds:** the implementations named by `implementation =` and `implementations = [...]` on the fields typed `Name` or `Arc<Name>`, a list's in the order written, in the order of the fields, then those of every `extend_static_dispatch(Name, [...])` present in the build. Each is a type written in full, as `implementation` takes it (§5.E). A type named more than once counts once. Sameness is decided by how the type is written, apart from a `self::` that starts any of its paths, since the macro runs before Rust resolves names: `Mock` and `self::Mock`, and `Cache<Order>` and `Cache<self::Order>`, are one entry, which keeps the spelling it is first written with, but `Mock` and `crate::shop::Mock` are two, and `<Name>::of` then cannot choose between them, which rustc reports as type annotations it needs, at the `implementation` of each field whose value it wraps ([#156](https://github.com/marlon-sousa/snapfit-rs/issues/156)); write each type one way, such as by importing it once with `use`. Inside a function that declares a type of the same name, `Mock` is the function's and `self::Mock` the module's, two types that snapfit cannot tell apart: the generated code checks that each other spelling of an entry names the type the entry holds, and when it does not, the container is refused at that spelling, saying that the two are written as one implementation but are two different types, and to name the module's type through `crate::`. A list of one implementation is allowed, such as a production build whose only other implementation is a mock added under `cfg(test)`. A list with none, such as a build whose only field of the type has `build = factory` with no `implementation` and whose mocks are added under `cfg(test)`, is refused at `static_dispatch_for`, offering `implementation` on a field, `implementations` beside `build = factory`, and `extend_static_dispatch`. A declared name that no field writes is accepted while its list is not empty, so that factories and tests can use it.
* **`extend_static_dispatch(<Name>, [...])`** adds implementations to a type the same container declares. Written as it is, it adds them to every build, such as an implementation that only a test or a factory of another container builds. Under a condition, it adds them to the builds where the condition holds: `#[cfg_attr(test, extend_static_dispatch(Db, [Mock]))]`. Rust removes the attribute where its condition is false before the container sees it, so the mocks, which can themselves be `#[cfg(test)]` types, exist only in test builds. Several may extend one type, and they only add. A missing list is refused, and an empty one adds nothing. A name the same container does not declare is refused, naming those it declares. Tests under `tests/` build the library without `cfg(test)`, so they extend under a feature instead, such as `cfg_attr(feature = "mocks", ...)`.
* **Where they go:** under `#[container]`, which reads and removes them, as it does `#[singleton]`. Written above `#[container]`, they are an attribute rustc cannot find, which it reports; snapfit, which receives every attribute of the struct, reads them too and reports what it finds, such as a type that holds no implementation; and an `extend_static_dispatch` under a `cfg_attr` whose condition is false is removed by Rust with no error, while the builds where the condition holds report it.
* **The generated name:** `<Name>` is a type alias with the container's visibility, declared next to the container, and the only name snapfit generates for it. Its doc comment names the trait and `<Name>::of`, so that a crate that denies `missing_docs` accepts it ([#156](https://github.com/marlon-sousa/snapfit-rs/issues/156)). It stands for one of snapfit's dispatch types, whose shape is not part of the API: code names `<Name>`, builds a value with `<Name>::of(value)`, and calls the trait on it. An implementation more private than the alias, such as a test-only `Mock` beside a `pub` container, makes rustc's `private_interfaces` lint warn wherever a public item names the alias. snapfit allows that lint on each item it generates that names a dispatched type: the alias, the getter of a dispatched field, its `build_<field>` method in the factories trait, its field in an inputs struct, and the getter of a dispatched transient (unstable). The user's own public items that name the alias still warn unless every implementation is at least as visible. Examples are a `pub fn new(db: Arc<Db>)` and the `build_<field>` methods of the user's `impl` of the factories trait. The user may allow the lint on them, as on that `impl`. A dependent generic over the trait names no implementation. Code outside the implementation's reach that handles a value of the alias, such as a getter called from another module, is refused by rustc as using a private type, so a private implementation suits code inside the module that can see it, such as its tests.
* **`<Name>::of(value)`** wraps an implementation into its variant. It accepts only a type on the list; any other is a compile error, set through `#[diagnostic::on_unimplemented]`, whose message names the type and says that it is not one of the implementations the dispatched type holds. The list itself is the one `static_dispatch_for` and its extensions write.
* **The checks at the attribute:** the type implements the trait, which fails with rustc's own error, pointing at the trait in `static_dispatch_for`, when the trait lacks `#[snapfit::static_dispatch]` or an implementation disagrees on an associated type; each implementation implements the trait, which fails with rustc's own error pointing at the field or the list that names it; and each other spelling of an implementation, with or without `self::`, names the type the list holds, refused in snapfit's own words at that spelling.

### The fields

A field's type decides how it is dispatched, so one container can hold both kinds, even for the same trait:

| Field's type | Dispatched | `implementation = X` |
| :--- | :--- | :--- |
| `Arc<dyn Database>` | Dynamically, through the trait object (§5.E) | Builds X and hands it out as the trait object |
| `Db` or `Arc<Db>`, through any of the paths of `Arc` that §3 reads, where `Db` is a name this container's `static_dispatch_for` declares | Statically | Builds X and wraps it in X's variant, then in `Arc::new` when the field is an `Arc` |
| Any other type | Not dispatched | Refused, as §5.E says |

* **With `implementation = X`:** X is built as §5.E says, by its `new` taking `deps` in order, `build = default`, `build = constructor(<fn>)` or `build = builder`, with `leaf_build` and `fallible = true` as for any field, then wrapped in its variant, and in `Arc::new` when the field is an `Arc`, as for any field (§3): snapfit adds no `Arc` the field does not declare. A field typed `Db` holds the value itself, and every dependent receives a clone of it, so every implementation on the list must then be `Clone`, or rustc says which is not. Each field states its own implementation's needs, so the variants never have to agree on their `deps`.
* **With `build = factory`:** the factory returns the field's type, `Name` or `Arc<Name>`, and chooses the variant while the program starts, such as `Arc::new(Db::of(Postgres::connect(&inputs.config)))`. Its `implementations` lists the variants it may return, always as a list, even of one: `#[singleton(deps = [config], build = factory, implementations = [Postgres, Sqlite])]`. They join the type's list like any field's, so a factory can return any variant on that list, also one another field or an extension adds, and `implementations` may be left out when the list already holds every variant the factory returns. Its `deps` are every input of every variant it may build, and the container builds all of them, also those the chosen variant does not use. A dependency that is costly to build, such as one that opens a connection, is better built by the factory itself, from a setting.
* **Refused at the field:** a field typed `Name` or `Arc<Name>` with neither `implementation` nor `build = factory`, asking for one of them; `implementations` without `build = factory`, asking for a factory or for `implementation`; `implementation` beside `build = factory`, asking for `implementations = [X]`; and a field typed with a path that ends in a declared name, such as `Arc<crate::shop::Db>`, asking for the name as declared.
* **What does not change:** the getter returns the field's type as declared, and every dependent, factory inputs included, receives a clone of it. `#[transient]` (unstable) builds a new value for every get, as §4 says.

### What it costs, and when to choose dynamic dispatch

* **A closed list:** adding an implementation means a field or an extension names it, and the container is compiled again.
* **Size:** a value of the type is at least as large as its largest implementation. It is kept as the field declares it: in an `Arc`, it is shared as a dynamic one is; as a value, every dependent receives a clone.
* **Naming the type:** a dependent that takes `Arc<Db>` names the container's alias, so it depends on the module that declares the container. A dependent generic over the trait, such as `Orders<D: Database>`, does not, and accepts the dispatched type and any single implementation alike.
* **Dynamic dispatch instead:** a trait of another crate, or a set of implementations that is open, such as plugins, uses `Arc<dyn Trait>` (§5.E), which needs no attribute on the trait, when a trait object can hold the trait. Of what static dispatch refuses, a trait object accepts only other receivers, such as `self: Box<Self>`, a supertrait of another crate that a trait object can hold, such as `Display`, and a refused method or generic associated type bounded by `where Self: Sized`, which the trait object leaves out. A trait with anything else static dispatch refuses has no dispatched form in 0.3.0: its field holds one implementation, named by its type or returned by a factory.

---

## 8. Asynchronous Containers (`mode = "async"`)

For applications where databases, message brokers, and clients require asynchronous setup, `snapfit` provides dedicated async support.

### Philosophy: Eager Async Boot, Synchronous Getters
* **`AppContainer::init().await`** handles all async setup at boot, each dependency after the ones it needs.
* **Independent async dependencies initialize concurrently** using `tokio::join!`.
* **Runtime getters (`container.get_db()`) remain synchronous, non-blocking, and instant** for web request handlers.

```rust
#[container(mode = "async")]
pub struct AppContainer {
    // Synchronous leaf dependency
    #[singleton]
    config: Arc<Config>,

    // Asynchronous constructor: Database::connect(config).await
    #[singleton(deps = [config], async, build = constructor(connect))]
    db: Arc<Database>,

    // Asynchronous constructor: PaymentGateway::init(config).await
    #[singleton(deps = [config], async, build = constructor(init))]
    payment: Arc<PaymentGateway>,

    // Synchronous dependency depending on async db
    #[singleton(deps = [db])]
    user_service: Arc<UserService>,
}
```

### Generated Initialization:
```rust
impl AppContainer {
    pub async fn init() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config = Arc::new(Config::new());

        // Independent branches run concurrently via tokio::join!
        let (db_res, payment_res) = tokio::join!(
            Database::connect(config.clone()),
            PaymentGateway::init(config.clone()),
        );
        let db = Arc::new(db_res?);
        let payment = Arc::new(payment_res?);

        // Sync constructor runs immediately
        let user_service = Arc::new(UserService::new(db.clone()));

        Ok(Self { config, db, payment, user_service })
    }

    // Synchronous, zero-cost getters:
    pub fn get_config(&self) -> Arc<Config> { self.config.clone() }
    pub fn get_db(&self) -> Arc<Database> { self.db.clone() }
    pub fn get_payment(&self) -> Arc<PaymentGateway> { self.payment.clone() }
    pub fn get_user_service(&self) -> Arc<UserService> { self.user_service.clone() }
}
```

---

## 9. Testing Overrides via Generated Builders

No `build.rs` or runtime configuration files are required for testing. The macro generates a fluent builder that allows overriding any node in the dependency graph:

```rust
#[tokio::test]
async fn test_controller_with_mock_db() {
    let mock_db = Arc::new(AppContainerDb::MockDb(MockDb::new()));

    // Override the DB in the DAG before startup:
    let container = AppContainer::builder()
        .with_db(mock_db)
        .init()
        .await
        .unwrap();

    // Controller receives the mock database automatically:
    let controller = container.get_controller();
    assert_eq!(controller.handle().await, "mocked_response");
}
```

---

## 10. Request-Scoped Child Containers

Child containers allow request-scoped dependencies (e.g. `UserSession`, `RequestId`) to borrow application singletons without re-creating them:

```rust
#[container(parent = AppContainer)]
pub struct RequestContainer<'a> {
    // Provided by HTTP request context
    #[singleton]
    session: Arc<UserSession>,

    // Combines parent's singleton DB with request-scoped session
    #[singleton(deps = [parent(db), session])]
    audit_logger: Arc<AuditLogger>,
}
```

### Usage in Web Handlers (e.g., Axum):
```rust
async fn handle_request(
    State(app): State<Arc<AppContainer>>,
    session: UserSession,
) -> impl IntoResponse {
    let req_container = app.create_scope(|builder| {
        builder.with_session(Arc::new(session))
    });

    let logger = req_container.get_audit_logger();
    logger.log("Action performed");
}
```

---

## 11. Crate Layout

How the crates and the macro's code are organised, and the rules the code follows, are in [ARCHITECTURE.md](ARCHITECTURE.md).

---

## 12. Container Introspection (planned)

> Not implemented in 0.1.0: `introspection = true` is a compile error and the descriptor types were removed from the runtime crate (see [§0](#0-what-ships-in-010)).

Users can request a description of the container's graph in the container settings:

```rust
#[container(introspection = true)]
pub struct AppContainer {
    #[singleton]
    config: Arc<Config>,
    #[singleton(deps = [config])]
    db: Arc<Database>,
    #[transient(deps = [config, db])]
    controller: UserController,
}
```

When `introspection = true`, the code generator emits:

```rust
impl AppContainer {
    /// Returns the complete compile-time graph descriptor
    pub fn dependency_graph() -> snapfit::introspection::GraphDescriptor {
        snapfit::introspection::GraphDescriptor {
            nodes: vec![
                snapfit::introspection::NodeDescriptor { name: "config".into(), lifecycle: "singleton".into() },
                snapfit::introspection::NodeDescriptor { name: "db".into(), lifecycle: "singleton".into() },
                snapfit::introspection::NodeDescriptor { name: "controller".into(), lifecycle: "transient".into() },
            ],
            edges: vec![
                snapfit::introspection::EdgeDescriptor { from: "db".into(), to: "config".into() },
                snapfit::introspection::EdgeDescriptor { from: "controller".into(), to: "config".into() },
                snapfit::introspection::EdgeDescriptor { from: "controller".into(), to: "db".into() },
            ],
        }
    }
}
```
