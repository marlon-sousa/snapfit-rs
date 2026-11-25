# The Snapfit Book

What **The Snapfit Book** contains, and how it is written. This document is to the book what [docs/SPEC.md](../docs/SPEC.md) is to the implementation: it defines the audience, the voice, the running story, the structure of every chapter, and the table of contents.

## Who the book is for

The book is for Rust developers building services and applications with `snapfit`, and for the AI coding agents building software alongside them. It is not an internal compiler manual: [docs/SPEC.md](../docs/SPEC.md) defines the specification, [docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md) documents how the procedural macro's code is organised, and the Rustdoc API reference documents every exported trait and type.

A reader finishes the book able to:

1. explain the problems of services that build their own dependencies, and what dependency injection is;
2. explain why `snapfit` keeps domain structs 100% free of framework macros and traits, and why declaring the dependency graph in a central container struct is safer than wiring by hand or runtime `TypeMap` service locators;
3. wire singletons, named constructors, factories, builders, multiple instances of the same type, a chosen implementation of a trait, a trait dispatched statically through a closed list of implementations, and components that can fail while they are built in a `#[container]`, and test a service on its own, with stand-ins for what it asks for;
4. read and fix the compile-time errors `snapfit` emits when a dependency graph has a cycle or a missing prerequisite.

A coding agent reading the book gains a structural advantage: in a `snapfit` codebase, changing a business rule touches only its pure-Rust struct, and changing how components are wired touches only one `#[container]` struct. Context is never scattered across dozens of files.

## How the book is written

### Voice and accessibility

1. Close, conversational, and welcoming prose, written for a developer who knows Rust and wants to understand what dependency injection is about, not as a report. The book walks beside the reader: "we" build the shop step by step, run it, and notice what goes wrong as it grows. A problem is shown before it is named, and is explained in plain words, often with an analogy from everyday life (a lamp wired into the wall, a ride requested in an app, a cloakroom that files coats by colour), choosing whichever analogy fits the idea best.
2. Every page is written for screen readers as a first-class requirement: a single `#` heading per page, logical heading hierarchy (`##` then `###`), numbered lists, and simple tables.
3. Diagrams are Mermaid flowcharts in fenced `mermaid` blocks that always declare `accTitle` and `accDescr`, and every diagram is immediately followed by a numbered list in prose stating everything the diagram shows. ASCII-art boxes and character-drawn trees are never used.
4. Every code listing is a directive (`example`, `output`, or `compile-error`) verified by `cargo xtask book-check`, as defined in [FORMAT.md](FORMAT.md).

### One story: the online shop

Every example in the book belongs to one small online shop processing customer orders, which grows naturally from chapter to chapter:

1. it reserves stock (`StockService`), charges payments through a `PaymentGateway` such as Stripe or a fake, and places orders (`OrderService`);
2. it loads shop configuration (`ShopConfig`) and tax rates (`TaxRateTable`);
3. it connects to catalog and order databases (`Database`), including separate read and write replicas (`read_db` and `write_db`).

Every chapter teaches only the stable API of `0.3`: nothing behind the `unstable` feature, and nothing planned for a later release. Every chapter's example is a complete, self-contained Rust program in `book/examples/` that compiles, runs, and prints deterministic output.

### The shape of a chapter

Every numbered chapter, and the quick start, tells its own story under its own level 2 headings, and ends with the same closing sections, so that a reader, or an agent, always knows where to find them. A chapter of Part 1 shows a problem; every other chapter teaches a feature.

A problem chapter (Part 1) writes the shop's code without `snapfit`, runs it, and follows what goes wrong as the shop grows. It closes with:

1. **What we found:** the problems the chapter met, each opening with its short name in bold, the recognised name where one exists, such as tight coupling. Later chapters refer to the problems by these names.

A feature chapter (every other chapter, and the quick start) shows the code of the feature in the shop's terms, runs it, and explains what happened. It closes with:

1. **Which problem this solves:** the problems of Part 1, by their bold names and with a link to the chapter that names them, that this feature answers, and how.
2. **Remember:** a short list of what to keep in mind, for readers and coding agents.

