use ahash::{HashSet, HashSetExt};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// A task. Contains the processing time and weight of the task.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Serialize, PartialEq)]
pub struct Task {
    pub time: u64,
    pub weight: u64,
}

/// A conflict between two tasks described by their indices.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Serialize, PartialEq)]
pub struct Conflict(usize, usize);

impl Conflict {
    /// Creates a new conflict between two tasks.
    #[must_use]
    pub const fn new(first: usize, second: usize) -> Self {
        Self(first, second)
    }
}

/// A conflict graph. Contains an edge for every pair of tasks that conflict.
#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(from = "Vec<Conflict>", into = "Vec<Conflict>")]
pub struct ConflictGraph {
    edges: Vec<HashSet<usize>>,
}

impl ConflictGraph {
    /// Returns whether the given tasks conflict.
    #[must_use]
    pub fn are_conflicted(&self, first: usize, second: usize) -> bool {
        self.edges
            .get(first)
            .map_or(false, |conflicts| conflicts.contains(&second))
    }

    /// Returns the conflicts of the given task.
    #[must_use]
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

        Self { edges }
    }
}

impl From<ConflictGraph> for Vec<Conflict> {
    fn from(conflicts: ConflictGraph) -> Self {
        let mut result = Self::new();

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

impl Instance {
    /// Creates a new instance of the scheduling problem without conflicts.
    #[must_use]
    pub const fn new_no_conflict(processors: usize, deadline: u64, tasks: Vec<Task>) -> Self {
        Self {
            processors,
            deadline,
            tasks,
            graph: ConflictGraph { edges: Vec::new() },
        }
    }

    /// Creates a new instance of the scheduling problem with conflicts.
    #[must_use]
    pub fn new(
        processors: usize,
        deadline: u64,
        tasks: Vec<Task>,
        conflicts: Vec<Conflict>,
    ) -> Self {
        Self {
            processors,
            deadline,
            tasks,
            graph: ConflictGraph::from(conflicts),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn instance_should_serialize() -> anyhow::Result<()> {
        let instance = Instance {
            processors: 2,
            deadline: 10,
            tasks: vec![Task { time: 1, weight: 1 }, Task { time: 2, weight: 2 }],
            graph: ConflictGraph::from(vec![Conflict(0, 1)]),
        };

        let serialized = crate::data::to_string(&instance)?;
        let mut reader = std::io::Cursor::new(serialized);
        let deserialized: Instance = crate::data::deserialize(&mut reader)?;

        assert_eq!(instance, deserialized);

        Ok(())
    }
}
