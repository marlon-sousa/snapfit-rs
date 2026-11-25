use super::*;
use crate::domain::{StaticDispatchProblem, TypeName};

#[test]
fn a_dispatched_field_wraps_its_implementation_in_its_variant_and_in_its_arc() {
    let db = TypeName::new("Db").unwrap();
    for (shape, wrapping) in [
        (TypeShape::Arc, Wrapping::ArcOfVariant),
        (TypeShape::Value, Wrapping::Variant),
    ] {
        let mut dispatched = dependency(shape, None);
        dispatched.implementation = Implementation::Named(variant("MySql"));
        let call = call_for(&dispatched, &Constructor::New, Some(&db));
        assert_eq!(call, Ok(constructor(wrapping)), "{shape:?}");
    }
}

#[test]
fn a_dispatched_field_without_an_implementation_or_a_factory_is_refused() {
    let db = TypeName::new("Db").unwrap();
    let call = call_for(
        &dependency(TypeShape::Arc, None),
        &Constructor::New,
        Some(&db),
    );

    assert_eq!(
        call,
        Err(DomainError::StaticDispatch(
            StaticDispatchProblem::WithoutImplementation {
                dependency: store(),
                name: db.clone(),
            }
        ))
    );
    let factory = dependency(TypeShape::Arc, Some(Construction::Factory));
    assert_eq!(
        call_for(&factory, &Constructor::New, Some(&db)),
        Ok(Call::Factory)
    );
}
