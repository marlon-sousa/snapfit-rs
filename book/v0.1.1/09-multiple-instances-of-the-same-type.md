# Multiple instances of the same type

Our shop is getting busy, so we split the database in two: reads go to a replica, and writes go to the primary. Both are a `Database`. Let's see how a container keeps two components of the same type apart, without inventing a new type for each.

## Two databases, one type

`Database` has a constructor for each role, and `CqrsOrderStore` takes two of them: one to read from, one to write to. Both arguments have the same type, `Arc<Database>`:

Rust, from the program [`ch09_multiple_instances`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.1/book/examples/ch09_multiple_instances.rs)

```rust
pub struct Database {
    role: &'static str,
}

impl Database {
    pub fn primary() -> Self {
        Self { role: "primary-rw" }
    }

    pub fn replica() -> Self {
        Self { role: "replica-ro" }
    }

    pub fn role(&self) -> &'static str {
        self.role
    }
}

pub struct CqrsOrderStore {
    read_db: Arc<Database>,
    write_db: Arc<Database>,
}

impl CqrsOrderStore {
    pub fn new(read_db: Arc<Database>, write_db: Arc<Database>) -> Self {
        Self { read_db, write_db }
    }

    pub fn describe_routing(&self) -> String {
        format!(
            "reads go to {}, writes go to {}",
            self.read_db.role(),
            self.write_db.role()
        )
    }
}
```

In the container they are simply two fields with two names, `read_db` and `write_db`, and `order_store` asks for both, by name:

Rust, from the program [`ch09_multiple_instances`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.1/book/examples/ch09_multiple_instances.rs)

```rust
#[container]
pub struct ShopContainer {
    #[singleton(build = constructor(replica))]
    read_db: Arc<Database>,

    #[singleton(build = constructor(primary))]
    write_db: Arc<Database>,

    #[singleton(deps = [read_db, write_db])]
    order_store: Arc<CqrsOrderStore>,
}
```

Let's check that reads and writes go where they should, and that we really have two databases:

Rust, from the program [`ch09_multiple_instances`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.1/book/examples/ch09_multiple_instances.rs)

```rust
fn main() {
    let container = ShopContainer::new();
    let read_db = container.get_read_db();
    let write_db = container.get_write_db();

    println!("{}", container.get_order_store().describe_routing());
    println!(
        "distinct allocations: {}",
        !Arc::ptr_eq(&read_db, &write_db)
    );
}
```

It prints:

What `ch09_multiple_instances` prints, recorded in [`ch09_multiple_instances.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.1/book/expected/ch09_multiple_instances.txt)

```text
reads go to replica-ro, writes go to primary-rw
distinct allocations: true
```

Remember the service locator of [Chapter 3](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.1/03-who-builds-everything.md), which lost the replica because it filed everything by type? Here nothing gets lost. It's like the keys on your key ring: two of them may be the same make, but one is labelled "front door" and the other "garage", and you never mix them up.

## How the container tells them apart

1. **Components have names.** In `snapfit`, a component is known by its field's name, `read_db` or `write_db`, never by its type.
2. **`deps` passes them in order.** Because `order_store` lists `deps = [read_db, write_db]`, `ShopContainer::new()` calls `CqrsOrderStore::new` with a clone of `read_db` first and a clone of `write_db` second.
3. **Each one has its own getter.** The container generates `get_read_db()` and `get_write_db()`, both returning an `Arc<Database>`, each pointing to its own database.

## What if we swap the names?

Point 2 deserves a second look. `deps` passes the databases in the order it lists them, and both have the same type. So what happens if someone, one busy afternoon, writes the names the other way round? Let's try it:

Rust, from the program [`ch09_swapped_deps`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.1/book/examples/ch09_swapped_deps.rs)

```rust
#[container]
pub struct ShopContainer {
    #[singleton(build = constructor(replica))]
    read_db: Arc<Database>,

    #[singleton(build = constructor(primary))]
    write_db: Arc<Database>,

    #[singleton(deps = [write_db, read_db])]
    order_store: Arc<CqrsOrderStore>,
}
```

Let's run it:

Rust, from the program [`ch09_swapped_deps`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.1/book/examples/ch09_swapped_deps.rs)

```rust
fn main() {
    let container = ShopContainer::new();
    println!("{}", container.get_order_store().describe_routing());
}
```

It compiles without a single warning, and it prints:

What `ch09_swapped_deps` prints, recorded in [`ch09_swapped_deps.txt`](https://github.com/marlon-sousa/snapfit-rs/blob/v0.1.1/book/expected/ch09_swapped_deps.txt)

```text
reads go to primary-rw, writes go to replica-ro
```

Reads now go to the primary, and writes go to the replica. That's the problem we called **arguments by position** in [Chapter 3](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.1/03-who-builds-everything.md). Naming components by field fixed the locator's problem, but a constructor still receives its arguments in order, and the compiler checks their types, not which one we meant. The next chapter shows two ways to pass dependencies by name, so that their order no longer matters.

## Which problem this solves

1. **One entry per type** ([Chapter 3](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.1/03-who-builds-everything.md)): `read_db` and `write_db` have the same type and are still two components, because `snapfit` names components by field. No wrapper type is needed to tell them apart.

## Remember

1. A container can hold as many fields of the same type as your application needs.
2. When a constructor takes several arguments of the same type, the order of the names in `deps` is the only thing that tells them apart, and the compiler can't catch a swapped pair. [Chapter 10](https://marlon-sousa.github.io/snapfit-rs/book/v0.1.1/10-passing-dependencies-by-name.md) passes them by name instead.
