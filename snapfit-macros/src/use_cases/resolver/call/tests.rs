use super::*;
use crate::domain::{Constructor, DependencyName, EnumDispatch, Execution, Prerequisite, TypeName};
use crate::test_support::{builder, builder_methods};

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

fn constructor(wrapping: Wrapping) -> Call {
    Call::Constructor {
        constructor: Constructor::New,
        wrapping,
    }
}

#[test]
fn a_plain_type_is_built_by_its_constructor_as_it_is() {
    let call = call_for(&dependency(TypeShape::Value, None), &Constructor::New);

    assert_eq!(call, Ok(constructor(Wrapping::None)));
}

#[test]
fn an_arc_is_built_by_its_inner_type_s_constructor_and_wrapped() {
    let call = call_for(&dependency(TypeShape::Arc, None), &Constructor::New);

    assert_eq!(call, Ok(constructor(Wrapping::Arc)));
}

#[test]
fn a_factory_builds_any_shape_as_it_is() {
    for shape in [TypeShape::Value, TypeShape::Arc, TypeShape::TraitObject] {
        let call = call_for(
            &dependency(shape, Some(Construction::Factory)),
            &Constructor::New,
        );
        assert_eq!(call, Ok(Call::Factory), "{shape:?}");
    }
}

#[test]
fn a_dispatched_trait_object_is_left_to_the_feature_gate() {
    let mut dispatched = dependency(TypeShape::TraitObject, None);
    let store = TypeName::new("Store").unwrap();
    dispatched.dispatch = Some(EnumDispatch::new(store.clone(), vec![store]));

    assert_eq!(
        call_for(&dispatched, &Constructor::New),
        Ok(constructor(Wrapping::None))
    );
}

#[test]
fn a_trait_object_without_a_factory_is_refused_naming_its_field() {
    let call = call_for(&dependency(TypeShape::TraitObject, None), &Constructor::New);

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
    assert_eq!(call_for(&bare_leaf, &leaf), default_call);

    let mut with_deps = dependency(TypeShape::Value, None);
    with_deps.prerequisites = vec![Prerequisite::new(DependencyName::new("config").unwrap())];
    assert_eq!(call_for(&with_deps, &leaf), Ok(constructor(Wrapping::None)));

    let own_build = dependency(
        TypeShape::Value,
        Some(Construction::Constructor(Constructor::New)),
    );
    assert_eq!(call_for(&own_build, &leaf), Ok(constructor(Wrapping::None)));
}

#[test]
fn leaf_build_leaves_an_async_leaf_to_its_own_constructor() {
    let mut async_leaf = dependency(TypeShape::Value, None);
    async_leaf.execution = Execution::Async;

    assert_eq!(
        call_for(&async_leaf, &Constructor::Default),
        Ok(constructor(Wrapping::None))
    );
}

#[test]
fn a_builder_belongs_to_the_inner_type_of_an_arc_and_its_value_is_wrapped() {
    let call = call_for(
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
    let call = call_for(
        &dependency(TypeShape::TraitObject, Some(builder())),
        &Constructor::New,
    );

    assert_eq!(call, Err(refusal(TraitObjectProblem::WithoutFactory)));
}

fn named(shape: TypeShape, construction: Option<Construction>) -> DependencyDeclaration<()> {
    let mut dependency = dependency(shape, construction);
    dependency.implementation = Implementation::Named;
    dependency
}

#[test]
fn an_arc_of_a_trait_object_is_built_by_the_named_implementation_and_wrapped() {
    let call = call_for(&named(TypeShape::ArcTraitObject, None), &Constructor::New);

    assert_eq!(call, Ok(constructor(Wrapping::Arc)));
}

#[test]
fn an_arc_of_a_trait_object_without_an_implementation_or_factory_is_refused() {
    let call = call_for(
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
            TypeShape::ArcTraitObject,
            Some(Construction::Factory),
            TraitObjectProblem::ImplementationWithFactory,
        ),
    ];

    for (shape, construction, problem) in cases {
        let call = call_for(&named(shape, construction), &Constructor::New);
        assert_eq!(call, Err(refusal(problem)), "{shape:?}");
    }
}
