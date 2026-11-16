# `wireup` — Technical Specification & Internal Architecture

> **Internal Hexagonal Architecture (Ports & Adapters) Blueprint**

---

## 1. Hexagonal Architecture (Ports & Adapters)

In `wireup_macros`, the compiler tools (`syn`, `darling`, `quote`, `proc_macro2`) are strictly quarantined to the outer adapter layers:
* **Inbound Adapter (Parser):** Uses `darling` and `syn` to adapt raw compiler tokens into **Pure Domain Models** (using only standard Rust types: `String`, `bool`, `Vec`, custom enums).
* **Core Domain & Use Cases:** Contains **zero compiler dependencies**. Knows nothing about `syn::Ident`, `syn::Type`, or `quote!`. Operates entirely on pure graph logic and instantiation plans.
* **Outbound Adapter (Code Emitter):** Implements an output port trait to transform the domain's `ConstructionPlan` into `proc_macro2::TokenStream` using `quote!`.

```text
       ┌────────────────────────────────────────────────────────┐
       │             Rust Compiler (TokenStream)                │
       └──────────────┬──────────────────────────▲──────────────┘
                      │                          │
        ┌─────────────▼────────────┐  ┌──────────┴─────────────┐
        │     Inbound Adapter      │  │    Outbound Adapter    │
        │      (syn + darling)     │  │        (quote!)        │
        └─────────────┬────────────┘  └──────────▲─────────────┘
                      │ Pure Domain IR           │ ConstructionPlan
                      ▼                          │
        ┌────────────────────────────────────────┴─────────────┐
        │               Core Domain & Use Cases                │
        │     (Zero syn, Zero darling, Zero quote! imports)    │
        │                                                      │
        │  • DeriveContainerUseCase   • GraphEngine (DAG)      │
        │  • CycleDetector            • TopologicalSorter      │
        └──────────────────────────────────────────────────────┘
```

---

## 2. Coding Standards & Engineering Principles

1. **Dependency Reuse Priority:**
   * Stdlib first (`std`) $\rightarrow$ existing workspace crates $\rightarrow$ well-known, actively maintained crates $\rightarrow$ custom implementation by hand.
2. **File Size Guideline (~200 Lines):**
   * Files should stay under ~200 lines as a soft limit. When approaching/exceeding 200 lines, split into focused private submodules.
3. **Self-Documenting Code:**
   * No redundant comments where code explains itself. Use descriptive names. Comments are reserved for non-obvious invariants or complex graph algorithms.
4. **Treat Warnings as Errors:**
   * All crates compile with `#![deny(warnings)]` and `cargo clippy -- -D warnings`.
5. **Minimal Visibility & Encapsulation:**
   * Default to private. Export only what is strictly necessary with `pub(crate)` or controlled `pub use` at the module root.
6. **Stop and Ask:**
   * If any question, ambiguity, or design trade-off arises, stop and ask the Tech Lead.

---

## 3. Crate Layout & Directory Structure

