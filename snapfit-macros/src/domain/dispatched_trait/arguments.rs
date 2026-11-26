//! Entity: the settings `#[snapfit::static_dispatch]` was given between its
//! parentheses, each as the user wrote it. Plays the role of its parent.

use derive_new::new;
use strum::EnumIs;

/// Every setting the attribute was given, in the order written.
#[derive(new, Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct DispatchArguments {
    pub(crate) settings: Vec<DispatchSetting>,
}

/// One setting of the attribute, between two commas.
#[derive(EnumIs, Debug, Clone, PartialEq, Eq)]
pub(crate) enum DispatchSetting {
    /// `crate = ...`, the path to snapfit.
    Runtime(RuntimeValue),
    /// `remote = <path>`, the trait of another crate it would copy.
    Remote,
    /// Anything else, such as `other` or `crate =` with nothing after it.
    Other,
}

/// What `crate = ...` was given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeValue {
    /// A path of modules, such as `di` or `::shop::di`.
    Path,
    /// A string, with its own text, such as `di`.
    Quoted(String),
    /// Anything else, as written, such as `3`.
    NotAPath(String),
    /// No value: `crate` alone.
    Bare,
}

/// Where a setting is among the attribute's settings, counted from 0, which
/// the diagnostic about it points at.
#[derive(new, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SettingPosition(pub(crate) usize);
