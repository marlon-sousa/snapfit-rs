//! Policy: how a cycle found by the walk is told.

use super::walk::OnPath;
use crate::domain::{DomainError, GraphProblem};

/// `members` each need the next, and the last needs the first. The cycle is
/// told from its member declared first, wherever the walk entered it.
pub(super) fn cycle_through<'a>(members: impl Iterator<Item = &'a OnPath>) -> Option<DomainError> {
    let members: Vec<&OnPath> = members.collect();
    let first = members
        .iter()
        .enumerate()
        .min_by_key(|(_, member)| member.position)
        .map_or(0, |(index, _)| index);
    let mut cycle = members
        .iter()
        .cycle()
        .skip(first)
        .take(members.len())
        .map(|member| member.name.clone());
    let start = cycle.next()?;
    Some(
        GraphProblem::Cycle {
            start,
            through: cycle.collect(),
        }
        .into(),
    )
}
