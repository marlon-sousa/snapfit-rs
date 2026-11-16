# The Wireup Book

What **The Wireup Book** contains, and how it is written. This document is to the book what [docs/SPEC.md](../docs/SPEC.md) is to the implementation: it defines the audience, the voice, the running story, the structure of every chapter, and the table of contents.

## Who the book is for

The book is for Rust developers building services and applications with `wireup`, and for the AI coding agents building software alongside them. It is not an internal compiler manual: [docs/SPEC.md](../docs/SPEC.md) defines the specification, [docs/TECHNICAL_SPEC.md](../docs/TECHNICAL_SPEC.md) documents the internal Hexagonal Architecture of the procedural macro, and the Rustdoc API reference documents every exported trait and type.

A reader finishes the book able to:

1. explain why `wireup` keeps domain structs 100% free of framework macros and traits, and why declaring the dependency graph in a central container struct is safer than invasive annotations or runtime `TypeMap` service locators;
2. wire singletons, transients, custom constructors, custom builders, and multiple instances of the same type in a `#[container]`;
3. initialize asynchronous containers eagerly at startup while keeping runtime getters synchronous and lock-free;
4. read and fix the compile-time errors `wireup` emits when a dependency graph has a cycle or a missing prerequisite.

A coding agent reading the book gains a structural advantage: in a `wireup` codebase, changing a business rule touches only its pure-Rust struct, and changing how components are wired touches only one `#[container]` struct. Context is never scattered across dozens of files.

## How the book is written

### Voice and accessibility

1. Plain, direct, and welcoming prose aimed at a developer who knows Rust and wants clean dependency management without boilerplate or runtime overhead.
2. Every page is written for screen readers as a first-class requirement: a single `#` heading per page, logical heading hierarchy (`##` then `###`), numbered lists, and simple tables.
3. Diagrams are Mermaid flowcharts in fenced `mermaid` blocks that always declare `accTitle` and `accDescr`, and every diagram is immediately followed by a numbered list in prose stating everything the diagram shows. ASCII-art boxes and character-drawn trees are never used.
4. Every code listing is a directive (`example`, `output`, or `compile-error`) verified by `cargo xtask book-check`, as defined in [FORMAT.md](FORMAT.md).

### One story: the online shop

Every example in the book belongs to one small online shop processing customer orders, which grows naturally from chapter to chapter:

1. it loads shop configuration (`ShopConfig`) and tax rates (`TaxRateTable`);
2. it connects to catalog and order databases (`Database`), including separate read and write replicas (`read_db` and `write_db`);
3. it processes payments (`PaymentGateway`), reserves inventory (`StockService`), and checks out orders (`OrderService` and `CheckoutController`).

Every chapter's example is a complete, self-contained Rust program in `book/examples/` that compiles, runs, and prints deterministic output.

### The shape of a chapter

Every numbered chapter, and the quick start, has the same four level 2 sections in exact order:

1. **What you want to do:** one or two sentences stating the goal in the shop's terms.
2. **The code:** directives showing the relevant regions of the runnable program and what it prints (or the compiler diagnostic for compile-error chapters).
3. **How it works:** numbered points explaining the mechanics in plain language.
4. **Remember:** a concise list of rules and invariants for readers and coding agents.

### Without Wireup: the section that closes Parts 1 and 2

Parts 1 and 2 close with a section headed **Without Wireup**. It shows the same shop written the way Rust services are often wired without `wireup` — manual constructor threading or a dynamic `HashMap<TypeId, Box<dyn Any>>` service locator — followed by the `wireup` version beside it. Both programs are compiled and run in `book/examples/` so every comparison is proven by code that runs.

## Contents (`v0.1.0`)

### Welcome

Unnumbered pages introducing `wireup`, the book's conventions, and a working container on one page:

1. **Welcome to Wireup** (`welcome-to-wireup.md`): What `wireup` is, why inverting dependency declarations keeps Rust domains pure, and how the book is organized.
2. **How to read this book** (`how-to-read-this-book.md`): Conventions, the online shop story, the four-part chapter structure, verified code directives, and screen-reader accessibility.
3. **Quick start** (`quick-start.md`): A complete shop container with pure domain types, `#[singleton]`, `#[transient]`, and zero-cost getters on one page (`quick_start.rs`).