### The problem first

The book does not open with `snapfit`. Part 1 follows the shop without it: services that build their own dependencies, then services that receive them, then the one place that has to build everything. Each step fixes a problem and names the next. Part 2 then shows the same shop with `snapfit` and maps every named problem to the chapter that answers it, and every feature chapter after it says which of those problems it solves. Every comparison is a program in `book/examples/` that compiles and runs.

### Limits, honestly

Each edition teaches only what its release does. Where an experienced reader would ask for something the release does not do, the book says so in one sentence, and may say that it could come in a later version, without promising what or when.

## Contents (`0.3`)

### Welcome

Unnumbered pages introducing `snapfit`, the book's conventions, and a working container on one page:

1. **Welcome to Snapfit** (`welcome-to-snapfit.md`): What `snapfit` is, why the book starts with the problem, and how it is organized.
2. **How to read this book** (`how-to-read-this-book.md`): Conventions, the online shop story, the two shapes of a chapter, verified code directives, and screen-reader accessibility.
3. **Quick start** (`quick-start.md`): A complete shop container with pure domain types, `#[singleton]` dependencies, and getters on one page (`quick_start.rs`).

### Part 1: Why dependency injection

Problem chapters. The shop without `snapfit`, one step at a time, until what is left is what `snapfit` solves.

1. **A service that builds its own dependencies** (`01-a-service-that-builds-its-own-dependencies.md`): Services that open their own database and connect to Stripe in `new()`, and the four problems that follow (`ch01_builds_its_own.rs`).
2. **Choosing between implementations** (`02-choosing-between-implementations.md`): A closed list, a type parameter, a trait object and an enum that forwards each call as ways to work with Stripe or a fake, and what each costs (`ch02_choosing_between_implementations.rs`).
3. **Who builds everything?** (`03-who-builds-everything.md`): The composition root built by hand and as a `TypeMap` service locator, the problems of each, and what dependency injection means (`ch03_manual_wiring.rs` and `ch03_service_locator.rs`).

### Part 2: Snapfit

4. **Why we built Snapfit** (`04-why-we-built-snapfit.md`): The shop of Chapter 3 built by a `#[container]`, what the macro does, every problem of Part 1 mapped to the chapter that answers it, and a comparison with the shapes of Chapter 3 (`ch04_why_snapfit.rs`).
* **What this version supports** (`what-this-version-supports.md`): Who `0.3` is for, what it does, what it does not do, and the plan for small, frequent releases.

### Part 3: Wiring a container

Covers every stable feature of `0.3`, one per chapter, and how to test the services a container wires:

