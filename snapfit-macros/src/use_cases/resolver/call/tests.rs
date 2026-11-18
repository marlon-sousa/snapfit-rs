use super::*;
use crate::domain::{Constructor, DependencyName, EnumDispatch, TypeName};

fn dependency(shape: TypeShape, construction: Option<Construction>) -> DependencyDeclaration<()> {
    let mut dependency = DependencyDeclaration::new(DependencyName::new("store").unwrap(), ());
    dependency.shape = shape;
    dependency.construction = construction;
    dependency
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

    assert_eq!(
        call,
        Err(DomainError::TraitObjectWithoutFactory {
            dependency: DependencyName::new("store").unwrap(),
        })
    );
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
    with_deps.prerequisites = vec![DependencyName::new("config").unwrap()];
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
    async_leaf.execution = crate::domain::Execution::Async;

    assert_eq!(
        call_for(&async_leaf, &Constructor::Default),
        Ok(constructor(Wrapping::None))
    );
}
