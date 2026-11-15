# AGENTS.md — Repository Instructions & Architectural Memory

> **Authoritative agent guidance for `wireup-rs`**

---

## 1. Project Overview & Identity

* **Crate Name:** `wireup` (runtime) / `wireup_macros` (proc-macro)
* **GitHub Repository:** `wireup-rs`
* **Purpose:** A zero-cost, compile-time verified, pure-Rust dependency injection framework with zero domain pollution, automated `enum_dispatch` trait polymorphism, and eager async boot.
* **Core Documentation Index:**
  * Conceptual Spec: [docs/SPEC.md](file:///c:/projects/project-x/docs/SPEC.md)
  * Technical Spec (Ports & Adapters): [docs/TECHNICAL_SPEC.md](file:///c:/projects/project-x/docs/TECHNICAL_SPEC.md)
  * Development Workflow: [docs/WORKFLOW.md](file:///c:/projects/project-x/docs/WORKFLOW.md)

---

## 2. Strict Architectural Invariants

Every agent (whether coder or reviewer) working in this repository **must** enforce these non-negotiable rules:

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

## 3. Code Quality & Verification Gates

Before submitting any work or marking a task done:
1. **Compilation:** `cargo check --workspace --all-targets` must pass with 0 errors.
2. **Clippy:** `cargo clippy --workspace --all-targets -- -D warnings` must produce 0 warnings.
3. **Formatting:** `cargo fmt --check` must be clean.
4. **Unit Tests:** Must run in < 5ms without `trybuild` compiler harnesses.
5. **No Blind Unwraps:** Never use `.unwrap()` in macro logic unless an invariant is strictly checked above it. Return `Result<T, DomainError>`.

---

## 4. Role Instructions for Subagents

### When Invoked as a Reviewer Subagent:
1. Audit the git diff strictly against Section 2 (Architectural Invariants).
2. Check for any leak of `syn` or `quote` into `domain` or `use_cases`.
3. Check for test coverage of both happy paths and edge cases (e.g. cycles, missing prerequisites).
4. Return a structured report:
   * **Result:** `PASS` or `CHANGES REQUESTED`
   * **Findings:** Specific file and line references if any invariants are violated.

---

## 5. Development Workflow

* All work proceeds via **Stacked PRs** (`stack/<number>-<name>`) as documented in [docs/WORKFLOW.md](file:///c:/projects/project-x/docs/WORKFLOW.md).
* BDD integration tests use `cucumber-rs` and reside in `tests/features/`.
