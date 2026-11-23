# Asynchronous containers

## What you want to do

The shop's `PaymentGateway` establishes its connection asynchronously (`pub async fn connect(...) -> Result<Self, ...>`). You want `ShopContainer::init().await` to resolve all asynchronous initialization at startup so that `container.get_orders()` remains 100% synchronous, non-blocking, and lock-free during request handling.

## The code

`PaymentGateway::connect` is an `async fn` returning a `Result`, while `ShopConfig::new` and `OrderService::new` are ordinary synchronous constructors:

```example
name: ch08_async_container
region: domain
```

Mark `ShopContainer` with `#[container(mode = "async")]` and mark the `gateway` field with `async`:

```example
name: ch08_async_container
region: container
```

Initialize the container once with `ShopContainer::init().await?`, then call `container.get_orders()` synchronously:

```example
name: ch08_async_container
region: run
```

It prints:

```output
name: ch08_async_container
```

## How it works

1. **Eager asynchronous boot (`init().await`):** Instead of `pub fn new() -> Self`, an asynchronous container generates `pub async fn init() -> Result<Self, Box<dyn Error + Send + Sync>>`. A field marked `fallible = true` has its error wrapped, after its `.await` when it is async, and that error reaches `init()`'s caller as the `snapfit::BuildError` of Chapter 13, boxed.
2. **Mixing synchronous and asynchronous nodes:** Inside `init()`, synchronous singletons (`config`) are constructed immediately, asynchronous singletons (`gateway`) are awaited and unwrapped with `?` before being wrapped in `Arc::new(...)`, and downstream synchronous singletons (`orders`) receive the resolved `Arc<PaymentGateway>`.
3. **Synchronous, lock-free getters:** Once `ShopContainer::init().await` completes, every singleton is already initialized and stored in the struct. Calling `container.get_orders()` or `container.get_gateway()` requires no `.await`, no `Mutex`, and no `OnceLock`.

## Remember

1. Add `async` inside `#[singleton(..., async, build = constructor(<fn>))]` when a singleton's constructor is an `async fn` returning `Result<T, E>`.
2. If any field in a container is marked `async`, `#[container]` automatically switches to asynchronous mode (unless `mode = "sync"` was explicitly forced, which fails compilation).
3. Runtime getters (`get_<name>()`) are always synchronous, even on an asynchronous container.
