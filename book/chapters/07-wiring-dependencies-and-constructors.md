# Wiring dependencies and custom constructors

So far every component had a constructor called `new`, and every field was an `Arc`. Real code is less tidy. Let's meet three components that each break one of those habits, and see how the container handles them.

## Three components with their own habits

1. `TaxRateTable` is a small struct that is cheap to clone. Wrapping it in an `Arc` would only add a pointer to follow.
2. `CatalogDb` isn't built with `new`. Its constructor is called `from_tax_table`, because that is what it does.
3. `ShippingClient` builds its endpoint from the catalog database, in a constructor called `for_catalog`.

Here they are, still plain Rust:

```example
name: ch07_wiring_and_constructors
region: domain
```

## Telling the container how to build them

`deps` says what a component needs, and `build` says which function builds it:

```example
name: ch07_wiring_and_constructors
region: container
```

Let's ask for all three:

```example
name: ch07_wiring_and_constructors
region: run
```

It prints:

```output
name: ch07_wiring_and_constructors
```

## How the container reads this

1. **A field doesn't have to be an `Arc`.** A singleton only has to implement `Clone`. `tax` is declared as a plain `TaxRateTable`, so the container stores it as it is, and `get_tax()` returns a clone of it. This is also the right choice for types that already share their insides, such as a connection pool or an HTTP client: wrapping those in an `Arc` would add a second layer of sharing for nothing.
2. **`build = constructor(...)` names the function to call.** On `db`, `build = constructor(from_tax_table)` tells the container to call `CatalogDb::from_tax_table` instead of `CatalogDb::new`, and to wrap the result in an `Arc`. You write the name bare, as Rust, not in quotes.
3. **The constructor receives what `deps` lists.** On `shipping`, the container calls `ShippingClient::for_catalog` with a clone of `db`. With several names in `deps`, the constructor receives them in that order.

Notice where the knowledge lives. `ShippingClient` knows how to make a shipping client out of a database, and nothing more. Which database it gets, and how that database was made, is written in one place: the container. It's like a relay race: each runner knows how to run their own leg and take the baton, and the team sheet says who runs after whom.

## Which problem this solves

1. **Mixed responsibilities** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): `ShippingClient::for_catalog` knows how to build a shipping client from a database, and nothing else. Which database it gets is decided once, in `ShopContainer`.
2. **Tight coupling** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): every constructor receives its dependencies as arguments, in the order `deps` lists them, and builds none of them itself.

## Remember

1. Leave out the `Arc` when a type is already cheap to clone, such as a small configuration struct, a connection pool or an HTTP client.
2. Use `build = constructor(<fn_name>)` when a type's constructor has another name; it returns `T`, and you write the name without quotes.
3. Put construction logic in the domain type's own constructor, where a unit test can call it too.
