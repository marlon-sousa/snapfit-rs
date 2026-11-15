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

## 2. Crate Layout & Directory Structure

```text
wireup/
├── Cargo.toml
├── wireup/                      # Runtime crate (Traits, Error types)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── container.rs         # Container & AsyncContainer traits
│       └── error.rs             # Runtime WireupError enum
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
        │       ├── mod.rs
        │       ├── quote_emitter_adapter.rs # Implements CodeEmitterPort
        │       ├── templates/
        │       │   ├── key_enum.rs
        │       │   ├── getters.rs
        │       │   ├── init_constructor.rs
        │       │   ├── builder.rs
        │       │   └── enum_dispatch.rs
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

## 3. Pure Domain Entities (Zero Compiler Dependencies)

Notice that all identifiers and types in the Domain IR are **pure standard Rust types** (`String`, standard enums). Not a single `syn::Ident` or `syn::Type` leaks into the domain!

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
    pub key_enum_name: String,
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

## 4. Port Interfaces

```rust
// wireup_macros/src/ports/parser_port.rs
use crate::domain::{ContainerIR, DomainError};

pub trait ContainerParserPort {
    fn parse_container(&self, input: &syn::DeriveInput) -> Result<ContainerIR, DomainError>;
}

// wireup_macros/src/ports/emitter_port.rs
use crate::domain::ConstructionPlan;
use proc_macro2::TokenStream;

pub trait CodeEmitterPort {
    fn emit_container(&self, plan: &ConstructionPlan) -> TokenStream;
}
```

---

## 5. Pure Use Case Orchestration

The controller/use-case wires together ports using **pure constructor injection**:

```rust
// wireup_macros/src/use_cases/derive_container.rs

use crate::ports::{ContainerParserPort, CodeEmitterPort};
use crate::use_cases::{GraphEngine, ExecutionPlanner};
use crate::domain::DomainError;
use proc_macro2::TokenStream;

pub struct DeriveContainerUseCase<P, E> 
where
    P: ContainerParserPort,
    E: CodeEmitterPort,
{
    parser: P,
    graph_engine: GraphEngine,
    planner: ExecutionPlanner,
    emitter: E,
}

