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
* **Dual Resolution APIs:** Direct, zero-cost generated getters (`get_<dep>()`) for fast production code, plus a type-checked key lookup (`get_dependency::<T>(key)`) with descriptive runtime diagnostics.
* **Auto-Generated Key Enum:** The macro automatically creates a strongly-typed `enum` of all dependency keys with `&str` and `FromStr` conversions.
* **First-Class Static Polymorphism:** Integrates automatic `enum_dispatch` generation for traits, enabling test mocking and swappable backends with zero vtable overhead.
* **Eager Async Boot with Synchronous Getters:** All async connections (databases, external APIs) are resolved during `Container::init().await`, while runtime getters remain **100% synchronous, non-blocking, and instant** for web request handlers.
* **Child Containers & Testing Overrides:** Fluent `.builder().with_*(mock)` overrides for testing, and macro-driven child containers that copy parent singletons for request-scoped workflows.

---

## 2. Declarative Container Syntax & Settings

A container in `wireup` is a standard Rust struct annotated with `#[derive(Container)]`. Global behavior is configured using the `#[container(...)]` helper attribute.

```rust
use std::sync::Arc;
use wireup::Container;

#[derive(Container)]
#[container(
    mode = "async",       // "async" | "sync" (auto-detected if any field is async)
    dispatch = "enum",    // "enum" (zero-cost static) | "dyn" (dynamic trait objects)
)]
pub struct AppContainer {
    // 1. Leaf dependency (calls Config::new())
    #[dep]
    config: Arc<Config>,

    // 2. Singleton with prerequisites (calls Database::new(config))
    #[dep(config)]
    db: Arc<Database>,

    // 3. Transient dependency: freshly instantiated on every get
    #[dep(config, db, transient)]
    user_controller: UserController,

    // 4. Custom constructor method name
    #[dep(config, constructor = "from_config")]
    cache: Arc<RedisCache>,

    // 5. User-provided custom factory (skips automatic ::new call)
    #[dep(custom = "build_auth_service")]
    auth: Arc<AuthService>,
}
```

### Container Settings Reference (`#[container(...)]`):

| Setting | Options | Default | Description |
| :--- | :--- | :--- | :--- |
| **`mode`** | `"async"`, `"sync"` | `"sync"` *(or auto `"async"` if any field is `async`)* | Controls whether the container produces `pub async fn init() -> Result<Self, ...>` or synchronous `pub fn new() -> Self`. |
| **`dispatch`** | `"enum"`, `"dyn"` | `"enum"` | Controls trait field resolution. `"enum"` generates `enum_dispatch` for 0 vtables; `"dyn"` produces `Arc<dyn Trait>` for open-ended plugin architectures. |
| **`parent`** | Type Identifier | None | Marks this container as a Child Scope copying/inheriting singletons from a parent container. |
| **`introspection`** | `bool` | `false` | When `true`, generates `.dependency_graph()` and runtime inspection methods for BDD testing or runtime visualization. |

---

## 3. The Generated Key Enum & String Mappings

When `#[derive(Container)]` processes the struct fields, it automatically generates a companion key enum. 

For the `AppContainer` above, the macro generates:

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AppContainerKey {
    Config,
    Db,
    UserController,
    Cache,
    Auth,
}

impl AppContainerKey {
    /// Returns the snake_case identifier matching the field name.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Db => "db",
            Self::UserController => "user_controller",
            Self::Cache => "cache",
            Self::Auth => "auth",
        }
    }
}

