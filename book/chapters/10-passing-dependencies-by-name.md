# Passing dependencies by name

At the end of the last chapter, we swapped two names in a `deps` list and the shop quietly started writing to its read replica. Nothing was wrong with the types, only with the order. Let's give the container a way to hand over dependencies by name, so that order stops mattering at all.

Think of a moving day. If the movers unload the boxes in the order they came off the truck, and you have to remember that the third box is the kitchen, a mix-up is only a matter of time. Write "kitchen" on the box, and it doesn't matter when it comes off the truck: it ends up in the kitchen. That's what we want for our databases.

## The same databases and order store

We keep the databases and `CqrsOrderStore` from [Chapter 9](09-multiple-instances-of-the-same-type.md), unchanged:

```example
name: ch10_factories
region: domain
```

## A factory: we build the field ourselves

Here is a container that builds the order store with a factory. Look closely at the `deps` list of `order_store`: it names `write_db` first, which is exactly the order that went wrong in the last chapter:

```example
name: ch10_factories
region: container
```

`order_store` says `build = factory`. That tells the container: "I'll build this one myself." The container generates a trait called `ShopContainerFactories`, with one function for each field built this way, and we implement it:

```example
name: ch10_factories
region: factory
```

Let's run it:

```example
name: ch10_factories
region: run
```

It prints:

```output
name: ch10_factories
```

Reads go to the replica, even though the list started with `write_db`. Let's look at why.

### How a factory works

1. **The container writes the trait, and we fill it in.** For a field called `order_store` in `ShopContainer`, the trait `ShopContainerFactories` gets a function called `build_order_store`, which returns the field's type, `Arc<CqrsOrderStore>`. If we forget the `impl`, or give it the wrong type, the program doesn't compile.
2. **The inputs arrive in a struct.** `build_order_store` receives one value, of type `ShopContainerOrderStoreInputs`, with one public field for each name in `deps`. Its fields are named after the container's fields and have their types: `read_db` and `write_db`, both `Arc<Database>`.
3. **We take them out by name.** `let ShopContainerOrderStoreInputs { read_db, write_db } = inputs;` matches by name, not by position. The order of `deps` decides nothing here, and a misspelt name doesn't compile.
4. **The rest is ordinary Rust.** Inside the function we can call any constructor, in any way we like. There is no `self`, because the container doesn't exist yet while it is being built: everything the factory needs arrives in its inputs.

## A builder: every input arrives through a setter

A factory means writing a function in the container. When a type already has a builder, the container can hand over the inputs by name with no code of ours at all.

The shop also wants a daily sales report. It reads today's sales from the live replica, and checks the totals against the ledger on the primary:

```example
name: ch10_builders
region: sales_report
```

