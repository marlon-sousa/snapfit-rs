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

#[test]
fn a_dispatched_factory_field_lists_the_variants_it_returns_and_names_no_single_one() {
    let db = TypeName::new("Db").unwrap();
    let mut factory = dependency(TypeShape::Arc, Some(Construction::Factory));
    factory.implementation = Implementation::Listed(vec![variant("MySql"), variant("Sqlite")]);
    assert_eq!(
        call_for(&factory, &Constructor::New, Some(&db)),
        Ok(Call::Factory)
    );

    factory.implementation = Implementation::Named(variant("MySql"));
    assert_eq!(
        call_for(&factory, &Constructor::New, Some(&db)),
        Err(DomainError::StaticDispatch(
            StaticDispatchProblem::ImplementationWithFactory {
                dependency: store(),
                name: db.clone(),
                implementation: TypeName::new("MySql").unwrap(),
            }
        ))
    );
}

#[test]
fn a_dispatched_field_listing_implementations_without_a_factory_is_refused() {
    let db = TypeName::new("Db").unwrap();
    let constructions = [
        None,
        Some(Construction::Constructor(Constructor::Default)),
        Some(Construction::Builder(builder_methods())),
    ];
    for shape in [TypeShape::Arc, TypeShape::Value] {
        for construction in constructions.clone() {
            let mut listing = dependency(shape, construction);
            listing.implementation = Implementation::Listed(vec![variant("MySql")]);
            let call = call_for(&listing, &Constructor::New, Some(&db));
            assert_eq!(
                call,
                Err(DomainError::StaticDispatch(
                    StaticDispatchProblem::ListWithoutFactory {
                        dependency: store(),
                        name: db.clone(),
                    }
                )),
                "{shape:?} {:?}",
                listing.construction
            );
        }
    }
}
