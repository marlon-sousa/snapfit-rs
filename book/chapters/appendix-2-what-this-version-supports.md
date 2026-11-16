# What this version supports

This edition of **The Wireup Book** covers `wireup` **`0.1.0`** (Minimum Supported Rust Version: **`1.85.0`**, Rust 2024 Edition).

## Capabilities in `0.1.0`

| Capability | Status in `0.1.0` | Details |
| :--- | :--- | :--- |
| **Pure-Rust domain types** | Supported | Zero macros or traits required on services, repositories, or controllers ([Chapter 2](02-pure-domain-types.md)). |
| **Compile-time DAG topological sort** | Supported | Kahn's algorithm orders initialization automatically regardless of field declaration order ([Chapter 3](03-read-the-container-know-the-graph.md)). |
| **Compile-time cycle and missing-prerequisite rejection** | Supported | Self-cycles, multi-node cycles, and unknown dependency names fail compilation with actionable diagnostics ([Chapter 9](09-mistakes-caught-by-the-compiler.md)). |
| **Singletons (`#[singleton]`)** | Supported | Supports both `Arc<T>` (auto-wrapped) and any `T: Clone` without forced `Arc` ([Chapter 4](04-installing-and-first-singleton.md), [Chapter 5](05-wiring-dependencies-and-constructors.md)). |
| **Custom constructors & container builders** | Supported | `constructor = "..."` and `builder = "..."` ([Chapter 5](05-wiring-dependencies-and-constructors.md)). |
| **Multiple instances of the same type** | Supported | Wired by container field name (`read_db`, `write_db`) without wrapper newtypes ([Chapter 6](06-multiple-instances-of-the-same-type.md)). |
| **Zero-byte transients (`#[transient]`)** | Supported | Stripped from container struct storage and freshly constructed on every `get_<name>()` call ([Chapter 7](07-transient-dependencies.md)). |
| **Asynchronous containers (`mode = "async"`)** | Supported | Eager asynchronous initialization via `init().await` with synchronous, lock-free runtime getters ([Chapter 8](08-asynchronous-containers.md)). |

## Planned for later releases

The following capabilities are specified in [docs/SPEC.md](https://github.com/marlon-sousa/wireup-rs/blob/main/docs/SPEC.md) and scheduled for upcoming releases:

1. **Auto-generated `enum_dispatch` static trait polymorphism (`#[dispatch(ImplA, ImplB)]`):** Rewrites `Arc<dyn Trait>` fields into static dispatch enums with zero vtable indirection.
2. **Auto-generated `<ContainerName>Builders` companion trait:** Generates a trait with exact method signatures for all fields declaring `builder = "..."` to enable IDE auto-scaffolding.
3. **Fork-and-rewire testing overrides (`fork_with()`):** Allows tests to override specific nodes in a container graph while automatically re-instantiating downstream dependents.
4. **Request-scoped child containers (`#[container(parent = ...)]`):** Lightweight child scopes that inherit parent singletons and combine them with request-local data.
5. **Container graph introspection (`introspection = true`):** Emits `.dependency_graph()` returning `GraphDescriptor` for runtime inspection and visualization.
