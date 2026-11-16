# AGENTS.md — Repository Instructions & Architectural Memory

> **Authoritative agent guidance for `snapfit-rs`**

---

## 1. Project Overview & Identity

* **Crate Name:** `snapfit` (runtime) / `snapfit-macros` (proc-macro)
* **GitHub Repository:** `snapfit-rs`
* **MSRV:** `1.85.0` (Rust 2024 Edition)
* **Purpose:** A zero-cost, compile-time verified, pure-Rust dependency injection framework with zero domain pollution, automated `enum_dispatch` trait polymorphism, and eager async boot.
* **Core Documentation Index:**
  * Conceptual Spec: [docs/SPEC.md](docs/SPEC.md)
  * Technical Spec (Ports & Adapters): [docs/TECHNICAL_SPEC.md](docs/TECHNICAL_SPEC.md)
  * Development Workflow: [docs/WORKFLOW.md](docs/WORKFLOW.md)

---

## 2. Strict Architectural Invariants

Every agent (coder or reviewer) working in this repository **must** enforce these non-negotiable rules:

### A. Hexagonal Architecture (Ports & Adapters)
* **The Domain & Use Cases represent pure, deterministic computation and business/graph logic.**
* **Decoupling from Side-Effects & Infrastructure:**
  * Even if a facility exists in the standard library (`std`), **side-effecting and infrastructural operations are strictly forbidden in `domain/` and `use_cases/`**:
    * ❌ No I/O or filesystem operations (`std::fs`, file writes/reads).
    * ❌ No channels or concurrency pipes (`std::sync::mpsc`, `crossbeam`) mediating external communication.
    * ❌ No compiler infrastructure, token streams, or AST crates (`syn::*`, `quote::*`, `darling::*`, `proc_macro::*`, `proc_macro2::*`).
  * The inbound adapter's sole job is to parse external compiler AST and adapt flow into domain types. The domain must never be coupled to how adapters parse or emit.
* **Permitted & Encouraged in `domain/` and `use_cases/`:**
  * High-quality, pure-Rust domain problem-solving crates (e.g. `petgraph` for DAG algorithms, `thiserror` for error modeling, pure in-memory data structures).
  * Domain IR identifiers and models must represent core domain concepts, not compiler syntax trees.

### B. Outer Adapters are Purpose-Built Integrations
* Adapters leverage specialized external crates suited for their integration responsibilities:
  * **Inbound Adapter (`adapters::inbound`):** Leverages `darling` and `syn` to parse external compiler syntax trees and adapt them into pure domain IR.
  * **Outbound Adapter (`adapters::outbound`):** Leverages `quote!` and `proc_macro2` to render a domain `ConstructionPlan` into compiler tokens.
* The architectural invariant is strictly directional: Outer adapters depend on Domain Ports; the Domain NEVER depends on Adapters or adapter technologies.

### C. Container Subclassing & Overrides
* Use **Option 1: The Fork & Re-wire Pattern** (`parent.fork_with().with_*(mock).build()`).
* Do NOT use `OnceLock` for container caches because it prevents child container overrides and downstream re-wiring.
* Unaffected singletons are cloned (`Arc::clone`), while dependents of the overridden node are automatically re-instantiated.

### D. Zero-Cost Trait Polymorphism & Resolution
* Trait fields with `#[dispatch(...)]` auto-generate static `enum_dispatch` enums.
* Zero `dyn Any` in resolution. Direct typed getters (`get_<dep>()`) are emitted for all singletons (cloned on read) and transients (freshly constructed on every call).

---

## 3. Coding Standards & Engineering Principles

1. **Dependency Reuse is an Engineering Judgment (No Hand-Rolled Boilerplate):**
   * **Engineering Judgment over Dogmatic Rules:** Preferring standard library types does NOT mean writing 300 lines of complex, error-prone algorithms or boilerplate by hand. Between writing a pure `std` implementation of 300 lines vs. using a battle-tested, focused, well-maintained crate, **use the crate**. Both the coder and the reviewer must actively apply this judgment.
   * **Tier 1:** Standard library first (`std`) — for standard in-memory types, data structures, and traits (strictly excluding I/O, channels, and side effects in the domain).
   * **Tier 2:** Crates already present in our workspace dependencies.
   * **Tier 3:** Well-known, actively maintained, and compatible external crates (e.g. `petgraph` for DAGs, `thiserror` for error derives, `strum` for enum utilities, `derive-new` for constructors, `tokio` for async).
   * **Tier 4:** Only if no suitable crate exists, write custom logic by hand. Hand-rolling graph traversal, cycle backtracking, manual `Display`/`Error` formatting, or manual `is_*`/constructor boilerplate when `petgraph`, `thiserror`, `strum`, and `derive-new` exist is a failure of judgment.
   * **Error Handling & Formatting:** All domain and runtime error enums **MUST derive `thiserror::Error`**. Never write manual `impl fmt::Display` or `impl std::error::Error` by hand.
