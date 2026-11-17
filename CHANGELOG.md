# Changelog

Every release of snapfit and snapfit-macros, which share one version. Each section is written in the release's pull request, from the changelog entries its commits carry as trailers, as [docs/WORKFLOW.md](docs/WORKFLOW.md), "Releases", describes, and becomes the notes of its [GitHub release](https://github.com/marlon-sousa/snapfit-rs/releases). Versions follow [Semantic Versioning](https://semver.org/).

## [0.1.0-alpha.1]

The first pre-release of 0.1.0, released on GitHub only: compile-time verified dependency injection, with the container declaring the wiring and the domain types left as plain Rust.

### Added

- `#[container]` on a struct with named fields generates `new()`, `Default`, an impl of the `Container` trait and a `get_<field>()` getter for each dependency, and keeps the struct's own attributes, documentation and visibility.
- `#[singleton]` fields are built once, when the container is built, and cloned on every read; a field typed `Arc<T>` is built as `Arc::new(T::new(...))`.
- `deps = [a, b]` passes the named fields to the constructor, in order, so that several fields of one type can be wired to different dependents.
- `constructor = connect` calls `T::connect(...)` instead of `T::new(...)`.
- Each field is built after the fields it needs, in the order the fields are declared wherever the graph allows.
- An unknown dependency, a dependency cycle, a trait-object field, a generic container, a `#[cfg]` on a dependency and a misspelt attribute are compile errors, each a sentence pointing at its field, and every problem of a container is reported in one compile.
- The Snapfit Book website, at https://marlon-sousa.github.io/snapfit-rs/, with the README as its front page and one edition of the book per release.
- Async containers, `#[transient]` dependencies and `builder = <fn>` ship behind the `unstable` feature, whose API may still change; planned features, such as `#[dispatch(...)]`, child containers and introspection, are compile errors that name them.
