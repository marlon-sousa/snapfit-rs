# Architecture

How snapfit's code is organised and written. [SPEC.md](SPEC.md) says what the macros accept, generate and refuse; this document says how the code that does it is laid out, and the rules every change follows. [AGENTS.md](../AGENTS.md) says how a change is made and reviewed against it. Changing a rule here is agreed with the maintainer first, like any other change.

The rules are modelled on those of [gherkin-glass](https://github.com/marlon-sousa/gherkin-glass/blob/main/docs/ARCHITECTURE.md), adapted to a procedural macro, which reads tokens and writes tokens and touches nothing else.

## The guiding principle

The domain decides; the adapters translate. The pure part of snapfit, which decides what a container is and how it is built, knows nothing of `syn`, `quote`, `darling` or `proc_macro2`. The adapters at both ends turn the user's tokens into the domain's data, and the domain's plan back into tokens, and decide nothing about containers themselves.

A procedural macro does no input or output: it is a pure function from tokens to tokens. So snapfit has no ports. A port is a trait the domain needs the world to implement, and a pure function never gets a trait around it only to be faked: it is already testable.

## The repository

- **`snapfit/`**, the crate users depend on: the `Container` trait and the re-export of the macro. Its `README.md` is a symbolic link to the one at the root and is its crate documentation on docs.rs, so the README's example is a doctest of `snapfit`.
- **`snapfit-macros/`**, the procedural macro, laid out as the rest of this document describes.
- **`tests/`**, the crate `snapfit_bdd`: the Gherkin features under `tests/features/` (`stable/`, and `unstable/`, run with the `unstable` feature), their step definitions, the containers they exercise, and the compile-fail fixtures under `tests/fixtures/compile_fail/`, compiled with [compile-outcomes](https://github.com/marlon-sousa/compile-outcomes-rs) against the build of snapfit the suite runs, whose diagnostics the steps check.
- **`book/`**, the crate `snapfit_book`: The Snapfit Book, whose chapters, examples, recorded outputs and compile-error fixtures `cargo xtask book-check` verifies. [book/BOOK.md](../book/BOOK.md) describes it.
- **`site/`**, the Astro site that renders the README as its front page and each edition of the book, checked by `npm run verify` and published onto the `site` branch by the site workflow.
- **`xtask/`** and **`scripts/gate.py`**, the quality gate, `cargo xtask ci`, whose checks CI runs, the tests on Linux, macOS and Windows; [WORKFLOW.md](WORKFLOW.md), section 7, lists them.
- **`.github/release/`**, the release gate and its fixtures, which the release and release checks workflows run, as [WORKFLOW.md](WORKFLOW.md), section 9, says.
- **`.github/workflows/`**, the CI, pull request, weekly, release, release checks and site workflows, as [WORKFLOW.md](WORKFLOW.md), section 7, lists them.
- **`clippy.toml`**, what clippy allows in tests.
- **`deny.toml`**, what cargo-deny allows of the dependencies.
- **`cliff.toml`** and **`CHANGELOG.md`**, the template git-cliff writes a release's notes with, and the notes of every release.
- **`docs/`**: [SPEC.md](SPEC.md), what the macros do; this document; [WORKFLOW.md](WORKFLOW.md), the stacked pull requests and the definition of done.
- **`.claude/skills/`**, the procedures an agent follows, such as having a change reviewed. They add no rule of their own: each points at the rules it follows.

## How an expansion runs

1. `lib.rs` defines the macro, `#[container]`, and hands its tokens to the handler.
2. The handler, `expansion.rs`, parses the tokens with `syn`, asks the inbound adapter for the container's declaration, asks the `plan_container` service for its plan, and asks the outbound adapter for the tokens of that plan. When any step finds problems, it asks the outbound adapter for one compiler error per problem instead, and keeps the struct as written.
3. The inbound adapter, `adapters/inbound`, reads the struct with `syn` and the attributes with `darling` into a `ContainerDeclaration`, and reports every field it cannot read.
4. The service, `use_cases/derive_container.rs`, runs every policy over the declaration: the feature gate, the declaration rules, the container's execution and the resolver, which orders the dependencies and links each step to the steps it needs. It reports every problem the policies find together.
5. The outbound adapter, `adapters/outbound`, writes the struct, `new()` or `init()`, the `Default` and `Container` impls and the getters from the plan with `quote`, or writes the compiler errors.

## Module roles

Every module plays one role, and says which on the first line of its `//!` doc comment, such as `//! Policy: whether the container is built at once by new() or awaited through init().`

1. **Entity**: data and the invariants it keeps, in `domain/`: the names, the declaration, the plan, the build.
2. **Error**: what makes snapfit refuse a container, in `domain/errors.rs`, as "Errors" below describes.
3. **Policy**: a pure decision about a container, in `use_cases/`: the feature gate, the declaration rules, the execution, the resolver and the call that builds each value.
4. **Service**: `use_cases/derive_container.rs`, which runs the policies over one declaration and gathers their problems.
5. **Adapter**: the code that speaks `syn`, `darling` and `quote`, in `adapters/`: the inbound adapter, the outbound adapter, and the syntax they share. Pure code that only one adapter uses to speak its library, such as reading a type's shape, is a private submodule of that adapter.
6. **Handler**: `expansion.rs`, the way in, which turns a macro's tokens into calls to the adapters and the service, and their results into tokens.
7. **Facade**: `lib.rs`, which holds only the macro, a one-line call to the handler.

A submodule that serves a single module, such as the resolver's walk state, plays its parent's role and says so on its role line.

Layering goes one way. The facade calls the handler; the handler calls the adapters and the service; the service calls policies; policies use entities and errors; adapters use entities and errors, never a policy or the service. No module reaches around another into its internals.

## The typed model

- **One model from parser to emitter.** `ContainerDeclaration<C, F>` and `ConstructionPlan<C, F>` carry the adapter's syntax, `C` for the container and `F` for each field, without reading it. The domain names no compiler type, yet the emitter gets back each field's own syntax with every decision about it, so it never looks a field up by name and never re-parses text.
- **Steps are linked, not named.** Each `Step` holds the steps of its prerequisites, shared with `Rc`, in the order they are passed. A step the plan holds always has the steps it needs, so the emitter has no lookup that could fail.
- **Adapters translate, they don't decide.** The inbound adapter reports what it sees: the field's lifecycle, its `deps`, whether it is `async`, how its `build` setting builds it, and its type's shape (`Arc<T>`, a trait object, or a plain value). Every choice made from those facts is a policy: whether the container is async, whether a value is wrapped in `Arc::new`, whether a trait object can be built, and what this build of snapfit accepts. A reading the adapter cannot turn into a declaration, such as a quoted function name or a `build` word that names no way of building, is an error of the adapter.
- **No compiler type in the domain.** `domain/` and `use_cases/` name no type or macro of `syn`, `quote`, `darling`, `proc_macro` or `proc_macro2`, which `cargo xtask ci` checks. Those types reach the domain only as the `C` and `F` parameters.
- **Diagnostics locate by subject.** An error names the dependency it is about, and the outbound adapter points the compiler error at that field's snapfit attribute, or at its name, or at the container's name when the error is about no field or the field is unknown. This is the one lookup by name, and it cannot fail.

## Folders and files

1. **The visibility ladder.** Private by default, `pub(crate)` when another module needs it, and bare `pub` only for the macros of `snapfit-macros` and the public API of `snapfit`. The `unreachable_pub` lint, set in the workspace's `[workspace.lints]`, catches a `pub` that is not re-exported.
2. **`module.rs` with a `module/` folder, never `mod.rs`**, so that every file's name says what it holds, which also reads better in a screen reader's list of editor tabs. `cargo xtask ci` refuses a `mod.rs`.
3. **Folders are named by role**: `domain/`, `use_cases/`, `adapters/`, and within `adapters/`, `inbound/` and `outbound/`. There is no `utils.rs` or `helpers.rs`.
4. **A module with submodules decides its public face**: its submodules are private, and it re-exports what the rest of the crate uses.
5. **Unit tests live with the code**, in `module/tests.rs`, declared with `#[cfg(test)] mod tests;` at the bottom of `module.rs`, in the module whose code decides what they check. Tests in `tests/` reach the macros only through the `snapfit` crate, as a user does: a running container, or a fixture compiled against `snapfit`.
6. **A file holds one thing, and its length is a signal.** A file holding two things or more, at any length, keeps one and moves each other to a private submodule. Past 200 lines `cargo xtask ci` fails, and the file splits along its seams. Each file is counted on its own, and a module's tests, in their own `tests.rs`, are counted apart from it.
7. **Import a symbol, then use it by its bare name**, except a crate's types used once where the crate's name says what they are, such as `syn::Ident` in an adapter.
8. **Helpers that the tests of several modules share** live in a test-only module, never copied from one test module to another: `test_support.rs` at the crate's root builds names and declarations for every layer's tests, and `adapters/outbound/test_support.rs` builds plans for the outbound adapter's.

## Writing the code

- **Formatting is plain `rustfmt`.**
- **Nothing panics while a macro expands.** Every failure is a `DomainError` reported as a compiler error. Clippy denies `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo` and `unimplemented`, as `[workspace.lints]` sets, and `clippy.toml` allows them in tests.
- **No `unsafe` code**: `[workspace.lints]` forbids it.
- **Reuse before writing.** Look first at the standard library, then at the crates already depended on, then at well-known crates. Write it ourselves only when none fits, or when a crate would replace only a couple of lines. A crate is judged by what it replaces across the code, not only by its first use.
- **Derive what a derive produces.** Errors derive with `thiserror`; `Display` with `derive_more`; a constructor that only assigns fields with `derive-new`; an enum's `is_*` predicates with `strum`'s `EnumIs`. An impl is written by hand only when it needs logic.
- **Types hold every constraint they can.** An enum for a closed set of values, never a string or a `bool` standing for one: `Lifecycle`, `Execution`, `Construction`, `TypeShape`, `TypeParameters`, `Introspection`, `FeatureGate`, `Feature`. A newtype for a name with rules of its own: `Name<K>`, never blank, whose kind keeps a dependency's name from being passed as a function's. A doc comment stating a constraint the type could enforce means the type is wrong. A newtype never derives `Deref`, which is for smart pointers.
- **A chain of iterator adapters reads as a sentence of names**, such as `.filter(is_async).map(async_in_sync_container)`. A closure in a chain is one call, one method call, one field read or one comparison with a value it captures; a closure that negates, matches, computes or builds a value becomes a named function.
- **Iterating to get a result uses adapters, not a `for` loop.** A `for` loop stays only when its body acts on each item, such as visiting each dependency or asserting in a test.
- **Every line does something.** No line that changes nothing.

## Errors

- **One enum, `DomainError`**, in `domain/errors.rs`, derived with `thiserror`. Its `Display` is the sentence the compiler shows, so all of an error's words are in one place. The parts several variants share are enums of their own in `domain/errors/parts.rs`: the `Feature` a build refuses and the `AttributeProblem` of an attribute that does not read. Only `AttributeProblem::Unreadable` carries words from outside, those `darling` or `syn` wrote about tokens they could not parse.
- **Every message must make sense spoken by a screen reader**: words, never arrows or chains of symbols. A cycle reads "service_a depends on service_b, which depends on service_a". A message names the field it is about and, where it can, says what to write instead, such as "build = constructor(connect)".
- **Every problem at once.** A function that can find several problems returns `DomainErrors`, which is never empty, and gathers them with `DomainErrors::both` and `DomainErrors::collect` instead of stopping at the first with `?`. It stops only where going on needs what failed: a field that cannot be read has no declaration to resolve.
- **Each error has a subject**, the dependency it is about, or none when it is about the whole container, and the diagnostic points there, as "The typed model" describes.

## Generated code

- **Generated names are a promise.** `new()`, `init()`, `get_<field>()` and the `Default` and `Container` impls are the public face of every container; changing how any of them is named or typed is agreed with the maintainer first.
- **Hygiene.** Generated code names everything by an absolute path, such as `::std::sync::Arc` and `::snapfit::Container`, and binds each singleton to a local named `__snapfit_<field>` with the field's span, so it never clashes with anything the user has in scope.
- **Construction order** is the order the fields are declared, each after the fields it needs, as SPEC.md, §6, says.

## Comments

A comment says only what the code cannot:

- the module's role line;
- an invariant or a hazard the code does not enforce itself;
- what an absent value, an error or a rejection carries.

One sentence, where it applies, once. Why a decision was taken, and what else was considered, belong in the pull request. A field, a function or a test whose name already says what it is gets no doc comment. No `TODO` comments. A measured fact names the program and its version, never a date. Code never cites an issue or a pull request.

## Tests

Cheapest first:

1. **Policy tests**, pure and in memory, over declarations whose syntax is `()` or a string: the resolver's order and errors, the feature gate, the rules, the execution.
2. **Adapter tests**: the inbound adapter over `parse_quote!` structs, checked as declarations and errors; the outbound adapter over plans the real parser and service build, checked as tokens.
3. **Handler tests**, tokens in and tokens out, checking the generated code and the messages of every `compile_error!`.
4. **Gherkin scenarios**, `tests/features/`, which run real containers built by the macros, and compile each fixture of `tests/fixtures/compile_fail/`, checking each message, the line it points at and how many errors the compiler reports. compile-outcomes compiles every fixture once per run, in a project of its own for each build under the suite's `CARGO_TARGET_TMPDIR`, with `snapfit` taken from its folder, with the `unstable` feature only when the suite runs with it, and the workspace's `Cargo.lock`. So a fixture meets the snapfit of the build that runs it, whatever other builds the target folder holds. The steps judge what it reports: whether the fixture compiled, and each error's message and the file and line it points at.

How a test is written:

- **A test's name states the rule it checks**, as a sentence, such as `a_cycle_reads_as_a_sentence_without_arrows`.
- **A fixed bug gets a regression test that fails without the fix**, and a Gherkin scenario when a user can meet the bug.
- **A setup that three or more tests of one module start the same way** becomes a helper of those tests, named for what it makes.
- **A rule checked on several inputs is one test over a table of cases**, each case's input and expectation side by side.
