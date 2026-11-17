# Development Workflow & Working Agreement (`snapfit-rs`)

> **Stacked PRs, Double-Loop BDD, and Agentic Review Loop**

---

## 1. Project Naming & Repository Identity

* **Crate Name (Crates.io):** `snapfit` (e.g. `snapfit = "0.1"`, `snapfit-macros = "0.1"`)
* **GitHub Repository Name:** `snapfit-rs` (`https://github.com/<user>/snapfit-rs`)
* **Git Branching Base:** `main`

---

## 2. Roles & Collaboration Model

| Role | Entity | Core Responsibilities |
| :--- | :--- | :--- |
| **Architect & Tech Lead** | **Human (You)** | • Final authority on API design, trade-offs, and feature acceptance.<br>• Approves stack scopes before implementation starts.<br>• Reviews and merges PRs. |
| **Implementer** | **Agent or contributor** | • Implements the change as [ARCHITECTURE.md](ARCHITECTURE.md) describes, following the rules of [AGENTS.md](../AGENTS.md).<br>• Writes the unit tests, Gherkin scenarios and fixtures the rules ask for.<br>• Runs `cargo xtask ci`. |
| **Independent Code Reviewer** | **Reviewer agent** | • Started fresh for each change, given the change and not its author's reasoning.<br>• Works through every item of the checklist in [AGENTS.md](../AGENTS.md), "Reviewing a change".<br>• Returns findings, each blocking or not, and a verdict before human sign-off. |

---

## 3. GitHub Stacked PR Strategy (`gh stack`)

To prevent blocking while maintaining small, reviewable diffs (< 300 lines of code per PR), development uses **Stacked PRs** (`gh stack` / Graphite / stacked Git branches).

### Stack Hierarchy (Phase 1: MVP & Phase 2: Advanced Capabilities):
```text
main (Stack 1: Workspace & Domain IR — PR #1)
 └── stack/02-graph-engine-and-planner
      └── stack/03-adapters-inbound-outbound
           └── stack/04-walking-skeleton-bdd (🎉 v0.1.0 MVP — First Gherkin Scenario GREEN)
                ├── [Milestone: The Snapfit Book & Documentation Site (site branch)]
                └── stack/05-async-container-boot
                     └── stack/06-transient-lifecycles
                          └── stack/07-enum-dispatch-polymorphism
                               └── stack/08-companion-builders (and #[external] / before_init hooks)
                                    └── stack/09-fork-and-rewire-overrides
                                         └── stack/10-child-containers-scoping
```

### 0.1.0 scope

`v0.1.0` ships the synchronous singleton container only. Async containers, transient dependencies and custom builders were implemented early (Stacks 2 to 4) and are kept behind the `unstable` Cargo feature until their stacks (05, 06, 08) settle the API. Planned features with no code generation (`dispatch`, `parent`, `introspection`) are compile errors. Items removed from the runtime crate for 0.1.0 are listed in [SPEC §0](SPEC.md#0-what-ships-in-010) so their stacks restore them.

The QA gate builds the `unstable` feature in `target/unstable`, separately from the stable build, because Cargo unifies features within one build and the stable suite must prove the gate rejects unstable APIs.

### Git Branch & Commit Conventions:
* **Branch Naming:** `stack/<number>-<short-description>` (e.g. `stack/02-graph-engine-and-planner`)
* **Commit Messages:** Follow [Conventional Commits](https://www.conventionalcommits.org/):
  * `feat(macros): add Kahn's algorithm topological sorter`
  * `test(domain): add cycle detection unit tests`
  * `docs(spec): document fork-and-rewire pattern`
  * `ci(qa): add cross-platform gate check`

---

## 4. Double-Loop BDD Strategy (Walking Skeleton)

Rather than waiting until the end to write integration tests, `snapfit` uses an **Outside-In Double-Loop BDD** approach:

1. **Executable Specifications Upfront:** We write the Gherkin `.feature` acceptance scenarios in `tests/features/` early.
2. **The Walking Skeleton (Stack 4):** As soon as Stack 4 lands (minimal end-to-end derive pipeline), we wire up `cucumber-rs`.
3. **Feature-by-Feature BDD:** Every subsequent capability (Async boot, Overrides, Enum Dispatch, Introspection) is delivered with its own Gherkin scenario that turns GREEN in that exact PR stack.

---

## 5. The Agentic PR Review Loop

For every PR slice in the stack:

1. **Implementation & Local Verification:** the implementer writes the code and its tests, and runs `cargo xtask ci`.
2. **Independent Review:** before the pull request is opened, a reviewer agent reviews the commits against the checklist in [AGENTS.md](../AGENTS.md), "Reviewing a change"; `.claude/skills/review-change` says how Claude starts it. Findings are fixed, or rejected with a reason the reviewer accepts or the maintainer decides. Every later push is reviewed the same way.
3. **Automated CI:** GitHub Actions runs `cargo xtask ci` on Linux, macOS and Windows.
4. **Human Review & Sign-Off:** the pull request is reported ready, with the review's verdict, once both the review and the CI of its latest push have passed. The maintainer approves and merges, or advances the stack.

---

## 6. Definition of Done (DoD) per Stack Slice

A slice is only ready for human merge when:
- [ ] **Clean Pre-Flight:** `cargo xtask ci` passes locally with 0 errors.
- [ ] **Clean Compilation:** `cargo check --workspace --all-targets` passes with 0 errors.
- [ ] **Zero Warnings:** `cargo clippy --workspace --all-targets -- -D warnings` passes with 0 warnings.
- [ ] **Properly Formatted:** `cargo fmt --check` succeeds.
- [ ] **100% Tests Pass:** All unit and BDD tests pass in `cargo test`.
- [ ] **Zero Compiler Leaks:** `cargo xtask arch-check` confirms zero `syn`, `quote`, or `darling` imports in `domain/` or `use_cases/`.
- [ ] **Rules Followed:** every behaviour has its scenario or fixture, and every fixed bug its regression test, as [AGENTS.md](../AGENTS.md), "Rules", asks.
- [ ] **Green CI Matrix:** GitHub Actions passes across Linux, macOS, and Windows.
- [ ] **Independent Review Approval:** the reviewer agent's verdict on the latest push is approve.

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

---

## 8. The Snapfit Book & Documentation Site (`site` branch)

Immediately following the completion of **Stack 4 (v0.1.0 MVP)**, development will deliver **The Snapfit Book** on a dedicated `site` branch for static documentation publishing (e.g., mdBook / GitHub Pages):

### Book Structure & Educational Goals:
1. **Introduction to Dependency Injection & Inversion of Control:**
   * Why DI matters in Rust without invasive runtime reflection or dynamic downcasting (`dyn Any`).
   * Component-Driven vs. Container-Driven DI: Why centralized manifests prevent domain pollution.
2. **The Snapfit Mental Model:**
   * Pure domain services (zero macros).
   * Container manifest (`#[container]`, `#[singleton]`, `#[transient]`).
   * Compile-time verification: how the macro orders the fields and reports unknown dependencies and cycles before the program compiles.
3. **Tutorial Walkthroughs (Human & Agent Friendly):**
   * Step-by-step: Building a clean Web API with Axum/Actix and Snapfit.
   * Multi-instance patterns (`read_db`, `write_db`).
   * Custom builders and companion traits (`<ContainerName>Builders`).
   * Testing overrides and the Fork-and-Rewire pattern.


