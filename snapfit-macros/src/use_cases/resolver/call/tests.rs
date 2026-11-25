use super::*;
use crate::domain::{Constructor, DependencyName, Execution, Prerequisite, TypeName, Variant};
use crate::test_support::{builder, builder_methods};

mod dispatched;
mod implementation;

fn dependency(shape: TypeShape, construction: Option<Construction>) -> DependencyDeclaration<()> {
    let mut dependency = DependencyDeclaration::new(store(), ());
    dependency.shape = shape;
    dependency.construction = construction;
    dependency
}

fn store() -> DependencyName {
    DependencyName::new("store").unwrap()
}

fn refusal(problem: TraitObjectProblem) -> DomainError {
    DomainError::TraitObject {
        dependency: store(),
        problem,
    }
}

fn variant(text: &str) -> Variant<()> {
    Variant::new(TypeName::new(text).unwrap(), ())
}

/// The call of a field that holds no dispatched type.
fn undispatched<T>(
    dependency: &DependencyDeclaration<(), T>,
    leaf: &Constructor,
) -> Result<Call, DomainError> {
    call_for(dependency, leaf, None)
}

fn constructor(wrapping: Wrapping) -> Call {
    Call::Constructor {
        constructor: Constructor::New,
        wrapping,
    }
}

#[test]
fn a_value_is_built_by_its_constructor_as_it_is_and_an_arc_by_its_inner_type_s() {
    let cases = [
        (TypeShape::Value, Wrapping::None),
        (TypeShape::KeepsTraitObject, Wrapping::None),
        (TypeShape::Arc, Wrapping::Arc),
        (TypeShape::ArcKeepsTraitObject, Wrapping::Arc),
    ];

    for (shape, wrapping) in cases {
        let call = undispatched(&dependency(shape, None), &Constructor::New);
        assert_eq!(call, Ok(constructor(wrapping)), "{shape:?}");
    }
}

#[test]
fn a_factory_builds_any_shape_as_it_is() {
    for shape in [TypeShape::Value, TypeShape::Arc, TypeShape::TraitObject] {
        let call = undispatched(
            &dependency(shape, Some(Construction::Factory)),
            &Constructor::New,
        );
        assert_eq!(call, Ok(Call::Factory), "{shape:?}");
    }
}

#[test]
fn a_trait_object_without_a_factory_is_refused_naming_its_field() {
    let call = undispatched(&dependency(TypeShape::TraitObject, None), &Constructor::New);

    assert_eq!(call, Err(refusal(TraitObjectProblem::WithoutFactory)));
}

#[test]
fn leaf_build_applies_only_to_fields_with_no_deps_and_no_build_of_their_own() {
    let leaf = Constructor::Default;
    let default_call = Ok(Call::Constructor {
        constructor: Constructor::Default,
        wrapping: Wrapping::None,
    });

    let bare_leaf = dependency(TypeShape::Value, None);
    assert_eq!(undispatched(&bare_leaf, &leaf), default_call);

    let mut with_deps = dependency(TypeShape::Value, None);
    with_deps.prerequisites = vec![Prerequisite::new(DependencyName::new("config").unwrap())];
    assert_eq!(
        undispatched(&with_deps, &leaf),
        Ok(constructor(Wrapping::None))
    );

    let own_build = dependency(
        TypeShape::Value,
        Some(Construction::Constructor(Constructor::New)),
    );
    assert_eq!(
        undispatched(&own_build, &leaf),
        Ok(constructor(Wrapping::None))
    );
}

#[test]
fn leaf_build_leaves_an_async_leaf_to_its_own_constructor() {
    let mut async_leaf = dependency(TypeShape::Value, None);
    async_leaf.execution = Execution::Async;

    assert_eq!(
        undispatched(&async_leaf, &Constructor::Default),
        Ok(constructor(Wrapping::None))
    );
}

#[test]
fn a_builder_belongs_to_the_inner_type_of_an_arc_and_its_value_is_wrapped() {
    let call = undispatched(
        &dependency(TypeShape::Arc, Some(builder())),
        &Constructor::New,
    );

    assert_eq!(
        call,
        Ok(Call::Builder {
            methods: builder_methods(),
            wrapping: Wrapping::Arc,
        })
    );
}

#[test]
fn a_trait_object_built_by_a_builder_is_refused_pointing_to_a_factory() {
    let call = undispatched(
        &dependency(TypeShape::TraitObject, Some(builder())),
        &Constructor::New,
    );

    assert_eq!(call, Err(refusal(TraitObjectProblem::WithoutFactory)));
}