impl<P, E> DeriveContainerUseCase<P, E>
where
    P: ContainerParserPort,
    E: CodeEmitterPort,
{
    pub fn new(parser: P, graph_engine: GraphEngine, planner: ExecutionPlanner, emitter: E) -> Self {
        Self { parser, graph_engine, planner, emitter }
    }

    pub fn execute(&self, raw_input: &syn::DeriveInput) -> Result<TokenStream, DomainError> {
        // 1. Inbound Adapter: Parse compiler AST -> Pure Domain IR
        let container_ir = self.parser.parse_container(raw_input)?;

        // 2. Pure Domain Logic: DAG Topological Sort & Cycle Detection
        let sorted_dependencies = self.graph_engine.topological_sort(&container_ir.dependencies)?;

        // 3. Pure Domain Logic: Plan sequential vs. parallel tokio::join batches
        let plan = self.planner.build_plan(&container_ir, sorted_dependencies)?;

        // 4. Outbound Adapter: Render ConstructionPlan -> proc_macro2::TokenStream via quote!
        Ok(self.emitter.emit_container(&plan))
    }
}
```

---

## 6. Testing Strategy: Total Isolation

Because of this separation, unit testing each component is clean, deterministic, and blazing fast:

### 1. Testing the Inbound Adapter (`darling` / `syn`)
Verify that attributes are parsed correctly without touching any graph logic:
```rust
#[test]
fn test_darling_parser_extracts_prerequisites() {
    let input: syn::DeriveInput = syn::parse_quote! {
        #[derive(Container)]
        pub struct App {
            #[dep(config, constructor = "connect")]
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
Test cycle detection using pure strings:
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
Feed a mock `ConstructionPlan` and assert that the generated string contains the expected Rust code:
```rust
#[test]
fn test_emitter_generates_getters() {
    let plan = mock_plan_with_dep("db", "Arc<Database>");
    let emitter = QuoteEmitterAdapter::default();

    let tokens = emitter.emit_container(&plan);
    let code = tokens.to_string();

    assert!(code.contains("pub fn get_db (& self) -> Arc < Database >"));
    assert!(code.contains("pub fn get_dependency < T"));
}
```
* **Result:** Every unit test runs in < 5 milliseconds in standard `cargo test`!

---

## 7. Integration Testing, BDD (Cucumber/Gherkin), & Container Introspection

For integration tests verifying end-to-end container assembly, `wireup` uses **BDD (Behavior-Driven Development)** with the [`cucumber`](https://crates.io/crates/cucumber) crate.

### 1. Macro-Generated Introspection (`introspection = true`)
Instead of complex `cfg` gates, users can explicitly request debug inspection methods directly in the derive settings:

```rust
#[derive(Container)]
#[container(
    mode = "async",
    introspection = true, // 🚀 Generates .dependency_graph() and .inspect_instances()
)]
pub struct AppContainer {
    #[dep]
    config: Arc<Config>,
    #[dep(config)]
    db: Arc<Database>,
    #[dep(config, db, transient)]
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
                ("db".into(), "config".into()),
                ("controller".into(), "config".into()),
                ("controller".into(), "db".into()),
            ],
        }
    }
}
```

---

### 2. Executable Gherkin Scenarios (`tests/features/container.feature`)

```gherkin
Feature: Container Dependency Resolution

  Scenario: Singleton dependencies are shared by reference
    Given an AppContainer with Config, Database, and UserController
    When the container is initialized
    Then the dependency "db" should depend on "config" in the dependency graph
    And the "user_controller" should receive the exact same "db" instance as the container

  Scenario: Transient dependencies produce fresh instances
    Given an initialized AppContainer
    When I request "user_controller" twice
    Then the two controller instances should have different memory addresses
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
    controller_1: Option<UserController>,
    controller_2: Option<UserController>,
}

#[when("the container is initialized")]
async fn init_container(world: &mut ContainerWorld) {
    world.container = Some(AppContainer::init().await.expect("Failed to boot container"));
}

#[then(regex = r#"the dependency "([^"]*)" should depend on "([^"]*)" in the dependency graph"#)]
fn verify_graph_edge(_world: &mut ContainerWorld, from: String, to: String) {
    let graph = AppContainer::dependency_graph();
    assert!(
        graph.edges.contains(&(from.clone(), to.clone())),
        "Expected graph edge from {} to {}", from, to
    );
}

#[then(regex = r#"the "([^"]*)" should receive the exact same "([^"]*)" instance as the container"#)]
fn verify_pointer_equality(world: &mut ContainerWorld, _consumer: String, _dep: String) {
    let container = world.container.as_ref().unwrap();
    let controller = container.get_controller();

    let container_db = container.get_db();
    let controller_db = controller.get_db_ref();

    // 🚀 Exact pointer equality assertion:
    assert!(
        Arc::ptr_eq(&container_db, &controller_db),
        "Expected controller's Database pointer to match Container's Database pointer"
    );
}

#[when(r#"I request "user_controller" twice"#)]
fn request_transient_twice(world: &mut ContainerWorld) {
    let container = world.container.as_ref().unwrap();
    world.controller_1 = Some(container.get_controller());
    world.controller_2 = Some(container.get_controller());
}

#[then("the two controller instances should have different memory addresses")]
fn verify_different_instances(world: &mut ContainerWorld) {
    let c1 = world.controller_1.as_ref().unwrap();
    let c2 = world.controller_2.as_ref().unwrap();

    assert!(
        !std::ptr::eq(c1, c2),
        "Expected transient instances to have different memory addresses"
    );
}
```

