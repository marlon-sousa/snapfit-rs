# Contents

**The Wireup Book** for Rust (`v0.1.0`), in reading order. It follows the plan in [BOOK.md](../BOOK.md) and the directive layout in [FORMAT.md](../FORMAT.md). Every program the chapters show is complete: those in [examples](../examples/) run with `cargo run -p wireup_book --example <name>` from the repository root.

## Welcome

- [Welcome to Wireup](welcome-to-wireup.md)
- [How to read this book](how-to-read-this-book.md)
- [Quick start](quick-start.md)

## Part 1: Introduction

1. [Why Wireup?](01-why-wireup.md)
2. [Pure domain types, one container](02-pure-domain-types.md)
3. [Read the container, know the graph](03-read-the-container-know-the-graph.md)
- [Without Wireup](part-1-without-wireup.md)

## Part 2: Wiring a Container

4. [Installing Wireup and your first singleton](04-installing-and-first-singleton.md)
5. [Wiring dependencies and custom constructors](05-wiring-dependencies-and-constructors.md)
6. [Multiple instances of the same type](06-multiple-instances-of-the-same-type.md)
7. [Transient dependencies](07-transient-dependencies.md)
8. [Asynchronous containers](08-asynchronous-containers.md)
- [Without Wireup](part-2-without-wireup.md)

## Part 3: Compile-Time Guarantees & Agents

9. [Mistakes caught by the compiler](09-mistakes-caught-by-the-compiler.md)
10. [Working with coding agents](10-working-with-coding-agents.md)

## Appendices

- Appendix 1: [Cheat sheet](appendix-1-cheat-sheet.md)
- Appendix 2: [What this version supports](appendix-2-what-this-version-supports.md)