impl std::convert::AsRef<str> for AppContainerKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::str::FromStr for AppContainerKey {
    type Err = wireup::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "config" => Ok(Self::Config),
            "db" => Ok(Self::Db),
            "user_controller" => Ok(Self::UserController),
            "cache" => Ok(Self::Cache),
            "auth" => Ok(Self::Auth),
            unknown => Err(wireup::Error::UnknownKey(unknown.to_string())),
        }
    }
}
```

---

## 4. Dual Resolution APIs

`wireup` provides two complementary ways to retrieve dependencies:

### A. Strongly-Typed Direct Getters (Zero-Cost, Compile-Time Checked)
For each field `foo: T`, the macro emits a direct getter:

```rust
impl AppContainer {
    pub fn get_config(&self) -> Arc<Config> { self.config.clone() }
    pub fn get_db(&self) -> Arc<Database> { self.db.clone() }
    pub fn get_user_controller(&self) -> UserController {
        // Transient: freshly evaluates constructor on every call
        UserController::new(self.get_config(), self.get_db())
    }
}
```

### B. Key-Based Resolution (`get_dependency::<T>`)
For dynamic dispatch, testing, or reflection-style access, the container provides generic key-based lookup accepting either `AppContainerKey` or string slices (`&str`):

```rust
impl AppContainer {
    /// Generic lookup accepting an enum key or &str.
    pub fn get_dependency<T: 'static + Clone>(&self, key: impl AsRef<str>) -> T {
        let key_str = key.as_ref();
        let enum_key: AppContainerKey = key_str.parse().unwrap_or_else(|_| {
            panic!("[wireup Error] Unknown dependency key: \"{}\"", key_str);
        });

        match enum_key {
            AppContainerKey::Config => self.downcast_dep::<Arc<Config>, T>(self.get_config(), key_str),
            AppContainerKey::Db => self.downcast_dep::<Arc<Database>, T>(self.get_db(), key_str),
            AppContainerKey::UserController => self.downcast_dep::<UserController, T>(self.get_user_controller(), key_str),
            AppContainerKey::Cache => self.downcast_dep::<Arc<RedisCache>, T>(self.get_cache(), key_str),
            AppContainerKey::Auth => self.downcast_dep::<Arc<AuthService>, T>(self.get_auth(), key_str),
        }
    }

    fn downcast_dep<Actual: 'static + Clone, Target: 'static>(&self, val: Actual, key: &str) -> Target {
        // Safely verify matching TypeIds and cast
        let any_val = &val as &dyn std::any::Any;
        match any_val.downcast_ref::<Target>() {
            Some(matched) => matched.clone(),
            None => {
                panic!(
                    "\n[wireup Error] Type mismatch for dependency key: \"{}\"\n\
                     Expected type: {}\n\
                     Actual type:   {}\n",
                    key,
                    std::any::type_name::<Target>(),
                    std::any::type_name::<Actual>()
                );
            }
        }
    }
}
```

---

## 5. Dependency Lifecycles & The Service Cache

`wireup` handles lifetimes with optimal memory and thread safety:

| Lifecycle | Declaration | Storage & Resolution Behavior |
| :--- | :--- | :--- |
| **Singleton** *(Default)* | `#[dep(...)]` | Constructed once during `init()`/`new()`, stored in the container struct as `Arc<T>` (or initialized lazily via `std::sync::OnceLock<Arc<T>>`). Thread-safe, lock-free clones on reads. |
| **Transient** | `#[dep(..., transient)]` | **Never cached in the service cache.** Evaluates the constructor on every getter call and returns a fresh instance. |
| **Scoped** | `#[container(parent = ...)]` | Defined inside a Child Container. Clones/borrows parent singletons and instantiates request-local instances for the scope duration. |

---

## 6. Dependency Wiring: Container-Driven vs. Component-Driven

`wireup` supports **Container-Driven Wiring** as its primary pattern, keeping business domain models 100% pure.

### Primary: Container-Driven Wiring (Zero Domain Macros)
The container declares which fields are passed into each constructor:
```rust
#[derive(Container)]
pub struct AppContainer {
    #[dep]
    config: Arc<Config>,

    #[dep(config)]
    db: Arc<Database>,

    #[dep(config, db, transient)]
    controller: Controller,
}
```
* **Domain Model:**
  ```rust
  pub struct Controller {
      config: Arc<Config>,
      db: Arc<Database>,
  }

  impl Controller {
      pub fn new(config: Arc<Config>, db: Arc<Database>) -> Self {
          Self { config, db }
      }
  }
  ```

### Optional Secondary: Parameter-Level Injection (`#[injectable]`)
For teams that prefer constructor-level self-wiring, `wireup` also provides an optional companion macro:
```rust
#[injectable]
impl UserService {
    pub fn new(
        #[inject("read_db")] db: Arc<Database>, // custom key
        #[inject] cache: Arc<Cache>,            // defaults to param name "cache"
    ) -> Self {
        Self { db, cache }
    }
}
```
When present, `#[injectable]` implements a trait allowing the container to auto-wire the component by reading its requested parameter keys.

---

