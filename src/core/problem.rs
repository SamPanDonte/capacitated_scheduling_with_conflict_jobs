use ahash::{HashSet, HashSetExt};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// A task. Contains the processing time and weight of the task.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Serialize, PartialEq)]
pub struct Task {
    pub time: u64,
    pub weight: u64,
}

/// A conflict between two tasks described by their indices.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct Conflict(usize, usize);

/// A conflict graph. Contains an edge for every pair of tasks that conflict.
#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(from = "Vec<Conflict>", into = "Vec<Conflict>")]
pub struct ConflictGraph {
    edges: Vec<HashSet<usize>>,
}

impl ConflictGraph {
    /// Returns whether the given tasks conflict.
    pub fn are_conflicted(&self, first: usize, second: usize) -> bool {
        self.edges
            .get(first)
            .map(|conflicts| conflicts.contains(&second))
            .unwrap_or(false)
    }

    /// Returns the conflicts of the given task.
    pub fn conflicts(&self, task: usize) -> &HashSet<usize> {
        static EMPTY: LazyLock<HashSet<usize>> = LazyLock::new(HashSet::new);

        self.edges.get(task).unwrap_or(&EMPTY)
    }
}

impl From<Vec<Conflict>> for ConflictGraph {
    fn from(conflicts: Vec<Conflict>) -> Self {
        let mut edges = Vec::new();

        for conflict in conflicts {
            while edges.len() <= conflict.0.max(conflict.1) {
                edges.push(HashSet::new());
            }

            edges[conflict.0].insert(conflict.1);
            edges[conflict.1].insert(conflict.0);
        }

        ConflictGraph { edges }
    }
}

impl From<ConflictGraph> for Vec<Conflict> {
    fn from(conflicts: ConflictGraph) -> Self {
        let mut result = Vec::new();

        for (from_vertex, adjacent_vertices) in conflicts.edges.into_iter().enumerate() {
            for to_vertex in adjacent_vertices {
                if to_vertex > from_vertex {
                    result.push(Conflict(from_vertex, to_vertex));
                }
            }
        }

        result
    }
}

/// An instance of the scheduling problem.
#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
pub struct Instance {
    pub processors: usize,
    pub deadline: u64,
    pub tasks: Vec<Task>,
    pub graph: ConflictGraph,
}
