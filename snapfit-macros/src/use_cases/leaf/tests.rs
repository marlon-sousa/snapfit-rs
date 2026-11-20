use super::*;
use crate::domain::{Construction, Constructor, Execution};
use crate::test_support::dependency;

#[test]
fn a_leaf_is_a_synchronous_field_with_no_deps_and_no_build_of_its_own() {
    assert!(is_leaf(&dependency("config", &[])));

    assert!(!is_leaf(&dependency("db", &["config"])));

    let mut own_build = dependency("config", &[]);
    own_build.construction = Some(Construction::Constructor(Constructor::New));
    assert!(!is_leaf(&own_build));

    let mut awaited = dependency("config", &[]);
    awaited.execution = Execution::Async;
    assert!(!is_leaf(&awaited));
}
