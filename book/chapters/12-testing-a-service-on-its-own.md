# Testing a service on its own

Back in [Chapter 1](01-a-service-that-builds-its-own-dependencies.md) we found our services hard to test, because each one built the real database and the real Stripe for itself. Since then our services receive what they need. In [Chapter 6](06-pure-domain-types.md) we tested a calculator by handing it a configuration, and in [Chapter 11](11-choosing-an-implementation.md) a second container built the shop around a fake gateway.

The calculator was easy, though: it needed one small value. Let's test something with a real graph below it, and see what receiving our dependencies has bought us, and what it hasn't yet.

## A checkout with a deep graph

Remember the `CheckoutController` of [Chapter 2](02-choosing-between-implementations.md), which held the order service? Let's give it a second job: before it places an order, it checks the stock. When a customer buys an item, the checkout asks the stock service whether the item is available and, if it is, asks the order service to place the order. Both services need the shop's configuration and a database. Here are the configuration and the database, a connection pool that opens when it is created:

```example
name: ch12_testing_by_hand
region: infrastructure
```

The two services read the database, and the configuration tells them the currency and how many items the shop always keeps back:

```example
name: ch12_testing_by_hand
region: services
```

And the checkout, which uses both:

```example
name: ch12_testing_by_hand
region: controller
```

The container wires them as we have done since [Chapter 7](07-wiring-dependencies-and-constructors.md):

```example
name: ch12_testing_by_hand
region: container
```

## Testing the checkout by hand

Now let's test one thing about the checkout: when an item is out of stock, it says so and places no order. The checkout is plain Rust, so the test builds one with `CheckoutController::new`, as [Chapter 6](06-pure-domain-types.md) taught us. But `new` wants an order service and a stock service, and each of those wants a configuration and a database. So the test builds all of it:

```example
name: ch12_testing_by_hand
region: by_hand
```

Let's run the shop, and then the test:

```example
name: ch12_testing_by_hand
region: run
```

It prints:

```output
name: ch12_testing_by_hand
```

The test passes, but look at what it took. We wanted to check one `if` in the checkout, and we wrote five lines of building before the line that checks it. The test opened a connection pool to a test database, so that database has to be running wherever the tests run. And whether the kettle is out of stock depends on what the test database holds today: if somebody adds kettles to it, our test breaks, though the checkout hasn't changed.

It also tests more than we asked. If the stock service gets its arithmetic wrong, this test fails too, and it tells us the checkout is broken. And every service we add below the checkout, a cache or a price list, is one more thing every checkout test has to build.

It's like checking that a new light switch works by building the whole house around it, power station included. Our services receive their dependencies now, which was the first half of the job. The checkout still asks for those exact services, though, so a test can only hand it those services, and everything they need.

## Asking for what the checkout needs

The checkout doesn't need `StockService`. It needs something that answers "is this item available?", and something that places an order. That's the same step [Chapter 2](02-choosing-between-implementations.md) took for payments: say what you need as a trait. Let's write two:

```example
name: ch12_mocking_services
region: traits
```

`Send + Sync` says that whatever implements the trait can be shared between threads, as a web server's handlers share the shop's services. Ignore the `#[automock]` lines for a moment; we'll come to them. The services stay as they were, and each now implements its trait:

```example
name: ch12_mocking_services
region: services
```

And the checkout asks for the traits, as trait objects:

```example
name: ch12_mocking_services
region: controller
```

In the container, the two services become fields typed as their trait, each naming its `implementation`, as in [Chapter 11](11-choosing-an-implementation.md). The `deps` still go to the implementation's `new`:

```example
name: ch12_mocking_services
region: container
```

Production builds exactly what it built before. What changed is what a test has to hand the checkout: anything that implements `Stock` and anything that implements `Orders`. Neither of those has to have a database behind it.

### A type parameter or a trait object

The checkout above holds `Arc<dyn Orders>` and `Arc<dyn Stock>`. As Chapter 2 showed, a service can take its dependencies as type parameters instead:

```example
name: ch12_generic_checkout
region: generic_controller
```

Both work for testing, as we'll see in a moment. The difference is the one [Chapter 11](11-choosing-an-implementation.md) described. `ExpressCheckout<O, S>` keeps its services and names them in its own type, so a container has to name concrete types for it, `Arc<ExpressCheckout<OrderService, StockService>>`, with concrete service fields beside it. `CheckoutController` keeps the same type whatever it is given, so it fits the trait fields above and every container that uses it. Choose the type parameter where each call must cost nothing extra, and the trait object where it keeps the wiring simpler.

## Stand-ins with a mocking library

Now the test needs something that implements `Stock` and says "no" when asked about the kettle. We could write a small struct for it, as Chapter 2 wrote `FakeGateway`. For one trait that's fine. But each test wants different answers, and wants to check what the checkout did: was `place` called, and with what? Writing a fake for every case quickly turns into a lot of small structs.

