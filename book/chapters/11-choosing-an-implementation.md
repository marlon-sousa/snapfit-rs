# Choosing an implementation

Back in [Chapter 2](02-choosing-between-implementations.md) we wanted `OrderService` to charge cards through Stripe in production and through a fake in tests, and we found that somebody still had to make that choice. Now we have a container, so let's let it choose, in exactly one place.

## Two gateways and two services

The gateways are the ones from Chapter 2, sharing a trait:

```example
name: ch11_choosing_an_implementation
region: gateways
```

The services take the trait in the two ways Chapter 2 showed. `OrderService` takes a trait object, `Arc<dyn PaymentGateway>`. `RefundService` takes a type parameter:

```example
name: ch11_choosing_an_implementation
region: services
```

## The container makes the choice

Each container makes the choice once, in the type of its `payments` field. `ShopContainer` is the production shop. `TestShopContainer` builds the same services around the fake:

```example
name: ch11_choosing_an_implementation
region: containers
```

Let's run both:

```example
name: ch11_choosing_an_implementation
region: run
```

It prints:

```output
name: ch11_choosing_an_implementation
```

Put the two containers side by side. Apart from their names, they differ only in the gateway's type. That's the ride app from Chapter 2: the services asked for something that charges cards, and the container decided which car to send.

## What happens in each container

1. **The field's type is the choice.** `payments: Arc<StripeGateway>` tells the container to call `StripeGateway::new()`. The container is the only place that says Stripe.
2. **A service that takes a trait object.** `OrderService::new` takes an `Arc<dyn PaymentGateway>`. The generated `new()` passes it a clone of the `Arc<StripeGateway>` field, and Rust converts it, as it would in any function call. That's why `orders` has the same type, `Arc<OrderService>`, in both containers.
3. **A service that takes a type parameter.** `RefundService<P>` works with any gateway, so its field names the concrete one: `Arc<RefundService<StripeGateway>>`. The type parameter shows up in the container, where the choice is made, and nowhere else.
4. **A container for tests.** `TestShopContainer` declares the same fields with `FakeGateway`. A test builds it with `TestShopContainer::new()` and gets the real services around the fake. When a test is about one service only, building that service directly with a fake is often simpler still ([Chapter 6](06-pure-domain-types.md)).

## Swapping a component

Now imagine the shop moves from Stripe to another provider. You write a type that implements `PaymentGateway`, change `payments: Arc<StripeGateway>` to that type, and recompile. For services that take a trait object, that's the whole change. A service with a type parameter names the gateway in its own field too, as `Arc<RefundService<StripeGateway>>` does, so that's one more line, still inside the container.

That isn't a special trick for payment gateways. It works for any field, and `TestShopContainer` shows why. Remember that the generated `new()` is ordinary Rust. Let's follow what it does with `FakeGateway`, and see what the new type had to offer:

1. **It has the constructor.** For `payments: Arc<FakeGateway>`, the container calls `FakeGateway::new()` and wraps the result in an `Arc`. So the new type needs to be built the way the attribute says: with `new` here, or with the constructor `build = constructor(...)` names, taking the fields in `deps`, in that order. For an `Arc<T>` field, that constructor lives on `T`.
2. **It can be cloned.** The getter and every field that needs `payments` receive a clone of it. An `Arc` can always be cloned; a field of any other type must implement `Clone`.
3. **The fields that need it accept it.** `orders` and `refunds` receive a clone of `payments`. `OrderService::new` takes an `Arc<dyn PaymentGateway>`, and `FakeGateway` implements `PaymentGateway`, so Rust converts it. `RefundService<P>` is generic over the gateway, so it accepts it too, and its field names the new type: `Arc<RefundService<FakeGateway>>`. A constructor that asked for exactly `Arc<FakeGateway>` would accept it as well.

That's the whole rule: you can change a field's type to any type that has the same constructor, can be cloned, and is accepted by the fields that need it. No service changes, and no attribute does. The only other line that changes is the type of a field that is generic over the new type, like `refunds`. If the new type doesn't fit, the program doesn't compile, so you find out right away.

## What a service keeps from the gateway

That other line comes from `RefundService<P>`, which names the gateway in its own type. Not every service that works with any gateway has to. What decides it is what the service keeps from the gateway once it is built, and there are three answers.

