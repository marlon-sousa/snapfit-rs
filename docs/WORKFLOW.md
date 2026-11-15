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
   * Runs local pre-flight QA: `cargo xtask ci` (validates formatting, clippy, architectural boundaries, unit tests, and docs).
2. **Agentic Code Review:**
   * Antigravity invokes an independent `Reviewer Subagent` (pinned to `pro`).
   * The subagent evaluates the diff against the **Architectural Invariants** in `AGENTS.md`.
   * Any findings (e.g., leaky types, missing edge cases) are fixed immediately.
3. **Automated CI & PR QA:**
   * GitHub Actions runs the cross-platform matrix (Linux, macOS, Windows) executing `cargo xtask ci`.
4. **Human Review & Sign-Off:**
   * Antigravity presents the PR summary, diff, and Reviewer Report.
   * You approve, merge (or advance the stack), and we proceed to the next slice.

---

## 6. Definition of Done (DoD) per Stack Slice

A slice is only ready for human merge when:
- [ ] **Clean Pre-Flight:** `cargo xtask ci` passes locally with 0 errors.
- [ ] **Clean Compilation:** `cargo check --workspace --all-targets` passes with 0 errors.
- [ ] **Zero Warnings:** `cargo clippy --workspace --all-targets -- -D warnings` passes with 0 warnings.
- [ ] **Properly Formatted:** `cargo fmt --check` succeeds.
- [ ] **100% Tests Pass:** All unit and BDD tests pass in `cargo test`.
- [ ] **Zero Compiler Leaks:** `cargo xtask arch-check` confirms zero `syn`, `quote`, or `darling` imports in `domain/` or `use_cases/`.
- [ ] **Green CI Matrix:** GitHub Actions passes across Linux, macOS, and Windows.
- [ ] **Reviewer Subagent Approval:** Review report from `wireup-reviewer` returns `PASS`.

---

## 7. Automated QA & CI Pipeline (`cargo xtask`)

To eliminate environmental drift between local development and CI, all quality checks are unified under the standard Rust **`cargo xtask`** pattern.

### Local QA Commands:
```bash
# Complete local pre-flight suite (runs format, clippy, boundary check, tests, docs)
cargo xtask ci

# Individual subcommands:
cargo xtask fmt-check   # Verifies rustfmt compliance
cargo xtask clippy      # Strict zero-warning clippy check
cargo xtask arch-check  # Scans domain/ and use_cases/ for prohibited compiler imports and ~200 line guideline
cargo xtask test        # Runs all workspace tests
cargo xtask doc         # Validates documentation builds with -D warnings
```

### GitHub Actions CI:
Every PR targeting `main` or `stack/*` triggers `.github/workflows/ci.yml`:
* **Matrix:** `ubuntu-latest`, `macos-latest`, `windows-latest`.
* **Toolchain:** Pinned Rust `1.85.0` (Rust 2024 Edition).
* **Execution:** Reuses `cargo xtask` steps so CI and local dev are 100% identical.

