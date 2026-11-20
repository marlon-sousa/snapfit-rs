# Cheat sheet

Every container attribute and field attribute in `snapfit` `0.2`, with a link to the chapter that teaches it.

## Container attributes (`#[container(...)]`)

| Declaration | What it does | Taught in |
| :--- | :--- | :--- |
| `#[container]` | Keeps the struct as written (attributes, docs, visibility), builds each field after the fields it needs, and generates `new()`, `Default`, `Container`, and `get_<name>()` methods; with a fallible field, `try_new()` and `FallibleContainer` instead of `new()`, `Default` and `Container`. | [Chapter 5](05-installing-and-first-singleton.md), [Chapter 12](12-when-building-can-fail.md) |
| `#[container(leaf_build = default)]` | Builds every field with no `deps` and no `build` of its own with `Default`; a field's own `build` always wins. Without it, such fields call `new()`. A fallible field can't be one of them: give it a `build` of its own. | [Chapter 7](07-wiring-dependencies-and-constructors.md), [Chapter 12](12-when-building-can-fail.md) |
| `#[container(leaf_build = constructor(<fn>))]` | Builds those same fields with `<Type>::<fn>()`; `constructor(new)` is what a container does without `leaf_build`. Unless `<fn>` is `new`, a fallible field can't be one of them. | [Chapter 7](07-wiring-dependencies-and-constructors.md) |

## Field attributes

| Declaration | Storage | Construction call | Taught in |
| :--- | :--- | :--- | :--- |
| `#[singleton]` | Stored in container; cloned on `get_<name>()`. | `<Type>::new()` (auto-wrapped in `Arc::new(...)` if field type is `Arc<T>`). | [Chapter 5](05-installing-and-first-singleton.md) |
| `#[singleton(deps = [a, b])]` | Stored in container; cloned on `get_<name>()`. | `<Type>::new(a.clone(), b.clone())`. | [Chapter 7](07-wiring-dependencies-and-constructors.md) |
| `#[singleton(build = default)]` | Stored in container; cloned on `get_<name>()`. | `<Type>::default()`; no `deps` allowed. | [Chapter 7](07-wiring-dependencies-and-constructors.md) |
| `#[singleton(deps = [a], build = constructor(from_a))]` | Stored in container; cloned on `get_<name>()`. | `<Type>::from_a(a.clone())`. | [Chapter 7](07-wiring-dependencies-and-constructors.md) |
| `#[singleton(deps = [a, b], build = factory)]` | Stored in container; cloned on `get_<name>()`. | `<Container>Factories::build_<name>(<Container><Name>Inputs { a: a.clone(), b: b.clone() })`, which you implement; the field may be a trait object. | [Chapter 10](10-passing-dependencies-by-name.md), [Chapter 11](11-choosing-an-implementation.md) |
| `#[singleton(deps = [a], implementation = StripeGateway)]` on `Arc<dyn Trait>` | Stored in container as declared; cloned on `get_<name>()`. | `Arc::new(<StripeGateway>::new(a.clone()))`, converted to the trait object; `build = constructor(...)`, `default` and `builder` work on `StripeGateway`. The getter and every dependent see `Arc<dyn Trait>`. | [Chapter 11](11-choosing-an-implementation.md) |
| `#[singleton(deps = [a, label = b], build = builder)]` | Stored in container; cloned on `get_<name>()`. | `<Type>::builder().a(a.clone()).label(b.clone()).build()`. | [Chapter 10](10-passing-dependencies-by-name.md) |
| `#[singleton(deps = [a], build = builder(start = make, finish = done))]` | Stored in container; cloned on `get_<name>()`. | `<Type>::make().a(a.clone()).done()`. | [Chapter 10](10-passing-dependencies-by-name.md) |
| `#[singleton(deps = [a], fallible = true, build = constructor(connect))]` | Stored in container; cloned on `get_<name>()`. | `<Type>::connect(a.clone())`, which returns a `Result`; its error becomes the `snapfit::BuildError` that `try_new()` returns. Works with `new`, a named constructor, a builder and a factory, whose `build_<name>` then returns `Result<FieldType, Box<dyn Error + Send + Sync>>`. | [Chapter 12](12-when-building-can-fail.md) |
