# `wireup` — Specification & Architecture Document

> **Zero-Cost, Compile-Time Verified, Pure-Rust Dependency Injection Framework**

---

## 1. Vision & Core Philosophy

Most dependency injection frameworks in Rust fall into two undesirable extremes:
1. **Invasive Java/C# ports** (`lockjaw`, `waiter`): Pollute business logic with intrusive macros (`#[inject]`, `#[component]`), require complex DSLs, and rely heavily on dynamic runtime downcasting (`dyn Any`).
2. **Dynamic Service Locators** (`axum::State`, `actix_web::web::Data`, `dptree`): Lack compile-time dependency graph verification, cannot detect cycles, and enforce `Arc<dyn Trait>` runtime indirection.

**`wireup`** solves this by inverting the dependency declaration: **the container struct declares the wiring, leaving domain structs 100% pure Rust.**

### Key Design Pillars:
* **Zero Domain Pollution:** Your services, repositories, and controllers require **zero framework macros or traits**. They are standard Rust structs with plain `pub fn new(...)` constructors.
* **Compile-Time Graph Verification:** The derive macro builds the Directed Acyclic Graph (DAG) during `cargo build` and rejects dependency cycles with compile errors.
* **Zero-Cost Direct Getters:** Direct, compile-time checked getters (`get_<dep>()`) for fast production code — singletons are cloned on read, and transients are freshly constructed on every call with zero `dyn Any` overhead.
* **First-Class Static Polymorphism:** Integrates automatic `enum_dispatch` generation for traits, enabling test mocking and swappable backends with zero vtable overhead.
* **Eager Async Boot with Synchronous Getters:** All async connections (databases, external APIs) are resolved during `Container::init().await`, while runtime getters remain **100% synchronous, non-blocking, and instant** for web request handlers.
* **Child Containers & Testing Overrides:** Fluent `.builder().with_*(mock)` overrides for testing, and macro-driven child containers that copy parent singletons for request-scoped workflows.

---

## 2. Declarative Container Syntax & Settings

A container in `wireup` is declared using the `#[container]` procedural attribute macro.

```rust
use std::sync::Arc;
use wireup::container;

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
    #[singleton(deps = [config, db], builder = "build_auth")]
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
| **`#[singleton(deps = [...], builder = "...")]`** | **Custom Builder** | Calls companion trait `<ContainerName>Builders` method for custom construction. |
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

`wireup` eliminates invasive domain macros entirely. Container wiring is declared centrally in the container definition, keeping all service structs 100% pure Rust.

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
       #[singleton(deps = [config, db], builder = "build_auth")]
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
* **Conditional Generation:** If no fields in the container declare `builder = "..."`, the companion trait is omitted entirely to prevent dead code or unused trait warnings.
* **Absolute Zero Domain Pollution:** Services, repositories, and controllers remain 100% pure standard Rust with zero framework dependencies.

---

## 6. Compile-Time DAG & Cycle Detection

Because all dependencies and their prerequisites are declared within the struct AST, the procedural macro performs graph analysis at compile time:

1. **Graph Construction:** Builds a Directed Acyclic Graph (DAG) using field names as nodes and `deps = [...]` prerequisites as directed edges.
2. **Topological Sort:** Runs Kahn's algorithm at macro expansion time to compute the exact instantiation sequence.
3. **Cycle Rejection:** If a dependency cycle is detected (e.g. `service_a -> service_b -> service_a`), the macro emits an immediate compile error:

```text
error: Dependency cycle detected in AppContainer:
   service_a -> service_b -> service_a
  --> src/main.rs:12:5
   |
12 |     #[singleton(deps = [service_b])]
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

---

## 7. Trait Abstraction via Auto-Generated `enum_dispatch`

To avoid `Arc<dyn Trait>` (heap allocation, vtable indirection, and inability to inline), `wireup` automatically generates static dispatch enums:

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
* **When dynamic dispatch is desired (Zero Trait Macros):** Omit `#[dispatch]` and use `Arc<dyn Database + Send + Sync>` directly. This requires zero annotations on the trait definition and supports dynamic plugin architectures.

---

## 8. Asynchronous Containers (`mode = "async"`)

For applications where databases, message brokers, and clients require asynchronous setup, `wireup` provides dedicated async support.

### Philosophy: Eager Async Boot, Synchronous Getters
* **`AppContainer::init().await`** handles all async setup topologically at boot.
* **Independent async dependencies initialize concurrently** using `tokio::join!`.
* **Runtime getters (`container.get_db()`) remain synchronous, non-blocking, and instant** for web request handlers.

```rust
#[container(mode = "async")]
pub struct AppContainer {
    // Synchronous leaf dependency
    #[singleton]
    config: Arc<Config>,

    // Asynchronous constructor: Database::connect(config).await
    #[singleton(deps = [config], async, constructor = "connect")]
    db: Arc<Database>,

    // Asynchronous constructor: PaymentGateway::init(config).await
    #[singleton(deps = [config], async, constructor = "init")]
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

## 11. Crate Layout & Architecture Blueprint

```text
wireup-rs/
├── Cargo.toml
├── wireup/                  # Core runtime crate (traits, error types, introspection)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Core exports, Container trait, Error
│       ├── container.rs     # Container and ContainerBuilder traits
│       ├── introspection.rs # GraphDescriptor, NodeDescriptor
│       └── error.rs         # WireupError definitions
├── wireup_macros/           # Procedural macros (Hexagonal Ports & Adapters)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Macro entry points (#[container])
│       ├── domain/          # Pure Domain IR, errors, and construction plan (No syn/quote)
│       ├── ports/           # ParserPort and EmitterPort trait boundaries
│       ├── use_cases/       # GraphEngine (Kahn's DAG topo sort), PlanContainer
│       └── adapters/
│           ├── inbound/     # syn/darling AST parser
│           └── outbound/    # quote token emitter
├── xtask/                   # Developer automation and CI gatekeeper
└── tests/                   # BDD integration test suite (cucumber-rs)
    └── features/
        └── container_resolution.feature
```