```text
wireup/
├── Cargo.toml
├── rust-toolchain.toml          # Pinned toolchain 1.85.0 (rustfmt, clippy, rust-src)
├── wireup/                      # Runtime crate (Traits, Error types)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── container.rs         # Container & AsyncContainer traits
│       ├── error.rs             # Runtime WireupError enum
│       └── introspection.rs     # GraphDescriptor, NodeDescriptor
│
└── wireup_macros/               # Procedural macro crate
    ├── Cargo.toml
    └── src/
        ├── lib.rs               # Proc-macro entry points (Handler layer)
        │
        ├── adapters/            # Outer Adapters Layer
        │   ├── inbound/         # Inbound: Adapting syn/darling to Domain IR
        │   │   ├── mod.rs
        │   │   ├── darling_models.rs   # darling derive structs (FromDeriveInput, etc.)
        │   │   └── syn_parser_adapter.rs # Implements ContainerParserPort
        │   │
        │   └── outbound/        # Outbound: Adapting Domain Plans to quote!
        │       ├── getters.rs
        │       ├── init_constructor.rs
        │       └── step_expr.rs
        │
        ├── domain/              # Pure Domain Entities (ZERO syn/quote dependencies)
        │   ├── mod.rs
        │   ├── ir.rs            # ContainerIR, DependencyIR, Lifecycle, TraitDispatch
        │   ├── plan.rs          # ConstructionPlan, StepPlan, ParallelBatch
        │   └── errors.rs        # DomainError (Cycles, MissingPrereqs, InvalidConfig)
        │
        ├── ports/               # Port Traits (Interfaces defining boundaries)
        │   ├── mod.rs
        │   ├── parser_port.rs   # ContainerParserPort
        │   └── emitter_port.rs  # CodeEmitterPort
        │
        └── use_cases/           # Application Business Logic
            ├── mod.rs
            ├── derive_container.rs # Orchestrates parsing -> graph -> plan -> emit
            ├── graph_engine.rs     # Pure Kahn's algorithm & cycle detection
            └── planner.rs          # Translates DAG into sequential/parallel execution steps
```

---

## 4. Pure Domain Entities (Zero Compiler Dependencies)

```rust
// wireup_macros/src/domain/ir.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lifecycle {
    Singleton,
    Transient,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraitDispatch {
    Enum { trait_name: String, impls: Vec<String> },
    Dyn { trait_type: String },
    None,
}

#[derive(Debug, Clone)]
pub struct DependencyIR {
    pub name: String,                  // e.g. "db"
    pub type_string: String,          // e.g. "Arc<Database>"
    pub prerequisites: Vec<String>,    // e.g. ["config"]
    pub lifecycle: Lifecycle,
    pub is_async: bool,
    pub constructor_name: String,     // e.g. "new", "connect"
    pub custom_factory: Option<String>,
    pub trait_dispatch: TraitDispatch,
}

#[derive(Debug, Clone)]
pub struct ContainerIR {
    pub name: String,                  // e.g. "AppContainer"
    pub is_async: bool,
    pub parent_type: Option<String>,
    pub introspection: bool,           // enabled via #[container(introspection = true)]
    pub dependencies: Vec<DependencyIR>,
}
```

```rust
// wireup_macros/src/domain/plan.rs

/// An optimized execution plan produced by the use case
#[derive(Debug, Clone)]
pub struct ConstructionPlan {
    pub container_name: String,
    pub is_async: bool,
    pub ordered_steps: Vec<InstantiationStep>,
    pub parallel_batches: Vec<Vec<String>>, // Groups of independent async tasks
}

#[derive(Debug, Clone)]
pub struct InstantiationStep {
    pub dep_name: String,
    pub dep_type: String,
    pub constructor_call: String,
    pub is_async: bool,
    pub lifecycle: Lifecycle,
}
```

---

## 5. Port Interfaces

```rust
// wireup_macros/src/ports/parser_port.rs
use crate::domain::{ContainerIR, DomainError};

pub trait ContainerParserPort {
    /// Parses an ItemStruct (from #[container]) or DeriveInput into pure domain IR.
    fn parse_container(&self, input: &syn::ItemStruct) -> Result<ContainerIR, DomainError>;
}

// wireup_macros/src/ports/emitter_port.rs
use crate::domain::ConstructionPlan;
use proc_macro2::TokenStream;

pub trait CodeEmitterPort {
    /// Emits the rewritten struct (transients stripped, traits rewritten to enums) and impl blocks.
    fn emit_container(&self, plan: &ConstructionPlan) -> TokenStream;
}
```

---

## 6. Pure Use Case Orchestration (100% Pure Standard Rust)

In strict adherence to Hexagonal Architecture, Use Cases know nothing about compiler ASTs or tokens. They accept pure `ContainerIR` and return a pure `ConstructionPlan`.