5. **Installing Snapfit and your first singleton** (`05-installing-and-first-singleton.md`): Adding `snapfit` to `Cargo.toml`, declaring a leaf `#[singleton]`, instantiating the container with `new()`, and verifying pointer identity across `get_<name>()` calls (`ch05_first_singleton.rs`).
6. **Pure domain types, one container** (`06-pure-domain-types.md`): Shows that domain structs need zero macros or traits — the exact same structs can be constructed directly in a unit test or wired by a `#[container]` (`ch06_pure_domain.rs`).
7. **Wiring dependencies and custom constructors** (`07-wiring-dependencies-and-constructors.md`): Declaring prerequisites with `deps = [...]`, building a field with `new`, `build = default` or a named constructor, `build = constructor(<fn>)`, building a container's leaves with `leaf_build = default`, and storing `T: Clone` types directly without forced `Arc<T>` wrapping (`ch07_wiring_and_constructors.rs` and `ch07_leaf_build.rs`).
8. **Read the container, know the graph** (`08-read-the-container-know-the-graph.md`): Shows how `#[container]` serves as an executable architecture manifest, and how fields declared in any order are each built after the fields they need (`ch08_container_graph.rs`).
9. **Multiple instances of the same type** (`09-multiple-instances-of-the-same-type.md`): Wiring `read_db` and `write_db` of the exact same `Arc<Database>` type by field name without wrapper newtypes, and the swap that positional `deps` can't catch (`ch09_multiple_instances.rs` and `ch09_swapped_deps.rs`).
10. **Passing dependencies by name** (`10-passing-dependencies-by-name.md`): `build = factory` and the generated `<Container>Factories` trait, `build = builder` with labels and `start`/`finish`, when to use which, and which logic belongs to a type and which to the wiring (`ch10_factories.rs` and `ch10_builders.rs`).
11. **Choosing an implementation** (`11-choosing-an-implementation.md`): Naming the implementation once, in a field's concrete type, for services that take a trait object, and a second container for tests (`ch11_choosing_an_implementation.rs`); a field typed as the trait that names its `implementation` (`ch11_trait_field.rs`), and why the getter then hides the concrete type (`ch11_trait_field_hides_stripe`); and a trait-object field built by a factory that reads the configuration (`ch11_configured_payments.rs`), with the error for a trait-object field built by neither (`ch11_trait_object_field`).
12. **Testing a service on its own** (`12-testing-a-service-on-its-own.md`): A checkout with a deep graph that a test has to build by hand (`ch12_testing_by_hand.rs`); services asked for as traits, held as trait objects, and stand-ins generated by mockall, with a word on other libraries and hand-written fakes (`ch12_mocking_services.rs`); and a database behind a trait, swapped for SQLite in one word (`ch12_swapping_the_database.rs`) and for an in-memory database in a test's container (`ch12_in_memory_database.rs`).
13. **Static dispatch** (`13-static-dispatch.md`): Price rules called on every line of a basket, why a trait object's pointer can start to show, `#[snapfit::static_dispatch]` on the trait, `static_dispatch_for` and fields typed `Arc<Prices>` naming their `implementation` (`ch13_static_dispatch.rs`); a test that builds `Prices::of` a stand-in added by `extend_static_dispatch` under `cfg_attr(test, ...)`; a factory that chooses the rule while the program starts, with `extra` (`ch13_choosing_the_rule.rs`); and what a dispatched trait may hold, with the error for a method that has no receiver (`ch13_rule_without_receiver`).
14. **Choosing between static and dynamic dispatch** (`14-choosing-static-or-dynamic-dispatch.md`): One container that dispatches payments dynamically and prices and receipts statically, a generic method no trait object can hold, a service generic over the trait so that a domain crate never names the container's type, and a side-by-side comparison with a rule of thumb (`ch14_static_or_dynamic.rs`).
15. **When building can fail** (`15-when-building-can-fail.md`): A database that may not answer and a factory that may refuse its configuration, `fallible = true`, `try_new()` and `FallibleContainer`, reading a `BuildError`, and what the compiler checks (`ch15_when_building_can_fail.rs`), with the error for a fallible field that `leaf_build` would build (`ch15_fallible_leaf_build`).

### Part 4: Compile-time guarantees and agents

16. **Mistakes caught by the compiler** (`16-mistakes-caught-by-the-compiler.md`): Every graph and configuration mistake `snapfit` turns into a compile-time error, with the real program and recorded `rustc` diagnostic (`ch16_unknown_dependency`, `ch16_dependency_cycle`, `ch16_self_dependency_cycle`, `ch16_quoted_constructor`, `ch16_several_mistakes`, `ch16_tuple_struct_container`).
17. **Working with coding agents** (`17-working-with-coding-agents.md`): Why separating pure domain modules from the central `#[container]` manifest prevents context bloat and wiring drift when working with AI coding agents, complete with a runnable example (`ch17_agent_workflow.rs`) and a copy-ready `AGENTS.md` rule block.

### Appendices

* **Appendix 1: Cheat sheet** (`appendix-1-cheat-sheet.md`): Every container setting and field attribute in one quick-reference table, linking to the chapter that teaches it.

### Held back for a later edition

Transient dependencies and asynchronous containers work in `0.3` but only behind the `unstable` feature, so their chapters are not part of the `0.3` editions. Their drafts, examples and expected output are kept in [`unreleased/`](unreleased/README.md) and return as Part 3 chapters when those APIs are stabilised.