1. **It keeps the gateway, and names it in its own type.** That's `RefundService<P>` from the start of the chapter, which stores `Arc<P>`. Each call goes straight to the concrete gateway's code. But the gateway's type is part of the service's type, so the container names it too: `Arc<RefundService<StripeGateway>>`.
2. **It keeps the gateway, without naming it in its own type.** Then it can hold the gateway only as a trait object. That's `OrderService`, which stores `Arc<dyn PaymentGateway>`. Each call goes through the trait object, and its field is `Arc<OrderService>` in every container.
3. **It keeps only a value it read from the gateway.** Say the gateway also tells us the provider's name:

   ```example
   name: ch11_generic_services
   region: trait
   ```

   A receipt printer needs that name, not the gateway. Its `new` is generic, but the struct isn't, so its type names no gateway, and its field is `Arc<ReceiptPrinter>` in every container. Each call costs nothing extra, because there is no gateway left to call:

   ```example
   name: ch11_generic_services
   region: receipts
   ```

   The `?Sized` in `P: PaymentGateway + ?Sized` lets `P` be the trait object itself, too, so `new` takes an `Arc<dyn PaymentGateway>` as happily as an `Arc<StripeGateway>`. We'll need that in a moment.

So keeping the gateway costs something either way: a type parameter that travels up to the container, or a trait object. Keeping only what you need costs nothing.

Think of saving a contact in your phone. You can save someone's whole business card, with their company printed on it, and then the company shows wherever the contact does. You can save the company's switchboard, which puts you through to whoever answers today, one step removed. Or, if all you ever needed was their address, you write that down and keep nothing else.

The first answer leaves two lines to change at a swap, the gateway's field and the service's. A plain Rust type alias brings that back to one:

```example
name: ch11_generic_services
region: container
```

`ShopGateway` is just another name for `StripeGateway`, and the compiler sees `StripeGateway` wherever it is written. snapfit only copies the name into the code it writes, so keep the `Arc<...>` in the field and give the alias to the gateway alone. Swapping the provider is now the alias's line. Here is the same container for tests, which differs only in its alias:

```example
name: ch11_generic_services
region: test_container
```

Let's run both:

```example
name: ch11_generic_services
region: run
```

It prints:

```output
name: ch11_generic_services
```

## Seeing the field as its trait

There is one thing the concrete field still gives away. `payments: Arc<StripeGateway>` makes `get_payments()` return `Arc<StripeGateway>`, so any code that uses the container can reach whatever Stripe offers beyond the trait. Say Stripe has a method of its own:

```example
name: ch11_trait_field
region: stripe
```

Code that calls `shop.get_payments().dashboard_url()` compiles today, and breaks the day the shop moves to another provider. The swap is no longer one line, and nobody notices until then.

What we'd like is for the field to promise only what the trait promises, and still have the container build Stripe. That's what `implementation` says. The field is the trait object, and the attribute names the type to build:

```example
name: ch11_trait_field
region: containers
```

Both containers' getters now return `Arc<dyn PaymentGateway>`, so code written against a getter names only the trait, and doesn't change at a swap. The receipt printer, generic only in its `new`, takes the trait object as it took Stripe. And the Stripe-only call is now refused, as it should be:

```compile-error
name: ch11_trait_field_hides_stripe
```

Let's run the containers:

```example
name: ch11_trait_field
region: run
```

It prints:

```output
name: ch11_trait_field
```

Here is what `implementation = StripeGateway` changes, and what it doesn't:

1. **The container builds the type it names.** It calls `StripeGateway::new()`, wraps the result in an `Arc`, and Rust converts it to `Arc<dyn PaymentGateway>`. `build = constructor(...)`, `build = default` and `build = builder` work as they would for an `Arc<StripeGateway>` field, on `StripeGateway`.
2. **Everyone else sees the trait.** The field, its getter, and every field that depends on it get `Arc<dyn PaymentGateway>`, the inputs struct of a factory included. Swapping the provider is the word after `implementation`.
3. **The type must implement the trait.** If it doesn't, the program doesn't compile, and the error points at the type you named.
4. **The field must be an `Arc` of a trait object.** On any other field, `implementation` is refused at the field, and the message says why.

This is the ride app of Chapter 2 again, seen from the rider's phone. The app promises a car, not which model, so you can't ask for the heated seats one model happens to have. That's exactly what lets the company change its fleet without a single rider noticing.

Two kinds of service don't fit the trait object as they stand. One keeps the gateway in its own type, like `RefundService<P>`: it needs one concrete type to name, and the field no longer has one, so a container with such a service keeps a concrete field and the alias of the previous section. The other is generic in its `new` without `?Sized`, which asks for a concrete type too; adding `?Sized` to its `new` is all it needs.

## Choosing while the program starts