### Part 1: Introduction

Makes one central argument: your domain types should remain 100% pure Rust, and you should be able to understand how an entire application is wired by reading one container struct.

1. **Why Wireup?** (`01-why-wireup.md`): Contrasts the three usual approaches in Rust (manual `main.rs` wiring chains, invasive `#[inject]` component annotations, and dynamic `TypeMap`/`dyn Any` service locators) against `wireup`'s unique five-pillar design (`ch01_manual_wiring.rs` and `ch01_why_wireup.rs`).
2. **Pure domain types, one container** (`02-pure-domain-types.md`): Shows that domain structs need zero macros or traits — the exact same structs can be constructed directly in a unit test or wired by a `#[container]` (`ch02_pure_domain.rs`).
3. **Read the container, know the graph** (`03-read-the-container-know-the-graph.md`): Shows how `#[container]` serves as an executable architecture manifest and topologically sorts dependencies at compile time regardless of field declaration order (`ch03_container_graph.rs`).
* **Without Wireup** (`part-1-without-wireup.md`): Side-by-side comparison of answering architectural questions in `ch01_manual_wiring.rs` versus `ch03_container_graph.rs`.

### Part 2: Wiring a Container

Covers every lifecycle and construction pattern in `v0.1.0`:

4. **Installing Wireup and your first singleton** (`04-installing-and-first-singleton.md`): Adding `wireup` to `Cargo.toml`, declaring a leaf `#[singleton]`, instantiating the container with `new()`, and verifying pointer identity across `get_<name>()` calls (`ch04_first_singleton.rs`).
5. **Wiring dependencies and custom constructors** (`05-wiring-dependencies-and-constructors.md`): Declaring prerequisites with `deps = [...]`, calling named constructors with `constructor = "..."`, using custom container builder methods with `builder = "..."`, and storing `T: Clone` types directly without forced `Arc<T>` wrapping (`ch05_wiring_and_constructors.rs`).
6. **Multiple instances of the same type** (`06-multiple-instances-of-the-same-type.md`): Wiring `read_db` and `write_db` of the exact same `Arc<Database>` type by field name without wrapper newtypes (`ch06_multiple_instances.rs`).
7. **Transient dependencies** (`07-transient-dependencies.md`): Declaring `#[transient]` components that are stripped from the container struct's memory layout and freshly constructed on every `get_<name>()` call (`ch07_transients.rs`).
8. **Asynchronous containers** (`08-asynchronous-containers.md`): Using `#[container(mode = "async")]` and `async` singleton constructors so all asynchronous I/O resolves eagerly during `init().await` while runtime getters remain synchronous and lock-free (`ch08_async_container.rs`).
* **Without Wireup** (`part-2-without-wireup.md`): Demonstrates the `TypeMap` overwrite bug when registering two databases of the same type (`part2_without_wireup.rs`) and contrasts it with `wireup`'s named-edge graph.

### Part 3: Compile-Time Guarantees & Agents

9. **Mistakes caught by the compiler** (`09-mistakes-caught-by-the-compiler.md`): Every graph and configuration mistake `wireup` turns into a compile-time error, with the real program and recorded `rustc` diagnostic (`ch09_unknown_dependency`, `ch09_dependency_cycle`, `ch09_self_dependency_cycle`, `ch09_conflicting_lifecycle`, `ch09_sync_mode_with_async_dep`, `ch09_async_transient`, `ch09_tuple_struct_container`).
10. **Working with coding agents** (`10-working-with-coding-agents.md`): Why separating pure domain modules from the central `#[container]` manifest prevents context bloat and wiring drift when working with AI coding agents, complete with a runnable example (`ch10_agent_workflow.rs`) and a copy-ready `AGENTS.md` rule block.

### Appendices

* **Appendix 1: Cheat sheet** (`appendix-1-cheat-sheet.md`): Every container setting and field attribute in one quick-reference table, linking to the chapter that teaches it.
* **Appendix 2: What this version supports** (`appendix-2-what-this-version-supports.md`): Summary of all capabilities shipped in `wireup` `0.1.0` and upcoming roadmap milestones.
