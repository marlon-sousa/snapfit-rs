# Cheat sheet

Every container attribute and field attribute in `snapfit` `0.1.0`, with a link to the chapter that teaches it.

## Container attributes (`#[container(...)]`)

| Declaration | What it does | Taught in |
| :--- | :--- | :--- |
| `#[container]` | Keeps the struct as written (attributes, docs, visibility), builds each field after the fields it needs, and generates `new()`, `Default`, `Container`, and `get_<name>()` methods. | [Chapter 5](05-installing-and-first-singleton.md) |
| `#[container(leaf_build = default)]` | Builds every field with no `deps` and no `build` of its own with `Default`; a field's own `build` always wins. Without it, such fields call `new()`. | [Chapter 7](07-wiring-dependencies-and-constructors.md) |
| `#[container(leaf_build = constructor(<fn>))]` | Builds those same fields with `<Type>::<fn>()`; `constructor(new)` is what a container does without `leaf_build`. | [Chapter 7](07-wiring-dependencies-and-constructors.md) |

## Field attributes

| Declaration | Storage | Construction call | Taught in |
| :--- | :--- | :--- | :--- |
| `#[singleton]` | Stored in container; cloned on `get_<name>()`. | `<Type>::new()` (auto-wrapped in `Arc::new(...)` if field type is `Arc<T>`). | [Chapter 5](05-installing-and-first-singleton.md) |
| `#[singleton(deps = [a, b])]` | Stored in container; cloned on `get_<name>()`. | `<Type>::new(a.clone(), b.clone())`. | [Chapter 7](07-wiring-dependencies-and-constructors.md) |
| `#[singleton(build = default)]` | Stored in container; cloned on `get_<name>()`. | `<Type>::default()`; no `deps` allowed. | [Chapter 7](07-wiring-dependencies-and-constructors.md) |
| `#[singleton(deps = [a], build = constructor(from_a))]` | Stored in container; cloned on `get_<name>()`. | `<Type>::from_a(a.clone())`. | [Chapter 7](07-wiring-dependencies-and-constructors.md) |
| `#[singleton(deps = [a, b], build = factory)]` | Stored in container; cloned on `get_<name>()`. | `<Container>Factories::build_<name>(<Container><Name>Inputs { a: a.clone(), b: b.clone() })`, which you implement; the field may be a trait object. | [Chapter 10](10-passing-dependencies-by-name.md), [Chapter 11](11-choosing-an-implementation.md) |
| `#[singleton(deps = [a, label = b], build = builder)]` | Stored in container; cloned on `get_<name>()`. | `<Type>::builder().a(a.clone()).label(b.clone()).build()`. | [Chapter 10](10-passing-dependencies-by-name.md) |
| `#[singleton(deps = [a], build = builder(start = make, finish = done))]` | Stored in container; cloned on `get_<name>()`. | `<Type>::make().a(a.clone()).done()`. | [Chapter 10](10-passing-dependencies-by-name.md) |

