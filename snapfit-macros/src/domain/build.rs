//! Entity: which build of snapfit is running, the stable one or the one with
//! the `unstable` Cargo feature.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FeatureGate {
    Stable,
    Unstable,
}
