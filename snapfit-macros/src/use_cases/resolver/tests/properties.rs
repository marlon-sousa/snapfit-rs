//! Tests: for any graph of fields, the resolver orders every field after
//! its prerequisites, or reports only cycles the graph really has.

use proptest::collection::vec;
use proptest::prelude::*;

use super::*;

/// Each field's prerequisites, by the position of the field they name.
type Graph = Vec<Vec<usize>>;

const MOST_FIELDS: usize = 8;
const MOST_PREREQUISITES: usize = 3;

fn field_name(position: usize) -> String {
    format!("field_{position}")
}

/// A graph of one to `MOST_FIELDS` fields, each naming up to
/// `MOST_PREREQUISITES` of them, itself and repeats included.
fn graphs() -> impl Strategy<Value = Graph> {
    (1..=MOST_FIELDS).prop_flat_map(graph_of)
}

fn graph_of(fields: usize) -> impl Strategy<Value = Graph> {
    vec(vec(0..fields, 0..=MOST_PREREQUISITES), fields)
}

/// A graph with no cycle, in any declaration order: each field names only
/// fields of a lower rank, and the ranks are shuffled into positions.
fn acyclic_graphs() -> impl Strategy<Value = Graph> {
    (1..=MOST_FIELDS).prop_flat_map(ranked_graph_of)
}

fn ranked_graph_of(fields: usize) -> impl Strategy<Value = Graph> {
    let ranks = vec(vec(0..MOST_FIELDS, 0..=MOST_PREREQUISITES), fields);
    let positions = Just((0..fields).collect::<Vec<_>>()).prop_shuffle();
    (ranks, positions).prop_map(placed_by_rank)
}

/// Rank `rank`'s prerequisites are the ranks below it, picked by `choices`;
/// rank `rank` is declared at `positions[rank]`.
fn placed_by_rank((choices, positions): (Vec<Vec<usize>>, Vec<usize>)) -> Graph {
    let mut graph = vec![Vec::new(); positions.len()];
    for (rank, picks) in choices.iter().enumerate().skip(1) {
        graph[positions[rank]] = picks.iter().map(|pick| positions[pick % rank]).collect();
    }
    graph
}

fn declarations(graph: &Graph) -> Vec<DependencyDeclaration<&'static str>> {
    graph.iter().enumerate().map(declaration).collect()
}

fn declaration(
    (position, prerequisites): (usize, &Vec<usize>),
) -> DependencyDeclaration<&'static str> {
    let prerequisite_names: Vec<String> = prerequisites.iter().copied().map(field_name).collect();
    let prerequisite_names: Vec<&str> = prerequisite_names.iter().map(String::as_str).collect();
    dependency(&field_name(position), &prerequisite_names)
}

/// Whether the graph has no cycle, found apart from the resolver: fields
/// whose prerequisites are all placed are placed until none is left to place.
fn is_acyclic(graph: &Graph) -> bool {
    let mut placed = vec![false; graph.len()];
    while let Some(next) = (0..graph.len()).find(|&field| can_place(graph, &placed, field)) {
        placed[next] = true;
    }
    placed.into_iter().all(|field| field)
}

fn can_place(graph: &Graph, placed: &[bool], field: usize) -> bool {
    !placed[field]
        && graph[field]
            .iter()
            .all(|&prerequisite| placed[prerequisite])
}

/// Each field's name with the name of each field it needs.
fn edges(graph: &Graph) -> HashSet<(String, String)> {
    graph
        .iter()
        .enumerate()
        .flat_map(|(position, prerequisites)| {
            prerequisites
                .iter()
                .map(move |&needed| (field_name(position), field_name(needed)))
        })
        .collect()
}

proptest! {
    #[test]
    fn a_graph_is_ordered_exactly_when_it_has_no_cycle(graph in graphs()) {
        prop_assert_eq!(resolve(container(declarations(&graph))).is_ok(), is_acyclic(&graph));
    }

    #[test]
    fn every_field_is_built_once_and_after_each_of_its_prerequisites(graph in acyclic_graphs()) {
        let resolved = resolve(container(declarations(&graph))).unwrap();
        let order: Vec<String> = step_names(&resolved.into_plan(Execution::Sync).construction_order)
            .into_iter()
            .map(str::to_string)
            .collect();
        let expected: HashSet<String> = (0..graph.len()).map(field_name).collect();

        prop_assert_eq!(order.len(), graph.len());
        prop_assert_eq!(order.iter().cloned().collect::<HashSet<_>>(), expected);
        for (field, prerequisite) in edges(&graph) {
            let built_at = order.iter().position(|name| *name == field).unwrap();
            let needed_at = order.iter().position(|name| *name == prerequisite).unwrap();
            prop_assert!(needed_at < built_at, "{field} is built before {prerequisite}");
        }
    }

    #[test]
    fn every_cycle_reported_is_a_cycle_of_the_graph(graph in graphs()) {
        let Err(errors) = resolve(container(declarations(&graph))) else {
            return Ok(());
        };
        let edges = edges(&graph);
        for error in errors {
            let DomainError::DependencyCycle { start, through } = error else {
                return Err(TestCaseError::fail(format!("not a cycle: {error}")));
            };
            let members: Vec<String> = std::iter::once(&start)
                .chain(&through)
                .map(|member| member.as_str().to_string())
                .collect();
            let unique: HashSet<&String> = members.iter().collect();
            prop_assert_eq!(unique.len(), members.len());
            for (field, prerequisite) in members.iter().zip(members.iter().cycle().skip(1)) {
                prop_assert!(
                    edges.contains(&(field.clone(), prerequisite.clone())),
                    "{field} does not need {prerequisite}"
                );
            }
        }
    }
}
