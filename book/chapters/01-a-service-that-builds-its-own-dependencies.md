# A service that builds its own dependencies

## What you want to do

The shop takes orders. Placing an order reserves the item in stock and charges the customer through Stripe, and a separate report reads the day's sales from the same database. You want to write these services the way most Rust code starts: each one creates what it needs in its own `new()`.

## The code

Two pieces of infrastructure, a database connection and a Stripe client. Each prints a line when it is created, so the output shows when that happens:

```example
name: ch01_builds_its_own
region: infrastructure
```

The services build their own dependencies. `StockService` opens the database, `OrderService` builds a `StockService` and connects to Stripe, and `ReportService` opens the database too:

```example
name: ch01_builds_its_own
region: services
```

The program creates an order service and a report service, and uses both:

```example
name: ch01_builds_its_own
region: run
```

It prints:

```output
name: ch01_builds_its_own
```

## What goes wrong

The program works, and for a small tool this is fine. The problems appear as the shop grows, and each one has a name that the rest of the book uses.

1. **The service picks its own dependencies.** `OrderService` decides that payments go through Stripe, with which key, and that stock lives in `postgres://shop`. To use another payment provider, or another database in staging, you edit `OrderService`, even though how an order is placed has not changed.
2. **It cannot be tested alone.** A unit test of `place_order` runs `OrderService::new()`, so it opens a database connection and connects to Stripe with the live key, as the output shows. There is no way to hand the service a fake.
3. **Nothing is shared.** `StockService` and `ReportService` each open their own connection to the same database: the output shows two of them. With ten services, that is ten connections, and nothing in the code says so.
4. **Two jobs in one place.** `OrderService` should know how to place an order. Here it also knows how to build a `StockService`, which means knowing that a stock service needs a database, and which one. Each service carries knowledge of everything below it.

## Remember

1. A service that calls the constructors of its own dependencies decides which implementation it uses, and every caller inherits that decision.
2. Building dependencies inside `new()` hides how many instances exist and makes the service impossible to test without the real infrastructure.
3. The next chapter takes the first step: the service stops choosing.
