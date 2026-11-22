# Contents

**The Snapfit Book** for Rust (`0.2`), in reading order. It follows the plan in [BOOK.md](../BOOK.md) and the directive layout in [FORMAT.md](../FORMAT.md). Every program the chapters show is complete: those in [examples](../examples/) run with `cargo run -p snapfit_book --example <name>` from the repository root.

## Welcome

- [Welcome to Snapfit](welcome-to-snapfit.md)
- [How to read this book](how-to-read-this-book.md)
- [Quick start](quick-start.md)

## Part 1: Why dependency injection

1. [A service that builds its own dependencies](01-a-service-that-builds-its-own-dependencies.md)
2. [Choosing between implementations](02-choosing-between-implementations.md)
3. [Who builds everything?](03-who-builds-everything.md)

## Part 2: Snapfit

4. [Why we built Snapfit](04-why-we-built-snapfit.md)
- [What this version supports](what-this-version-supports.md)

## Part 3: Wiring a container

5. [Installing Snapfit and your first singleton](05-installing-and-first-singleton.md)
6. [Pure domain types, one container](06-pure-domain-types.md)
7. [Wiring dependencies and custom constructors](07-wiring-dependencies-and-constructors.md)
8. [Read the container, know the graph](08-read-the-container-know-the-graph.md)
9. [Multiple instances of the same type](09-multiple-instances-of-the-same-type.md)
10. [Passing dependencies by name](10-passing-dependencies-by-name.md)
11. [Choosing an implementation](11-choosing-an-implementation.md)
13. [When building can fail](13-when-building-can-fail.md)

## Part 4: Compile-time guarantees and agents

14. [Mistakes caught by the compiler](14-mistakes-caught-by-the-compiler.md)
15. [Working with coding agents](15-working-with-coding-agents.md)

## Appendices

- Appendix 1: [Cheat sheet](appendix-1-cheat-sheet.md)