A mocking library writes those stand-ins for us. In this book we use [mockall](https://docs.rs/mockall/0.15.0/mockall/), the most widely used one for Rust. It isn't the only choice: another library, or fakes written by hand, work just as well, because `snapfit` never sees them. A stand-in is just another type that implements the trait.

That's what `#[automock]` on our traits was for. It generates a `MockStock` and a `MockOrders`, each implementing its trait, which a test fills with the answers it wants.

In your own crate, mockall goes in the `[dev-dependencies]` of `Cargo.toml`, as `mockall = "0.15"`, so the crates that depend on yours never build it. And the traits carry `#[cfg_attr(test, mockall::automock)]`, so the mocks exist only when the crate's own tests are built. Integration tests under `tests/` build the crate without `cfg(test)`, so they don't see these mocks; they can use the in-memory approach at the end of this chapter instead.

Here is the out-of-stock test again, with stand-ins:

```example
name: ch12_mocking_services
region: test_out_of_stock
```

In your own crate, each of these test functions is a `#[test]` inside a `#[cfg(test)]` module; in the book they are called from `main`, so that we can show them running.

Read it like a script for two actors. The stock stand-in expects to be asked about the kettle, exactly once, and answers `false`. The order stand-in expects `place` never to be called. If the checkout calls `place` anyway, or never asks about the kettle, the test fails and mockall says which expectation wasn't met. Nothing else exists: no configuration, no database, no services.

Think of a flight simulator. A pilot practises an engine failure in a cockpit that answers exactly like the plane would, without an airport, without passengers, and without waiting for an engine to fail. The checkout is the pilot; mockall builds the cockpit.

The other path checks what the checkout passes on:

```example
name: ch12_mocking_services
region: test_in_stock
```

And the same stand-ins work for the generic `ExpressCheckout`, which takes them as its type parameters:

```example
name: ch12_generic_checkout
region: test_generic
```

Let's run the shop, then the first two tests:

```example
name: ch12_mocking_services
region: run
```

It prints:

```output
name: ch12_mocking_services
```

The shop opened its connection pool, as production should. The tests opened nothing. The generic test, run the same way, prints:

```output
name: ch12_generic_checkout
```

## Swapping the database

A stand-in for a service tests the checkout on its own. Sometimes we want the opposite: the real checkout and the real services together, without the real database. And sometimes the database itself changes: the shop moves from PostgreSQL to SQLite. Both are the same move, one level down: put the database behind a trait. Like `Orders` and `Stock`, it is `Send + Sync`, so the whole graph can be shared between threads:

```example
name: ch12_swapping_the_database
region: trait
```

PostgreSQL is one implementation:

```example
name: ch12_swapping_the_database
region: postgres
```

The services take an `Arc<dyn Database>` instead of the database's own type. They and the checkout are otherwise the ones of the previous section, asking for `Orders` and `Stock`:

```example
name: ch12_swapping_the_database
region: services
```

And the container names PostgreSQL once:

```example
name: ch12_swapping_the_database
region: before
```

Now the shop moves to SQLite. We write the new implementation:

```example
name: ch12_swapping_the_database
region: sqlite
```

And the container changes in one word, after the `implementation` of `db`. No service changes, and neither does the checkout. We keep both containers in the example so you can compare them:

```example
name: ch12_swapping_the_database
region: after
```

Let's run both shops:

```example
name: ch12_swapping_the_database
region: run
```

It prints:

```output
name: ch12_swapping_the_database
```

Tests use the same move. An in-memory database implements the trait with a few items written into it:

```example
name: ch12_in_memory_database
region: in_memory
```

A container for tests names it:

```example
name: ch12_in_memory_database
region: test_container
```

And a test builds that container and uses the real checkout, the real services and the real configuration, with only the database replaced. The shop keeps one item back, so the only lamp can't be sold, and a kettle can:

```example
name: ch12_in_memory_database
region: test
```

Let's run it:

```example
name: ch12_in_memory_database
region: run
```

It prints:

```output
name: ch12_in_memory_database
```

The test knows exactly what the database holds, because the in-memory database that the test's container names holds only what we wrote into it, and no server has to be running.

## Which kind of test to write

1. **A stand-in for each dependency** tests one service on its own: its decisions, and what it asks of the services it uses. Use a mocking library, or small fakes, behind the traits the service asks for.
2. **A container of the test's own, whose field names another `implementation`,** tests the real services together, with the slow or outside part, such as a database, swapped for one the test controls.
3. **The real container** belongs to the few tests that check the whole application against real infrastructure.

## Which problem this solves

1. **Hard to test** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): a service that asks for traits is tested with stand-ins, and nothing below them is built. A container whose database is a trait object is tested with an in-memory database.
2. **Tight coupling** ([Chapter 1](01-a-service-that-builds-its-own-dependencies.md)): the checkout no longer names `OrderService` or `StockService`, and the services no longer name PostgreSQL. Only the container does.
3. **Spreading type parameters** ([Chapter 2](02-choosing-between-implementations.md)): a checkout that takes trait objects keeps one type whatever it is given, in tests and in every container.

## Remember

1. A service is easy to test on its own when it asks for traits, not for other services' concrete types.
2. A field typed as its trait, with `implementation` and `deps`, builds the real service for production; a test builds the service it wants to check directly, with stand-ins.
3. mockall's `#[automock]` generates a stand-in for a trait, whose expectations say what it answers and how often it must be called. Other libraries, or fakes written by hand, work as well.
4. Put mocks behind `#[cfg_attr(test, mockall::automock)]` and mockall in `[dev-dependencies]`, so production builds never include them.
5. A dependency behind a trait, such as the database, is swapped in one word after `implementation`, for a new database or for an in-memory one in a test's container.
