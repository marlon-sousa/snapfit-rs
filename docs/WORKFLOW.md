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

### Stack Hierarchy (Phase 1: Core v0.1.0 & Phase 2: Advanced Capabilities):
```text
main (Stack 1: Workspace & Domain IR — PR #1)
 └── stack/02-graph-engine-and-planner
      └── stack/03-adapters-inbound-outbound
           └── stack/04-walking-skeleton-bdd (🎉 v0.1.0 — First Gherkin Scenario GREEN)
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
* **Commit types:** `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert` or `test`; a formatting-only change is a `refactor`.
* **Changelog entries:** a `feat`, a `fix` or a breaking change (`!` before the colon) ends with at least one trailer, `Added:`, `Changed:`, `Fixed:` or `Removed:`, each a one-line entry of [CHANGELOG.md](../CHANGELOG.md) as a user reads it, or with `Changelog: none` when a user meets no change. git-cliff writes a release's notes from them, as `cliff.toml` says. For example:

  ```text
  fix(macros): name the Arc to keep in the stable trait-object advice

  Fixed: the error for a trait-object field on the stable build says to hold an Arc of the concrete type.
  ```

  The pull request workflow checks every commit of a pull request against this convention, with the `commit-messages` action of [itinera-dev/actions](https://github.com/itinera-dev/actions). Merge commits are not checked.

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
3. **Automated CI:** GitHub Actions runs the checks of `cargo xtask ci`, the tests on Linux, macOS and Windows, as section 7 says.
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
- [ ] **Green CI:** every check of the CI and pull request workflows passes, including the tests on Linux, macOS and Windows and on the minimum supported Rust.
- [ ] **Every checkbox ticked:** the description has no unticked checkbox, such as a manual check with a screen reader.
- [ ] **Independent Review Approval:** the reviewer agent's verdict on the latest push is approve.

---

## 7. Automated QA & CI Pipeline (`cargo xtask`)

To eliminate environmental drift between local development and CI, all quality checks are unified under the standard Rust **`cargo xtask`** pattern.

### Local QA Commands

`cargo xtask ci` runs every check below, in this order. Each runs on its own as `cargo xtask <mode>`:

| Mode | What it checks |
| :--- | :--- |
| `fmt` | rustfmt compliance. |
| `clippy` | clippy with every warning an error, on the default and the `unstable` builds. |
| `arch` | no compiler type in `domain/` or `use_cases/`, no `mod.rs`, and files under about 200 lines. |
| `test` | every unit test and Gherkin scenario, on the default and the `unstable` builds. |
| `doc` | the documentation of both builds, with every warning an error; docs.rs documents snapfit with `unstable`. |
| `deny` | the licences, advisories and sources of the dependencies, as `deny.toml` allows them. It needs cargo-deny, installed with `cargo install cargo-deny --version 0.20.2 --locked`. |
| `package` | snapfit and snapfit-macros built from their packages alone, as crates.io receives them, with `cargo publish --workspace --dry-run`. |

`cargo xtask book-check` runs only the tests of The Snapfit Book: it compiles and runs every example and checks every recorded output and compiler message. It is not a mode of `cargo xtask ci`, whose `test` mode already runs the book's tests with the rest of the workspace.

### The toolchain

`rust-toolchain.toml` pins the toolchain every contributor, cloud session and CI job builds with, a recent stable Rust. The minimum Rust version promised to users is `rust-version` in `Cargo.toml`, 1.85.0, which its own CI job checks. Raising either is a change of its own.

### GitHub Actions

Every action a workflow uses is pinned to a commit, with its version in a comment beside it.

* **`.github/workflows/ci.yml`**, on every pull request and every push to `main` or a `stack/*` branch: each mode of `cargo xtask ci` in a job of its own on Linux, the `test` mode on Linux, macOS and Windows, and the `test` mode again with the minimum supported Rust.
* **`.github/workflows/pull-request.yml`**, on every pull request, whatever its base: every commit message keeps the convention of section 3, and the description has no unticked checkbox. Ticking a checkbox runs only this workflow again.
* **`.github/workflows/weekly.yml`**, each Monday: the `test` mode on Rust beta, so that a change in the next Rust that breaks snapfit, such as a compiler message a compile-fail fixture checks, is known before it is released.
* **`.github/workflows/release.yml`**, on a tag `v<version>`: section 9.

---

## 8. The Snapfit Book & Documentation Site (`site` branch)

**The Snapfit Book** lives in `book/`: [book/BOOK.md](../book/BOOK.md) says what it contains and how it is written, and [book/FORMAT.md](../book/FORMAT.md) defines the directives its chapters use. Each edition teaches only the stable API of its release.

---

## 9. Releases

snapfit and snapfit-macros share one version, `version` in the workspace's `Cargo.toml`, and follow [Semantic Versioning](https://semver.org/). The generated names and the attributes SPEC.md describes are part of what is promised.

### Preparing a release

A release starts as a pull request, titled "Release <version>", whose commit `chore(release): release <version>`:

1. raises `version` under `[workspace.package]` and the `version` requirements of snapfit and snapfit-macros under `[workspace.dependencies]` in `Cargo.toml`, and the versions in `Cargo.lock` with `cargo update --workspace`;
2. writes the version's section at the top of [CHANGELOG.md](../CHANGELOG.md), starting from what `git cliff --unreleased --tag v<version> --strip all` prints, which is the heading `## [<version>]` and the entries of the commits since the last release;
3. reconciles those entries, since a commit's entries cannot change once it merges: entries a later commit made untrue become one that says what the release holds, entries that say the same thing merge, and a change a user meets that carries no entry gets one.

`cliff.toml` is written for git-cliff 2.14.2, which installs with `cargo install git-cliff --version 2.14.2 --locked`.

A pre-release is named with a dot before its number, such as `0.1.0-alpha.1` or `0.2.0-rc.1`. Only a version without a pre-release part and a release candidate, `rc.N`, go to crates.io; an alpha, a beta or any other pre-release is released on GitHub only.

### Tagging it

Once the pull request has merged, the maintainer tags the commit the merge put on `main` and pushes the tag, such as `git tag -a v0.1.0 <commit> -m "snapfit 0.1.0"` and `git push origin v0.1.0`. The release workflow then runs:

1. **release gate**, which refuses the tag unless it is `v` and the version in `Cargo.toml`, its commit is on `main`, and `CHANGELOG.md` has notes for the version, reporting every reason, and tells whether the version goes to crates.io;
2. **checks**, every job of `ci.yml` again on the tagged commit;
3. **release tools**, the release gate and the crates.io rule against their fixtures under `.github/release/`;
4. **the crates' packages**, the `.crate` files of snapfit-macros and snapfit;
5. **crates.io**, for a version that goes there, once every job before it passed;
6. **GitHub release**, with the `create-release` action of [itinera-dev/actions](https://github.com/itinera-dev/actions): the version's section of `CHANGELOG.md` as its notes, a pre-release for a version such as `0.2.0-rc.1`, and the packages attached.

A pull request that changes how releases are made runs the release tools and packs the crates, without releasing anything.

### Publishing to crates.io

crates.io's trusted publishing lets the workflow publish without a token, but it is set up on a crate that already exists there. So the first version, 0.1.0, is published by hand: after the gate passes, the maintainer runs `cargo publish --workspace --locked` at the tagged commit, from Linux or macOS, which publishes snapfit-macros, then snapfit. Each crate's licence files are links to the ones at the root, which a Windows checkout without symbolic links would pack as text. Until trusted publishing is set up for both crates, the crates.io job only runs `cargo publish --workspace --dry-run --locked`. A later pull request then makes the job publish, in a `crates-io` environment, with the `publish-crates` action of itinera-dev/actions, which publishes each crate whose version is not on crates.io yet.