2. **File Size Guideline (~200 Lines):**
   * Keep files under ~200 lines as a soft limit. If a file exceeds this threshold, treat it as a signal to refactor into cohesive private submodules.
3. **Self-Documenting Code (No Obvious Comments):**
   * Do not write redundant comments where code explains itself. Use descriptive function, variable, and type names. Reserve comments strictly for non-obvious invariants or complex graph algorithms.
4. **Treat Warnings as Errors:**
   * All code must compile with zero warnings: `#![deny(warnings)]`, `cargo clippy --workspace --all-targets -- -D warnings`.
5. **Minimal Visibility & Encapsulation:**
   * Default to private. Export only what is strictly necessary.
   * Modules expose their interface through `pub(crate) use` or controlled `pub use` at the module root.
6. **Stop and Ask:**
   * If any question, ambiguity, or design trade-off arises during implementation, **stop and ask the Tech Lead** before making assumptions.
7. **Strongly-Typed Domain Modeling (No Anonymous Tuples):**
   * Never use anonymous tuples (e.g. `(String, String)`) or loose primitives where a domain concept exists.
   * If a concept or relationship can be modeled, define an explicit named struct (e.g. `EdgeDescriptor { pub from: String, pub to: String }`). Every field must convey intent.
8. **Anti-Corruption Service Boundaries (Domain In, Domain Out):**
   * If an external crate is highly specific, complex, or exposes idiosyncratic types, never bleed its types into higher-level consumers (e.g. controllers, use case orchestrators, or domain entities).
   * Encapsulate the crate inside a focused service or adapter that operates strictly on **Domain In, Domain Out**.
   * The consumer interacts exclusively with clean domain contracts; the third-party crate's implementation details and external types remain fully quarantined inside that service.
9. **Modern Rust Module Layout (No `mod.rs`):**
   * Modules declare `<module_name>.rs` at the parent directory level, and private submodules reside in a `<module_name>/` folder.
   * `mod.rs` is strictly forbidden across the repository (enforced automatically by `cargo xtask arch-check`).
10. **Types Enforce Invariants (Make Illegal States Unrepresentable):**
    * **Types Must Be the Invariant:** Whenever a domain concept has invariants or constraints, encode them into the type system at construction time so that downstream code is guaranteed to receive valid data without defensive checks.
    * **Non-Zero & Numeric Invariants:** If a number cannot be zero, use standard library types like `NonZeroU32`, `NonZeroUsize`, etc.
    * **Non-Blank & Distinct Strings:** If an identifier cannot be empty or blank (e.g. `DependencyName`, `ContainerName`, `TypeName`), enforce that invariant on construction (via `TryFrom` or validated constructors) so blank values are unrepresentable.
    * **Distinct Concepts over Raw Primitives:** Keep distinct concepts as distinct types (e.g. `DependencyName` vs `TypeName`) to prevent accidental argument swapping at compile time.
    * **Judgment over Dogma:** If a value is genuinely unconstrained or arbitrary data without domain rules, keep it simple. Apply invariants where they provide real compiler-enforced safety and prevent illegal states.
11. **Small, Single-Responsibility Functions & Idiomatic Functional Composition:**
    * **Read Like Prose (One Level of Abstraction per Function):** Every function should read like clear English prose at a single level of abstraction (~3–10 lines). Never mix multiple loops, validation passes, and state mutations in one function.
    * **Prefer Iterator Combinators & Functional Pipelines over Imperative Loops:** Use `.try_for_each(...)`, `.find(...)`, `.find_map(...)`, `.any(...)`, `.all(...)`, `.filter(...)`, `.map_or(...)`, `.then(...)`, `.then_some(...)`, and `.ok_or_else(...)` instead of imperative `for` + `if` + `break`/`return` boilerplate.
    * **Extract Focused Helpers & Propagate Early with `?`:** Extract inner steps, predicates, and state transitions into small, well-named private methods or helper structs (`CycleFinder`, `IndexedGraph`) and compose them cleanly with `?`.
    * **Flat Control Flow:** Zero deep nesting (`for` -> `if` -> `for` -> `if`).
