# Welcome to Snapfit

Hello, and welcome to **The Snapfit Book**.

`snapfit` is a compile-time dependency injection library for Rust. You declare your application's components in one struct, and `snapfit` writes the code that builds them, in the right order, each once, and refuses a missing dependency or a circle while the program compiles. Your services stay plain Rust: they never mention `snapfit`.

## Why we start with the problem

A tool that solves a problem you haven't met yet is hard to judge. So this book doesn't start with `snapfit`. It starts with an ordinary online shop, written the way most Rust programs start, and we grow it together. Every time the shop grows, something starts to hurt, and we stop to look at why, give the problem a name, and try a fix. By the end of Part 1, what is left is exactly what `snapfit` was built for, and every chapter after that tells you which of those problems it answers.

If you already know dependency injection and just want to see the code, the [Quick start](quick-start.md) shows a complete container on one page.

## How the book is laid out

- **[Part 1: Why dependency injection](01-a-service-that-builds-its-own-dependencies.md)** starts from services that build their own dependencies, tries a few ways of letting them work with more than one implementation, and ends with the one place that builds everything: the composition root, and what dependency injection means.
- **[Part 2: Snapfit](04-why-we-built-snapfit.md)** shows the same shop built by a `snapfit` container, how it answers each problem of Part 1, and what the `0.2` releases can and can't do.
- **[Part 3: Wiring a container](05-installing-and-first-singleton.md)** teaches the features of `0.2` one at a time: shared components, dependencies and constructors, construction order, several components of one type, passing dependencies by name with factories and builders, choosing an implementation, and components that can fail while they are built.
- **[Part 4: Compile-time guarantees and agents](13-mistakes-caught-by-the-compiler.md)** shows every mistake `snapfit` refuses while compiling, with the compiler's exact message, and how the design helps when coding agents work on your code.
- **[Appendix](appendix-1-cheat-sheet.md)**: every attribute on one page.
