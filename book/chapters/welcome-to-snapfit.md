# Welcome to Snapfit

Hello, and welcome to **The Snapfit Book**.

`snapfit` is a compile-time dependency injection library for Rust. You declare your application's components in one struct, and `snapfit` generates the code that builds them, in the right order, each once, and refuses a missing dependency or a cycle when the program compiles. Your services stay plain Rust: they never mention `snapfit`.

## Why start with the problem

A library that solves a problem you have not met yet is hard to judge. So the book does not start with `snapfit`. It starts with an ordinary shop whose services build their own dependencies, and follows it as it grows: each step fixes one problem and uncovers the next, until the remaining problems are the ones `snapfit` was built for. Every problem gets a short name, and the chapters that follow say which of them they answer.

If you already know dependency injection and want to see the code, the [Quick start](quick-start.md) shows a complete container on one page.

## How the book is laid out

- **[Part 1: Why dependency injection](01-a-service-that-builds-its-own-dependencies.md)** starts from services that build their own dependencies, shows the ways to let them work with more than one implementation, and ends with the one place that builds everything, which is what dependency injection means.
- **[Part 2: Snapfit](04-why-we-built-snapfit.md)** shows the same shop built by a `snapfit` container, how it answers each problem of Part 1, and what version `0.1.0` supports and does not.
- **[Part 3: Wiring a container](05-installing-and-first-singleton.md)** teaches the features of `0.1.0` one at a time: shared components, dependencies and constructors, construction order, several components of one type, and choosing an implementation.
- **[Part 4: Compile-time guarantees and agents](11-mistakes-caught-by-the-compiler.md)** shows every mistake `snapfit` refuses when compiling, with the compiler's exact message, and how the design helps when coding agents work on your code.
- **[Appendix](appendix-1-cheat-sheet.md)**: every attribute on one page.
