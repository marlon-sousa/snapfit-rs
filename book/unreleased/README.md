# Unreleased chapters

These chapters were written for `v0.1.0` but cover APIs that ship only behind snapfit's `unstable` feature, so they are not part of the `v0.1.0` edition of the book and are not built or checked by `cargo xtask book-check`:

1. `chapters/07-transient-dependencies.md`, with `examples/ch07_transients.rs` and `expected/ch07_transients.txt`.
2. `chapters/08-asynchronous-containers.md`, with `examples/ch08_async_container.rs` and `expected/ch08_async_container.txt`.

When transients and async containers are stabilised (see [What ships in 0.1.0](../../docs/SPEC.md#0-what-ships-in-010)):

1. move the chapter back into `book/chapters/` as Part 2 chapters and renumber the chapters after it;
2. move the example and expected output back into `book/examples/` and `book/expected/`;
3. link the chapters from `book/chapters/README.md` and `book/BOOK.md`;
4. update the cheat sheet and Appendix 2, and restore the Quick start's transient `checkout` if it still reads best that way;
5. run `cargo xtask book-check` and re-record any changed output.
