# Cheat sheet

Every stable container setting and field attribute in `snapfit` `0.1.0`, with a link to the chapter that teaches it. The unstable ones are listed at the end.

## Container attributes (`#[container(...)]`)

| Declaration | What it does | Taught in |
| :--- | :--- | :--- |
| `#[container]` | Keeps the struct as written (attributes, docs, visibility), topologically sorts dependencies, and generates `new()`, `Default`, `Container`, and `get_<name>()` methods. | [Chapter 4](04-installing-and-first-singleton.md) |
| `#[derive(Container)]` | Generates `new()` and getters as a derive, leaving the struct untouched (supports `#[singleton]` fields only). | Same as `#[container]` ([Chapter 4](04-installing-and-first-singleton.md)) |

## Field attributes

| Declaration | Lifecycle & storage | Construction call | Taught in |
| :--- | :--- | :--- | :--- |
| `#[singleton]` | Stored in container; cloned on `get_<name>()`. | `<Type>::new()` (auto-wrapped in `Arc::new(...)` if field type is `Arc<T>`). | [Chapter 4](04-installing-and-first-singleton.md) |
| `#[singleton(deps = [a, b])]` | Stored in container; cloned on `get_<name>()`. | `<Type>::new(a.clone(), b.clone())`. | [Chapter 5](05-wiring-dependencies-and-constructors.md) |
| `#[singleton(deps = [a], constructor = from_a)]` | Stored in container; cloned on `get_<name>()`. | `<Type>::from_a(a.clone())`. | [Chapter 5](05-wiring-dependencies-and-constructors.md) |

## Unstable attributes (`features = ["unstable"]`)

These work in `0.1.0` but their API may still change, so they need `snapfit = { version = "0.1", features = ["unstable"] }` and are not taught in this edition.

| Declaration | What it does |
| :--- | :--- |
| `#[container(mode = "async")]` | Generates `pub async fn init() -> Result<Self, Box<dyn Error + Send + Sync>>`; getters stay synchronous. |
| `#[singleton(deps = [a], async, constructor = connect)]` | Calls `<Type>::connect(a.clone()).await?` during `init()`. |
| `#[transient(deps = [a, b])]` | Not stored; built fresh on every `get_<name>()`. |
| `#[singleton(deps = [a], builder = build_x)]` | Calls `<Container>::build_x(a.clone())`, which returns the exact field type. |
