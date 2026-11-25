# Static dispatch

So far, every trait our container handed out was a trait object, `Arc<dyn PaymentGateway>` or `Arc<dyn Database>`. That was the right call for gateways and databases: each call does real work, such as a network round trip, and the pointer a trait object follows on the way is lost in it. Now let's meet a part of the shop that is called far more often, and does far less each time: prices.

## A price on every line

Every item in a basket gets a price, and how it is priced depends on who is buying. A guest pays the list price, and a member gets a discount that the shop's configuration sets:

```example
name: ch13_static_dispatch
region: config
```

So the price is a trait with two rules, the list price and the member price. Ignore the attribute on the trait for a moment; we'll come to it:

```example
name: ch13_static_dispatch
region: trait
```

```example
name: ch13_static_dispatch
region: rules
```

A basket asks the rule for the price of every item, and adds them up:

```example
name: ch13_static_dispatch
region: basket
```

## A pointer on every call

We know how to hand `Basket` its rule: as an `Arc<dyn PriceRule>`, as [Chapter 11](11-choosing-an-implementation.md) did for payments. That works. But look at `total`. It calls `price` once for every item, of every basket, of every customer, and each of those calls goes through the trait object: follow the pointer to the rule's table of methods, find `price`, and call it. The compiler can't see which rule is behind the pointer, so it can't fold the rule's one line of arithmetic into the loop. For a payment that takes a network round trip, that detour is nothing. For a rule that subtracts a percentage, the detour can be a good part of the work.

Whether it matters for your shop is something only measuring can tell. But when it does, [Chapter 2](02-choosing-between-implementations.md) showed us a way out: an enum with one variant per rule, which implements the trait with a `match` the compiler can see through. And it showed us what that costs: **forwarding by hand**. Our trait has two methods and two rules, so that's four arms, each of which only passes the call along. A third rule adds an arm to every method, and a stand-in for tests adds another.

Writing those arms is exactly the kind of work a macro is good at.

## Letting snapfit write the enum

That's what the attribute on the trait was for. `#[snapfit::static_dispatch]` reads the trait's methods, and writes the forwarding for snapfit's own dispatch types, in the trait's module. It is the one `snapfit` attribute that sits outside the container, because the container can't see a trait's methods: only the trait knows them. The rules stay plain types with a plain `impl`.

The container does the rest:

```example
name: ch13_static_dispatch
region: container
```

`#[static_dispatch_for(PriceRule as Prices)]` declares a type, `Prices`, that stands for every price rule the container's fields name. Each field typed `Arc<Prices>` says which rule it holds with `implementation`, just as a trait-object field did in [Chapter 11](11-choosing-an-implementation.md), and builds it the same way: `ListPrice::new()` for `prices`, and `MemberPrice::from_config` with the configuration for `member_prices`. Ignore the `cfg_attr` line for now; it's for tests.

`Basket::new` takes an `Arc<Prices>`, and `total` calls `price` on it like on any other value that implements the trait. There is no `match` in the basket, and no type parameter either. Let's run the shop:

```example
name: ch13_static_dispatch
region: run
```

It prints:

```output
name: ch13_static_dispatch
```

The guest's basket and the member's basket are the same `Basket` type, holding the same `Prices` type, each built around its own rule.

## What snapfit wrote

1. **One type for the list.** `Prices` is a type alias that the container writes next to itself, with the container's visibility. It stands for one of snapfit's dispatch types, holding either a `ListPrice` or a `MemberPrice`. What that type looks like inside is snapfit's business: your code names `Prices`, and calls the trait on it.
2. **The list comes from the fields.** `Prices` holds the implementation of every field typed `Prices` or `Arc<Prices>`, in the order of the fields. A rule two fields name counts once. Later in this chapter we'll add rules that only a test or a factory builds.
3. **Every call is a `match`.** The trait's attribute implements `PriceRule` for the dispatch type: each method looks at which rule the value holds, and calls that rule directly. That is the enum of Chapter 2, and the compiler sees through it the same way.
4. **The field is built like any other.** The container builds the implementation, with its `new`, `build = constructor(...)`, `build = default` or `build = builder`, and its `deps`, wraps it in its variant, and then in an `Arc`, because the field says `Arc`.
5. **Mistakes don't compile.** A type in `implementation` that doesn't implement the trait is refused where the field names it, and a trait without `#[snapfit::static_dispatch]` is refused where `static_dispatch_for` names it.

