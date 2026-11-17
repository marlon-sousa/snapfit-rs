# Choosing an implementation

Back in [Chapter 2](02-choosing-between-implementations.md) we wanted `OrderService` to charge cards through Stripe in production and through a fake in tests, and we found that somebody still had to make that choice. Now we have a container, so let's let it choose, in exactly one place.

## Two gateways and two services

The gateways are the ones from Chapter 2, sharing a trait:

```example
name: ch10_choosing_an_implementation
region: gateways
```

The services take the trait in the two ways Chapter 2 showed. `OrderService` takes a trait object, `Arc<dyn PaymentGateway>`. `RefundService` takes a type parameter:

```example
name: ch10_choosing_an_implementation
region: services
```

## The container makes the choice

Each container makes the choice once, in the type of its `payments` field. `ShopContainer` is the production shop. `TestShopContainer` builds the same services around the fake:

```example
name: ch10_choosing_an_implementation
region: containers
```

Let's run both:

```example
name: ch10_choosing_an_implementation
region: run
```

It prints:

```output
name: ch10_choosing_an_implementation
```

Put the two containers side by side. Apart from their names, they differ only in the gateway's type. That's the ride app from Chapter 2: the services asked for something that charges cards, and the container decided which car to send.

## What happens in each container

1. **The field's type is the choice.** `payments: Arc<StripeGateway>` tells the container to call `StripeGateway::new()`. The container is the only place that says Stripe.
2. **A service that takes a trait object.** `OrderService::new` takes an `Arc<dyn PaymentGateway>`. The generated `new()` passes it a clone of the `Arc<StripeGateway>` field, and Rust converts it, as it would in any function call. That's why `orders` has the same type, `Arc<OrderService>`, in both containers.
3. **A service that takes a type parameter.** `RefundService<P>` works with any gateway, so its field names the concrete one: `Arc<RefundService<StripeGateway>>`. The type parameter shows up in the container, where the choice is made, and nowhere else.
4. **A container for tests.** `TestShopContainer` declares the same fields with `FakeGateway`. A test builds it with `TestShopContainer::new()` and gets the real services around the fake. When a test is about one service only, building that service directly with a fake is often simpler still ([Chapter 6](06-pure-domain-types.md)).

## Swapping a component

Now imagine the shop moves from Stripe to another provider. You write a type that implements `PaymentGateway`, change `payments: Arc<StripeGateway>` to that type, and recompile. For services that take a trait object, that's the whole change. A service with a type parameter names the gateway in its own field too, as `Arc<RefundService<StripeGateway>>` does, so that's one more line, still inside the container.

That isn't a special trick for payment gateways. It's a general rule, and it's worth knowing well. The generated `new()` is ordinary Rust: for each field, it calls the constructor its attribute names, on the field's type (or on `T`, for an `Arc<T>` field, which it then wraps in an `Arc`). It passes that constructor clones of the fields in its `deps`, and passes clones of the field on to the constructors of the fields that list it. So you can change a field's type to any other type that fits those calls, recompile, and the application works with it. A type fits when:

1. **It has the constructor.** It, or `T` for an `Arc<T>` field, has the constructor the attribute calls, `new` or the one `constructor =` names, taking the fields in `deps`, in that order.
2. **It can be cloned.** An `Arc<T>` field always can. A field of any other type must implement `Clone`, because its getter and the fields that need it receive clones.
3. **The fields that need it accept it.** Each constructor that receives the field takes the new type, because it asks for exactly that type, because it is generic over a trait the new type implements (the type of that field then names the new type too, as `Arc<RefundService<FakeGateway>>` does), or because it takes an `Arc<dyn Trait>` of a trait the new type implements.

No service changes, and no attribute does. If the new type doesn't fit, the program doesn't compile, so you find out right away.

## Why the field itself is concrete

You might wonder why we don't simply write `payments: Arc<dyn PaymentGateway>` in the container. The container builds each field by calling a constructor of the field's type, and a trait object has no constructor: there is nothing to call. So a field typed as a trait object is refused, at the field, and the message tells you what to write instead:

```compile-error
name: ch10_trait_object_field
```

A concrete field, passed to services that take the trait, gives you the same freedom.

## Which problem this solves

1. **Tight coupling** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): no service names a gateway. The type of the `payments` field is the only choice.
2. **A closed list** ([Chapter 2](02-choosing-between-implementations.md)): adding a provider means writing a type that implements `PaymentGateway` and naming it in a field. No service changes.
3. **Spreading type parameters** ([Chapter 2](02-choosing-between-implementations.md)): a service can take a trait object, and its type stays the same whatever the gateway. A service that takes a type parameter still carries it, but only the container names the concrete type.
4. **Hard to test** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): `TestShopContainer` builds the real services around a fake gateway.
5. **Someone still has to choose** ([Chapter 2](02-choosing-between-implementations.md)): the container chooses, once, in the types of its fields.

## Remember

1. Name the implementation once, in the type of a container field, and let services take a trait object or a type parameter.
2. A field can change to any other type that has the same constructor, can be cloned, and is accepted by the fields that need it. No service or attribute changes; a field whose type is generic over it, such as `Arc<RefundService<StripeGateway>>`, names the new type too.
3. A trait object keeps a service's type the same whatever the gateway; a type parameter avoids the indirect call, and its concrete type is named in the container.
4. To build the application around different implementations, such as fakes for tests, declare a second container with the same fields.
