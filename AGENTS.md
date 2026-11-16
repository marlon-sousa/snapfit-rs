# AGENTS.md — Repository Instructions & Architectural Memory

> **Authoritative agent guidance for `wireup-rs`**

---

## 1. Project Overview & Identity

* **Crate Name:** `wireup` (runtime) / `wireup_macros` (proc-macro)
* **GitHub Repository:** `wireup-rs`
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
* **The Domain & Use Cases MUST be 100% pure standard Rust.**
* **Forbidden Imports in `domain/` and `use_cases/`:**
  * ❌ `syn::*` (no `syn::Ident`, `syn::Type`, etc.)
  * ❌ `quote::*` (no `quote!`, `format_ident!`)
  * ❌ `darling::*`
  * ❌ `proc_macro::*` or `proc_macro2::*`
* All identifiers in the Domain Intermediate Representation (IR) must be standard library types (`String`, `bool`, `Vec`, custom enums).

### B. Outer Adapters are Strictly Isolated
* **Inbound Adapter (`adapters::inbound`):** The ONLY place where `darling` and `syn` are permitted. Responsible for converting AST into Domain IR.
* **Outbound Adapter (`adapters::outbound`):** The ONLY place where `quote!` is permitted. Responsible for rendering a `ConstructionPlan` into tokens.

### C. Container Subclassing & Overrides
* Use **Option 1: The Fork & Re-wire Pattern** (`parent.fork_with().with_*(mock).build()`).
* Do NOT use `OnceLock` for container caches because it prevents child container overrides and downstream re-wiring.
* Unaffected singletons are cloned (`Arc::clone`), while dependents of the overridden node are automatically re-instantiated.

### D. Zero-Cost Trait Polymorphism
* Trait fields default to `dispatch = "enum"` which auto-generates static `enum_dispatch` enums.
* Avoid `dyn Any` in normal resolution. Direct typed getters (`get_<dep>()`) must always be emitted alongside generic key lookup.

---

## 3. Coding Standards & Engineering Principles

1. **Dependency Reuse Priority:**
   * **Tier 1:** Standard library first (`std`).
   * **Tier 2:** Crates already present in our workspace dependencies.
   * **Tier 3:** Well-known, actively maintained, and compatible external crates (e.g. `darling`, `quote`, `syn`, `tokio`, `thiserror`).
   * **Tier 4:** Only if none exist, write custom logic by hand.
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

---

## 4. Role Instructions for Subagents

### `wireup-coder` (Lead Implementer):
* Follows the 7 engineering principles above.
* Runs `cargo fmt`, `cargo clippy -- -D warnings`, and in-memory unit tests before submitting.
* Keeps tests fast (< 5ms) without `trybuild`.

### `wireup-reviewer` (Independent Reviewer):
* Audits diffs against Section 2 (Architectural Invariants) and Section 3 (Coding Standards).
* Checks for:
  * Any compiler types leaking into `domain` or `use_cases`.
  * Violation of Dependency Reuse Priority (e.g. hand-rolled `Display`/`Error` formatting instead of `thiserror`, or custom logic where established crates exist like `petgraph`).
  * Anonymous tuples or loose primitives where a named struct should be used.
  * Files approaching/exceeding ~200 lines without justification.
  * Over-commented trivial code or missing error handling.
  * Excessive public exports (`pub` instead of `pub(crate)`).
* Returns a structured report:
  * **Result:** `PASS` or `CHANGES REQUESTED`
  * **Findings:** Specific file and line references if any invariants are violated.

---

## 5. Development Workflow

* All work proceeds via **Stacked PRs** (`stack/<number>-<name>`) as documented in [docs/WORKFLOW.md](docs/WORKFLOW.md).
* BDD integration tests use `cucumber-rs` and reside in `tests/features/`.