Think of the speed-dial buttons on an old phone. Button 2 rings home, button 3 rings the office, and pressing one dials at once, without searching the address book. The buttons are set in advance, though: to add someone, you program another button. A trait object is the address book, which reaches anyone but looks them up every time. `Prices` is the row of buttons, and the container programs them from its fields.

## Without the `Arc`

Our fields say `Arc<Prices>`, so the rule is built once and every basket shares it. A field can also be typed `Prices` itself, without the `Arc`. It then holds the value, and every field that needs it receives a clone, as any field that isn't an `Arc` does since [Chapter 7](07-wiring-dependencies-and-constructors.md). Every rule on the list must then implement `Clone`, or the compiler says which one doesn't. Either way, `snapfit` adds no `Arc` your field doesn't declare.

## Testing with a rule no field names

A test of the basket would like a rule that answers whatever the test says, so that the expected total is easy to work out. Here is one, `FixedPrice`, and the test that uses it:

```example
name: ch13_static_dispatch
region: test
```

The test builds the basket directly, as [Chapter 6](06-pure-domain-types.md) taught us, without a container. `Prices::of(FixedPrice(100))` puts the stand-in in its variant, and the basket can't tell it from a real rule. The book's check runs this test with `cargo test`, so you won't see it in the output above.

`Prices::of` accepts only a type on the list, though, and no field of the container names `FixedPrice`. That's what the line we skipped is for:

1. **`extend_static_dispatch(Prices, [FixedPrice])`** adds `FixedPrice` to the list of `Prices`, beside the rules the fields name.
2. **`cfg_attr(test, ...)`** keeps that line only when the crate's own tests are built. Rust removes the attribute from every other build before the container sees it, so production holds only the two real rules, and `FixedPrice`, which is itself `#[cfg(test)]`, doesn't exist there at all.

A type that isn't on the list doesn't compile: `Prices::of(HalfPrice)` is refused, saying that `HalfPrice` is not one of the implementations this dispatched type holds. Several extensions can add to one type, each under its own condition, and they only ever add. Written without `cfg_attr`, an extension adds to every build, for a rule that only code outside the container builds. Integration tests under `tests/` build your crate without `cfg(test)`, just as [Chapter 12](12-testing-a-service-on-its-own.md) said of mocks, so they extend the list under a feature instead, such as `cfg_attr(feature = "mocks", ...)`.

One more detail: `FixedPrice` is `pub`. `Basket::new` is public and names `Prices`, so if a rule on the list were more private than `Basket::new`, rustc's `private_interfaces` lint would warn about `Basket::new`, and about every other public item of yours that names `Prices`, such as the methods of your impl of a factories trait. Making the stand-in as visible as the code that names `Prices` keeps the build quiet; you can also allow that lint on your own item.

## Choosing while the program starts

On sale days, the whole shop is cheaper, and the configuration says by how much:

```example
name: ch13_choosing_the_rule
region: config
```

```example
name: ch13_choosing_the_rule
region: sale
```

Now the container can't build the rule itself, because it doesn't know whether today is a sale day until the configuration has been read. That's the situation at the end of [Chapter 11](11-choosing-an-implementation.md), and the answer is the same: a factory. It returns the field's type, `Arc<Prices>`, and picks the variant:

```example
name: ch13_choosing_the_rule
region: container
```

