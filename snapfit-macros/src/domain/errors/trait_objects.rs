//! Error: why a field that holds a trait object, or names an implementation, cannot be built.

use crate::domain::names::DependencyName;

/// What is wrong with how the field is built, worded after the field's name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TraitObjectProblem {
    /// A trait object held in anything but an `Arc`, built by no factory.
    WithoutFactory,
    /// An `Arc<dyn Trait>` that names no implementation and has no factory.
    WithoutImplementation,
    ImplementationWithoutTraitObject,
    ImplementationOutsideArc,
    ImplementationWithFactory,
}

impl TraitObjectProblem {
    /// The words after the field's name; a factory is offered by the
    /// method named after `field`.
    pub(crate) fn worded(self, field: &DependencyName) -> String {
        let factory = format!(
            "add build = factory to the field and build it in the build_{} method of the container's factories trait",
            field.without_raw_prefix()
        );
        match self {
            Self::WithoutFactory => {
                format!("has a trait-object type, which has no constructor; {factory}")
            }
            Self::WithoutImplementation => format!(
                "has a trait-object type, which has no constructor; name the type the container builds, such as implementation = StripeGateway, or {factory}"
            ),
            Self::ImplementationWithoutTraitObject => "names an implementation, but its type holds no trait object; write the trait object as the field's type, such as Arc<dyn PaymentGateway>, or remove implementation".to_owned(),
            Self::ImplementationOutsideArc => "names an implementation, but holds its trait object in something other than an Arc; write the field as an Arc of the trait object, such as Arc<dyn PaymentGateway>, or remove implementation and build the field with build = factory".to_owned(),
            Self::ImplementationWithFactory => "names an implementation and is built with build = factory, which returns the field itself; remove implementation, or remove build = factory so that the container builds the implementation".to_owned(),
        }
    }
}
