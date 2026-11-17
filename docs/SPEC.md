# `snapfit` — Specification & Architecture Document

> **Zero-Cost, Compile-Time Verified, Pure-Rust Dependency Injection Framework**

---

## 0. What ships in 0.1.0

This document describes the full design. Only part of it ships as a stable API in 0.1.0; the rest is either behind the `unstable` Cargo feature or not implemented yet. Using a planned feature, or an unstable one without the feature, is a compile error that names the feature.

| Capability | 0.1.0 status | Where it is specified |
| :--- | :--- | :--- |
| `#[container]` and `#[derive(Container)]` with `#[singleton]` fields | Stable | §2, §3, §4 |
| `deps = [...]` named edges, multiple instances of one type | Stable | §2, §5.A |
| `constructor = <fn>` (a bare function name, `constructor = connect`) | Stable | §5.A |
| Compile-time missing-dependency and cycle detection | Stable | §6 |
| `Container` trait, `new()`, `Default` | Stable | §2 |
| Async containers: `mode = "async"`, `#[singleton(async)]`, `init().await` (sequential) | Unstable | §8 |
| `#[transient]` dependencies | Unstable | §4 |
| `builder = <fn>` (associated function on the container) | Unstable | §5.B |
| `<ContainerName>Builders` companion trait | Planned | §5.B |
| Concurrent async boot (`join!` over independent async singletons) | Planned | §8 |
| `#[dispatch(...)]` static trait dispatch | Planned | §7 |
| Test overrides (`builder().with_*()` / `fork_with()`) | Planned | §9 |
| Child containers (`parent = ...`, `parent(dep)`) | Planned | §10 |
| Introspection (`introspection = true`, `dependency_graph()`) | Planned | §12 |
| `#[external]` inputs and `before_init` computed fields | Planned | §2 |

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

### Decided syntax for later versions

* **Async containers get their own macro** (decided 2026-11-16, not implemented yet): `#[async_container]` replaces `#[container(mode = "async")]`, and `mode` is removed. Inside it, async dependencies are marked with the bare `async` flag (`#[singleton(deps = [config], async, constructor = connect)]`); everything else stays sync. `async` inside a plain `#[container]` becomes an error pointing to `#[async_container]`. Until then the unstable `mode = "async"` syntax keeps working.

### Container shapes refused in 0.1.0

These are compile errors that point at the offending field or container, instead of generating code that does not compile:

* **Generic containers** (`struct Repo<T>`): planned; the generated `impl` blocks and getters need a design for bounds first.
* **`#[cfg]` / `#[cfg_attr]` on a dependency field**: conditional dependencies would change the graph per build, so they are refused.
* **Trait-object fields** (`Arc<dyn PaymentGateway>`) without `builder = <fn>`: a trait object has no constructor to call. On the stable build the error tells the user to replace `dyn` and the trait in the field's type with the concrete type that implements the trait, keeping the `Arc` around it (`Arc<StripeGateway>`); a constructor that takes `Arc<dyn PaymentGateway>` still accepts that field, because Rust converts the `Arc` where the generated code passes it. On the `unstable` build the error suggests a builder named after the field (`builder = build_payments`) instead. `#[dispatch(...)]` is another way once implemented.
* **`constructor` or `builder` values that are not a plain function name** (`"connect()"`, `"Db::connect"`).

---

## 1. Vision & Core Philosophy

Most dependency injection frameworks in Rust fall into two undesirable extremes:
1. **Invasive Java/C# ports** (`lockjaw`, `waiter`): Pollute business logic with intrusive macros (`#[inject]`, `#[component]`), require complex DSLs, and rely heavily on dynamic runtime downcasting (`dyn Any`).
2. **Dynamic Service Locators** (`axum::State`, `actix_web::web::Data`, `dptree`): Lack compile-time dependency graph verification, cannot detect cycles, and enforce `Arc<dyn Trait>` runtime indirection.

**`snapfit`** solves this by inverting the dependency declaration: **the container struct declares the wiring, leaving domain structs 100% pure Rust.**

### Key Design Pillars:
* **Zero Domain Pollution:** Your services, repositories, and controllers require **zero framework macros or traits**. They are standard Rust structs with plain `pub fn new(...)` constructors.
* **Compile-Time Graph Verification:** The derive macro builds the Directed Acyclic Graph (DAG) during `cargo build` and rejects dependency cycles with compile errors.
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

    // 5. Custom builder (when constructor isn't standard ::new)
    #[singleton(deps = [config, db], builder = build_auth)]
    auth: Arc<AuthService>,

    // 6. Static polymorphism (Post-MVP): Macro rewrites field to generated enum!
    #[dispatch(PostgresDb, MockDb)]
    database: Arc<dyn Database>,
}
```

### Container Settings Reference (`#[container(...)]`):

