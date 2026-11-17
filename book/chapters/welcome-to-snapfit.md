# Welcome to Snapfit

Hello, and welcome to **The Snapfit Book**.

`snapfit` is a zero-cost, compile-time verified dependency injection framework for Rust. It is built around one architectural inversion: **the container struct declares the dependency graph, leaving your domain structs 100% pure Rust.**

## What we are solving

As a Rust application grows beyond a few structs, wiring components together usually pushes teams toward one of three compromises:

1. **Invasive component macros** (`#[inject]`, `#[component]`, framework traits) scattered across every business service and repository.
2. **Dynamic service locators** (`TypeMap` or `Arc<dyn Any + Send + Sync>`) that defer missing-dependency checks to runtime, pay dynamic downcasting overhead, and collide whenever two instances share the same Rust type.
3. **Hand-rolled `main.rs` initialization chains** where dozens of `Arc::clone` calls hide which services are shared and what depends on what.

Keeping domain logic clean and making the dependency graph explicit in one place was good engineering when only humans maintained the code. When AI coding agents work alongside your team, it becomes a matter of safety: an agent starting a session with zero memory can read one `#[container]` struct to understand the entire system's wiring, and can change a business rule inside a pure Rust struct without touching any framework macro.

## How the book is laid out

- **[Part 1: Introduction](01-why-snapfit.md)** makes the case for container-driven dependency injection, shows why domain types need zero annotations, and demonstrates how reading one container struct tells you the full dependency graph.
- **[Part 2: Wiring a Container](04-installing-and-first-singleton.md)** walks through every stable wiring feature in `v0.1.0`: singletons, custom constructors, and multiple instances of the same type (`read_db` and `write_db`).
- **[Part 3: Compile-Time Guarantees & Agents](07-mistakes-caught-by-the-compiler.md)** shows every mistake `snapfit` catches at compile time with the exact `rustc` error messages, and how to structure `AGENTS.md` so coding agents take full advantage of `snapfit`.
- **[Appendices](appendix-1-cheat-sheet.md)** provide a one-page attribute cheat sheet and the capability summary for `v0.1.0`.
