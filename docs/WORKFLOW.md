# Development Workflow & Working Agreement (`wireup-rs`)

> **Stacked PRs, Double-Loop BDD, and Agentic Review Loop**

---

## 1. Project Naming & Repository Identity

* **Crate Name (Crates.io):** `wireup` (e.g. `wireup = "0.1"`, `wireup_macros = "0.1"`)
* **GitHub Repository Name:** `wireup-rs` (`https://github.com/<user>/wireup-rs`)
* **Git Branching Base:** `main`

---

## 2. Roles & Collaboration Model

| Role | Entity | Core Responsibilities |
| :--- | :--- | :--- |
| **Architect & Tech Lead** | **Human (You)** | • Final authority on API design, trade-offs, and feature acceptance.<br>• Approves stack scopes before implementation starts.<br>• Reviews and merges PRs. |
| **Lead Implementer** | **Antigravity (Agent)** | • Implements clean Rust adhering strictly to Ports & Adapters.<br>• Writes comprehensive in-memory unit tests alongside code.<br>• Runs formatting, linting, and compile checks. |
| **Independent Code Reviewer** | **Reviewer Subagent** | • Spawned fresh for each PR slice to act as a second pair of eyes.<br>• Verifies architectural boundaries (no compiler leaks into domain).<br>• Audits for unhandled `unwrap`s, edge cases, and test coverage.<br>• Emits a formal PR Review Report before human sign-off. |

---

## 3. GitHub Stacked PR Strategy (`gh stack`)

To prevent blocking while maintaining small, reviewable diffs (< 300 lines of code per PR), development uses **Stacked PRs** (`gh stack` / Graphite / stacked Git branches).

### Stack Hierarchy:
```text
main
 └── stack/01-workspace-domain-ir
      └── stack/02-graph-engine
           └── stack/03-inbound-darling
                └── stack/04-outbound-quote (Walking Skeleton)
                     └── stack/05-use-case-derive
                          └── stack/06-enum-dispatch
                               └── stack/07-runtime-fork-rewire
                                    └── stack/08-bdd-cucumber-suite
```

### Git Branch & Commit Conventions:
* **Branch Naming:** `stack/<number>-<short-description>` (e.g. `stack/01-domain-ir`)
* **Commit Messages:** Follow [Conventional Commits](https://www.conventionalcommits.org/):
  * `feat(macros): add Kahn's algorithm topological sorter`
  * `test(domain): add cycle detection unit tests`
  * `docs(spec): document fork-and-rewire pattern`

---

## 4. Double-Loop BDD Strategy (Walking Skeleton)

Rather than waiting until the end to write integration tests, `wireup` uses an **Outside-In Double-Loop BDD** approach:

1. **Executable Specifications Upfront:** We write the Gherkin `.feature` acceptance scenarios in `tests/features/` early.
2. **The Walking Skeleton (Stack 4):** As soon as Stack 4 lands (minimal end-to-end derive pipeline), we wire up `cucumber-rs`.
3. **Feature-by-Feature BDD:** Every subsequent capability (Async boot, Overrides, Enum Dispatch, Introspection) is delivered with its own Gherkin scenario that turns GREEN in that exact PR stack.

---

## 5. The Agentic PR Review Loop

For every PR slice in the stack:

1. **Implementation & Local Verification:**
   * Antigravity writes the module + unit tests.
   * Runs `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`.
2. **Agentic Code Review:**
   * Antigravity invokes an independent `Reviewer Subagent`.
   * The subagent evaluates the diff against the **Architectural Invariants** in `AGENTS.md`.
   * Any findings (e.g., leaky types, missing edge cases) are fixed immediately.
3. **Human Review & Sign-Off:**
   * Antigravity presents the PR summary, diff, and Reviewer Report.
   * You approve, merge (or advance the stack), and we proceed to the next slice.

---

## 6. Definition of Done (DoD) per Stack Slice

A slice is only ready for human merge when:
- [ ] **Clean Compilation:** `cargo check --workspace --all-targets` passes with 0 errors.
- [ ] **Zero Warnings:** `cargo clippy --workspace --all-targets -- -D warnings` passes with 0 warnings.
- [ ] **Properly Formatted:** `cargo fmt --check` succeeds.
- [ ] **100% Tests Pass:** All unit and BDD tests pass in `cargo test`.
- [ ] **Zero Compiler Leaks:** No `syn::Ident`, `syn::Type`, or `quote!` types imported in `domain/` or `use_cases/`.
- [ ] **Reviewer Subagent Approval:** Review report returns `PASS`.