## 7. Compile-Time DAG & Cycle Detection

Because all dependencies and their prerequisites are declared within the struct AST, the procedural macro performs graph analysis at compile time:

1. **Graph Construction:** Builds a Directed Acyclic Graph (DAG) using field names as nodes and `#[dep(...)]` arguments as directed edges.
2. **Topological Sort:** Runs Kahn's algorithm at macro expansion time to compute the exact instantiation sequence.
3. **Cycle Rejection:** If a dependency cycle is detected (e.g. `service_a -> service_b -> service_a`), the macro emits an immediate compile error:

```text
error: Dependency cycle detected in AppContainer:
   service_a -> service_b -> service_a
  --> src/main.rs:12:5
   |
12 |     #[dep(service_b)]
   |     ^^^^^^^^^^^^^^^^^
```

---

## 8. Trait Abstraction via Auto-Generated `enum_dispatch`

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

#[derive(Container)]
pub struct AppContainer {
    // Automatically generates an enum AppDb implementing Database
    #[dep(trait = Database, impls = [PostgresDb, MockDb])]
    db: Arc<AppDb>,

    #[dep(db)]
    controller: Controller,
}
```

### Generated Code:
```rust
#[derive(Clone)]
#[enum_dispatch::enum_dispatch(Database)]
pub enum AppDb {
    PostgresDb(PostgresDb),
    MockDb(MockDb),
}
```
* **Performance:** 100% static dispatch, compiler-inlined calls, zero `dyn Any`, zero `vtable`.
* **When `dispatch = "dyn"` is selected:** Emits `Arc<dyn Database + Send + Sync>` instead, supporting open-ended dynamic plugin architectures.

---

## 9. Asynchronous Containers (`mode = "async"`)

For applications where databases, message brokers, and clients require asynchronous setup, `wireup` provides dedicated async support.

### Philosophy: Eager Async Boot, Synchronous Getters
* **`AppContainer::init().await`** handles all async setup topologically at boot.
* **Independent async dependencies initialize concurrently** using `tokio::join!`.
* **Runtime getters (`container.get_db()`) remain synchronous, non-blocking, and instant** for web request handlers.

```rust
#[derive(Container)]
#[container(mode = "async")]
pub struct AppContainer {
    // Synchronous leaf dependency
    #[dep]
    config: Arc<Config>,

    // Asynchronous constructor: Database::connect(config).await
    #[dep(config, async, constructor = "connect")]
    db: Arc<Database>,

    // Asynchronous constructor: PaymentGateway::init(config).await
    #[dep(config, async, constructor = "init")]
    payment: Arc<PaymentGateway>,

    // Synchronous dependency depending on async db
    #[dep(db)]
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

## 10. Testing Overrides via Generated Builders

No `build.rs` or runtime configuration files are required for testing. The macro generates a fluent builder that allows overriding any node in the dependency graph:

```rust
#[tokio::test]
async fn test_controller_with_mock_db() {
    let mock_db = Arc::new(AppDb::MockDb(MockDb::new()));

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

## 11. Request-Scoped Child Containers

Child containers allow request-scoped dependencies (e.g. `UserSession`, `RequestId`) to borrow application singletons without re-creating them:

```rust
#[derive(Container)]
#[container(parent = AppContainer)]
pub struct RequestContainer<'a> {
    // Provided by HTTP request context
    #[dep]
    session: Arc<UserSession>,

    // Combines parent's singleton DB with request-scoped session
    #[dep(parent(db), session)]
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

## 12. Crate Layout Blueprint

```text
wireup/
├── Cargo.toml
├── wireup/                  # Core runtime crate (traits, error types)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Core traits, Container, Injectable, Error
│       └── error.rs         # WireupError definitions
├── wireup_macros/           # Procedural macros
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # #[derive(Container)], #[injectable]
│       ├── ast.rs           # Parsing container struct fields & #[dep] / #[container] attrs
│       ├── graph.rs         # DAG construction, topological sort, cycle detection
│       ├── codegen.rs       # Emitting getters, builders, and ::init()
│       └── enum_dispatch.rs # Auto-generating static dispatch enums
└── tests/                   # Integration tests
    ├── sync_container.rs
    ├── async_container.rs
    ├── overrides.rs
    ├── child_container.rs
    └── enum_dispatch_test.rs
```
