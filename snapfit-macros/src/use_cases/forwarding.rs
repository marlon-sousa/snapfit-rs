//! Policy: whether a `match` can forward an item of a dispatched trait, and how.

use crate::domain::{
    AssociatedTypeShape, DispatchProblem, DomainError, DomainErrors, ForwardedItem, ItemPosition,
    ItemProblem, MethodShape, PlacedItem, Rebuilding, Receiver, ReturnShape, TraitItem,
    TraitItemDeclaration, TraitName, TypeParameters,
};

#[cfg(test)]
mod tests;

/// The item as the impls forward it, or every reason they cannot.
pub(crate) fn forwarding<M, A>(
    trait_name: &TraitName,
    placed: PlacedItem<M, A>,
) -> Result<ForwardedItem<M, A>, DomainErrors> {
    let position = placed.position;
    match placed.declaration {
        TraitItemDeclaration::Method {
            name,
            syntax,
            shape,
        } => {
            let item = TraitItem::Method(name);
            DomainErrors::check(refusals(
                trait_name,
                &item,
                position,
                method_problems(shape),
            ))?;
            Ok(ForwardedItem::Method {
                syntax,
                rebuilding: rebuilding(shape.returns),
                execution: shape.execution,
                safety: shape.safety,
            })
        }
        TraitItemDeclaration::AssociatedType {
            name,
            syntax,
            shape,
        } => {
            let item = TraitItem::AssociatedType(name);
            DomainErrors::check(refusals(trait_name, &item, position, type_problems(shape)))?;
            Ok(ForwardedItem::AssociatedType(syntax))
        }
        TraitItemDeclaration::AssociatedConst(name) => {
            let item = TraitItem::AssociatedConst(name);
            Err(refusal(trait_name, &item, position, ItemProblem::AssociatedConst).into())
        }
        TraitItemDeclaration::Macro(name) => {
            let item = TraitItem::Macro(name);
            Err(refusal(trait_name, &item, position, ItemProblem::Macro).into())
        }
    }
}

/// Every reason a method's signature cannot be forwarded, in the order it is written.
fn method_problems(shape: MethodShape) -> impl Iterator<Item = ItemProblem> {
    let receiver = match shape.receiver {
        Receiver::None => Some(ItemProblem::NoReceiver),
        Receiver::Other => Some(ItemProblem::OtherReceiver),
        Receiver::Value | Receiver::Reference => None,
    };
    let self_use = shape
        .self_use
        .is_elsewhere()
        .then_some(ItemProblem::SelfElsewhere);
    let self_bound = shape
        .self_bound
        .is_other()
        .then_some(ItemProblem::SelfBound);
    let returns = shape
        .returns
        .is_impl_trait()
        .then_some(ItemProblem::ImplTraitReturn);
    receiver
        .into_iter()
        .chain(self_use)
        .chain(self_bound)
        .chain(returns)
}

/// Every reason an associated type cannot be forwarded.
fn type_problems(shape: AssociatedTypeShape) -> impl Iterator<Item = ItemProblem> {
    let parameters = match shape.parameters {
        TypeParameters::None => None,
        TypeParameters::Declared => Some(ItemProblem::GenericAssociatedType),
    };
    let self_use = shape
        .self_use
        .is_elsewhere()
        .then_some(ItemProblem::SelfInBounds);
    let presence = shape
        .presence
        .is_conditional()
        .then_some(ItemProblem::ConditionalAssociatedType);
    parameters.into_iter().chain(self_use).chain(presence)
}

fn rebuilding(returns: ReturnShape) -> Rebuilding {
    match returns {
        ReturnShape::ItSelf => Rebuilding::IntoVariant,
        ReturnShape::Plain | ReturnShape::ImplTrait => Rebuilding::None,
    }
}

fn refusals(
    trait_name: &TraitName,
    item: &TraitItem,
    position: ItemPosition,
    problems: impl Iterator<Item = ItemProblem>,
) -> Vec<DomainError> {
    problems
        .map(|problem| refusal(trait_name, item, position, problem))
        .collect()
}

fn refusal(
    trait_name: &TraitName,
    item: &TraitItem,
    position: ItemPosition,
    problem: ItemProblem,
) -> DomainError {
    DomainError::DispatchedTrait(DispatchProblem::Item {
        trait_name: trait_name.clone(),
        item: item.clone(),
        position,
        problem,
    })
}