The factory chooses, but `Prices` still needs to know which rules it may choose from, so that it can hold either. That's what `implementations`, with an s, says on a factory field: not the one rule the container builds, which is what `implementation` names, but the list of rules the factory may return, `implementations = [ListPrice, SalePrice]`. They join the list of `Prices` like the rule of any other field. It is always a list, even of one rule, and writing `implementation` on a factory field is refused, pointing you to `implementations`. If we left the list out, no field would name a rule, the list would be empty, and the container would be refused at `static_dispatch_for`:

```compile-error
name: ch13_prices_without_variants
```

An older compiler, such as Rust 1.85, may also report your impl of the factories trait, which the refused container doesn't write; adding the list makes those errors go too.

Let's run it, once with today's configuration, and once calling the factory on its own with a sale:

```example
name: ch13_choosing_the_rule
region: run
```

It prints:

```output
name: ch13_choosing_the_rule
```

The factory's `deps` are every input of every rule it might build, and the container builds all of them, even those today's rule doesn't use. Ours needs only the configuration. If one rule needed something costly to build, such as a connection, the factory could build it itself, and only when that rule is chosen.

## What a trait can hold

Because `#[snapfit::static_dispatch]` writes a `match` for every method, a method has to have a value to match on, and every rule has to agree on what the method means. So the attribute accepts methods that take `self`, `&self` or `&mut self`, with any arguments other than `Self`, generic methods included, default methods, methods that return `Self`, which come back wrapped in the variant they came from, and associated types that every rule gives the same type.

It refuses what it can't forward, each at the item it is about, and all of them in the same build. Among them are a method with no receiver, an associated constant, a receiver such as `self: Box<Self>`, `Self` in an argument or inside a return type, such as `Option<Self>`, a generic associated type, a return type that is `impl Trait`, a macro called among the trait's items, and an `unsafe trait`. Here is the first of those, a rule that wants a constructor in the trait:

```compile-error
name: ch13_rule_without_receiver
```

`PriceRule::standard()` has no value to look at, so there is no way to tell which rule should answer. The message offers the two ways out: give it a receiver, or move it out of the trait, to the rules' own `impl` blocks.

A supertrait needs the attribute too, when it is a trait of your own crate, because the dispatch type has to implement every trait the trait asks for. `Send`, `Sync`, `Sized`, `Clone` and `Debug` need nothing. A trait that has a supertrait of another crate, such as `Display`, can't be dispatched statically, and neither can a trait of another crate: Rust lets a crate implement another crate's trait only for its own types. Those stay with trait objects, when a trait object can hold them, and the next chapter looks at how to choose between the two.

## Which problem this solves

1. **Forwarding by hand** ([Chapter 2](02-choosing-between-implementations.md)): `#[snapfit::static_dispatch]` writes the `match` of every method for every implementation, and the container collects the implementations from its fields.
2. **A closed list** ([Chapter 2](02-choosing-between-implementations.md)): the list is still closed, which is what lets the compiler see every case, but it lives in the container alone. Adding a rule means a type and a field or a factory's list that names it, and no service changes.
3. **Spreading type parameters** ([Chapter 2](02-choosing-between-implementations.md)): `Basket` takes one type, `Arc<Prices>`, whatever rule it holds, and nothing above it needs a parameter.
4. **Hard to test** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): a test builds `Prices::of(FixedPrice(100))` itself, from a stand-in that an extension adds to the list in test builds only.

## Remember

1. Mark the trait with `#[snapfit::static_dispatch]`, once, where it is defined; the implementations stay plain types.
2. Declare the type under `#[container]` with `#[static_dispatch_for(Trait as Name)]`, and type the fields `Name` or `Arc<Name>`, each with its `implementation`, or with `build = factory`.
3. The list is the fields' implementations, then every `extend_static_dispatch(Name, [...])` in the build; put test stand-ins under `cfg_attr(test, ...)`.
4. `Name::of(value)` builds a variant, and accepts only a type on the list.
5. A factory returns `Name` or `Arc<Name>`, and lists the implementations it may return beside it: `build = factory, implementations = [A, B]`.
6. Static dispatch fits a closed list of implementations of your own trait whose calls are frequent and cheap; measure before you trade a trait object for it.
