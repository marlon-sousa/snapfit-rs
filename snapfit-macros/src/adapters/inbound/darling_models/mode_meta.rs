//! Adapter: the value of `mode = ...` on `#[container]`, as darling reads it.

use darling::FromMeta;

/// The mode as written. The adapter checks it, so that its error names the
/// mode without darling's location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::adapters::inbound) enum ModeMeta {
    /// The string's own text, such as `sync`.
    Written(String),
    /// `mode` with no value.
    Bare,
}

impl FromMeta for ModeMeta {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self::Written(value.to_owned()))
    }

    /// A setting with no value, which darling would refuse in its own words.
    fn from_word() -> darling::Result<Self> {
        Ok(Self::Bare)
    }
}
