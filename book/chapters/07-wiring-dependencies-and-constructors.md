# Wiring dependencies and custom constructors

## What you want to do

Not every struct uses a zero-argument `new()` or needs to be wrapped in `Arc`. In the shop:

- `TaxRateTable` is a small `Clone` struct that you want stored directly by value without `Arc`;
- `CatalogDb` is constructed via `CatalogDb::from_tax_table(tax)` instead of `new`;
- `ShippingClient` assembles its endpoint from the database via `ShippingClient::for_catalog(db)`.

## The code

Here are the three domain structs:

```example
name: ch07_wiring_and_constructors
region: domain
```

Here is `ShopContainer`, combining bare `Clone` storage and two named constructors:

```example
name: ch07_wiring_and_constructors
region: container
```

And here is `main` resolving all three singletons:

```example
name: ch07_wiring_and_constructors
region: run
```

It prints:

```output
name: ch07_wiring_and_constructors
```

## How it works

1. **No forced `Arc<T>` (`tax: TaxRateTable`):** Singletons only require `T: Clone`. Because `TaxRateTable` is declared without `Arc<...>`, `ShopContainer` stores `TaxRateTable` inline and `get_tax()` returns `self.tax.clone()` with zero heap indirection. This is also ideal for types that are already internally reference-counted, such as connection pools or HTTP clients.
2. **Named constructors (`constructor = from_tax_table`):** Setting `constructor = from_tax_table` on `db: Arc<CatalogDb>` tells `snapfit` to call `CatalogDb::from_tax_table(tax.clone())` and wrap the returned `CatalogDb` in `Arc::new(...)`.
3. **Constructors receive their prerequisites (`deps = [db], constructor = for_catalog`):** `snapfit` calls `ShippingClient::for_catalog(db.clone())`, passing the prerequisites in the order listed in `deps`, so the endpoint can be assembled from the database's DSN inside the domain type itself.

## Which problem this solves

1. **Two jobs in one place** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): `ShippingClient::for_catalog` knows how to build a shipping client from a database, and nothing else. Which database it gets, and how that database was built, is written once, in `ShopContainer`.
2. **The service picks its own dependencies** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): every constructor receives its dependencies as arguments, in the order `deps` lists them.

## Remember

1. Omit `Arc<...>` whenever a singleton type already implements `Clone` cheaply (such as small config structs, `PgPool`, or `reqwest::Client`).
2. Use `constructor = <fn_name>` when the target type's constructor has a custom name and returns `T`. Write the name bare, without quotes.
3. Put construction logic in the domain type's own named constructor, where a unit test can call it too.
