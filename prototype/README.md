# Prototype: `impl Trait` in field types (throwaway, not for merging)

Asked for in the project thread "Discuss generics for implementation fields" on 2026-11-23, to see what editors show on a field typed `impl Trait`.

- `snapfit-macros/src/expansion/impl_fields.rs` rewrites the struct before it is read. A field typed `impl Trait` with `implementation = X` becomes `X`. An implementation may hold `impl Trait` itself, such as `Controller<impl SvcTrait>`. Every `impl Trait`, in a field or in an implementation, becomes the implementation of the one field among that field's own `deps` that declared the same trait.
- `impl-fields-app` is a container that uses it: `svc: impl SvcTrait`, `controller: impl ControllerTrait` built as `Controller<impl SvcTrait>`, and `api: Api<impl ControllerTrait>`.
- `lsp_probe.py` drives rust-analyzer over LSP and asks for hover, go-to-definition and completion at given places.

Run `cargo run` in `impl-fields-app`, and `python3 -I ../lsp_probe.py "$PWD" '<checks>'` from it.
