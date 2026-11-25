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
| **Architect & Tech Lead** | **Human (You)** | • Agrees the issues every change starts from.<br>• Final authority on API design, trade-offs, and feature acceptance.<br>• Approves stack scopes before implementation starts.<br>• Reviews and merges PRs. |
| **Implementer** | **Agent or contributor** | • Implements the change as [ARCHITECTURE.md](ARCHITECTURE.md) describes, following the rules of [AGENTS.md](../AGENTS.md).<br>• Writes the unit tests, Gherkin scenarios and fixtures the rules ask for.<br>• Runs `cargo xtask ci`. |
| **Independent Code Reviewer** | **Reviewer agent** | • Started fresh for each change, given the change and not its author's reasoning.<br>• Works through every item of the checklist in [AGENTS.md](../AGENTS.md), "Reviewing a change".<br>• Returns findings, each blocking or not, and a verdict before human sign-off. |

---

## 3. Stacked pull requests

To prevent blocking while keeping diffs small and reviewable (under about 300 lines of code per pull request), work is split into stacked pull requests. Each pull request's base is the branch of the one below it, and the pull requests of one piece of work are grouped into a GitHub stack, which shows them in order. A branch can have any name: its pull request's base and its GitHub stack place it in the stack, and CI checks it through its pull request (section 7).

### Roadmap

Stacks 1 to 4 built the core of `v0.1.0`:

