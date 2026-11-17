# What this version supports

This edition of **The Snapfit Book** covers `snapfit` **`0.1.0`** (Minimum Supported Rust Version: **`1.85.0`**, Rust 2024 Edition).

## Capabilities in `0.1.0`

| Capability | Status in `0.1.0` | Details |
| :--- | :--- | :--- |
| **Pure-Rust domain types** | Supported | Zero macros or traits required on services, repositories, or controllers ([Chapter 2](02-pure-domain-types.md)). |
| **Construction order worked out at compile time** | Supported | Fields can be declared in any order; each is built after the fields it needs ([Chapter 3](03-read-the-container-know-the-graph.md)). |
| **Compile-time cycle and missing-prerequisite rejection** | Supported | Self-cycles, multi-node cycles, and unknown dependency names fail compilation, every one in the same build, each at its field ([Chapter 7](07-mistakes-caught-by-the-compiler.md)). |
| **Singletons (`#[singleton]`)** | Supported | Supports both `Arc<T>` (auto-wrapped) and any `T: Clone` without forced `Arc` ([Chapter 4](04-installing-and-first-singleton.md), [Chapter 5](05-wiring-dependencies-and-constructors.md)). |
| **Named constructors** | Supported | `constructor = <fn>` ([Chapter 5](05-wiring-dependencies-and-constructors.md)). |
| **Multiple instances of the same type** | Supported | Wired by container field name (`read_db`, `write_db`) without wrapper newtypes ([Chapter 6](06-multiple-instances-of-the-same-type.md)). |
