# How to read this book

A few words on how the book works, so you know what to expect on every page, whether you read it with your eyes, with a screen reader, or as a coding agent.

## One story: the online shop

We don't invent unrelated `Foo` and `Bar` types on every page. Every example belongs to one small online shop that reserves stock, charges payments and reports its sales, and that grows from chapter to chapter. Each chapter's example is a complete Rust program in `book/examples/` that you can run on its own with `cargo run -p snapfit_book --example <name>`.

## How a chapter goes

The chapters of Part 1 tell a story. We write some code, run it, and look at what goes wrong. Each time we find a problem, we explain it, often with an example from everyday life, and give it a short name. Every chapter of Part 1 ends with **What we found**, a list of the problems it named, in bold, so you can find them again.

Every other chapter teaches one feature of `snapfit`. It shows the code, runs it, and explains what happened. It ends with two short sections:

1. **Which problem this solves:** the problems of Part 1 this feature answers, by their bold names, and how.
2. **Remember:** the few things worth keeping in mind, for you and for any agent working with you.

## Every listing is compiled and checked

No multi-line Rust program in this book is pasted into the text by hand:

- Code blocks come straight from `book/examples/<name>.rs`.
- Output blocks come from the recorded output in `book/expected/<name>.txt`.
- Compiler error blocks, as in [Chapter 11](11-choosing-an-implementation.md) and [Chapter 16](16-mistakes-caught-by-the-compiler.md), come from programs in `book/compile-errors/<name>.rs`, followed by the recorded compiler message in `book/compile-errors/<name>.txt`.

`cargo xtask book-check` compiles and runs every example and compares every recorded output and compiler message, and every pull request has to pass the same check. If a page shows it, it works.

## Diagrams and screen readers

Every diagram is drawn with Mermaid, has an accessible title and description, and is followed by a numbered list that says in words everything the diagram shows. You never need to see a box or an arrow to follow along.

## Where to go further

- The [Quick start](quick-start.md) runs a complete container on one page.
- The [specification (`docs/SPEC.md`)](https://github.com/marlon-sousa/snapfit-rs/blob/main/docs/SPEC.md) defines exactly what `snapfit` accepts, generates and refuses.
- The [architecture (`docs/ARCHITECTURE.md`)](https://github.com/marlon-sousa/snapfit-rs/blob/main/docs/ARCHITECTURE.md) explains how the code of `snapfit-macros` is organised.