The two lines marked `#[bon::bon]` and `#[builder(start_fn = builder)]` come from [bon](https://docs.rs/bon), a popular crate that writes builders. Rust has no named arguments, and a builder is the usual way around that: instead of `SalesReport::new(a, b)`, you call `SalesReport::builder().live(a).ledger(b).build()`, and every argument arrives through a method named after it. bon writes that builder for us from the constructor, and `start_fn = builder` asks it to keep `SalesReport::new` public too, so the constructor can still be called directly. Notice that `SalesReport` still knows nothing about `snapfit`.

Here is a container for the report. Again, its `deps` list names `write_db` first:

```example
name: ch10_builders
region: container
```

`sales_report` says `build = builder`. This time the container writes no trait for us to fill in: it calls the report's own builder. Let's run it:

```example
name: ch10_builders
region: run
```

It prints:

```output
name: ch10_builders
```

The report reads from the replica too. Here is how.

### How a builder works

1. **`build = builder` calls the type's builder.** For `sales_report`, the container writes `SalesReport::builder()`, then one setter call for each name in `deps`, then `.build()`, and wraps the result in an `Arc`.
2. **A bare name is a setter of the same name.** Had the report's constructor taken `read_db` and `write_db`, `deps = [read_db, write_db]` would call `.read_db(...)` and `.write_db(...)`.
3. **A label renames the setter.** The report calls its inputs `live` and `ledger`, which say what they mean to a report. `ledger = write_db` means "call `.ledger(...)` with the `write_db` field". Labels are only accepted together with `build = builder`, because only a builder has setters to name.
4. **The compiler checks the names.** A setter that doesn't exist, because a name or a label is misspelt, is a compile error with any builder. Naming the same setter twice in one `deps` list is refused by `snapfit` itself, at the field. And with bon, forgetting a setter is a compile error too. That last check comes from bon, whose builders are written as a typestate: the builder's type changes with every setter called, so `build()` only exists once every required setter has been called.
5. **Other names for the ends.** If a builder starts with something other than `builder()`, or finishes with something other than `build()`, say so with `build = builder(start = make, finish = done)`. You can give either one, or both.

You don't need bon, or any builder crate. The container only calls a starting function, the setters and a finishing function, so a builder you write by hand works just as well, as long as its finishing function returns the value itself, or a `Result` when the field says it can fail, as [Chapter 13](13-when-building-can-fail.md) shows. We still recommend a crate such as bon, for the same reason you'd use any well-tested crate instead of rewriting it: it is less code to maintain. Keep in mind that the missing-setter check comes from typestate builders such as bon's. A builder written by hand only has it if you write it as a typestate too.

## Which one should I use?

Both take the order of `deps` out of the picture, in different ways. In a factory, every input is named where you pass it, so a swap would be written in plain sight, in a few lines of Rust you can read. With a builder, the compiler checks every setter's name. With bare names, the field and the setter are the same word, so a swap can't be written at all; with a label, each pairing is written in plain sight in the attribute, as in a factory. They also differ in where the code lives.

1. **Use a builder when the type already has one, or can have one.** The container needs no code at all, and the type keeps its own builder, which is useful anywhere else you build it, such as in a test.
2. **Use a factory when building the field is the container's business.** A factory works for any type, including types you don't own and can't add a builder to. It is also how a container field that is a trait object can be chosen while the program starts, as we'll see in [Chapter 11](11-choosing-an-implementation.md).
3. **Leave it alone when nothing can be swapped.** A constructor whose arguments all have different types, such as `OrderService::new(stock, payments)`, is already safe: passing them the wrong way round doesn't compile. Plain `deps` is shorter.

## Where the logic goes

There's a second question hidden in the first one: where should the code that builds something live? A useful rule is to ask who the logic belongs to.

1. **Logic that belongs to the type goes in the type.** The report deciding what its summary says is about what a sales report is. It lives in `SalesReport`'s constructor, which is also the body of the function bon turns into a builder. A unit test reaches it without any container, with `SalesReport::new(...)` or `SalesReport::builder()`, as [Chapter 6](06-pure-domain-types.md) showed.
2. **Logic that belongs to the wiring goes in a factory.** Choosing which implementation to use, or adapting one component into what another needs, is about how this application is put together, not about any one type. A factory is the place for it, next to the container that describes the rest of the wiring.

Think of a restaurant. How a dish is cooked belongs to the chef's recipe, whichever restaurant serves it. Which supplier the vegetables come from belongs to the restaurant's manager. Keep each decision where it belongs, and both are easy to change.

A factory builds one field from the inputs its `deps` list. When building it can fail, such as a connection that may be refused, the factory returns a `Result`, as we'll see in [Chapter 13](13-when-building-can-fail.md).

## Which problem this solves

1. **Arguments by position** ([Chapter 3](03-who-builds-everything.md)): a factory takes its inputs out of a struct by name, and a builder receives them through setters named after them, so the order of `deps` no longer decides who gets what.
2. **Mixed responsibilities** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): logic that belongs to a type stays in its constructor or builder, and logic that belongs to the wiring goes in a factory, next to the container.

## Remember

1. When a constructor takes two dependencies of the same type, pass them by name, with `build = builder`, where the compiler checks each name, or `build = factory`, where each name is written next to the argument it fills.
2. `build = factory` makes the container generate `<Container>Factories`, with a `build_<field>` function that receives a `<Container><Field>Inputs` struct and returns the field's type; destructure the inputs by name.
3. `build = builder` calls `builder()`, one setter for each name in `deps`, then `build()`; a label, as in `ledger = write_db`, renames a setter, and `build = builder(start = .., finish = ..)` renames the ends.
4. Any builder whose finishing function returns the value works, but a typestate builder, such as bon's, is what turns a forgotten setter into a compile error.
5. Logic about what a type is goes in the type; logic about how the application is wired goes in a factory.
