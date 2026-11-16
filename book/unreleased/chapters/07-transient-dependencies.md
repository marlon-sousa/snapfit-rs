# Transient dependencies

## What you want to do

Every time a customer checks out a cart, you want `container.get_checkout()` to construct a brand-new `CartCheckout` instance (with its own cart ID) wired with the shared `Arc<ShopConfig>` singleton, without wasting memory for `CartCheckout` inside `ShopContainer`.

## The code

`CartCheckout` assigns a fresh sequential `cart_id` each time `CartCheckout::new` is called:

```example
name: ch07_transients
region: domain
```

In `ShopContainer`, `config` is a `#[singleton]` and `checkout` is a `#[transient]`:

```example
name: ch07_transients
region: container
```

Calling `get_checkout()` twice produces two distinct `CartCheckout` values that share the same `ShopConfig` singleton, and `size_of::<ShopContainer>()` equals `size_of::<Arc<ShopConfig>>()`:

```example
name: ch07_transients
region: run
```

It prints:

```output
name: ch07_transients
```

## How it works

1. **Struct storage stripping:** The `#[container]` attribute macro rewrites `pub struct ShopContainer` so that only `#[singleton]` fields are kept in the struct definition. The `#[transient]` field `checkout` is completely removed from the struct's memory layout (`size_of::<ShopContainer>() == size_of::<Arc<ShopConfig>>()`).
2. **On-demand construction in the getter:** The generated `get_checkout(&self) -> CartCheckout` method resolves its prerequisites via their getters (`let config = self.get_config();`) and invokes `CartCheckout::new(config.clone())` on every call.
3. **No `Clone` requirement on transients:** Because `get_checkout()` constructs and returns a fresh owned `CartCheckout` each time, transient types do not need to implement `Clone`.

## Remember

1. Use `#[transient]` for stateless controllers, request handlers, or per-operation objects that should be freshly created on each `get_<name>()` call.
2. `#[transient]` fields require the `#[container]` attribute macro (not `#[derive(Container)]`), because only an attribute macro can strip fields from the struct definition.
3. Transient constructors must be synchronous because `get_<name>()` is always a synchronous method.
