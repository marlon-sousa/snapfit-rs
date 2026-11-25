//! Error: the sentences of a static dispatch problem that depend on its
//! attribute or on the names a container declares. Plays the role of its parent.

use super::DispatchAttributeKind;
use crate::domain::errors::sentences::join_names;
use crate::domain::names::TypeName;

pub(super) fn example(kind: DispatchAttributeKind) -> &'static str {
    match kind {
        DispatchAttributeKind::Declaration => "static_dispatch_for(Database as Db)",
        DispatchAttributeKind::Extension => "extend_static_dispatch(Db, [Mock])",
    }
}

/// The names a container does declare, or that it declares none.
pub(super) fn undeclared(name: &TypeName, declared: &[TypeName]) -> String {
    let extended =
        format!("extend_static_dispatch extends '{name}', which this container does not declare");
    match declared {
        [] => {
            format!("{extended}; declare it first, such as static_dispatch_for(Database as {name})")
        }
        _ => format!(
            "{extended}; extend one of the types its static_dispatch_for declares: [{}]",
            join_names(declared)
        ),
    }
}