| Setting | Options | Default | Description |
| :--- | :--- | :--- | :--- |
| **`mode`** | `"async"`, `"sync"` | `"sync"` *(or auto `"async"` if any field is `async`)* | Controls whether the container produces `pub async fn init() -> Result<Self, ...>` or synchronous `pub fn new() -> Self`. |
| **`parent`** | Type Identifier | None | Marks this container as a Child Scope copying/inheriting singletons from a parent container. |
| **`introspection`** | `bool` | `false` | When `true`, generates `.dependency_graph()` and runtime inspection methods for BDD testing or runtime visualization. |

### Field-Level Attribute Vocabulary:

| Attribute | Role | Structural & Memory Impact |
| :--- | :--- | :--- |
| **`#[singleton]`** | **Singleton** | Stored in container struct. Initialized once at boot. Cloned on read. |
| **`#[singleton(deps = [a, b])]`** | **Singleton (With Dependencies)** | Stored in container struct. Created at boot passing dependencies in order. |
| **`#[transient]`** | **Transient** | **Stripped from struct memory storage.** Reconstructed on each `get_<name>()`. |
| **`#[transient(deps = [a])]`** | **Transient (With Dependencies)** | **Stripped from struct memory storage.** Reconstructed on each `get_<name>()` passing `a`. |
| **`#[singleton(deps = [...], builder = <fn>)]`** | **Custom Builder** | Calls companion trait `<ContainerName>Builders` method for custom construction. |
| **`#[dispatch(ImplA, ImplB)]`** | **Static Trait (Post-MVP)** | Auto-generates `enum_dispatch` and **rewrites the struct field type** to the enum. |
| **`#[external]`** | **External Input (Post-MVP)** | Passed as a parameter to `new(...)` / `init(...)` and exposed as a wireable leaf node in the DAG. |
| *(Unannotated field)* | **Pre-Init Computed (Post-MVP)** | Populated via an optional `before_init` hook prior to DAG wiring and exposed as a wireable node in the DAG. |



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

## 5. Dependency Wiring & Companion Builders Trait

`snapfit` eliminates invasive domain macros entirely. Container wiring is declared centrally in the container definition, keeping all service structs 100% pure Rust.

### A. Standard Constructors (`::new`)
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

### B. Custom Builders via Companion Trait (`<ContainerName>Builders`)
When a dependency requires custom construction (such as unpacking config fields, calling a non-`new` constructor, or configuring an external client):

1. Declare the builder method name:
   ```rust
   #[container]
   pub struct AppContainer {
       #[singleton]
       config: Arc<Config>,

       #[singleton(deps = [config])]
       db: Arc<Database>,

       // Custom builder declared on the field:
       #[singleton(deps = [config, db], builder = build_auth)]
       auth: Arc<AuthService>,
   }
   ```

2. The `#[container]` macro auto-generates a companion trait with the exact signature:
   ```rust
   pub trait AppContainerBuilders {
       fn build_auth(config: &Config, db: Arc<Database>) -> Arc<AuthService>;
   }
   ```

3. Implement the companion trait on your container:
   ```rust
   impl AppContainerBuilders for AppContainer {
       fn build_auth(config: &Config, db: Arc<Database>) -> Arc<AuthService> {
           AuthService::from_vault(config.vault_url(), db)
       }
   }
   ```

* **IDE Auto-Scaffolding:** In `rust-analyzer` or any LSP-enabled editor, typing `impl AppContainerBuilders for AppContainer {}` provides an immediate "Implement missing members" code action that generates the exact function stub with correct types.
* **Conditional Generation:** If no fields in the container declare `builder = <fn>`, the companion trait is omitted entirely to prevent dead code or unused trait warnings.
* **Absolute Zero Domain Pollution:** Services, repositories, and controllers remain 100% pure standard Rust with zero framework dependencies.

---

## 6. Compile-Time Graph Checks & Diagnostics

Because all dependencies and their prerequisites are declared within the struct, the procedural macro checks the whole graph at compile time:

1. **Construction order:** Each field's `deps = [...]` names the fields it needs. The macro visits the fields in the order they are declared and builds each one after the fields it needs, so the construction order follows the declaration wherever the graph allows it. Declaring a field before the fields it needs is allowed.
2. **Unknown dependencies:** A `deps` entry that names no field of the container is refused, naming every unknown entry of that field and every field the container can wire.
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
* **When dynamic dispatch is desired (Zero Trait Macros):** Omit `#[dispatch]` and use `Arc<dyn Database + Send + Sync>` directly, built by a `builder = <fn>`. This requires zero annotations on the trait definition and supports dynamic plugin architectures. Builders are unstable in 0.1.0, so on the stable build the field holds the concrete type and a constructor takes the trait object instead (§0, "Container shapes refused in 0.1.0").

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
    #[singleton(deps = [config], async, constructor = connect)]
    db: Arc<Database>,

    // Asynchronous constructor: PaymentGateway::init(config).await
    #[singleton(deps = [config], async, constructor = init)]
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