1. Workspace and domain model (PR #1).
2. Graph engine and planner.
3. Inbound and outbound adapters.
4. Walking skeleton, with the first Gherkin scenario green: `v0.1.0`. The Snapfit Book and its site follow it.

What comes after it, such as async containers, transients, test overrides and child containers, is planned in the [open issues labelled `feature`](https://github.com/marlon-sousa/snapfit-rs/issues?q=is%3Aissue%20is%3Aopen%20label%3Afeature), as section 10 says.

### 0.1.0 scope

`v0.1.0` ships the synchronous singleton container, with fields built by `new`, `Default`, a named constructor, a factory or a builder. Async containers and transient dependencies were implemented early (Stacks 2 to 4) and are kept behind the `unstable` Cargo feature until their issues settle the API. Planned features with no code generation (`dispatch`, `parent`, `introspection`) are compile errors. Items removed from the runtime crate for 0.1.0 are listed in [SPEC §0](SPEC.md#0-what-ships-in-010) so that the work on their issues restores them.

The QA gate runs the Gherkin suite once on the stable build, whose scenarios prove the gate rejects unstable APIs, and once with the `unstable` feature. Each run compiles the compile-fail fixtures against snapfit with its own features, so both builds share one target folder.

### Commit conventions
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

### Naming the issue

Every pull request names an open issue of this repository, as section 10 says, unless every one of its commits, merges aside, is a `test` and it changes no `Cargo.toml` or `Cargo.lock`: `Closes #N` when it finishes the issue, `Refs #N` when it does part of it. The layers of a stack each say `Refs #N`, and the top one also says `Closes #N` once it targets `main`: GitHub links a closing keyword only on a pull request into the default branch, so before that the check finds only `Refs #N`. A pull request doing the work of several issues names each of them. A pull request never closes an issue of another repository, which GitHub would close the moment it merged, before that repository's own work.

---

## 4. Double-Loop BDD Strategy (Walking Skeleton)

Rather than waiting until the end to write integration tests, `snapfit` uses an **Outside-In Double-Loop BDD** approach:

1. **Executable Specifications Upfront:** We write the Gherkin `.feature` acceptance scenarios in `tests/features/` early.
2. **The Walking Skeleton (Stack 4):** As soon as Stack 4 lands (minimal end-to-end derive pipeline), we wire up `cucumber-rs`.
3. **Feature-by-Feature BDD:** Every subsequent capability (Async boot, Overrides, Static Dispatch, Introspection) is delivered with its own Gherkin scenario that turns GREEN in that exact PR stack.

---

## 5. The Agentic PR Review Loop

For every PR slice in the stack:

1. **Implementation & Local Verification:** starting from its agreed issues, the implementer writes the code and its tests, and runs `cargo xtask ci`.
2. **Independent Review:** before the pull request is opened, a reviewer agent reviews the commits, given the issues they belong to, against the checklist in [AGENTS.md](../AGENTS.md), "Reviewing a change"; `.claude/skills/review-change` says how Claude starts it. Findings are fixed, or rejected with a reason the reviewer accepts or the maintainer decides. Every later push is reviewed the same way.
3. **Automated CI:** GitHub Actions runs the checks of `cargo xtask ci`, the tests on Linux, macOS and Windows, as section 7 says.
4. **Human Review & Sign-Off:** the pull request is reported ready, with the review's verdict, once both the review and the CI of its latest push have passed. The maintainer approves and merges, or advances the stack.

---

## 6. Definition of Done (DoD) per Stack Slice

A slice is only ready for human merge when:
- [ ] **Issues named:** the description names its open issues, with `Closes #N` or `Refs #N`, unless every commit, merges aside, is a `test` and no `Cargo.toml` or `Cargo.lock` changes.
- [ ] **Clean Pre-Flight:** `cargo xtask ci` passes locally with 0 errors.
- [ ] **Clean Compilation:** `cargo check --workspace --all-targets` passes with 0 errors.
- [ ] **Zero Warnings:** `cargo clippy --workspace --all-targets -- -D warnings` passes with 0 warnings.
- [ ] **Properly Formatted:** `cargo fmt --check` succeeds.
- [ ] **100% Tests Pass:** All unit and BDD tests pass in `cargo test`.
- [ ] **Zero Compiler Leaks:** `cargo xtask arch-check` confirms zero `syn`, `quote`, or `darling` imports in `domain/` or `use_cases/`.
- [ ] **Rules Followed:** every behaviour has its scenario or fixture, and every fixed bug its regression test, as [AGENTS.md](../AGENTS.md), "Rules", asks.
- [ ] **Green CI:** every check of the CI and pull request workflows passes, and of the release checks workflow when it runs, including the tests on Linux, macOS and Windows and on the minimum supported Rust.
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
| `test` | every unit test and Gherkin scenario, on the default and the `unstable` builds, every doctest, such as the README's example, and the tests of the gate's own scripts. Some scenarios lint a fixture with clippy, so a toolchain that runs them needs the clippy component, which the CI jobs on the minimum Rust version and on beta install. |
| `doc` | the documentation of both builds, with every warning an error; docs.rs documents the default build only, the stable API. It also checks that `snapfit/README.md`, the crate's documentation, is what the root README gives, as `cargo xtask crate-readme` writes it. |
| `deny` | the licences, advisories and sources of the dependencies, as `deny.toml` allows them. It needs cargo-deny, installed with `cargo install cargo-deny --version 0.20.2 --locked`. |
| `package` | snapfit and snapfit-macros built from their packages alone, as crates.io receives them, with `cargo publish --workspace --dry-run`. |

`cargo xtask crate-readme` writes `snapfit/README.md`, which crates.io and docs.rs show, from the root README, as the site sees it: without the `<!-- github-only -->` blocks, with the `<!-- site-only` blocks shown, and with a link to the site in place of `<!-- snapfit:releases -->`. Run it after changing the root README, and commit the file it writes. It is not a mode of `cargo xtask ci`, whose `doc` mode checks that the file is up to date.

`cargo xtask book-check` runs only the tests of The Snapfit Book: it compiles and runs every example and checks every recorded output and compiler message. It is not a mode of `cargo xtask ci`, whose `test` mode already runs the book's tests with the rest of the workspace.

### The toolchain

`rust-toolchain.toml` pins the toolchain every contributor, cloud session and CI job builds with, a recent stable Rust. The minimum Rust version promised to users is `rust-version` in `Cargo.toml`, 1.85.0, which its own CI job checks. Raising either is a change of its own.

### GitHub Actions

Every action a workflow uses is pinned to a commit, with its version in a comment beside it.

* **`.github/workflows/ci.yml`**, on every pull request, whatever its base, and every push to `main`. A branch is checked through its pull request, so a push to a branch with no pull request, or whose pull request has a merge conflict, runs no checks. It runs each mode of `cargo xtask ci` in a job of its own on Linux, the `test` mode on Linux, macOS and Windows, the `test` mode again with the minimum supported Rust, and `npm run verify` in `site/`, which builds the site and checks its pages, links, accessibility and layout.
* **`.github/workflows/pull-request.yml`**, on every pull request, whatever its base: the description names an open issue of this repository, unless every commit other than a merge is a `test` and no `Cargo.toml` or `Cargo.lock` changes, and neither the description nor a commit message closes an issue of another repository, with the `pr-has-issue` and `no-cross-repo-closing` actions of itinera-dev/actions; every commit message keeps the convention of section 3, and the description has no unticked checkbox. Ticking a checkbox runs only this workflow again.
* **`.github/workflows/weekly.yml`**, each Monday: the `test` mode on Rust beta, so that a change in the next Rust that breaks snapfit, such as a compiler message a compile-fail fixture checks, is known before it is released.
* **`.github/workflows/release.yml`**, on a tag `v<version>`: section 9.
* **`.github/workflows/release-checks.yml`**, on a pull request that changes how releases are made, and called by the release workflow: the release tools and the crates' packages, as section 9 says.
* **`.github/workflows/site.yml`**, called by the release workflow for a release, or run by hand with a release's version: publishes that release's README front page and book edition onto the `site` branch, as section 8 says. The README and the book come from the release's tag, and the site's tools and `site/releases.json` from the commit the workflow runs on, so a fix to the tools, run by hand from `main`, can publish the newest release, whose tag holds the old tools. Run for an older release, it would put that release's README on the front page. The page check finds the editions published before on the `site` branch, which it does not build again.

---

## 8. The Snapfit Book & Documentation Site (`site` branch)

**The Snapfit Book** lives in `book/`: [book/BOOK.md](../book/BOOK.md) says what it contains and how it is written, and [book/FORMAT.md](../book/FORMAT.md) defines the directives its chapters use. Each edition teaches only the stable API of its release. The site in `site/` renders the book and the README, and the site workflow publishes them onto the `site` branch, which GitHub Pages serves at <https://marlon-sousa.github.io/snapfit-rs/>, the `homepage` of the crates. Pages is set once, in the repository's settings, to deploy from the `site` branch's root.

---

## 9. Releases

snapfit and snapfit-macros share one version, `version` in the workspace's `Cargo.toml`, and follow [Semantic Versioning](https://semver.org/). The generated names and the attributes SPEC.md describes are part of what is promised.

### Preparing a release

A release starts as a pull request, titled "Release <version>", whose commit `chore(release): release <version>`:

1. raises `version` under `[workspace.package]` and the `version` requirements of snapfit and snapfit-macros under `[workspace.dependencies]` in `Cargo.toml`, snapfit-macros' with `=`, because the code it generates calls the hidden items of the snapfit of the same version, and the versions in `Cargo.lock` with `cargo update --workspace`;
2. writes the version's section at the top of [CHANGELOG.md](../CHANGELOG.md), starting from what `git cliff --unreleased --tag v<version> --strip all` prints, which is the heading `## [<version>]` and the entries of the commits since the last release. A section the commits cannot write, such as 0.1.0's, which says what the first version on crates.io holds as a whole, is written by hand from SPEC.md, §0 ([#34](https://github.com/marlon-sousa/snapfit-rs/issues/34));
3. reconciles those entries, since a commit's entries cannot change once it merges: entries a later commit made untrue become one that says what the release holds, entries that say the same thing merge, and a change a user meets that carries no entry gets one;
4. points the chapter links of the README's "What Works Today", which crates.io, docs.rs and the site show, at the tag `v<version>`, the tag of this release, so that the README on crates.io and docs.rs, which never changes, keeps linking to the book it was released with. The tag does not exist until the maintainer pushes it, just after the merge, so those links do not resolve on `main` until then. In 0.2.0, whose pull request added those links, a `docs(readme)` commit of the release pull request did this step.

`cliff.toml` is written for git-cliff 2.14.2, which installs with `cargo install git-cliff --version 2.14.2 --locked`.

A pre-release is named with a dot before its number, such as `0.1.0-alpha.1` or `0.2.0-rc.1`. Only a version without a pre-release part and a release candidate, `rc.N`, go to crates.io; an alpha, a beta or any other pre-release is released on GitHub only.

### Tagging it

Once the pull request has merged, the maintainer tags the commit the merge put on `main` and pushes the tag, such as `git tag -a v0.1.0 <commit> -m "snapfit 0.1.0"` and `git push origin v0.1.0`. The release workflow then runs:

1. **release gate**, which refuses the tag unless it is `v` and the version in `Cargo.toml`, its commit is on `main`, and `CHANGELOG.md` has notes for the version, reporting every reason, and tells whether the version goes to crates.io and to the site;
2. **checks**, every job of `ci.yml` again on the tagged commit;
3. **release checks**, the release checks workflow:
   * **release tools**, the release gate, the crates.io rule and the site rule against their fixtures under `.github/release/`;
   * **the crates' packages**, the `.crate` files of snapfit-macros and snapfit;
4. **crates.io**, for a version that goes there, once every job before it passed;
5. **GitHub release**, with the `create-release` action of [itinera-dev/actions](https://github.com/itinera-dev/actions): the version's section of `CHANGELOG.md` as its notes, a pre-release for a version such as `0.2.0-rc.1`, and the packages attached;
6. **the site**, for a version without a pre-release part only, once the GitHub release exists: the site workflow publishes the release's book edition and front page, as section 8 says. The gate tells which versions do, with `.github/release/site.sh`, which the release tools check too.

A pull request that changes how releases are made, such as a change to the release workflows, `.github/release/`, `cliff.toml`, the root `Cargo.toml`, `Cargo.lock` or the toolchain, runs the same release checks workflow on its own, without releasing anything. The release workflow runs only on a tag, so a pull request lists none of its jobs.

### Publishing to crates.io

crates.io's trusted publishing lets the workflow publish without a token. The crates.io job runs in a `crates-io` environment that only `v*` tags may deploy to; `rust-lang/crates-io-auth-action` exchanges the workflow's identity for a short-lived crates.io token, and `cargo publish --workspace --locked` uses it to publish snapfit-macros, then snapfit ([#35](https://github.com/marlon-sousa/snapfit-rs/issues/35)). It works once both crates' settings on crates.io name `release.yml` and that environment as a trusted publisher, and the environment and its tag rule are set in the repository's settings; no token is kept anywhere. Trusted publishing is set up on a crate that already exists on crates.io, so the first version, 0.1.0, was published by hand. A crate's licence files are links to the ones at the root, which is why a publish runs on Linux: a Windows checkout without symbolic links would pack them as text.

If snapfit-macros is published and snapfit then fails, running the job again fails on snapfit-macros, which already exists. The jobs after the crates.io job do not run either. The maintainer then finishes the release by hand:

- publishes snapfit at the tagged commit with `cargo publish --package snapfit --locked`;
- makes the GitHub release;
- for a version without a pre-release part, runs the site workflow from the Actions tab with the release's version.

---

## 10. Issues come first

Every change starts as an issue, and is agreed there before any code is written: building the wrong thing is the expensive mistake, and the issue is the cheapest place to find it. The issues are also the record of every decision taken about snapfit, what it does and how it is built and released, so that a decision can be found again by its kind.

The one exception is a change whose commits, merges aside, are all `test` commits and that changes no `Cargo.toml` or `Cargo.lock`. It changes neither what snapfit does nor how its code is organised or released, so it holds no decision of its own: what it checks was agreed already. A test that needs a new dependency or another version of one, even one used only by tests, is not that exception: choosing a crate is an `architecture` decision, which starts from its issue. The pull request workflow asks for an issue whenever a `Cargo.toml` or a `Cargo.lock` changes, renamed or removed ones included.

* **A feature** says what is wanted and why, and what it must not break, such as a generated name. It can be short; the details are worked out in the issue.
* **A bug** says what happened, what was expected, and the smallest container that shows it.
* **A change to the architecture, the process or the documentation** says what changes and why, and what else was considered.
* **Labels:** each issue carries one label, which says what kind of decision it holds, so that decisions can be found again:
  * `feature`, something new a user meets;
  * `bug`, something a user meets that is wrong;
  * `architecture`, how the code is organised or written, refactors and the choice of dependencies included, and the rules on code of docs/ARCHITECTURE.md and AGENTS.md;
  * `process`, CI, this workflow, releases, and the process rules and review checklist of AGENTS.md;
  * `docs`, the documentation a user reads, such as README.md and the book.

  One issue per kind: a pull request that holds decisions of several kinds names one issue of each kind.

A decision the maintainer takes about an issue, wherever it is taken, such as in a chat, is recorded at once as a comment on the issue, headed "Decided so far (continued)", dated, saying that the maintainer decided it, and saying so when an agent wrote it. Once an issue is agreed, every decision taken in its comments is written into its body, which then binds the work. The comments stay as the history of how each decision was reached, and nothing in them may contradict the body.

[SPEC.md](SPEC.md) describes the whole design and, in §0, what ships; the open `feature` issues hold what is planned and when. A feature's behaviour is written or settled in SPEC.md in the pull request that builds it, as [AGENTS.md](../AGENTS.md), rule 1, asks.
