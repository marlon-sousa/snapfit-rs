# Who builds everything?

Our services now receive what they need instead of building it. That moved every decision out of them, and those decisions have to land somewhere. Let's give them one place to land.

## Services that receive

First, the services, rewritten in the new style. Each one takes what it uses as arguments. `OrderService` receives a stock service and any payment gateway, and the database is shared through `Arc`, so that several services can hold the same one:

```example
name: ch03_manual_wiring
region: domain
```

None of them builds anything any more. Which means something else has to.

## One place that builds everything

Let's write it. `AppState` opens the database once, hands a clone of the `Arc` to each service that needs it, and decides that payments go through Stripe:

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

Look at the first line: one connection this time, shared by the stock service and the report service.

## There's a name for this

What we have been doing since the last chapter has a name: dependency injection. It sounds grander than it is. It only means that a component receives what it depends on instead of building it. That's all.

The one place that builds the components and connects them, our `AppState`, has a name too: the composition root. Think of the stage manager of a theatre. The actors don't bring their own props, lights and scenery; they find them in place when they walk on stage. Someone set all of that up, in the right order, before the curtain rose. That someone is the composition root.

Our services are already written the right way. What is left to look at is the composition root itself.

## Where it starts to hurt

The compiler checks `AppState`, which is great: if we forget to pass something, the program doesn't build. But as the shop grows, four things start to hurt.

The first is the order. `build` has to open the database before it builds the stock service, because the stock service needs it. Today that is four lines and easy to see. When the database needs a configuration, and the configuration needs a secrets client, we reorder the lines ourselves, every time. It's like assembling a wardrobe without the instructions: you can do it, but you have to remember that the frame goes in before the doors. We order by hand.

The second is sharing. The stock service is built once and shared. But look at `order_service()`: it builds a new `OrderService` every time it is called. From outside, both look the same, and only reading the code tells you which is which. What is shared and what is not is a convention that everybody has to remember.

The third is the big picture. To learn what depends on what, you read every constructor call in `build`. Nothing shows the whole graph at once: it is there, but you can't see it.

The fourth is quieter. Every constructor call in `build` passes its arguments by position. `OrderService::new` takes a stock service and then a gateway, and because those have different types, passing them the wrong way round doesn't compile. Now picture a constructor that takes two arguments of the same type, such as a database to read from and a database to write to. Swap them, and the compiler is perfectly happy, because both fit. It's like plugging the hot and the cold pipes of a shower into the wrong taps: they are the same size, everything connects, and you find out when you step in. The compiler checks types, not intentions.

## Another shape: the service locator

Many frameworks take a different road. Instead of one function that builds everything, they keep a map from a type to an instance. You register each component once, and anything that needs one asks for it by its type while the program runs. Here is one, small enough to read in a minute:

```example
name: ch03_service_locator
region: locator
```

Let's use it. We register two databases, a replica and then a primary, and a Stripe gateway. Then we ask for them back:

```example
name: ch03_service_locator
region: run
```

It prints:

```output
name: ch03_service_locator
```

Two surprises. We registered two databases, but the map kept one: the primary replaced the replica without a word, so every read now goes to the primary. And we registered the gateway as `StripeGateway` but asked for an `Arc<StripeGateway>`, so the locator found nothing.

Think of a cloakroom that files coats by colour instead of giving you a ticket. Hand in two black coats, and the second one takes the first one's hook. Describe yours slightly wrong, and you find out only at the end of the evening, when you're about to leave. The compiler couldn't see either mistake: the program starts, and fails only when it asks for the component.

There is one more trap, and it's subtler. Once a service can reach the locator, it's tempting to hand it the whole locator and let it ask for whatever it needs. Then the service's constructor no longer says what the service depends on. You find out by reading its body, or by running it. It's like a recipe that says "take what you need from the pantry" instead of listing its ingredients: you discover that you're out of eggs halfway through the cake. And the service is back to choosing its own dependencies, as in the first chapter, only now without the compiler's help.

## What we found

1. **Ordering by hand.** The composition root has to build every component after the ones it needs, and you keep that order yourself.
2. **Sharing by convention.** Nothing in the code says which components are shared and which are built again each time; only reading it tells you.
3. **An invisible graph.** What depends on what is spread over every constructor call, and nothing shows it in one place.
4. **Arguments by position.** Constructors receive their arguments in order, so two arguments of the same type can be swapped without the compiler noticing.
5. **Run-time failures.** A service locator finds a missing or mistyped component only when the program asks for it, not when it compiles.
6. **One entry per type.** A locator keyed by type keeps one component per type, so two databases need wrapper types to tell them apart.
7. **Hidden dependencies.** A service that is handed the locator no longer says what it needs, and picks its own dependencies again.

Building the composition root ourselves is the right idea. It is just tedious to write and easy to get wrong. The next chapter shows a composition root that you declare, and that the compiler writes and checks for you.
