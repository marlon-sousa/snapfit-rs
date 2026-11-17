# Who builds everything?

## What you want to do

Every service now receives its dependencies instead of building them. You want one place in the program that builds them all, each once, in the right order, and hands each service what it needs.

## The code

### Building by hand

The services take what they use as arguments. `OrderService` receives a stock service and any payment gateway, and the database is shared through `Arc`:

```example
name: ch03_manual_wiring
region: domain
```

`AppState` is the one place that builds everything. It opens the database once, passes clones of the `Arc` to the services that need it, and decides that payments go through Stripe:

```example
name: ch03_manual_wiring
region: manual_assembly
```

```example
name: ch03_manual_wiring
region: run
```

It prints:

```output
name: ch03_manual_wiring
```

### A service locator

Another common shape is a map from a type to an instance: you register each component once, and anything that needs one asks for it by type at run time:

```example
name: ch03_service_locator
region: locator
```

Here two databases are registered, a replica and then a primary, and the gateway is registered as `StripeGateway` but asked for as `Arc<StripeGateway>`:

```example
name: ch03_service_locator
region: run
```

It prints:

```output
name: ch03_service_locator
```

## What goes wrong

### The name for this: dependency injection

What this chapter does has a name. **Dependency injection** means that a component receives what it depends on instead of building it. The one place in the program that builds the components and connects them, `AppState` here, is called the composition root. The services of this chapter are already written that way, so the problems left are in the composition root.

### Building by hand

1. **The order is kept by hand.** `AppState::build` must open the database before it builds the stock service, and the stock service must exist before `order_service()` can build an order service. Give the database a dependency of its own, and you reorder the lines yourself.
2. **Sharing is a convention.** The stock service is built once and shared, but `order_service()` builds a new `OrderService` each time it is called. Both look the same from outside, and only reading the code tells which is which.
3. **The graph is nowhere.** To learn what depends on what, you read every constructor call in `build`, and nothing shows the whole picture at once.

### A service locator

1. **Mistakes appear at run time.** The gateway was registered under one type and asked for under another, so the locator found nothing. The compiler cannot see this: the program starts, and the first order fails.
2. **One entry per type.** The second database replaced the first without a word, so every read now goes to the primary. Two instances of one type need wrapper types to tell them apart.
3. **Services drift back to choosing.** When a service is handed the locator and asks it for what it needs, it is picking its own dependencies again, only later and without the compiler's help.

## Remember

1. Dependency injection means components receive their dependencies; the composition root is the one place that builds and connects them.
2. Building by hand is checked by the compiler, but the order, the sharing and the graph live in your head.
3. A service locator moves mistakes to run time and keys everything by type.
4. The next chapter shows the composition root `snapfit` builds for you, and how it answers each problem named so far.
