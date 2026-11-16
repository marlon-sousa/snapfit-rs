# Wiring dependencies and custom constructors

## What you want to do

Not every struct uses a zero-argument `new()` or needs to be wrapped in `Arc`. In the shop:

- `TaxRateTable` is a small `Clone` struct that you want stored directly by value without `Arc`;
- `CatalogDb` is constructed via `CatalogDb::from_tax_table(tax)` instead of `new`;
- `ShippingClient` needs custom URL assembly via a helper method on the container (`build_shipping`).

## The code

Here are the three domain structs:

```example
name: ch05_wiring_and_constructors
region: domain
```

Here is `ShopContainer`, combining bare `Clone` storage, `constructor = "..."`, and `builder = "..."`:

```example
name: ch05_wiring_and_constructors
region: container
```

And here is `main` resolving all three singletons:

```example
name: ch05_wiring_and_constructors
region: run
```

It prints:

```output
name: ch05_wiring_and_constructors
```

## How it works

1. **No forced `Arc<T>` (`tax: TaxRateTable`):** Singletons only require `T: Clone`. Because `TaxRateTable` is declared without `Arc<...>`, `ShopContainer` stores `TaxRateTable` inline and `get_tax()` returns `self.tax.clone()` with zero heap indirection. This is also ideal for types that are already internally reference-counted, such as connection pools or HTTP clients.
2. **Named constructors (`constructor = "from_tax_table"`):** Setting `constructor = "from_tax_table"` on `db: Arc<CatalogDb>` tells `wireup` to call `CatalogDb::from_tax_table(tax.clone())` and wrap the returned `CatalogDb` in `Arc::new(...)`.
3. **Custom container builders (`builder = "build_shipping"`):** When construction requires custom formatting or unpacking fields, `builder = "build_shipping"` delegates to `ShopContainer::build_shipping(db.clone())`. Unlike `constructor`, a custom `builder` method returns the exact declared field type (`Arc<ShippingClient>`).

## Remember

1. Omit `Arc<...>` whenever a singleton type already implements `Clone` cheaply (such as small config structs, `PgPool`, or `reqwest::Client`).
2. Use `constructor = "<fn_name>"` when the target type's constructor has a custom name and returns `T`.
3. Use `builder = "<method_name>"` when you want to call an associated function on the container struct that returns the exact field type.
4. A field cannot specify both `constructor` and `builder` at the same time.
