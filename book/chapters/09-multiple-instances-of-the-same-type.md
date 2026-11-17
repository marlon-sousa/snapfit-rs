# Multiple instances of the same type

## What you want to do

The shop separates read queries from write commands using two `Arc<Database>` instances: a read-only replica (`read_db`) and a read-write primary (`write_db`). You want `CqrsOrderStore` to receive both databases without inventing `ReadDb` and `WriteDb` wrapper structs.

## The code

`Database` and `CqrsOrderStore` are plain domain structs where both database handles have the exact same type, `Arc<Database>`:

```example
name: ch09_multiple_instances
region: domain
```

In `ShopContainer`, `read_db` and `write_db` are two fields of type `Arc<Database>`, and `order_store` wires them by field name:

```example
name: ch09_multiple_instances
region: container
```

Running the container verifies that `read_db` and `write_db` are distinct instances routed to the right roles:

```example
name: ch09_multiple_instances
region: run
```

It prints:

```output
name: ch09_multiple_instances
```

## How it works

1. **Named-edge graph:** In `snapfit`, nodes in the dependency graph are identified by their **container field name** (`read_db`, `write_db`), never by `TypeId`.
2. **Positional mapping from `deps`:** Because `order_store` declares `deps = [read_db, write_db]`, `ShopContainer::new()` calls `CqrsOrderStore::new(read_db.clone(), write_db.clone())`.
3. **Distinct getters:** `ShopContainer` generates `get_read_db(&self) -> Arc<Database>` and `get_write_db(&self) -> Arc<Database>`, returning the two separate allocations.

## Remember

1. You can declare as many fields of the same Rust type in a `#[container]` as your architecture requires.
2. Always verify the order of field names in `deps = [read_db, write_db]` when a constructor takes multiple arguments of the same type.
