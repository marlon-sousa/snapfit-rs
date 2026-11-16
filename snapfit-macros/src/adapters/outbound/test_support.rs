use crate::adapters::SynParserAdapter;
use crate::domain::ConstructionPlan;
use crate::ports::{ContainerParserPort, ContainerSyntax};
use crate::use_cases::DeriveContainerUseCase;

/// Runs the real parser and use case so emitter tests start from the same
/// plan and syntax the macro would see.
pub(super) fn plan_for(item: syn::ItemStruct) -> (ConstructionPlan, ContainerSyntax) {
    let ir = SynParserAdapter::new().parse_container(&item).unwrap();
    let plan = DeriveContainerUseCase::new().execute(ir).unwrap();
    let syntax = ContainerSyntax::new(
        &item.ident,
        &item.vis,
        &item.attrs,
        &item.generics,
        &item.fields,
    );
    (plan, syntax)
}

pub(super) fn compact(tokens: impl ToString) -> String {
    tokens.to_string().split_whitespace().collect()
}