So far the choice was made when we wrote the code: one container says Stripe, the other says fake. But a real shop often decides when it starts. The staging server should use the fake, production should use Stripe, and the configuration says which. Now the container itself can't name a concrete type for `payments`, because it doesn't know which one it will be until the configuration has been read.

We already have a field that is simply "something that charges cards", `Arc<dyn PaymentGateway>`. But `implementation` names its type when we write the code, and now we don't know it yet. Let's see what happens if we write the field with a plain `#[singleton]` and nothing else:

```compile-error
name: ch11_trait_object_field
```

The container builds a plain field by calling a constructor of its type, and a trait object has no constructor: there is nothing to call. So the field is refused, and the message offers two ways out. `implementation` names one type for good. A factory chooses while the program starts. Choosing from the configuration is exactly the wiring logic that [Chapter 10](10-passing-dependencies-by-name.md) said belongs in a factory.

Here is the configuration, which says which provider to use. To keep the example short it derives `Default`, which asks for the fake; a real `ShopConfig` would read the setting in its constructor, from an environment variable for example:

```example
name: ch11_configured_payments
region: config
```

And the container, whose factory reads the configuration and picks the gateway:

```example
name: ch11_configured_payments
region: configured
```

Because a factory is an ordinary function, we can also call it on its own, with a configuration that asks for Stripe, without building any container. That's handy in a test:

```example
name: ch11_configured_payments
region: factory_alone
```

Let's run it:

```example
name: ch11_configured_payments
region: run
```

It prints:

```output
name: ch11_configured_payments
```

The configuration asks for the fake, so `orders` charges through the fake, and the factory on its own picks Stripe when asked.

You may recognise that `match` from [Chapter 2](02-choosing-between-implementations.md), where a closed list of providers made every service change when a provider was added. Here the list lives in one place only, the wiring. Adding a provider means a new type and one more line in this factory; `OrderService` and every other service stay as they are.

Think of a theatre with an understudy. The play is the same every night, and the part is the same, but which actor walks on is decided that evening, by the stage manager, from the evening's call sheet. `OrderService` is the play: it doesn't care who plays the gateway.

## Which way to choose

1. **Name the concrete type in the field when the choice is fixed, and a service keeps the gateway in its own type, like `RefundService<P>`.** The container needs no code, a second container covers tests, and a type alias keeps a swap to one line. That's `ShopContainer` and `TestShopContainer`.
2. **Type the field as the trait and name its `implementation` when the choice is fixed, and no service needs the concrete type.** The getter promises only the trait, and a swap is one word.
3. **Use a factory when the choice is made while the program starts**, typically from configuration. The field is `Arc<dyn PaymentGateway>`, so its getter returns the trait object too.

## Which problem this solves

1. **Tight coupling** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): no service names a gateway. The type of the `payments` field, or its `implementation`, is the only choice.
2. **A closed list** ([Chapter 2](02-choosing-between-implementations.md)): adding a provider means writing a type that implements `PaymentGateway` and naming it in a field. No service changes.
3. **Spreading type parameters** ([Chapter 2](02-choosing-between-implementations.md)): a service can take a trait object, and its type stays the same whatever the gateway. A service that takes a type parameter still carries it, but only the container names the concrete type.
4. **Hard to test** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): `TestShopContainer` builds the real services around a fake gateway.
5. **Someone still has to choose** ([Chapter 2](02-choosing-between-implementations.md)): the container chooses, once, in the type of a field, in its `implementation`, or in a factory when the choice depends on the configuration.

## Remember

1. Name the implementation once, in the type of a container field, and let services take a trait object or a type parameter.
2. A field can change to any other type that has the same constructor, can be cloned, and is accepted by the fields that need it. No service or attribute changes; a field whose type is generic over it, such as `Arc<RefundService<StripeGateway>>`, names the new type too.
3. A trait object keeps a service's type the same whatever the gateway; a type parameter avoids the indirect call, and its concrete type is named in the container.
4. A service needs the gateway in its type, or as a trait object, only if it keeps the gateway. A service that keeps only what it read needs neither; with `?Sized` on its `new`, it takes a trait-object field too. A type alias keeps the swap of a generic service to one line.
5. `implementation = StripeGateway` on an `Arc<dyn PaymentGateway>` field builds Stripe, and shows everyone else only the trait.
6. To build the application around different implementations, such as fakes for tests, declare a second container with the same fields.
7. When the choice is made while the program starts, type the field as the trait object and build it with `build = factory`; the factory reads the configuration and returns the implementation.
