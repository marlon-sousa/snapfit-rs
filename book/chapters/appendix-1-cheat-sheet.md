# Cheat sheet

Every container attribute and field attribute in `snapfit` `0.1.0`, with a link to the chapter that teaches it.

## Container attributes (`#[container(...)]`)

| Declaration | What it does | Taught in |
| :--- | :--- | :--- |
| `#[container]` | Keeps the struct as written (attributes, docs, visibility), builds each field after the fields it needs, and generates `new()`, `Default`, `Container`, and `get_<name>()` methods. | [Chapter 5](05-installing-and-first-singleton.md) |

## Field attributes

| Declaration | Storage | Construction call | Taught in |
| :--- | :--- | :--- | :--- |
| `#[singleton]` | Stored in container; cloned on `get_<name>()`. | `<Type>::new()` (auto-wrapped in `Arc::new(...)` if field type is `Arc<T>`). | [Chapter 5](05-installing-and-first-singleton.md) |
| `#[singleton(deps = [a, b])]` | Stored in container; cloned on `get_<name>()`. | `<Type>::new(a.clone(), b.clone())`. | [Chapter 7](07-wiring-dependencies-and-constructors.md) |
| `#[singleton(deps = [a], constructor = from_a)]` | Stored in container; cloned on `get_<name>()`. | `<Type>::from_a(a.clone())`. | [Chapter 7](07-wiring-dependencies-and-constructors.md) |

