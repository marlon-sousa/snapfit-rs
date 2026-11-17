# Choosing between implementations

## What you want to do

Production charges customers through Stripe, and tests must never do that: they need a fake gateway that only pretends. You want `OrderService` to work with either, without editing it each time.

## The code

First, the two gateways share a trait, so that the service can talk to either one:

```example
name: ch02_choosing_between_implementations
region: gateways
```

### A closed list

The first idea is to keep the choice inside the service, and let the caller pick from a list:

```example
name: ch02_choosing_between_implementations
region: closed_list
```

### A type parameter

The second idea moves the choice out of the service. `OrderService<P>` works with any gateway, and whoever builds it decides which. Anything that holds an `OrderService` must name `P` too, as `CheckoutController` does:

```example
name: ch02_choosing_between_implementations
region: generic
```

### A trait object

The third idea also moves the choice out, but erases the type: the service holds an `Arc<dyn PaymentGateway>`, and does not know which gateway is behind it:

```example
name: ch02_choosing_between_implementations
region: trait_object
```

### All three in use

```example
name: ch02_choosing_between_implementations
region: run
```

It prints:

```output
name: ch02_choosing_between_implementations
```

## What goes wrong

1. **A closed list.** `OrderServiceWithList` can be tested now, but it still picks its own dependencies: it knows every provider by name and builds each one. Adding PayPal means editing the service again.
2. **Type parameters spread.** `OrderService<P>` no longer chooses, and calls to the gateway add no run-time cost. But `CheckoutController` has to become `CheckoutController<P>`, and so does everything that holds a controller. Give the service a database and a stock service the same way, and every type above it carries three parameters it does not care about.
3. **The choice still has to be made.** With a type parameter or a trait object, the service receives its gateway instead of building it. That fixes the first two problems of [Chapter 1](01-a-service-that-builds-its-own-dependencies.md): the service no longer picks its own dependencies, and a test can hand it a fake. But someone now has to choose Stripe, build it, and pass it in. That someone is the subject of the next chapter.

A trait object costs a little at run time: one pointer to follow on each call, and the compiler usually cannot inline the call. For a service that talks to a database or a payment provider, that cost is far below the cost of the call itself. Use a type parameter where it matters, and a trait object where it keeps the code simpler.

## Remember

1. A service that receives its dependencies, as a type parameter or as a trait object, no longer decides which implementation it uses.
2. Type parameters add no run-time cost, but they spread to every type that holds the service.
3. Receiving dependencies moves the decision to whoever builds the service. Something still has to make it.
