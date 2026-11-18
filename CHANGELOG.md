# Changelog

Every release of snapfit and snapfit-macros, which share one version. Each section is written in the release's pull request, from the changelog entries its commits carry as trailers, as [docs/WORKFLOW.md](docs/WORKFLOW.md), "Releases", describes, and becomes the notes of its [GitHub release](https://github.com/marlon-sousa/snapfit-rs/releases). Versions follow [Semantic Versioning](https://semver.org/).

## [0.1.0-alpha.2]

The second pre-release of 0.1.0, released on GitHub only: one `build` setting chooses how each field is built, with a constructor, `Default`, a factory you write or its type's builder.

### Added

- `build = default` builds a field with `Default::default()`, and `#[container(leaf_build = ...)]` builds every field without deps or a build of its own as `default` or `constructor(<fn>)`.
- `build = factory` builds a field with the `build_<field>` method you write in the generated `<Container>Factories` trait, which receives its dependencies by name.
- `build = builder` builds a field with its type's builder, passing each dependency to the setter of its name, and `deps = [disk = folder_files]` names another setter.
- `build = builder(start = draft, finish = publish)` names the functions a builder's chain starts and finishes with.
- A `build` value snapfit does not know is refused, suggesting each way of building a field.
- `build = default` is refused on an async field, which `leaf_build` skips (unstable).

### Changed

- A named constructor is written `build = constructor(connect)` instead of `constructor = connect`.
- A trait-object field is told to build it with a factory, on both builds.
- An unknown setting on a field or on `#[container]` lists the settings it takes.

### Fixed

- `deps` values passed by a `macro_rules!` as `$e:expr` are read as if written by hand, instead of being refused.
- Every bad entry of `deps = [...]` is reported in one compile, each in a sentence that says what to write instead.
- `deps(config)` is refused with a message saying to write `deps = [config]`.
- A repeated `deps` entry, such as `deps = [config, config]`, is refused, naming the field.
- An unknown dependency that a field's `deps` lists more than once is named once in its error.
- An error naming several unknown dependencies of a field says "unknown dependencies", in the plural.
- A raw identifier, such as `r#config`, names the same field as the plain word, in a field's name and in `deps`.

### Removed

- The unstable `builder = <fn>` setting, replaced by `build = factory`, which does not build an async field yet: `build = factory` and `build = builder` on an async field are refused as planned (unstable).

## [0.1.0-alpha.1]

The first pre-release of 0.1.0, released on GitHub only: compile-time verified dependency injection, with the container declaring the wiring and the domain types left as plain Rust.

### Added

- `#[container]` on a struct with named fields generates `new()`, `Default`, an impl of the `Container` trait and a `get_<field>()` getter for each dependency, and keeps the struct's own attributes, documentation and visibility.
- `#[singleton]` fields are built once, when the container is built, and cloned on every read; a field typed `Arc<T>` is built as `Arc::new(T::new(...))`.
- `deps = [a, b]` passes the named fields to the constructor, in order, so that several fields of one type can be wired to different dependents.
- `constructor = connect` calls `T::connect(...)` instead of `T::new(...)`.
- Each field is built after the fields it needs, in the order the fields are declared wherever the graph allows.
- An unknown dependency, a dependency cycle, a trait-object field, a generic container, a `#[cfg]` on a dependency and a misspelt attribute are compile errors, each a sentence pointing at its field, and every problem of a container is reported in one compile.
- The Snapfit Book, in `book/`, whose examples and compiler messages CI checks; its website, with the README as its front page and one edition of the book per release, is first published with 0.1.0.
- Async containers, `#[transient]` dependencies and `builder = <fn>` ship behind the `unstable` feature, whose API may still change; planned features, such as `#[dispatch(...)]`, child containers and introspection, are compile errors that name them.
