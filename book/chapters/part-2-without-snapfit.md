# Without Snapfit

In [Chapter 6](06-multiple-instances-of-the-same-type.md) we wired `read_db` and `write_db` of the same `Arc<Database>` type using `snapfit`. What happens when you try to do the same thing with a dynamic `TypeMap` service locator (`HashMap<TypeId, Box<dyn Any + Send + Sync>>`)?

## The `TypeId` overwrite trap

Here is a standard `TypeMapLocator` alongside `snapfit`'s `NamedDatabaseContainer`:

```example
name: part2_without_snapfit
region: typemap_collision
```

```example
name: part2_without_snapfit
region: snapfit_named_edges
```

Now register both the read replica and the primary database in `TypeMapLocator`, and compare it with `NamedDatabaseContainer`:

```example
name: part2_without_snapfit
region: run
```

When run, the program prints:

```output
name: part2_without_snapfit
```

## What changed and why it matters

1. **Silent overwrite in `TypeMap`:** Because `Arc::new(Database::replica())` and `Arc::new(Database::primary())` have the exact same `TypeId::of::<Arc<Database>>()`, the second `register` call silently overwrites the read replica. Every read query in the application is now unknowingly routed to the primary database.
2. **Wrapper pollution to work around `TypeMap`:** To prevent that collision in a type-keyed DI framework, you have to invent wrapper types (`pub struct ReadDatabase(pub Arc<Database>);`) and leak them into your domain constructors.
3. **Zero wrappers with `snapfit`:** Because `snapfit` resolves edges by container field name (`read_db` and `write_db`), `Database` stays a single pure domain type and both instances coexist, each wired by its field name.
