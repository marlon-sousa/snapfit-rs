# Choosing between implementations

At the end of the last chapter, our services were deciding everything themselves. Let's start with the case that hurts the most: payments. In production, `OrderService` must charge cards through Stripe. In a test, it must never do that.

## Asking for what you need

When you order a ride in a transport app, you ask for a car and a driver to take you from home to the airport. You don't choose the engine, and you don't hire the driver. What you care about is getting there.

Some details can matter, though. If you are travelling with three suitcases, you ask for a car with a big boot. You still don't build the car: you say what you need, and any car that fits will do.

In Rust, saying what you need is what a trait is for. `OrderService` doesn't need Stripe. It needs something that can charge a card. So let's write that down, and give it two implementations: the real one, and a fake that only pretends:

```example
name: ch02_choosing_between_implementations
region: gateways
```

Now, how does `OrderService` get one? There are four common ways, and each one teaches us something.

## First try: a list of options

The first idea is to keep the choice inside the service, and let the caller pick from a list:

```example
name: ch02_choosing_between_implementations
region: closed_list
```

This works, and a test can now pass `Provider::Fake`. But look at what the service still knows: every provider, by name, and how to build each one. When the shop adds PayPal, we edit `OrderServiceWithList` again, even though placing an order hasn't changed at all.

It's like a vending machine with a fixed set of slots: to sell a new drink, someone has to open the machine and add a slot. We'll call this a closed list: you can't add an option without opening up the code that uses the options.

## Second try: a type parameter

The second idea moves the choice out of the service. `OrderService<P>` works with any gateway, and whoever builds it decides which one:

```example
name: ch02_choosing_between_implementations
region: generic
```

This is nice. The service no longer chooses, and because the compiler knows the exact type, calls to the gateway cost nothing extra when the program runs. But look at `CheckoutController`. It holds an `OrderService`, so it has to become `CheckoutController<P>` too, even though it doesn't care how payments are made. Whatever holds the controller needs `P` as well, and so on, all the way up. Give the service a database and a stock service the same way, and every type above it carries three parameters it doesn't care about.

It's like a parcel inside a parcel inside a parcel, where every box has to repeat the label of the box inside it. The type parameters spread.

## Third try: a trait object

The third idea also moves the choice out, but hides the type. The service holds an `Arc<dyn PaymentGateway>`: a pointer to some gateway whose name it doesn't know:

```example
name: ch02_choosing_between_implementations
region: trait_object
```

`SharedOrderService` keeps the same type whatever gateway it receives, so nothing that holds it has to change. This is the ride from the start of the chapter: the service asks for something that charges cards, and takes whatever comes.

There is a small price. Calling through a trait object means following one more pointer on each call, and the compiler usually can't inline the call. For a service that talks to a database or a payment provider, that cost is tiny next to the call itself. A good rule of thumb: use a type parameter where every nanosecond counts, and a trait object where it keeps the code simpler.

## Fourth try: an enum that forwards

Can we have both: one type that doesn't spread, and calls the compiler can see through? We can, if we know every gateway in advance. Let's write an enum with one variant per gateway, and implement the trait for the enum itself. Each method looks at which variant it holds, and calls that gateway:

```example
name: ch02_choosing_between_implementations
region: enum
```

`OrderServiceWithEnum` holds a `Gateway`, so its type never changes, and nothing above it needs a parameter. Whoever builds the service chooses the variant, so the choice is out of the service, as with the last two tries. And a call is a `match` over two cases that the compiler can see in full, so it can inline the gateway's code, as it does for a type parameter.

The list is closed again, but it lives in one place now, outside the service: adding PayPal means a new variant, not a change to `OrderServiceWithEnum`. What we pay is the `match` itself. Our trait has one method and two gateways, so we wrote two arms. A trait with five methods and four implementations needs twenty arms, each of which does nothing but pass the call along, and every new implementation adds an arm to every method. It's like a switchboard operator who connects each call by hand: quick, as long as the board is small, and every new line means rewiring the board.

## All four, side by side

```example
name: ch02_choosing_between_implementations
region: run
```

It prints:

```output
name: ch02_choosing_between_implementations
```

## The choice didn't go away

With the type parameter, the trait object and the enum, the service receives its gateway instead of building it. That already fixes two problems from the last chapter: the service is no longer coupled to Stripe, and a test can hand it the fake.

But look at `main`. Somebody still writes `StripeGateway`, builds it, and passes it in. We moved the decision out of the service; we didn't make it disappear. And as the shop grows, whoever makes these decisions has more and more to build. The next chapter looks at who that is.

## What we found

1. **A closed list.** A service that picks from a list of implementations can be tested, but it still knows every implementation, and adding one means editing it.
2. **Spreading type parameters.** A type parameter costs nothing when the program runs, but it spreads to every type that holds the service.
3. **Forwarding by hand.** An enum that implements the trait keeps the list in one place, outside the service, and its calls can be inlined like a type parameter's, but every method passes every call along by hand, and every new implementation adds an arm to each of them.
4. **Someone still has to choose.** A service that receives its dependencies no longer decides which ones it gets, so something else has to build them and pass them in.
