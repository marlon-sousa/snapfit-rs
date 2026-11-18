# Wiring dependencies and custom constructors

So far every component had a constructor called `new`, and every field was an `Arc`. Real code is less tidy. Let's meet four components that each break one of those habits, and see how the container handles them.

## Four components with their own habits

1. `TaxRateTable` is a small struct that is cheap to clone. Wrapping it in an `Arc` would only add a pointer to follow.
2. `DiscountCodes` has no constructor at all. It derives `Default`, and an empty list of codes is exactly what the shop starts with.
3. `CatalogDb` isn't built with `new`. Its constructor is called `from_tax_table`, because that is what it does.
4. `ShippingClient` builds its endpoint from the catalog database, in a constructor called `for_catalog`.

Here they are, still plain Rust, together with `GiftWrapOptions`, which we'll need in a moment:

```example
name: ch07_wiring_and_constructors
region: domain
```

## Telling the container how to build them

`deps` says what a component needs, and `build` says how to build it:

```example
name: ch07_wiring_and_constructors
region: container
```

Let's ask for all four:

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
2. **Without `build`, the container calls `new`.** `tax` says nothing about how it is built, so the container calls `TaxRateTable::new()`.
3. **`build = default` uses `Default`.** On `discounts`, the container calls `DiscountCodes::default()`. A value built this way receives nothing, so a field with `build = default` can't list `deps`.
4. **`build = constructor(...)` names the function to call.** On `db`, `build = constructor(from_tax_table)` tells the container to call `CatalogDb::from_tax_table` instead of `CatalogDb::new`, and to wrap the result in an `Arc`. You write the name bare, as Rust, not in quotes.
5. **The constructor receives what `deps` lists.** On `shipping`, the container calls `ShippingClient::for_catalog` with a clone of `db`. With several names in `deps`, the constructor receives them in that order.

Notice where the knowledge lives. `ShippingClient` knows how to make a shipping client out of a database, and nothing more. Which database it gets, and how that database was made, is written in one place: the container. It's like a relay race: each runner knows how to run their own leg and take the baton, and the team sheet says who runs after whom.

## A container full of defaults

Some containers hold many small settings structs, and most of them simply derive `Default`. Writing `build = default` on every one of them gets old quickly. Let's say it once, for the whole container:

```example
name: ch07_wiring_and_constructors
region: leaf_build
```

`leaf_build = default` changes how the container builds its leaves: the fields that need nothing and say nothing about how they are built. `discounts` and `gift_wrap` are built with `Default`. `tax` still wants its own constructor, which gives the shop its 10% rate, so it says `build = constructor(new)`, and a field's own `build` always wins. The last line of the output shows both.

Think of a hotel that makes every bed with the same white sheets unless the guest asked for something else when booking. The housekeeping rule is written once, and the guests with a wish write it on their own card. Without `leaf_build`, the container behaves as before, and calls `new` on every field that says nothing.

## Which problem this solves

1. **Mixed responsibilities** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): `ShippingClient::for_catalog` knows how to build a shipping client from a database, and nothing else. Which database it gets is decided once, in `ShopContainer`.
2. **Tight coupling** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): every constructor receives its dependencies as arguments, in the order `deps` lists them, and builds none of them itself.

## Remember

1. Leave out the `Arc` when a type is already cheap to clone, such as a small configuration struct, a connection pool or an HTTP client.
2. Without `build`, a field is built with `new`, which receives the fields `deps` lists, in that order.
3. Use `build = default` for a type that derives `Default`, and `#[container(leaf_build = default)]` when most leaves of a container do; a field's own `build` always wins.
4. Use `build = constructor(<fn_name>)` when a type's constructor has another name; it returns `T`, and you write the name without quotes.
5. Put construction logic in the domain type's own constructor, where a unit test can call it too.
