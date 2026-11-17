use super::*;
use crate::domain::{Constructor, DependencyName, EnumDispatch, FunctionName, TypeName};

fn dependency(shape: TypeShape, construction: Construction) -> DependencyDeclaration<()> {
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

fn builder() -> Construction {
    Construction::Builder(FunctionName::new("build_store").unwrap())
}

#[test]
fn a_plain_type_is_built_by_its_constructor_as_it_is() {
    let call = call_for(&dependency(TypeShape::Value, Construction::default()));

    assert_eq!(call, Ok(constructor(Wrapping::None)));
}

#[test]
fn an_arc_is_built_by_its_inner_type_s_constructor_and_wrapped() {
    let call = call_for(&dependency(TypeShape::Arc, Construction::default()));

    assert_eq!(call, Ok(constructor(Wrapping::Arc)));
}

#[test]
fn a_builder_builds_any_shape_as_it_is() {
    let call = call_for(&dependency(TypeShape::TraitObject, builder()));

    assert_eq!(
        call,
        Ok(Call::Builder(FunctionName::new("build_store").unwrap()))
    );
}

#[test]
fn a_trait_object_without_a_builder_is_refused_naming_its_field() {
    let call = call_for(&dependency(TypeShape::TraitObject, Construction::default()));

    assert_eq!(
        call,
        Err(DomainError::TraitObjectWithoutBuilder {
            dependency: DependencyName::new("store").unwrap()
        })
    );
}

#[test]
fn a_dispatched_trait_object_is_left_to_the_feature_gate() {
    let mut dispatched = dependency(TypeShape::TraitObject, Construction::default());
    let store = TypeName::new("Store").unwrap();
    dispatched.dispatch = Some(EnumDispatch::new(store.clone(), vec![store]));

    assert_eq!(call_for(&dispatched), Ok(constructor(Wrapping::None)));
}
