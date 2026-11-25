//! Policy: whether a `match` can forward an item of a dispatched trait, and how.

use crate::domain::{
    DispatchProblem, DomainError, DomainErrors, ForwardedItem, ItemProblem, MethodShape,
    Rebuilding, Receiver, ReturnShape, TraitItem, TraitItemDeclaration, TraitName, TypeParameters,
};

#[cfg(test)]
mod tests;

/// The item as the impls forward it, or every reason they cannot.
pub(crate) fn forwarding<M, A>(
    trait_name: &TraitName,
    declaration: TraitItemDeclaration<M, A>,
) -> Result<ForwardedItem<M, A>, DomainErrors> {
    match declaration {
        TraitItemDeclaration::Method {
            name,
            syntax,
            shape,
        } => {
            let item = TraitItem::Method(name);
            let problems = method_problems(shape)
                .map(|problem| refusal(trait_name, &item, problem))
                .collect();
            DomainErrors::check(problems)?;
            Ok(ForwardedItem::Method {
                syntax,
                rebuilding: rebuilding(shape.returns),
            })
        }
        TraitItemDeclaration::AssociatedType {
            name,
            syntax,
            parameters,
        } => match parameters {
            TypeParameters::None => Ok(ForwardedItem::AssociatedType(syntax)),
            TypeParameters::Declared => Err(refusal(
                trait_name,
                &TraitItem::AssociatedType(name),
                ItemProblem::GenericAssociatedType,
            )
            .into()),
        },
        TraitItemDeclaration::AssociatedConst(name) => Err(refusal(
            trait_name,
            &TraitItem::AssociatedConst(name),
            ItemProblem::AssociatedConst,
        )
        .into()),
        TraitItemDeclaration::Macro(name) => {
            Err(refusal(trait_name, &TraitItem::Macro(name), ItemProblem::Macro).into())
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
    let returns = shape
        .returns
        .is_impl_trait()
        .then_some(ItemProblem::ImplTraitReturn);
    receiver.into_iter().chain(self_use).chain(returns)
}

fn rebuilding(returns: ReturnShape) -> Rebuilding {
    match returns {
        ReturnShape::ItSelf => Rebuilding::IntoVariant,
        ReturnShape::Plain | ReturnShape::ImplTrait => Rebuilding::None,
    }
}

fn refusal(trait_name: &TraitName, item: &TraitItem, problem: ItemProblem) -> DomainError {
    DomainError::DispatchedTrait(DispatchProblem::Item {
        trait_name: trait_name.clone(),
        item: item.clone(),
        problem,
    })
}
