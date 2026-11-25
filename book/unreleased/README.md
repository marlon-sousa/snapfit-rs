# Unreleased chapters

These chapters were written for `v0.1.0` but cover APIs that ship only behind snapfit's `unstable` feature, so they are not part of the `0.2` and `0.3` editions of the book and are not built or checked by `cargo xtask book-check`:

1. `chapters/07-transient-dependencies.md`, with `examples/ch07_transients.rs` and `expected/ch07_transients.txt`.
2. `chapters/08-asynchronous-containers.md`, with `examples/ch08_async_container.rs` and `expected/ch08_async_container.txt`.

When transients and async containers are stabilised (see [What ships in 0.1.0](../../docs/SPEC.md#0-what-ships-in-010)):

1. move the chapter back into `book/chapters/` as Part 3 chapters and renumber the chapters after it;
2. move the example and expected output back into `book/examples/` and `book/expected/`;
3. link the chapters from `book/chapters/README.md` and `book/BOOK.md`;
4. update the cheat sheet and "What this version supports", and restore the Quick start's transient `checkout` if it still reads best that way;
5. rewrite each chapter in the book's teaching voice, as `book/BOOK.md` describes, closing with "Which problem this solves" and "Remember";
6. run `cargo xtask book-check` and re-record any changed output.
