# Choosing an implementation

## What you want to do

Production charges customers through Stripe, and a test must use a fake gateway. You want `OrderService` and `RefundService` to work with either, without naming Stripe anywhere but in the one place that builds the shop.

## The code

The two gateways share a trait:

```example
name: ch10_choosing_an_implementation
region: gateways
```

The services take the trait, in the two ways [Chapter 2](02-choosing-between-implementations.md) showed: `OrderService` takes a trait object, and `RefundService` takes a type parameter:

```example
name: ch10_choosing_an_implementation
region: services
```

Each container makes the choice once, in the type of its `payments` field. `ShopContainer` is the production shop, and `TestShopContainer` builds the same services around the fake:

```example
name: ch10_choosing_an_implementation
region: containers
```

```example
name: ch10_choosing_an_implementation
region: run
```

It prints:

```output
name: ch10_choosing_an_implementation
```

## How it works

1. **The field's type is the choice.** `payments: Arc<StripeGateway>` tells the container to call `StripeGateway::new()`. The container is the only place that chooses Stripe.
2. **Services that take a trait object.** `OrderService::new` takes an `Arc<dyn PaymentGateway>`. The generated `new()` passes it a clone of the `Arc<StripeGateway>` field, and Rust converts it, as it would in any function call. `orders` keeps the type `Arc<OrderService>` in both containers.
3. **Services that take a type parameter.** `RefundService<P>` works with any gateway, and its field names the concrete type: `Arc<RefundService<StripeGateway>>`. The type parameter appears in the container, where the choice is made, and in no other service.
4. **Swapping is a change to the container only.** Compare the two containers: apart from their names, they differ only in the gateway's type. To move production from Stripe to another provider, you write a type that implements `PaymentGateway`, change `payments: Arc<StripeGateway>` to that type, and recompile. With services that take a trait object, that is the whole change. A service with a type parameter names the gateway in its own field as well, `Arc<RefundService<StripeGateway>>`, so it is one more line: the cost of a type parameter, kept inside the container.
5. **A test container.** `TestShopContainer` declares the same fields with `FakeGateway`. A test builds it with `TestShopContainer::new()` and gets the real services around the fake. For a test of one service, building that service directly with a fake is often simpler still ([Chapter 6](06-pure-domain-types.md)).
6. **Why the field itself is concrete.** The container builds each field by calling a constructor of the field's type. A trait object has no constructor, so a field typed `Arc<dyn PaymentGateway>` is refused. The concrete field, passed to services that take the trait, gives the same freedom.

### The rule: any type that fits works

Everything above follows from one rule. The generated `new()` is ordinary Rust: for each field, it calls the constructor its attribute names on the field's type, passing clones of the fields in its `deps`, and it passes the field on to the constructors of the fields that list it. So you can change a field's type to any other type that fits those two calls, recompile, and the application works with it:

1. **The new type has the constructor.** It has the constructor the attribute calls, `new` or the one `constructor =` names, taking the fields in `deps`, in that order.
2. **The fields that need it accept it.** Each constructor that receives the field takes the new type: because it asks for exactly that type, because it is generic over a trait the new type implements (the type of that field then names the new type too, as `Arc<RefundService<FakeGateway>>` does), or because it takes an `Arc<dyn Trait>` of a trait the new type implements.

No service changes, and no attribute does. If the new type does not fit, the program does not compile.

## Which problem this solves

1. **The service picks its own dependencies** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): no service names a gateway. The type of the `payments` field is the only choice.
2. **A closed list** ([Chapter 2](02-choosing-between-implementations.md)): adding a provider means writing a type that implements `PaymentGateway` and naming it in a field. No service changes.
3. **Type parameters spread** ([Chapter 2](02-choosing-between-implementations.md)): a service can take a trait object, and its type stays the same whatever the gateway. A service that takes a type parameter still carries it, but only the container names the concrete type.
4. **It cannot be tested alone** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): `TestShopContainer` builds the real services around a fake gateway.
5. **The choice still has to be made** ([Chapter 2](02-choosing-between-implementations.md)): the container makes it, once, in the types of its fields.

## Remember

1. Name the implementation once, in the type of a container field; let services take a trait object or a type parameter.
2. Changing a field to any other type that has the same constructor and that the fields needing it accept, through a trait object, a type parameter or the exact type, works with no other change.
3. A trait object keeps the service's type the same whatever the gateway; a type parameter avoids the indirect call, and its concrete type is named in the container.
4. To build the application around different implementations, such as fakes for tests, declare a second container with the same fields.
