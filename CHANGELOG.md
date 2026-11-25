# Changelog

Every release of snapfit and snapfit-macros, which share one version. Each section is written in the release's pull request, from the changelog entries its commits carry as trailers, except 0.1.0's, written by hand for those meeting snapfit for the first time, as [docs/WORKFLOW.md](docs/WORKFLOW.md), "Releases", describes, and becomes the notes of its [GitHub release](https://github.com/marlon-sousa/snapfit-rs/releases). Versions follow [Semantic Versioning](https://semver.org/).

## [0.3.0]

Static dispatch: a field typed with a name the container declares holds one of a closed list of a trait's implementations, and calls it through a `match` instead of a vtable.

### Added

- `#[snapfit::static_dispatch]` on a trait lets a container dispatch it statically, through a `match` over a closed list of its implementations.
- `#[static_dispatch_for(Trait as Name)]` under `#[container]` declares the dispatched type `Name`, which holds the implementations that the fields typed `Name` or `Arc<Name>` name with `implementation =` or `implementations = [...]`; `extend_static_dispatch(Name, [...])` adds others, such as a test's mocks under `#[cfg_attr(test, ...)]`.
- `Name::of(value)` wraps an implementation into its variant of the dispatched type, so that a test can build any variant itself; a type that is not on the list is a compile error that says so.
- `implementations = [A, B]` beside `build = factory` on a field of a dispatched type lists the variants its factory returns.
- A field whose type keeps a trait object deeper in its type arguments, such as `Arc<Controller<Arc<dyn PaymentGateway>>>`, is built by its constructor, builder or `Default` instead of needing `build = factory`.
- The book's 0.3 edition teaches static dispatch in chapter 13, and how to choose between static and dynamic dispatch, with what a field of several types holds, in chapter 14; the chapters that followed are now chapters 15 to 17.

### Changed

- A field marked `#[dispatch(...)]` is refused with a message that names `static_dispatch_for`, instead of as a planned feature.
- An `implementation` on a field that keeps its trait object deeper in its type, such as `Arc<Controller<Arc<dyn PaymentGateway>>>`, is refused with words that ask for a field of its own for the trait object, instead of asking for `build = factory`.
- A type that keeps a trait object deeper but has no constructor snapfit can call, such as `Option<Box<dyn PaymentGateway>>`, is reported by rustc where the container builds it, instead of by snapfit asking for `build = factory`.
- `implementation = [A, B]` is refused with words that point to `implementations`, instead of as a token that was not expected.
- The settings a field's error offers for a misspelt setting include `implementations`.

### Fixed

- A public container no longer fails a crate that warns on or denies `missing_docs`: `new()`, `try_new()`, `init()` and every getter carry a doc comment, as a dispatched type's alias does.
- The doc of a raw container's factories trait names it without `r#`.
- A field whose `Arc` type a `macro_rules!` passes as `$t:ty`, such as `Arc<Mailer>` or `Arc<dyn Reader>` with an `implementation`, is built inside its `Arc`, as when the type is written by hand.

## [0.2.0]

Fields typed as their trait, and fields whose construction can fail.

### Added

- `implementation = <Type>` on an `Arc<dyn Trait>` field, such as `#[singleton(implementation = StripeGateway)] payments: Arc<dyn PaymentGateway>`, so that the field, its getter and every service that depends on it see the trait object, and a swap changes one word.
- `fallible = true` on a field whose constructor, builder or factory returns a `Result`; its container is built by `try_new()`, which returns a `snapfit::BuildError` naming the field that failed.
- `snapfit::BuildError`, which names the container and the field that failed and keeps the field's own error as its source.
- `snapfit::FallibleContainer`, the trait of a container whose `try_new()` can fail.
- The book's 0.2 edition teaches fields typed as their trait, testing a service on its own, fields whose construction can fail, and factories before builders, with a table of every way to build a field.

### Changed

- The error for an `Arc<dyn Trait>` field without a factory now also offers `implementation`.
- The README is a quick start, with a line for each feature that links to the chapter of the book teaching it.
- snapfit requires snapfit-macros of exactly its own version, because the code the macros write calls snapfit's hidden items of that version.

### Fixed

- A bad `build`, `leaf_build`, `mode` or `parent` is reported even when another setting of the same attribute is also wrong, instead of only once that one is fixed.

## [0.1.1]

### Added

- snapfit's page on docs.rs shows the README and documents the `container` macro and the `Container` trait.

### Changed

- snapfit's README on crates.io and docs.rs is written for users: it leaves out the badges, the unstable and planned APIs and how to contribute, apart from a pointer to the repository, and links to every release of the book on the site.

## [0.1.0]

The first release of snapfit on crates.io: compile-time verified dependency injection, with the container declaring the wiring and the domain types left as plain Rust. It holds everything 0.1.0-alpha.1 and 0.1.0-alpha.2 brought, described here as a whole for anyone who did not try them. It needs Rust 1.85.0 or newer, and is licensed under MIT or Apache 2.0.

### Added

- `#[container]` on a struct with named fields generates `new()`, `Default`, an impl of the `Container` trait and a `get_<field>()` getter for each dependency, and keeps the struct's own attributes, documentation and visibility.
- `#[singleton]` fields are built once, when the container is built, and cloned on every read; a field typed `Arc<T>` is built as `Arc::new(T::new(...))`.
- `deps = [a, b]` passes the named fields to the constructor, in order, so that several fields of one type can be wired to different dependents.
- One `build` setting chooses how a field is built: `build = constructor(connect)` calls `T::connect(...)`, `build = default` calls `Default::default()`, `build = factory` calls the `build_<field>` method you write in the generated `<Container>Factories` trait, which receives its dependencies by name, and `build = builder` passes each dependency to the setter of its name on the type's builder.
- `deps = [disk = folder_files]` passes a field to another setter of a builder, and `build = builder(start = draft, finish = publish)` names the functions a builder's chain starts and finishes with.
- `#[container(leaf_build = ...)]` builds every field without deps or a build of its own as `default` or `constructor(<fn>)`.
- Each field is built after the fields it needs, in the order the fields are declared wherever the graph allows.
- Attributes are read the same when a `macro_rules!` passes their values, and a raw identifier, such as `r#config`, names the same field as the plain word.
- Mistakes are compile errors, each a sentence pointing at its field or, when no field is at fault, at the container. For example: a field without `#[singleton]`, an unknown dependency, a dependency cycle, a trait-object field without a factory, a tuple struct or generic container, a second `#[container]`, a `#[cfg]` on a dependency, a misspelt attribute or setting, a `build` value snapfit does not know, a `constructor(...)` value that is not a plain function name, `build = default` with `deps`, a label without `build = builder`, two fields for one builder setter, and a bad or repeated `deps` entry.
- The problems found while reading the fields are reported together in one compile; once every field reads, the unknown dependencies, cycles and other rules of the declaration are reported together.
- The Snapfit Book, in `book/`, teaches the stable API with examples and compiler messages that CI checks; its website, with the README as its front page and one edition of the book per release, is first published with this release.
- Async containers and `#[transient]` dependencies ship behind the `unstable` feature, whose API may still change; the stable build refuses them with an error that names the feature. With it, `build = default`, `build = factory` and `build = builder` on an async field are refused.
- `#[dispatch(...)]`, child containers (`parent = ...`) and introspection (`introspection = true`) are planned, and are compile errors that say so.

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