12. **Co-Located, Module-Scoped Unit Tests:**
    * **Tests Belong to the Module They Test:** Never lump tests for multiple distinct submodules into a single monolithic parent `tests.rs` file.
    * **Submodule Test Layout:** Each module `<name>.rs` declares its own `#[cfg(test)] mod tests;` residing at `<name>/tests.rs`.
    * **Focused Scope & Direct Encapsulation:** Internal modules test their own invariants, algorithms, and private helpers directly (`graph_engine/tests.rs` tests graph sorting and cycles; `planner/tests.rs` tests step building and wrapper stripping; `derive_container/tests.rs` tests use-case orchestration).

---

## 4. Role Instructions for Subagents

### `snapfit-coder` (Lead Implementer):
* Follows the 12 engineering principles above.
* **Exercises Engineering Judgment:** Never writes 300 lines of complex algorithmic or boilerplate logic by hand when an established, focused crate solves the problem cleanly without side effects or adapter coupling.
* Writes small, single-responsibility methods composed with `?` and iterator combinators, and co-locates unit tests in `<module_name>/tests.rs`.
* Runs `cargo fmt`, `cargo clippy -- -D warnings`, and in-memory unit tests before submitting.
* Keeps tests fast (< 5ms) without `trybuild`.

### `snapfit-reviewer` (Independent Reviewer):
* Audits diffs across **both** dimensions: **Functional & Behavioral Correctness** AND **Architecture & Coding Standards**.
* **1. Functional & Behavioral Verification (Primary Mandate):**
  * **Spec Conformance:** Verifies that all functional requirements in [docs/SPEC.md](docs/SPEC.md), [docs/TECHNICAL_SPEC.md](docs/TECHNICAL_SPEC.md), and the active stack's scope are accurately and completely implemented.
  * **Algorithmic & Domain Logic Audit:** Traces through graph algorithms, state transitions, error paths, and edge cases (e.g., complex multi-SCC graphs, non-cycle nodes pointing into/out of cycles, diamond dependencies, empty inputs, argument ordering, wrapper type stripping, async propagation, custom builder invocation semantics) to uncover subtle bugs or false assumptions.
  * **Test Quality & Edge-Case Coverage:** Audits unit and BDD tests to verify they assert exact behavioral outcomes and tricky edge cases, rather than superficial happy-path checks.
* **2. Architectural & Engineering Judgment (Not Just Checklists):**
  * Audits diffs against Section 2 (Architectural Invariants) and Section 3 (Coding Standards).
  * **Enforces Function Simplicity & Module-Scoped Tests (Principles 11 & 12):** Rejects long or deeply nested functions that mix multiple concerns, and rejects monolithic test files that lump tests from separate modules together.
  * Actively challenges hand-rolled logic: asks whether custom code should be replaced by an established, battle-tested crate (e.g. `petgraph`, `thiserror`).
  * Checks for side-effect leakage: verifies that NO I/O (`std::fs`), channels/pipes (`std::sync::mpsc`, `crossbeam`), or compiler infrastructure (`syn`, `quote`, `darling`) leak into `domain` or `use_cases`.
  * Verifies strongly-typed modeling: flags anonymous tuples or loose primitives where a named struct should be used.
  * Enforces the ~200 LOC file length soft limit and minimal visibility (`pub(crate)` over `pub`).
* Returns a structured report:
  * **Result:** `PASS` or `CHANGES REQUESTED`
  * **Functional Findings:** Verification of behavioral requirements, algorithmic edge cases, and test coverage (with specific file:line references for any bugs or gaps).
  * **Architectural Findings:** Specific file:line references if any invariants or coding standards are violated.

---

## 5. Development Workflow

* All work proceeds via **Stacked PRs** (`stack/<number>-<name>`) as documented in [docs/WORKFLOW.md](docs/WORKFLOW.md).
* BDD integration tests use `cucumber-rs` and reside in `tests/features/`.
