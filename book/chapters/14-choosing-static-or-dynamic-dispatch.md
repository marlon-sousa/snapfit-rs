# Choosing between static and dynamic dispatch

We now have two ways to hand a service "something that implements this trait". A trait object, `Arc<dyn PaymentGateway>`, is called dynamic dispatch, and a type like `Prices` from [Chapter 13](13-static-dispatch.md) is called static dispatch. Both pick the implementation that answers a call while the program runs; what differs is how they find it. Neither is better in general, so let's first look at what each one holds, then go through the shop, decide for each part, and see what each choice costs.

## What a field of several types holds

Rust wants to know how big every value is before the program runs. That's how it lays out a struct, field after field, and how it knows how much room a local variable takes. Most of the time we never notice, because each field has one type, and each type has one size.

A field that may hold any of several types breaks that. `ListPrice` holds nothing at all, `MemberPrice` holds a percentage, and a rule we write next year might hold a whole table of discounts. If the field could be any of them, how much room should the struct keep for it? There are two ways out, and they are exactly our two kinds of dispatch.

1. **Keep the value somewhere else, and hold a pointer to it.** A pointer is always the same size, whatever it points at, so the field's size is settled. That's what `dyn` means: the value lives behind the pointer, and the compiler no longer knows which type it is. So beside the pointer, Rust keeps a second one, to a small table of the methods of the type behind it. A call follows that table to the right method. This is dynamic dispatch, and because the compiler never needs the list of types, it can hold any implementation, even one another crate writes years later.
2. **Keep room for the largest, and a tag saying which one is there.** This is an enum: it is as big as its biggest variant, plus a small tag. A call checks the tag in a `match`, and each arm calls its own implementation directly. That check happens while the program runs too, but it goes through no pointer and no table, and the compiler sees every arm, so it can fold a short method, such as our price arithmetic, right into the code that calls it. This is static dispatch, and what it costs is the list: the enum can hold only the types it was written with.

## One container, both kinds

Here are three parts of the shop. Payments we know well:

```example
name: ch14_static_or_dynamic
region: payments
```

Prices are the rules of the last chapter, with one method this time:

```example
name: ch14_static_or_dynamic
region: prices
```

And receipts are new. A receipt format writes one line for a label and an amount, and the label can be anything that can be displayed, a `&str`, a `String` or a number, so `line` is a generic method:

```example
name: ch14_static_or_dynamic
region: receipts
```

One container holds all three, and makes a different choice for each:

```example
name: ch14_static_or_dynamic
region: container
```

Let's run it:

```example
name: ch14_static_or_dynamic
region: run
```

It prints:

```output
name: ch14_static_or_dynamic
```

Here is why each part got the dispatch it got:

1. **Payments are dynamic.** A charge happens once per order, and takes a trip over the network that costs far more than any pointer. Providers come and go, the trait needs no attribute, and every service that takes `Arc<dyn PaymentGateway>` names only the trait. There is nothing to gain from a closed list.
2. **Prices are static.** A price is worked out for every item of every basket, in a loop, and the work is a line of arithmetic. The rules are known in advance and are our own. That is exactly where the `match` the compiler can see through pays off.
3. **Receipts are static, because they have to be.** A trait object can't hold a trait with a generic method: the table behind the pointer would need one entry for every type `L` could ever be. So rustc refuses `Arc<dyn ReceiptFormat>`, with error E0038, saying that `line` has generic type parameters. Static dispatch forwards a generic method like any other, because each arm of the `match` calls the implementation's own generic method. Both receipt fields hold the same `Receipts` type, one plain and one shouting.

Static and dynamic dispatch can even serve the same trait in one container: a field typed `Arc<dyn PriceRule>` with its `implementation` would sit happily beside `prices`. The field's type decides.

## Who names the dispatched type

Look at `Checkout`, which uses both prices and payments:

```example
name: ch14_static_or_dynamic
region: checkout
```

`payments` is a trait object, so `Checkout` names only the trait. For prices, though, we had a choice. The `Basket` of [Chapter 13](13-static-dispatch.md) took `Arc<Prices>`, and `Prices` is a type the container declares. That's fine while the services and the container live side by side. But [Chapter 6](06-pure-domain-types.md) suggested keeping domain types in a crate of their own, which the application's crate, the one with the container, depends on. Such a domain crate can't name `Prices` at all: it would have to depend on the crate that depends on it.

