# Without Snapfit

In [Chapter 1](01-why-snapfit.md) we built the shop with manual constructor threading (`ch01_manual_wiring.rs`), and in [Chapter 3](03-read-the-container-know-the-graph.md) we built a five-component shop with `#[container]` (`ch03_container_graph.rs`).

## Answering three questions about the system

Suppose a new teammate — or a coding agent — joins the project and asks three questions:

1. **Which components are shared singletons, and which are created fresh per call?**
   - **Without Snapfit (`ch01_manual_wiring.rs`):** You must inspect both the fields of `ManualAppState` and every helper method (`build_order_service`) to see what is stored in `Arc` vs. constructed on the fly.
   - **With Snapfit (`ch03_container_graph.rs`):** Every field in `ShopContainer` states its lifecycle (`#[singleton]`) and its prerequisites (`deps = [...]`).
2. **What happens if I reorder fields or add a new dependency to `Database`?**
   - **Without Snapfit:** You must manually reorder the `let` bindings inside `ManualAppState::build()` and thread the new `Arc::clone` call through every constructor.
   - **With Snapfit:** You add the field name to `deps = [...]` on `db`. `snapfit` recomputes the initialization order automatically at compile time.
3. **Where do I look to understand the architecture of the whole service?**
   - **Without Snapfit:** Across `main.rs`, state structs, and factory methods.
   - **With Snapfit:** In the single `#[container]` struct declaration.
