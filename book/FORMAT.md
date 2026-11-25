# How the book is laid out

Where each part of the book lives in `book/`, and how a chapter embeds code that is compiled and run. [BOOK.md](BOOK.md) defines what the book contains; this document defines how it is stored and verified.

## Chapters and files

1. Each chapter is one Markdown file under `book/chapters/`, named after its two-digit number and slug, for example `book/chapters/04-why-we-built-snapfit.md`. The `book/chapters/README.md` beside it lists the parts and chapters in reading order.
2. An unnumbered page inside a part is a file named after its title without a number, for example `what-this-version-supports.md`.
3. Each appendix is a file named `appendix-N-short-name.md`.
4. Each welcome page before Part 1 is a file named after its title without a number, for example `quick-start.md`. The contents list them unnumbered under the heading "Welcome".
5. Every numbered chapter, and the quick start, opens with its title as the only level 1 heading (`#`), and chooses its own level 2 (`##`) and level 3 (`###`) headings for the story it tells. It ends with the closing sections of [BOOK.md](BOOK.md), "The shape of a chapter", as level 2 headings, in order: "What we found" for a problem chapter of Part 1; "Which problem this solves" and then "Remember" for every other chapter. A closing heading is used only at the end.
6. An unnumbered page, an appendix or a welcome page other than the quick start opens with its own title, and chooses its own level 2 and level 3 headings.
7. Every name a chapter gives to an example or a compile error starts with the chapter's number (`ch05_first_singleton`), or is `quick_start` for the quick start, so names never clash between chapters.
8. Any page may show a region of another chapter's example by naming it, so the same code is never written twice.
9. A chapter never holds a multi-line Rust program in its prose. Code that compiles, what it prints, and what the compiler refuses are named by directives, below, so every listing in the book is compiled and checked in CI. Only a short fragment that is not a program, such as one attribute (`#[singleton]`) or a type's name (`Arc<Config>`), may be written inline in prose.

## Directives

A directive is a fenced code block whose info string is the directive's kind (`example`, `output`, or `compile-error`), holding `key: value` lines. It names a file inside `book/`, and the site generator replaces it with that file's content. On GitHub a directive displays as a small block naming what will appear there.

### Example

````markdown
```example
name: ch05_first_singleton
region: container
```
````

Shows the lines of `book/examples/ch05_first_singleton.rs` between the comment lines `// ANCHOR: container` and `// ANCHOR_END: container`, with every anchor comment line removed and indentation normalized. Without `region`, shows the whole example with anchor lines removed. Every example is a complete runnable program; the region is only the slice the chapter highlights at that point.

### Output

````markdown
```output
name: ch05_first_singleton
```
````

Shows what `book/examples/ch05_first_singleton.rs` prints to standard output, as recorded in `book/expected/ch05_first_singleton.txt`.

### Compile error

````markdown
```compile-error
name: ch16_dependency_cycle
```
````

Shows a program that the Rust compiler refuses to compile (`book/compile-errors/ch16_dependency_cycle.rs`), followed by the compiler's diagnostic message as recorded in `book/compile-errors/ch16_dependency_cycle.txt`.

## Verification directories

Inside `book/`:

1. `chapters/`: the Markdown files of the book, ordered by `chapters/README.md`.
2. `examples/<name>.rs`: complete runnable Rust programs, compiled and executed by `cargo xtask book-check`.
3. `expected/<name>.txt`: the exact standard output each example prints, compared byte-for-byte on every check.
4. `compile-errors/<name>.rs` and `compile-errors/<name>.txt`: programs that must fail compilation, paired with their recorded compiler error output. Paths inside recorded compiler output are normalized to `book/compile-errors/<name>.rs` so diagnostics are identical across machines and operating systems.

Every example prints deterministic output on every run: timestamps, random values, or raw memory addresses are never printed directly.
