//! Error: why one setting of `#[snapfit::static_dispatch]` is refused. Plays
//! the role of its parent.

use thiserror::Error;

use super::super::{ArgumentProblem, FeatureProblem};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum SettingProblem {
    #[error(
        "#[snapfit::static_dispatch] takes no arguments but crate, given once, the path to snapfit when a crate depends on it under another name; write it alone, as #[snapfit::static_dispatch], or as #[snapfit::static_dispatch(crate = di)]"
    )]
    Unknown,
    /// What is wrong with the value of a `crate` setting, in the words the
    /// container gives it.
    #[error("{0}")]
    Runtime(ArgumentProblem),
    /// A setting snapfit plans, such as `remote`.
    #[error(transparent)]
    Planned(FeatureProblem),
}
