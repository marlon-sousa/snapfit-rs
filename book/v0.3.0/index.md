# Contents

**The Snapfit Book** for Rust (`0.3`), in reading order. It follows the plan in [BOOK.md](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/BOOK.md) and the directive layout in [FORMAT.md](https://github.com/marlon-sousa/snapfit-rs/blob/v0.3.0/book/FORMAT.md). Every program the chapters show is complete: those in [examples](https://github.com/marlon-sousa/snapfit-rs/tree/v0.3.0/book/examples/) run with `cargo run -p snapfit_book --example <name>` from the repository root.

## Welcome

- [Welcome to Snapfit](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/welcome-to-snapfit.md)
- [How to read this book](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/how-to-read-this-book.md)
- [Quick start](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/quick-start.md)

## Part 1: Why dependency injection

1. [A service that builds its own dependencies](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/01-a-service-that-builds-its-own-dependencies.md)
2. [Choosing between implementations](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/02-choosing-between-implementations.md)
3. [Who builds everything?](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/03-who-builds-everything.md)

## Part 2: Snapfit

4. [Why we built Snapfit](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/04-why-we-built-snapfit.md)

- [What this version supports](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/what-this-version-supports.md)

## Part 3: Wiring a container

5. [Installing Snapfit and your first singleton](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/05-installing-and-first-singleton.md)
6. [Pure domain types, one container](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/06-pure-domain-types.md)
7. [Wiring dependencies and custom constructors](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/07-wiring-dependencies-and-constructors.md)
8. [Read the container, know the graph](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/08-read-the-container-know-the-graph.md)
9. [Multiple instances of the same type](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/09-multiple-instances-of-the-same-type.md)
10. [Passing dependencies by name](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/10-passing-dependencies-by-name.md)
11. [Choosing an implementation](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/11-choosing-an-implementation.md)
12. [Testing a service on its own](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/12-testing-a-service-on-its-own.md)
13. [Static dispatch](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/13-static-dispatch.md)
14. [Choosing between static and dynamic dispatch](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/14-choosing-static-or-dynamic-dispatch.md)
15. [When building can fail](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/15-when-building-can-fail.md)

## Part 4: Compile-time guarantees and agents

16. [Mistakes caught by the compiler](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/16-mistakes-caught-by-the-compiler.md)
17. [Working with coding agents](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/17-working-with-coding-agents.md)

## Appendices

- Appendix 1: [Cheat sheet](https://marlon-sousa.github.io/snapfit-rs/book/v0.3.0/appendix-1-cheat-sheet.md)
