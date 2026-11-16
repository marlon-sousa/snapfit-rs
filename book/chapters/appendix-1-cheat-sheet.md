# Cheat sheet

Every container setting and field attribute in `wireup` `0.1.0`, with a link to the chapter that teaches it.

## Container attributes (`#[container(...)]`)

| Declaration | What it does | Taught in |
| :--- | :--- | :--- |
| `#[container]` | Rewrites the struct (stripping `#[transient]` fields), topologically sorts dependencies, and generates `new()`, `Default`, `Container`, and `get_<name>()` methods. | [Chapter 4](04-installing-and-first-singleton.md) |
| `#[container(mode = "sync")]` | Forces synchronous mode (`pub fn new() -> Self`). Fails at compile time if any field is marked `async`. | [Chapter 9](09-mistakes-caught-by-the-compiler.md) |
| `#[container(mode = "async")]` | Enables asynchronous boot (`pub async fn init() -> Result<Self, Box<dyn Error + Send + Sync>>`) while keeping all `get_<name>()` getters synchronous. | [Chapter 8](08-asynchronous-containers.md) |
| `#[derive(Container)]` | Generates `new()` / `init()` and getters without rewriting the struct fields (supports `#[singleton]` fields only). | [Chapter 7](07-transient-dependencies.md) |

## Field attributes

| Declaration | Lifecycle & storage | Construction call | Taught in |
| :--- | :--- | :--- | :--- |
| `#[singleton]` | Stored in container; cloned on `get_<name>()`. | `<Type>::new()` (auto-wrapped in `Arc::new(...)` if field type is `Arc<T>`). | [Chapter 4](04-installing-and-first-singleton.md) |
| `#[singleton(deps = [a, b])]` | Stored in container; cloned on `get_<name>()`. | `<Type>::new(a.clone(), b.clone())`. | [Chapter 5](05-wiring-dependencies-and-constructors.md) |
| `#[singleton(deps = [a], constructor = "from_a")]` | Stored in container; cloned on `get_<name>()`. | `<Type>::from_a(a.clone())`. | [Chapter 5](05-wiring-dependencies-and-constructors.md) |
| `#[singleton(deps = [a], builder = "build_x")]` | Stored in container; cloned on `get_<name>()`. | `<Container>::build_x(a.clone())` (returns exact field type). | [Chapter 5](05-wiring-dependencies-and-constructors.md) |
| `#[singleton(deps = [a], async, constructor = "connect")]` | Stored in container; initialized during `init().await`; cloned synchronously on `get_<name>()`. | `<Type>::connect(a.clone()).await?`. | [Chapter 8](08-asynchronous-containers.md) |
| `#[transient]` | Stripped from container struct (`0` bytes stored); reconstructed on every `get_<name>()`. | `<Type>::new()`. | [Chapter 7](07-transient-dependencies.md) |
| `#[transient(deps = [a, b])]` | Stripped from container struct (`0` bytes stored); reconstructed on every `get_<name>()`. | `<Type>::new(self.get_a(), self.get_b())`. | [Chapter 7](07-transient-dependencies.md) |
