# `snapfit` — Specification & Architecture Document

> **Zero-Cost, Compile-Time Verified, Pure-Rust Dependency Injection Framework**

---

## 0. What ships in 0.1.0

This document describes the full design. Only part of it ships as a stable API in 0.1.0; the rest is either behind the `unstable` Cargo feature or not implemented yet. Using a planned feature, or an unstable one without the feature, is a compile error that names the feature.

| Capability | 0.1.0 status | Where it is specified |
| :--- | :--- | :--- |
| `#[container]` with `#[singleton]` fields | Stable | §2, §3, §4 |
| `deps = [...]` named edges, multiple instances of one type | Stable | §2, §5.A |
| `build = default` and `build = constructor(<fn>)` (a bare function name, `build = constructor(connect)`) | Stable | §5.A |
| `#[container(leaf_build = ...)]` for fields with no `deps` | Stable | §2, §5 |
| Factories: `build = factory` and the generated `<ContainerName>Factories` trait | Stable | §5.B |
| Builders: `build = builder`, dependency labels and `builder(start = ..., finish = ...)` | Stable | §5.C |
| Compile-time missing-dependency and cycle detection | Stable | §6 |
| `Container` trait, `new()`, `Default` | Stable | §2 |
| Async containers: `mode = "async"`, `#[singleton(async)]`, `init().await` (sequential) | Unstable | §8 |
| `#[transient]` dependencies | Unstable | §4 |
| Concurrent async boot (`join!` over independent async singletons) | Planned | §8 |
| `#[dispatch(...)]` static trait dispatch | Planned | §7 |
| Test overrides (`builder().with_*()` / `fork_with()`) | Planned | §9 |
| Child containers (`parent = ...`, `parent(dep)`) | Planned | §10 |
| Introspection (`introspection = true`, `dependency_graph()`) | Planned | §12 |
| `#[external]` inputs and `before_init` computed fields | Planned | §2 |

