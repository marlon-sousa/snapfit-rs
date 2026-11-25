//! Entity: what a field's type is, and which type the container builds for
//! it. Plays the role of its parent.

use strum::EnumIs;

use crate::domain::dispatched_type::Variant;

/// What the field's type is, as far as building it goes.
#[derive(EnumIs, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum TypeShape {
    #[default]
    Value,
    /// `Arc<T>`, whose `T` is the type the constructor belongs to.
    Arc,
    /// `Arc<dyn Trait>`, which holds the value of the implementation the
    /// field names.
    ArcTraitObject,
    /// Any other type whose value is a trait object, which has no
    /// constructor of its own.
    TraitObject,
    /// A value that keeps a trait object deeper in its type arguments, such
    /// as behind a pointer of its own, which its constructor takes.
    KeepsTraitObject,
    /// `Arc<T>` whose `T` keeps a trait object deeper in its type arguments.
    ArcKeepsTraitObject,
}

/// Which type the container builds a field's value as.
#[derive(EnumIs, Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum Implementation<T> {
    /// The field's own type, or `T` when the field holds `Arc<T>`.
    #[default]
    OfField,
    /// The type of `implementation = ...`, whose value the field's trait
    /// object, or its dispatched type, holds.
    Named(Variant<T>),
    /// The types of `implementation = [...]`, which a factory may return as
    /// the field's dispatched type.
    Listed(Vec<Variant<T>>),
}

impl<T> Implementation<T> {
    /// The types the field names, which its dispatched type holds.
    pub(crate) fn variants(&self) -> &[Variant<T>] {
        match self {
            Self::OfField => &[],
            Self::Named(variant) => std::slice::from_ref(variant),
            Self::Listed(variants) => variants,
        }
    }
}
