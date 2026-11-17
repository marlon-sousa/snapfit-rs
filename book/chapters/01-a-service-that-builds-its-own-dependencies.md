# A service that builds its own dependencies

Let's build a small online shop together. It doesn't need much to start: customers place orders, each order reserves an item in stock and charges the customer's card, and at the end of the day someone wants a sales report.

## Two pieces of infrastructure

A shop like this talks to the outside world. Ours talks to a database, where the stock and the sales live, and to Stripe, which charges cards. Here they are. Each one prints a line when it is created, so that later we can see exactly when that happens:

```example
name: ch01_builds_its_own
region: infrastructure
```

## Assembling the services

Now we need the services themselves. The stock service reserves items, so it needs the database. The simplest thing we can do is open the database right there, in its `new()`:

```example
name: ch01_builds_its_own
region: stock
```

The order service needs to reserve stock and to charge the card. Following the same idea, it creates its own stock service and connects to Stripe:

```example
name: ch01_builds_its_own
region: orders
```

The report service reads the day's sales, so it opens the database too:

```example
name: ch01_builds_its_own
region: reports
```

Perfect. Let's put them to work:

```example
name: ch01_builds_its_own
region: run
```

It prints:

```output
name: ch01_builds_its_own
```

It works, and for a small tool this would be fine. Most Rust programs start exactly like this. But let's see what happens as the shop grows.

## What if we want to test?

Say we want a test for `place_order`. The test has to create an `OrderService`, and the only way to do that is `OrderService::new()`. Look at the output again: creating one opens a connection to the real database and connects to Stripe with the live key. Every run of our test would reserve real stock and charge real money.

We can't hand the service a fake payment gateway, because the service never asks for one. It goes and gets the real thing itself. `OrderService` is welded to Stripe and to `postgres://shop`, not because placing an order needs Stripe, but because the service decided, on its own, that payments go through Stripe. This is called tight coupling.

Think of a lamp. You can wire a lamp straight into the wall. It works, and it keeps working until you want to move it to another room, or check it on a test bench. Then you need an electrician. A lamp with a plug works in any socket: it only cares that electricity comes out of the wall, and where that electricity comes from is not its business.

Our services are wired into the wall, and that is also why they are hard to test: a test can't run one without the real database and the real Stripe behind it.

## Two connections where one would do

Count the lines that say "opening a connection". There are two. `StockService` opened one, and `ReportService` opened another, to the same database. Nobody decided that the shop should have two connections. It just happened, because each service built its own. With ten services there would be ten, and nothing in the code would tell you so.

It's like two flatmates each signing their own internet contract for the same flat. Each one solved their own problem, and together they pay twice.

## A chef who also has to farm

Finally, look at what `OrderService` does besides placing orders. It decides how its stock service is made, by calling `StockService::new()`, and when. So every order service drags in whatever `StockService::new` opens: a database connection today, and if the stock service one day needs a cache, every order service quietly opens one too. A test of placing an order runs all of that as well. Every service ends up building everything below it.

That's like a chef who, to cook dinner, must also grow the vegetables. A good chef cares that the tomatoes are ripe, not how to grow them. Using your ingredients and producing them are two different jobs.

## What we found

1. **Tight coupling.** A service that calls the constructors of its dependencies decides which implementations it uses, and everyone who uses the service inherits that decision.
2. **Hard to test.** A service that builds its own infrastructure can't be tested without that infrastructure.
3. **Duplicated resources.** When every service builds what it needs, nothing is shared, and nobody can see how many instances exist.
4. **Mixed responsibilities.** A service that builds its dependencies has two jobs: its own work, and building everything below it.

All four come from the same place: the service builds what it needs instead of receiving it. In the next chapter, our services stop choosing.