Each unstable and planned capability has an [open issue labelled `feature`](https://github.com/marlon-sousa/snapfit-rs/issues?q=is%3Aissue%20is%3Aopen%20label%3Afeature), which holds what is decided about it and what it must not break.

Enable the unstable APIs with:

```toml
[dependencies]
snapfit = { version = "0.1", features = ["unstable"] }
```

### Removed from the 0.1.0 runtime crate (to restore later)

These items existed before 0.1.0 but nothing generated or used them, so they were removed rather than shipped as unused public API. Each one comes back with the feature that needs it:

* **`AsyncContainer` trait** (`fn init() -> impl Future<Output = Result<Self, Self::Error>>`): return with stabilised async containers. The generated `init()` currently returns `Box<dyn Error + Send + Sync>`, which cannot satisfy the old `Error: std::error::Error` bound, so the error type must be designed first (ideally naming which dependency failed).
* **`introspection` module** (`GraphDescriptor`, `NodeDescriptor`, `EdgeDescriptor`): return with `introspection = true`.
* **`SnapfitError` and the `Error` alias** (`ResolutionFailed`, `MissingDependency`, `InitializationFailed`): return when async initialization errors get dependency context.
* **`ConstructionPlan::parallel_batches`** (groups of independent async singletons): the planner computed them but the generator never used them. They return with concurrent async boot.

### Removed from the 0.1.0 macros

* **`#[derive(Container)]`**: removed before 0.1.0 ([#32](https://github.com/marlon-sousa/snapfit-rs/issues/32)), so that a container has one way to be declared. A derive can only add code next to the struct, never rewrite it, so it could never take a transient, a dispatched field or a parent, each of which leaves the struct changed; on the stable build it generated the same code as `#[container]`. `#[derive(Container)]` is now the compiler's own "cannot find derive macro" error. The `Container` trait stays. Adding a derive form back later would break no one, but it needs a reason `#[container]` cannot meet.

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
* **Trait-object fields** (`Arc<dyn PaymentGateway>`) without `build = factory`: a trait object has no constructor or builder to call. The error, the same on both builds, tells the user to build the field with a factory (§5.B), whatever wrapper held the trait object. Holding the concrete type (`Arc<StripeGateway>`) still works, and a constructor that takes `Arc<dyn PaymentGateway>` accepts that field, because Rust converts the `Arc` where the generated code passes it. `#[dispatch(...)]` is another way once implemented.
* **`build` values that are not a way of building** (§5), and **`constructor(...)` values that are not a plain function name** (`"connect()"`, `Db::connect`).
* **`build = default` with `deps`**: `Default::default()` takes nothing, so the field's `deps` would never be passed.
* **`build = default` on an async dependency** (unstable): `Default::default()` cannot be awaited.
* **`build = factory` on an async dependency** (unstable): planned, refused as not implemented yet, because a factory that awaits needs an async trait method, designed with `#[async_container]` ([#37](https://github.com/marlon-sousa/snapfit-rs/issues/37)).
* **A dependency label without `build = builder`** (`deps = [disk = folder_files]` on a field built another way): only a builder has setters for a label to name.
* **A `deps` entry that is not another field's name**, a label written `setter = field` or `parent(<name>)`, such as `config + 1` or `a::b = config`, and a `deps` that is not a list in square brackets, such as `deps = config`, or that writes its list in parentheses, such as `deps(config)`, which is told to write the same entries in square brackets, `deps = [config]` ([#69](https://github.com/marlon-sousa/snapfit-rs/issues/69)). Each bad entry is its own error at the field, and every one is reported in the same compile, beside any other problem of the same attribute ([#65](https://github.com/marlon-sousa/snapfit-rs/issues/65)).
* **A repeated `deps` entry**, the same field passed again to the same setter, such as `deps = [config, config]` or `deps = [disk = folder_files, disk = folder_files]`: it is refused at the field, once for each entry repeated ([#69](https://github.com/marlon-sousa/snapfit-rs/issues/69)). A builder may still pass one field to two setters, such as `deps = [reader = pool, writer = pool]`.
* **Two `deps` entries that pass different fields to the same builder setter**, and **`builder(...)` settings other than `start` and `finish` with a bare function name each.**
* **`build = builder` on an async dependency** (unstable): planned, refused as not implemented yet, like an async factory.

---

## 1. Vision & Core Philosophy

Most dependency injection frameworks in Rust fall into two undesirable extremes:
1. **Invasive Java/C# ports** (`lockjaw`, `waiter`): Pollute business logic with intrusive macros (`#[inject]`, `#[component]`), require complex DSLs, and rely heavily on dynamic runtime downcasting (`dyn Any`).
2. **Dynamic Service Locators** (`axum::State`, `actix_web::web::Data`, `dptree`): Lack compile-time dependency graph verification, cannot detect cycles, and enforce `Arc<dyn Trait>` runtime indirection.

**`snapfit`** solves this by inverting the dependency declaration: **the container struct declares the wiring, leaving domain structs 100% pure Rust.**

### Key Design Pillars:
* **Zero Domain Pollution:** Your services, repositories, and controllers require **zero framework macros or traits**. They are standard Rust structs with plain `pub fn new(...)` constructors.
* **Compile-Time Graph Verification:** The container macro builds the Directed Acyclic Graph (DAG) during `cargo build` and rejects dependency cycles with compile errors.
* **Zero-Cost Direct Getters:** Direct, compile-time checked getters (`get_<dep>()`) for fast production code — singletons are cloned on read, and transients are freshly constructed on every call with zero `dyn Any` overhead.
* **First-Class Static Polymorphism:** Integrates automatic `enum_dispatch` generation for traits, enabling test mocking and swappable backends with zero vtable overhead.
* **Eager Async Boot with Synchronous Getters:** All async connections (databases, external APIs) are resolved during `Container::init().await`, while runtime getters remain **100% synchronous, non-blocking, and instant** for web request handlers.
* **Child Containers & Testing Overrides:** Fluent `.builder().with_*(mock)` overrides for testing, and macro-driven child containers that copy parent singletons for request-scoped workflows.

---

## 2. Declarative Container Syntax & Settings

A container in `snapfit` is declared using the `#[container]` procedural attribute macro.

```rust
use std::sync::Arc;
use snapfit::container;

#[container]
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

    // 6. Static polymorphism (Planned): Macro rewrites field to generated enum!
    #[dispatch(PostgresDb, MockDb)]
    database: Arc<dyn Database>,
}
```

### Container Settings Reference (`#[container(...)]`):

| Setting | Options | Default | Description |
| :--- | :--- | :--- | :--- |
| **`mode`** | `"async"`, `"sync"` | `"sync"` *(or auto `"async"` if any field is `async`)* | Controls whether the container produces `pub async fn init() -> Result<Self, ...>` or synchronous `pub fn new() -> Self`. |
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
| **`#[dispatch(ImplA, ImplB)]`** | **Static Trait (Planned)** | Auto-generates `enum_dispatch` and **rewrites the struct field type** to the enum. |
| **`#[external]`** | **External Input (Planned)** | Passed as a parameter to `new(...)` / `init(...)` and exposed as a wireable leaf node in the DAG. |
| *(Unannotated field)* | **Pre-Init Computed (Planned)** | Populated via an optional `before_init` hook prior to DAG wiring and exposed as a wireable node in the DAG. |

A container written by a `macro_rules!` is read as if written by hand: a setting's value, a `deps` entry or any part of one that the macro passes as `$e:expr` means what the same words would mean typed into the attribute ([#64](https://github.com/marlon-sousa/snapfit-rs/issues/64)).



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

---

## 4. Dependency Lifecycles & Struct Storage Rewriting

The `#[container]` attribute macro actively shapes the memory layout of the generated container struct:

| Lifecycle | Declaration | Storage in Container Struct | Resolution Behavior |
| :--- | :--- | :--- | :--- |
| **Singleton** | `#[singleton]` or `#[singleton(deps = [...])]` | **Stored in struct.** | Constructed once during `init()`/`new()`. Returned by `.clone()`. |
| **Transient** | `#[transient]` or `#[transient(deps = [...])]` | **Stripped from struct memory.** | **Zero struct bytes allocated.** Reconstructed on every `get_<name>()` call. |
| **Static Trait** | `#[dispatch(...)] name: dyn Trait` | **Field rewritten to static enum.** | Enum generated via `enum_dispatch`; zero heap allocation or vtable overhead. |
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

A container can say once how its leaves are built: `#[container(leaf_build = default)]` builds every field that has no `deps` and no `build` of its own as if it said `build = default`, which suits a project whose leaf types all derive `Default`. `leaf_build` applies to synchronous fields only, because `Default::default()` and a constructor chosen for the whole container cannot be awaited; an async field keeps its own constructor. `leaf_build` takes the values a leaf can take, `default` or `constructor(<fn>)`; leaving it out is `constructor(new)`, the behaviour above. `leaf_build` only changes the default for leaves, such as from `new` to `default`: a field's own `build` always wins, so a leaf that says `build = constructor(sandbox)` is still built by `sandbox`; a field with `deps` is never affected; and every field still needs its `#[singleton]`. Any other value, such as `leaf_build = factory`, is refused at the container's name.

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

* **Names:** the trait is `<ContainerName>Factories`. Each method is `build_<field>`, an associated function with no `self`, because the container does not exist yet while it is being built, and it returns the field's type exactly as declared. Each inputs struct is `<ContainerName><Field>Inputs`, the field's name in PascalCase (`order_store` gives `ShopOrderStoreInputs`).
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
* **Trait objects:** a trait-object field has no type to call `builder()` on, so `build = builder` on it is refused with the trait-object error of §0, which points to a factory.
* **Async (unstable):** `build = builder` on an async dependency is planned, refused as not implemented yet, for the same reason as an async factory.
* **No dependency on a builder crate:** snapfit only writes the chain of calls. The builder crate, if any, is the user's own dependency.

---

## 6. Compile-Time Graph Checks & Diagnostics

Because all dependencies and their prerequisites are declared within the struct, the procedural macro checks the whole graph at compile time:

1. **Construction order:** Each field's `deps = [...]` names the fields it needs. The macro visits the fields in the order they are declared and builds each one after the fields it needs, so the construction order follows the declaration wherever the graph allows it. Declaring a field before the fields it needs is allowed.
2. **Unknown dependencies:** A `deps` entry that names no field of the container is refused, naming every unknown entry of that field once, even one it lists twice, and every field the container can wire.
3. **Cycles:** A field that needs itself, directly or through other fields, is refused. The message reads as a sentence, without arrows, so that a screen reader speaks it naturally, and it points at the first field of the cycle in declaration order:

```text
error: dependency cycle: service_a depends on service_b, which depends on service_a
  --> src/main.rs:12:5
   |
12 |     #[singleton(deps = [service_b])]
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

A field that needs only itself reads `dependency cycle: service_a depends on itself`.

4. **Every problem at once:** The macro reports every problem it finds in one compile, each as its own error pointing at the field it is about (its snapfit attribute, or its name when it has none), or at the container's name when no field is at fault. Problems found while reading the fields, such as a missing `#[singleton]` or a quoted function name, are reported together; once every field reads, the unknown dependencies, cycles, refused features and other rules of the declaration are reported together. The struct is kept as written, without snapfit's attributes, so that the compiler adds no follow-on errors about a missing type.
5. **Only what the build accepts:** snapfit's own advice on what to write offers only what the build in use accepts, and never a planned feature. On the stable build it offers no unstable feature: a field with no snapfit attribute reads `field 'config' has no snapfit attribute; add #[singleton] to have the container build it`, and an unknown mode reads `invalid container mode 'threaded': write mode = "sync", or leave mode out`, the same on both builds. A feature the user wrote is still named when it is refused, such as `#[dispatch(...)]` being planned. One exception is not snapfit's own advice: for a misspelt or unknown setting, darling, which reads the attributes, either suggests the closest name it knows, such as "Did you mean `build`?", or lists the names it knows, such as "Available values: `async`, `build`, `deps`", which could in principle include an unstable or planned setting.

---

## 7. Trait Abstraction via Auto-Generated `enum_dispatch`

To avoid `Arc<dyn Trait>` (heap allocation, vtable indirection, and inability to inline), `snapfit` automatically generates static dispatch enums:

### User Code:
```rust
pub trait Database {
    fn query(&self, sql: &str) -> String;
}

pub struct PostgresDb;
impl Database for PostgresDb { ... }

pub struct MockDb;
impl Database for MockDb { ... }

#[container]
pub struct AppContainer {
    // Declares the trait and concrete implementations via #[dispatch(...)]
    #[dispatch(PostgresDb, MockDb)]
    db: Arc<dyn Database>,

    #[transient(deps = [db])]
    controller: Controller,
}
```

### Generated Code (Struct Rewriting & Enum Dispatch):
The attribute macro automatically creates the static dispatch enum and **rewrites the struct field type** so the user never has to invent or manage intermediate enum types:

```rust
// 1. Auto-generated static dispatch enum:
#[derive(Clone)]
#[enum_dispatch::enum_dispatch(Database)]
pub enum AppContainerDb {
    PostgresDb(PostgresDb),
    MockDb(MockDb),
}

// 2. Struct field is automatically rewritten by the attribute macro:
pub struct AppContainer {
    db: Arc<AppContainerDb>, // Rewritten from Arc<dyn Database>
}
```
* **Performance:** 100% static dispatch, compiler-inlined calls, zero `dyn Any`, zero `vtable`.
* **Trait Annotation Contract:** Because Rust procedural macros operate syntactically and cannot inspect external trait method definitions, static dispatch leverages `enum_dispatch` (which requires `#[enum_dispatch]` on the trait).
* **When dynamic dispatch is desired (Zero Trait Macros):** Omit `#[dispatch]` and use `Arc<dyn Database + Send + Sync>` directly, built by a factory (`build = factory`, §5.B). This requires zero annotations on the trait definition and supports dynamic plugin architectures.

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