```rust
// wireup_macros/src/use_cases/plan_container.rs

use crate::domain::{ContainerIR, ConstructionPlan, DomainError};
use crate::use_cases::{GraphEngine, ExecutionPlanner};

pub struct PlanContainerUseCase {
    graph_engine: GraphEngine,
    planner: ExecutionPlanner,
}

impl PlanContainerUseCase {
    pub fn new(graph_engine: GraphEngine, planner: ExecutionPlanner) -> Self {
        Self { graph_engine, planner }
    }

    pub fn execute(&self, container_ir: ContainerIR) -> Result<ConstructionPlan, DomainError> {
        // 1. Pure Domain Logic: DAG Topological Sort & Cycle Detection
        let sorted_dependencies = self.graph_engine.topological_sort(&container_ir.dependencies)?;

        // 2. Pure Domain Logic: Plan sequential vs. parallel tokio::join batches
        let plan = self.planner.build_plan(&container_ir, sorted_dependencies)?;

        Ok(plan)
    }
}
```

### The End-to-End Orchestration in `wireup_macros/src/lib.rs` (Outer Handler Layer):
```rust
#[proc_macro_attribute]
pub fn container(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = syn::parse_macro_input!(input as syn::ItemStruct);

    // 1. Inbound Adapter: Compiler AST -> Pure Domain IR
    let ir = SynParserAdapter::parse(&item_struct)?;

    // 2. Pure Use Case: Domain IR -> ConstructionPlan (ZERO compiler types)
    let plan = PlanContainerUseCase::default().execute(ir)?;

    // 3. Outbound Adapter: ConstructionPlan -> proc_macro2::TokenStream
    QuoteEmitterAdapter::emit(&plan).into()
}
```

---

## 7. Testing Strategy: Total Isolation

Because of this separation, unit testing each component is clean, deterministic, and blazing fast:

### 1. Testing the Inbound Adapter (`darling` / `syn`)
```rust
#[test]
fn test_darling_parser_extracts_prerequisites() {
    let input: syn::DeriveInput = syn::parse_quote! {
        #[container]
        pub struct App {
            #[singleton(deps = [config], constructor = "connect")]
            db: Arc<Database>,
        }
    };

    let parser = SynParserAdapter::default();
    let ir = parser.parse_container(&input).unwrap();

    assert_eq!(ir.dependencies[0].name, "db");
    assert_eq!(ir.dependencies[0].prerequisites, vec!["config"]);
    assert_eq!(ir.dependencies[0].constructor_name, "connect");
}
```

### 2. Testing the Pure Graph Engine (Zero Compiler Types)
```rust
#[test]
fn test_graph_detects_cycle() {
    let engine = GraphEngine::default();
    let deps = vec![
        mock_dep("a", vec!["b"]),
        mock_dep("b", vec!["a"]),
    ];

    let result = engine.topological_sort(&deps);
    assert!(matches!(result, Err(DomainError::CycleDetected(_))));
}
```

### 3. Testing the Outbound Adapter (`quote!`)
```rust
#[test]
fn test_emitter_generates_getters() {
    let plan = mock_plan_with_dep("db", "Arc<Database>");
    let emitter = QuoteEmitterAdapter::default();

    let tokens = emitter.emit_container(&plan);
    let code = tokens.to_string();

    assert!(code.contains("pub struct AppContainer"));
    assert!(code.contains("pub fn get_db (& self) -> Arc < Database >"));
}
```

---

## 8. Integration Testing, BDD (Cucumber/Gherkin), & Container Introspection

