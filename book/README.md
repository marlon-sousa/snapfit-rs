# The Snapfit Book

The source of **The Snapfit Book**: its plan in [BOOK.md](BOOK.md), its layout and directive specification in [FORMAT.md](FORMAT.md), its chapters in [chapters/](chapters/README.md), and the runnable programs and compiler diagnostics they embed.

## Running and verifying the examples

From the repository root:

- `cargo xtask book-check` compiles and runs every program in `examples/`, compares what each prints against `expected/`, compiles every program in `compile-errors/` to verify its compiler message matches `compile-errors/*.txt`, and checks every directive in `chapters/`.
- `cargo run -p snapfit_book --example quick_start` runs a single example.
