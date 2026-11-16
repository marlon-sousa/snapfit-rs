# How the book is laid out

Where each part of the book lives in `book/`, and how a chapter embeds code that is compiled and run. [BOOK.md](BOOK.md) defines what the book contains; this document defines how it is stored and verified.

## Chapters and files

1. Each chapter is one Markdown file under `book/chapters/`, named after its two-digit number and slug, for example `book/chapters/01-why-wireup.md`. The `book/chapters/README.md` beside it lists the parts and chapters in reading order.
2. Each "Without Wireup" section is a file of its own, named after the part it closes, for example `part-1-without-wireup.md`.
3. Each appendix is a file named `appendix-N-short-name.md`.
4. Each welcome page before Part 1 is a file named after its title without a number, for example `quick-start.md`. The contents list them unnumbered under the heading "Welcome".
5. Every numbered chapter, and the quick start, opens with its title as the only level 1 heading (`#`), followed by the four sections of [BOOK.md](BOOK.md) as level 2 headings (`##`), in order: "What you want to do", "The code", "How it works", and "Remember". It may use level 3 headings (`###`) inside them.
6. A "Without Wireup" section opens with the level 1 heading `# Without Wireup`, and an appendix or other welcome page opens with its own title; they choose their own level 2 and level 3 headings.
7. Every name a chapter gives to an example or a compile error starts with the chapter's number (`ch04_first_singleton`), with the part's for a "Without Wireup" section (`part2_without_wireup`), or is `quick_start` for the quick start, so names never clash between chapters.
8. Any chapter, "Without Wireup" section, or appendix may show a region of another chapter's example by naming it, so the same code is never written twice.
9. A chapter never holds a multi-line Rust program in its prose. Code that compiles, what it prints, and what the compiler refuses are named by directives, below, so every listing in the book is compiled and checked in CI. Only a short fragment that is not a program, such as one attribute (`#[singleton]`) or a type's name (`Arc<Config>`), may be written inline in prose.

## Directives

A directive is a fenced code block whose info string is the directive's kind (`example`, `output`, or `compile-error`), holding `key: value` lines. It names a file inside `book/`, and the site generator replaces it with that file's content. On GitHub a directive displays as a small block naming what will appear there.

### Example

````markdown
```example
name: ch04_first_singleton
region: container
```
````

Shows the lines of `book/examples/ch04_first_singleton.rs` between the comment lines `// ANCHOR: container` and `// ANCHOR_END: container`, with every anchor comment line removed and indentation normalized. Without `region`, shows the whole example with anchor lines removed. Every example is a complete runnable program; the region is only the slice the chapter highlights at that point.

### Output

````markdown
```output
name: ch04_first_singleton
```
````

Shows what `book/examples/ch04_first_singleton.rs` prints to standard output, as recorded in `book/expected/ch04_first_singleton.txt`.

### Compile error

````markdown
```compile-error
name: ch09_dependency_cycle
```
````

Shows a program that the Rust compiler refuses to compile (`book/compile-errors/ch09_dependency_cycle.rs`), followed by the compiler's diagnostic message as recorded in `book/compile-errors/ch09_dependency_cycle.txt`.

## Verification directories

Inside `book/`:

1. `chapters/`: the Markdown files of the book, ordered by `chapters/README.md`.
2. `examples/<name>.rs`: complete runnable Rust programs, compiled and executed by `cargo xtask book-check`.
3. `expected/<name>.txt`: the exact standard output each example prints, compared byte-for-byte on every check.
4. `compile-errors/<name>.rs` and `compile-errors/<name>.txt`: programs that must fail compilation, paired with their recorded compiler error output. Paths inside recorded compiler output are normalized to `book/compile-errors/<name>.rs` so diagnostics are identical across machines and operating systems.

Every example prints deterministic output on every run: timestamps, random values, or raw memory addresses are never printed directly.