For integration tests verifying end-to-end container assembly, `wireup` uses **BDD** with the [`cucumber`](https://crates.io/crates/cucumber) crate.

### 1. Macro-Generated Introspection (`introspection = true`)
Users can explicitly request debug inspection methods directly in the derive settings:

```rust
#[container(
    mode = "async",
    introspection = true, // 🚀 Generates .dependency_graph()
)]
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
    /// Returns the complete compile-time DAG descriptor
    pub fn dependency_graph() -> wireup::introspection::GraphDescriptor {
        wireup::introspection::GraphDescriptor {
            nodes: vec![
                wireup::introspection::NodeDescriptor { name: "config".into(), lifecycle: "singleton".into() },
                wireup::introspection::NodeDescriptor { name: "db".into(), lifecycle: "singleton".into() },
                wireup::introspection::NodeDescriptor { name: "controller".into(), lifecycle: "transient".into() },
            ],
            edges: vec![
                wireup::introspection::EdgeDescriptor { from: "db".into(), to: "config".into() },
                wireup::introspection::EdgeDescriptor { from: "controller".into(), to: "config".into() },
                wireup::introspection::EdgeDescriptor { from: "controller".into(), to: "db".into() },
            ],
        }
    }
}
```

---

### 2. Executable Gherkin Scenarios (`tests/features/container_resolution.feature`)

```gherkin
Feature: Container Dependency Resolution

  As a Rust developer building services with wireup
  I want container-managed singletons to resolve topologically
  So that dependent components receive shared instances without domain pollution

  @mvp
  Scenario: Singletons are instantiated once and shared by reference
    Given an AppContainer with leaf Config, Database, and UserService
    When the container is instantiated
    Then the container should provide a Config singleton
    And the container should provide a Database singleton
    And the container should provide a UserService singleton
    And the Database singleton should reference the exact same Config instance as the container
    And the UserService singleton should reference the exact same Database instance as the container

  @mvp
  Scenario: Multiple dependencies of the same type are distinguished by field name
    Given an AppContainer with "read_db" and "write_db" singletons of type Database
    When the container is instantiated
    Then the container should provide separate "read_db" and "write_db" instances
    And the "read_db" instance should have a different memory address from the "write_db" instance
```

---

### 3. Step Definitions & Pointer Equality (`cucumber-rs`)

```rust
// tests/steps/container_steps.rs

use cucumber::{given, when, then, World};
use std::sync::Arc;

#[derive(Default, World)]
pub struct ContainerWorld {
    container: Option<AppContainer>,
}

#[when("the container is instantiated")]
fn instantiate_container(world: &mut ContainerWorld) {
    world.container = Some(AppContainer::new());
}

#[then(regex = r#"the Database singleton should reference the exact same Config instance as the container"#)]
fn verify_db_config_equality(world: &mut ContainerWorld) {
    let container = world.container.as_ref().unwrap();
    let container_config = container.get_config();
    let db_config = container.get_db().get_config_ref();

    // 🚀 Exact pointer equality assertion:
    assert!(
        Arc::ptr_eq(&container_config, &db_config),
        "Expected Database's Config pointer to match Container's Config pointer"
    );
}

#[then(regex = r#"the UserService singleton should reference the exact same Database instance as the container"#)]
fn verify_user_service_db_equality(world: &mut ContainerWorld) {
    let container = world.container.as_ref().unwrap();
    let container_db = container.get_db();
    let service_db = container.get_user_service().get_db_ref();

    assert!(
        Arc::ptr_eq(&container_db, &service_db),
        "Expected UserService's Database pointer to match Container's Database pointer"
    );
}
```

---

## 9. Automated Architectural Boundary Enforcement (`cargo xtask arch-check`)

To guarantee that the **Hexagonal Architecture** invariants are never violated as the codebase evolves, the workspace includes an automated verification task (`cargo xtask arch-check`).

### Automated Invariant Checks:
1. **Zero Compiler Leaks in Domain & Use Cases:**
   * Scans all Rust source files in `wireup_macros/src/domain/` and `wireup_macros/src/use_cases/`.
   * Forbids any import or reference to compiler crates:
     * ❌ `syn::`
     * ❌ `quote::`
     * ❌ `darling::`
     * ❌ `proc_macro::`
     * ❌ `proc_macro2::`
   * CI fails immediately if a compiler type leaks into domain or use case modules.
2. **File Size Guideline Enforcement (~200 Lines):**
   * Inspects all `.rs` files across the workspace.
   * Emits warnings if a file approaches or exceeds the ~200-line soft limit, signaling the author to refactor into cohesive submodules.

