use super::*;

fn named(shape: TypeShape, construction: Option<Construction>) -> DependencyDeclaration<()> {
    let mut dependency = dependency(shape, construction);
    dependency.implementation = Implementation::Named(variant("Stripe"));
    dependency
}

#[test]
fn an_arc_of_a_trait_object_is_built_by_the_named_implementation_and_wrapped() {
    let call = undispatched(&named(TypeShape::ArcTraitObject, None), &Constructor::New);

    assert_eq!(call, Ok(constructor(Wrapping::Arc)));
}

#[test]
fn an_arc_of_a_trait_object_without_an_implementation_or_factory_is_refused() {
    let call = undispatched(
        &dependency(TypeShape::ArcTraitObject, None),
        &Constructor::New,
    );

    assert_eq!(
        call,
        Err(refusal(TraitObjectProblem::WithoutImplementation))
    );
}

#[test]
fn an_implementation_is_refused_where_it_cannot_be_built() {
    let cases = [
        (
            TypeShape::Value,
            None,
            TraitObjectProblem::ImplementationWithoutTraitObject,
        ),
        (
            TypeShape::Arc,
            None,
            TraitObjectProblem::ImplementationWithoutTraitObject,
        ),
        (
            TypeShape::TraitObject,
            None,
            TraitObjectProblem::ImplementationOutsideArc,
        ),
        (
            TypeShape::KeepsTraitObject,
            None,
            TraitObjectProblem::ImplementationKeptInside,
        ),
        (
            TypeShape::ArcKeepsTraitObject,
            None,
            TraitObjectProblem::ImplementationKeptInside,
        ),
        (
            TypeShape::ArcTraitObject,
            Some(Construction::Factory),
            TraitObjectProblem::ImplementationWithFactory,
        ),
    ];

    for (shape, construction, problem) in cases {
        let call = undispatched(&named(shape, construction), &Constructor::New);
        assert_eq!(call, Err(refusal(problem)), "{shape:?}");
    }
}

#[test]
fn a_list_of_implementations_is_refused_on_every_field_that_holds_no_dispatched_type() {
    let shapes = [
        TypeShape::Value,
        TypeShape::Arc,
        TypeShape::ArcTraitObject,
        TypeShape::TraitObject,
        TypeShape::KeepsTraitObject,
        TypeShape::ArcKeepsTraitObject,
    ];

    for shape in shapes {
        for construction in [None, Some(Construction::Factory)] {
            let mut listing = dependency(shape, construction.clone());
            listing.implementation = Implementation::Listed(vec![variant("Stripe")]);
            let call = undispatched(&listing, &Constructor::New);
            assert_eq!(
                call,
                Err(refusal(TraitObjectProblem::ImplementationList)),
                "{shape:?} {construction:?}"
            );
        }
    }
}
