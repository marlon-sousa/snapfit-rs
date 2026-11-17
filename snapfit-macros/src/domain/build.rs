//! Entity: which build of snapfit is running, the stable one or the one with
//! the `unstable` Cargo feature.

/// Planned features, with no code generation yet, are refused by every build.
/// Features whose API is not settled for 0.1.0 are accepted only by a build
/// with the `unstable` Cargo feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FeatureGate {
    Stable,
    Unstable,
}