So `Checkout` is generic over the trait instead, `Checkout<P: PriceRule>`, and the container names the type once, in its field: `Arc<Checkout<Prices>>`. This is the type parameter of [Chapter 2](02-choosing-between-implementations.md), and it still spreads: a domain service that held the checkout would hold an `Arc<Checkout<P>>`, and be generic over `P` too. But it stops at the container, which fills it in once. A service in the container's own crate has the other choice, the one `Basket` made: take `Arc<Prices>`, and carry no parameter at all. `Checkout`'s constructor takes an `Arc<P>`, and the field's type tells Rust that `P` is `Prices`.

It makes testing simpler too. A generic checkout takes any single rule, so a test hands it a stand-in directly, with no `Prices`, no list and no extension:

```example
name: ch14_static_or_dynamic
region: test
```

## Side by side

| Question | Dynamic dispatch, `Arc<dyn Trait>` | Static dispatch, `Name` or `Arc<Name>` |
| :--- | :--- | :--- |
| Which implementations can it hold? | Any type that implements the trait, from any crate | The list the container collects from its fields, factory lists included, and its extensions |
| How does a call find its implementation? | While the program runs, through a pointer to the table of methods; the compiler rarely inlines it | While the program runs, through a `match` on the tag, with no pointer or table; the compiler sees every arm and can inline it |
| What does the trait need? | Nothing, but it must be one a trait object can hold: no generic methods, for example | `#[snapfit::static_dispatch]`, so it must be a trait of your own crate |
| Generic methods? | No | Yes |
| A supertrait of another crate, such as `Display`? | Yes | No |
| What is stored? | A pointer to the value, at its own size, and one to its table of methods | Room for the largest implementation and a tag saying which one it holds, behind the `Arc` or held by value |
| What do dependents name? | The trait | The container's type, or a type parameter the container fills in |
| How does a test hand in a stand-in? | Any type that implements the trait | A type on the list, through an extension, or directly to a generic dependent |

## A rule of thumb

1. **Start with a trait object.** It asks nothing of the trait, keeps the list open, and lets every service name only the trait. For most services, such as databases, gateways and clients, its cost is lost in the work each call does.
2. **Choose static dispatch for a closed list of your own trait whose calls are frequent and cheap,** such as rules, formats or strategies called inside loops, after measuring that the trait object's cost shows. Choose it too when the trait has a generic method, which no trait object can hold.
3. **Keep to a trait object for a trait of another crate,** and for an open set of implementations, such as plugins that other crates provide.

Some traits fit neither: one with an associated constant, for example, is refused by both. Its field then holds one implementation, named by the field's own type, as in [Chapter 11](11-choosing-an-implementation.md), or returned by a factory.

Picking a dispatch is like picking between a taxi rank and the shop's own delivery vans. The rank takes any car that turns up, and you wait a moment for each one. The vans are a fixed fleet you chose in advance, parked at the door and ready to go, but only those vans, and buying another means planning a space for it.

## Which problem this solves

1. **A closed list** ([Chapter 2](02-choosing-between-implementations.md)): static dispatch closes the list only where we choose it, in the container. Wherever the list must stay open, a trait object keeps it open.
2. **Spreading type parameters** ([Chapter 2](02-choosing-between-implementations.md)): a service in the container's crate takes `Arc<Prices>` and carries no parameter. A service in a domain crate is generic over the trait instead, and the parameter reaches only the domain types that hold it, then stops at the container, which names `Checkout<Prices>` once.
3. **Forwarding by hand** ([Chapter 2](02-choosing-between-implementations.md)): neither way asks for a hand-written `match`. A trait object needs none, and `#[snapfit::static_dispatch]` writes it.

## Remember

1. Both kinds pick the implementation while the program runs: a trait object through a pointer and a table of methods, a dispatched type through a `match` on its tag, whose every arm the compiler sees.
2. A field's type decides its dispatch: `Arc<dyn Trait>` is dynamic, and `Name` or `Arc<Name>`, for a name the container's `static_dispatch_for` declares, is static. One container can hold both.
3. Prefer a trait object until a closed list of cheap, frequent calls, or a generic method, asks for static dispatch.
4. A trait of another crate, or one with a supertrait of another crate such as `Display`, is dispatched dynamically.
5. A service that lives outside the container's crate is generic over the trait, and the container names the dispatched type in the field, such as `Arc<Checkout<Prices>>`.
