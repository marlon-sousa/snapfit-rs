# How to read this book

Every page in this book follows a small set of conventions so that both human readers (including screen-reader users) and coding agents can navigate straight to what they need.

## One story: the online shop

Instead of inventing unrelated `Foo` and `Bar` types on every page, every example belongs to one small online bookshop (`Northwind Books`) that loads configuration, connects to catalog and order databases, checks fraud rules, and processes customer checkouts. Each chapter's example is a complete, standalone Rust program in `book/examples/` that you can run on its own with `cargo run -p snapfit_book --example <name>`.

## The shape of a chapter

Every numbered chapter, and the [Quick start](quick-start.md), has the exact same four level 2 sections in order:

1. **What you want to do:** one or two sentences stating the goal in the shop's terms.
2. **The code:** the relevant slices of the chapter's compiled program, followed by the exact output it prints.
3. **How it works:** a numbered walkthrough explaining each mechanism in plain language.
4. **Remember:** a short checklist of invariants to keep in mind.

## Every listing is compiled and verified

No multi-line Rust program in this book is pasted by hand into the prose:

- Code blocks pull marked regions directly from `book/examples/<name>.rs`.
- Output blocks pull the recorded standard output from `book/expected/<name>.txt`.
- Compiler error blocks in [Chapter 7](07-mistakes-caught-by-the-compiler.md) pull invalid programs from `book/compile-errors/<name>.rs` and the recorded compiler diagnostics from `book/compile-errors/<name>.txt`.

Every pull request and release runs `cargo xtask book-check` to compile and execute every example and verify every recorded output and compiler message.

## Diagrams and screen-reader accessibility

Every diagram is drawn with Mermaid, includes an accessible title and description (`accTitle` and `accDescr`), and is immediately followed by a numbered list in the text that states everything the diagram shows. You never need to see a visual box or arrow to follow the explanation.

## Where to go further

- [Quick start](quick-start.md) runs a complete container of singletons on one page.
- [Specification (`docs/SPEC.md`)](https://github.com/marlon-sousa/snapfit-rs/blob/main/docs/SPEC.md) defines the formal syntax and semantics of `snapfit`.
- [Technical Architecture (`docs/TECHNICAL_SPEC.md`)](https://github.com/marlon-sousa/snapfit-rs/blob/main/docs/TECHNICAL_SPEC.md) explains the internal Hexagonal Architecture (Ports and Adapters) of `snapfit-macros`.
