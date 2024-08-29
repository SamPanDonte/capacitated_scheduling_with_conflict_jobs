use std::iter::{once, repeat};

/// A weighted graph.
#[derive(Clone, Debug, Default)]
pub struct Graph {
    max_weight: i128,
    edges: Vec<(usize, usize, i128)>,
    endpoints: Vec<usize>,
    neighbors: Vec<Vec<usize>>,
}

impl Graph {
    /// Adds an edge to the graph.
    pub fn add_edge(&mut self, from: usize, to: usize, weight: impl Into<i128>) {
        let weight = weight.into();

        self.max_weight = self.max_weight.max(weight);

        let max_vertex = from.max(to);
        if max_vertex >= self.neighbors.len() {
            self.neighbors.resize(max_vertex + 1, Vec::new());
        }

        self.edges.push((from, to, weight));
        self.neighbors[to].push(self.endpoints.len());
        self.endpoints.push(from);
        self.neighbors[from].push(self.endpoints.len());
        self.endpoints.push(to);
    }

    /// Returns whether the graph is empty (has no edges).
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Returns the number of vertices in the graph.
    pub fn vertex_count(&self) -> usize {
        self.neighbors.len()
    }

    /// Returns the max weight of edges in the graph.
    pub const fn max_weight(&self) -> i128 {
        self.max_weight
    }
}

/// Find the maximum weighted matching in a graph.
/// Has a time complexity of `O(n^3)`.
///
/// Arguments:
/// - `graph`: The graph to find the matching in.
/// - `max_card`: Whether to find the maximum cardinality matching or the maximum weight matching.
pub fn gabow_algo(graph: &Graph, max_cardinality: bool) -> Vec<Option<usize>> {
    if graph.is_empty() {
        return Vec::new();
    }

    let n = graph.vertex_count();
    let w = graph.max_weight();

    let algorithm = Algorithm {
        graph,
        mate: vec![None; n],
        label: vec![0; n * 2],
        label_end: vec![None; n * 2],
        blossom: (0..n).collect(),
        blossom_parent: vec![None; n * 2],
        blossom_children: vec![None; n * 2],
        blossom_base: (0..n).map(Some).chain(repeat(None).take(n)).collect(),
        blossom_endpoints: vec![None; n * 2],
        best_edge: vec![None; n * 2],
        blossom_best_edges: vec![None; n * 2],
        unused_blossom: (n..n * 2).collect(),
        dual_var: repeat(w).take(n).chain(repeat(0).take(n)).collect(),
        allow_edge: vec![false; graph.edges.len()],
        queue: Vec::new(),
    };

    algorithm.run(max_cardinality)
}

#[derive(Debug)]
struct Algorithm<'a> {
    graph: &'a Graph,
    mate: Vec<Option<usize>>,
    label: Vec<isize>,
    label_end: Vec<Option<usize>>,
    blossom: Vec<usize>,
    blossom_parent: Vec<Option<usize>>,
    blossom_children: Vec<Option<Vec<usize>>>,
    blossom_base: Vec<Option<usize>>,
    blossom_endpoints: Vec<Option<Vec<usize>>>,
    best_edge: Vec<Option<usize>>,
    blossom_best_edges: Vec<Option<Vec<usize>>>,
    unused_blossom: Vec<usize>,
    dual_var: Vec<i128>,
    allow_edge: Vec<bool>,
    queue: Vec<usize>,
}

impl<'a> Algorithm<'a> {
    fn slack(&self, k: usize) -> i128 {
        let edge = &self.graph.edges[k];
        self.dual_var[edge.0] + self.dual_var[edge.1] - 2 * edge.2
    }

    fn assign_label(&mut self, vertex: usize, label: isize, endpoint: Option<usize>) {
        let blossom = self.blossom[vertex];

        self.label[vertex] = label;
        self.label[blossom] = label;
        self.label_end[vertex] = endpoint;
        self.label_end[blossom] = endpoint;
        self.best_edge[vertex] = None;
        self.best_edge[blossom] = None;

        if label == 1 {
            let leaves = blossom_leaves(blossom, self.graph.vertex_count(), &self.blossom_children);
            self.queue.extend(leaves);
        } else if label == 2 {
            let base = self.blossom_base[blossom].unwrap_or_else(cannot_happen);
            let mate = self.mate[base].unwrap_or_else(cannot_happen);
            self.assign_label(self.graph.endpoints[mate], 1, Some(mate ^ 1));
        }
    }

    fn scan_blossom(&mut self, v: usize, w: usize) -> Option<usize> {
        let mut path = Vec::new();
        let mut base = None;
        let mut v = Some(v);
        let mut w = Some(w);

        while let Some(vertex) = v {
            let b = self.blossom[vertex];
            if self.label[b] & 4 != 0 {
                base = self.blossom_base[b];
                break;
            }

            path.push(b);
            self.label[b] = 5;

            if let Some(endpoint) = self.label_end[b] {
                let b = self.blossom[self.graph.endpoints[endpoint]];
                v = Some(self.graph.endpoints[self.label_end[b].unwrap_or_else(cannot_happen)]);
            } else {
                v = None;
            }

            if w.is_some() {
                std::mem::swap(&mut v, &mut w);
            }
        }

        for b in path {
            self.label[b] = 1;
        }

        base
    }

    fn add_blossom(&mut self, base: usize, edge: usize) {
        let (mut v, mut w, _) = self.graph.edges[edge];
        let bb = self.blossom[base];
        let mut bv = self.blossom[v];
        let mut bw = self.blossom[w];

        let blossom = self.unused_blossom.pop().unwrap_or_else(cannot_happen);

        self.blossom_base[blossom] = Some(base);
        self.blossom_parent[blossom] = None;
        self.blossom_parent[bb] = Some(blossom);

        let mut path = Vec::new();
        let mut endpoints = Vec::new();

        while bv != bb {
            self.blossom_parent[bv] = Some(blossom);
            path.push(bv);

            let endpoint = self.label_end[bv].unwrap_or_else(cannot_happen);
            endpoints.push(endpoint);

            v = self.graph.endpoints[endpoint];
            bv = self.blossom[v];
        }

        path.push(bb);
        path.reverse();
        endpoints.reverse();
        endpoints.push(2 * edge);

        while bw != bb {
            self.blossom_parent[bw] = Some(blossom);
            path.push(bw);

            let endpoint = self.label_end[bw].unwrap_or_else(cannot_happen);
            endpoints.push(endpoint ^ 1);

            w = self.graph.endpoints[endpoint];
            bw = self.blossom[w];
        }

        self.label[blossom] = 1;
        self.label_end[blossom] = self.label_end[bb];
        self.dual_var[blossom] = 0;

        self.blossom_children[blossom] = Some(path);
        self.blossom_endpoints[blossom] = Some(endpoints);

        for v in blossom_leaves(blossom, self.graph.vertex_count(), &self.blossom_children) {
            if self.label[self.blossom[v]] == 2 {
                self.queue.push(v);
            }
            self.blossom[v] = blossom;
        }

        let mut best_edge_to = vec![None; self.graph.vertex_count() * 2];

        let path = self.blossom_children[blossom].as_ref();
        for &bv in path.unwrap_or_else(cannot_happen) {
            let neighbors = if let Some(list) = self.blossom_best_edges[bv].clone() {
                vec![list]
            } else {
                blossom_leaves(bv, self.graph.vertex_count(), &self.blossom_children)
                    .map(|v| self.graph.neighbors[v].iter().map(|&p| p / 2).collect())
                    .collect()
            };

            for neighbor in neighbors {
                for k in neighbor {
                    let (mut i, mut j, _) = self.graph.edges[k];
                    if self.blossom[j] == blossom {
                        std::mem::swap(&mut i, &mut j);
                    }
                    let bj = self.blossom[j];
                    if bj != blossom
                        && self.label[bj] == 1
                        && !best_edge_to[bj].is_some_and(|x| self.slack(k) >= self.slack(x))
                    {
                        best_edge_to[bj] = Some(k);
                    }
                }
            }

            self.blossom_best_edges[bv] = None;
            self.best_edge[bv] = None;
        }

        self.best_edge[blossom] = None;
        let blossom_best_edges = best_edge_to.into_iter().flatten().collect();
        for &k in &blossom_best_edges {
            if !self.best_edge[blossom].is_some_and(|x| self.slack(k) >= self.slack(x)) {
                self.best_edge[blossom] = Some(k);
            }
        }
        self.blossom_best_edges[blossom] = Some(blossom_best_edges);
    }

    fn expand_blossom(&mut self, b: usize, end_stage: bool) {
        // maybe using Option::take() would be better
        let b_children = self.blossom_children[b].clone();
        let b_children = b_children.unwrap_or_else(cannot_happen);
        for &s in &b_children {
            self.blossom_parent[s] = None;
            if s < self.graph.vertex_count() {
                self.blossom[s] = s;
            } else if end_stage && self.dual_var[s] == 0 {
                self.expand_blossom(s, end_stage);
            } else {
                for v in blossom_leaves(s, self.graph.vertex_count(), &self.blossom_children) {
                    self.blossom[v] = s;
                }
            }
        }

        if !end_stage && self.label[b] == 2 {
            let endpoint = self.label_end[b].unwrap_or_else(cannot_happen);
            let entry_child = self.blossom[self.graph.endpoints[endpoint ^ 1]];

            let j = b_children.iter().position(|&x| x == entry_child);
            let mut j = j.unwrap_or_else(cannot_happen);
            let (rev, end_ptr) = if j & 1 != 0 { (false, 0) } else { (true, 1) };

            let mut p = self.label_end[b].unwrap_or_else(cannot_happen);

            let b_end = self.blossom_endpoints[b].take();
            let b_end = b_end.unwrap_or_else(cannot_happen);

            while j != 0 {
                self.label[self.graph.endpoints[p ^ 1]] = 0;
                self.label[self.graph.endpoints[b_children[j - end_ptr] ^ end_ptr ^ 1]] = 0;
                self.assign_label(self.graph.endpoints[p ^ 1], 2, Some(p));
                self.allow_edge[b_end[j - end_ptr] / 2] = true;
                j = if rev { j - 1 } else { j + 1 } % b_children.len();

                p = b_end[j - end_ptr] ^ end_ptr;
                self.allow_edge[p / 2] = true;
                j = if rev { j - 1 } else { j + 1 } % b_children.len();
            }

            let mut bv = b_children[j];

            self.label[self.graph.endpoints[p ^ 1]] = 2;
            self.label[bv] = 2;
            self.label_end[self.graph.endpoints[p ^ 1]] = Some(p);
            self.label_end[bv] = Some(p);
            self.best_edge[bv] = None;

            j = if rev { b_children.len() - 1 } else { 1 };
            while b_children[j] != entry_child {
                bv = b_children[j];
                if self.label[bv] == 1 {
                    j = if rev { j - 1 } else { j + 1 } % b_children.len();
                    continue;
                }

                let mut v = 0;
                for v2 in blossom_leaves(bv, self.graph.vertex_count(), &self.blossom_children) {
                    v = v2;
                    if self.label[v2] != 0 {
                        break;
                    }
                }

                if self.label[v] != 0 {
                    self.label[v] = 0;
                    let index = self.blossom_base[bv].and_then(|x| self.mate[x]);
                    self.label[self.graph.endpoints[index.unwrap_or_else(cannot_happen)]] = 0;
                    self.assign_label(v, 2, self.label_end[v]);
                }

                j = if rev { j - 1 } else { j + 1 } % b_children.len();
                j %= b_children.len();
            }
        }

        self.label[b] = -1;
        self.label_end[b] = None;
        self.blossom_children[b] = None;
        self.blossom_endpoints[b] = None;
        self.blossom_base[b] = None;
        self.blossom_best_edges[b] = None;
        self.best_edge[b] = None;
        self.unused_blossom.push(b);
    }

    fn augment_blossom(&mut self, blossom: usize, vertex: usize) {
        let mut t = vertex;
        while self.blossom_parent[t] != Some(blossom) {
            t = self.blossom_parent[t].unwrap_or_else(cannot_happen);
        }

        if t >= self.graph.vertex_count() {
            self.augment_blossom(t, vertex);
        }

        let blossom_len = self.blossom_children[blossom].as_ref();
        let blossom_len = blossom_len.unwrap_or_else(cannot_happen).len();

        let i = self.blossom_children[blossom]
            .as_ref()
            .map(std::iter::IntoIterator::into_iter)
            .and_then(|mut x| x.position(|&x| x == t))
            .unwrap_or_else(cannot_happen);
        let mut j = i;
        let (rev, end_ptr) = if j & 1 != 0 { (false, 0) } else { (true, 1) };
        while j != 0 {
            j = if rev { j - 1 } else { j + 1 } % blossom_len;
            t = self.blossom_children[blossom]
                .as_ref()
                .unwrap_or_else(cannot_happen)[j];
            let p = self.blossom_endpoints[blossom]
                .as_ref()
                .unwrap_or_else(cannot_happen)[(j + blossom_len - end_ptr) % blossom_len]
                ^ end_ptr;
            if t >= self.graph.vertex_count() {
                self.augment_blossom(t, self.graph.endpoints[p]);
            }

            j = if rev { j - 1 } else { j + 1 } % blossom_len;

            t = self.blossom_children[blossom]
                .as_ref()
                .unwrap_or_else(cannot_happen)[j];
            if t >= self.graph.vertex_count() {
                self.augment_blossom(t, self.graph.endpoints[p ^ 1]);
            }

            self.mate[self.graph.endpoints[p]] = Some(p ^ 1);
            self.mate[self.graph.endpoints[p ^ 1]] = Some(p);
        }

        self.blossom_children[blossom]
            .as_mut()
            .unwrap_or_else(cannot_happen)
            .rotate_left(i);
        self.blossom_endpoints[blossom]
            .as_mut()
            .unwrap_or_else(cannot_happen)
            .rotate_left(i);
        self.blossom_base[blossom] = self.blossom_base[self.blossom_children[blossom]
            .as_ref()
            .unwrap_or_else(cannot_happen)[0]];
    }

    fn augment_matching(&mut self, edge: usize) {
        let (v, w, _) = self.graph.edges[edge];
        for (mut s, mut p) in [(v, 2 * edge + 1), (w, 2 * edge)] {
            loop {
                let bs = self.blossom[s];
                if bs >= self.graph.vertex_count() {
                    self.augment_blossom(bs, s);
                }

                self.mate[s] = Some(p);

                let Some(bs_endpoint) = self.label_end[bs] else {
                    break;
                };

                let t = self.graph.endpoints[bs_endpoint];
                let bt = self.blossom[t];
                let t_endpoint = self.label_end[bt].unwrap_or_else(cannot_happen);

                s = self.graph.endpoints[t_endpoint];
                let j = self.graph.endpoints[t_endpoint ^ 1];

                if bt >= self.graph.vertex_count() {
                    self.augment_blossom(bt, j);
                }

                self.mate[j] = self.label_end[bt];

                p = t_endpoint ^ 1;
            }
        }
    }

    fn apply_delta(&mut self, max_cardinality: bool) -> DeltaType {
        let mut delta = if max_cardinality {
            None
        } else {
            let min = self.dual_var[..self.graph.vertex_count()].iter().min();
            Some((*min.unwrap_or_else(cannot_happen), DeltaType::Vertex))
        };

        for vertex in 0..self.graph.vertex_count() {
            if self.label[self.blossom[vertex]] == 0 {
                if let Some(edge) = self.best_edge[vertex] {
                    let d = self.slack(edge);
                    if delta.map_or(true, |(delta, _)| d < delta) {
                        delta = Some((d, DeltaType::Slack(edge)));
                    }
                }
            }
        }

        for b in 0..self.graph.vertex_count() * 2 {
            if self.label[b] == 1 && self.blossom_parent[b].is_none() {
                if let Some(edge) = self.best_edge[b] {
                    let d = self.slack(edge) / 2;
                    if delta.map_or(true, |(delta, _)| d < delta) {
                        delta = Some((d, DeltaType::HalfSlack(edge)));
                    }
                }
            }
        }

        for b in self.graph.vertex_count()..self.graph.vertex_count() * 2 {
            if self.label[b] == 2
                && self.blossom_parent[b].is_none()
                && self.blossom_base[b].is_some()
                && delta.map_or(true, |(delta, _)| self.dual_var[b] < delta)
            {
                delta = Some((self.dual_var[b], DeltaType::Blossom(b)));
            }
        }

        let (delta, delta_type) = delta.unwrap_or_else(|| {
            let min = self.dual_var[..self.graph.vertex_count()].iter().min();
            (0.max(*min.unwrap_or_else(cannot_happen)), DeltaType::Vertex)
        });

        for v in 0..self.graph.vertex_count() {
            if self.label[self.blossom[v]] == 1 {
                self.dual_var[v] -= delta;
            } else if self.label[self.blossom[v]] == 2 {
                self.dual_var[v] += delta;
            }
        }

        for b in self.graph.vertex_count()..self.graph.vertex_count() * 2 {
            if self.blossom_base[b].is_some() && self.blossom_parent[b].is_none() {
                if self.label[b] == 1 {
                    self.dual_var[b] += delta;
                } else if self.label[b] == 2 {
                    self.dual_var[b] -= delta;
                }
            }
        }

        delta_type
    }

    fn run(mut self, max_cardinality: bool) -> Vec<Option<usize>> {
        for _ in 0..self.graph.vertex_count() {
            self.label.fill(0);
            self.best_edge.fill(None);
            self.blossom_best_edges[self.graph.vertex_count()..].fill(None);
            self.allow_edge.fill(false);
            self.queue.clear();

            for v in 0..self.graph.vertex_count() {
                if self.mate[v].is_none() && self.label[self.blossom[v]] == 0 {
                    self.assign_label(v, 1, None);
                }
            }

            let mut augmented = false;
            loop {
                while let Some(v) = self.queue.pop() {
                    for &p in &self.graph.neighbors[v] {
                        let k = p / 2;
                        let w = self.graph.endpoints[p];
                        let k_slack = self.slack(k);
                        if self.blossom[v] == self.blossom[w] {
                            continue;
                        }
                        if !self.allow_edge[k] && k_slack <= 0 {
                            self.allow_edge[k] = true;
                        }
                        if self.allow_edge[k] {
                            if self.label[self.blossom[w]] == 0 {
                                self.assign_label(w, 2, Some(p ^ 1));
                            } else if self.label[self.blossom[w]] == 1 {
                                if let Some(base) = self.scan_blossom(v, w) {
                                    self.add_blossom(base, k);
                                } else {
                                    self.augment_matching(k);
                                    augmented = true;
                                    break;
                                }
                            } else if self.label[w] == 0 {
                                self.label[w] = 2;
                                self.label_end[w] = Some(p ^ 1);
                            }
                        } else if self.label[self.blossom[w]] == 1 {
                            let b = self.blossom[v];
                            if !self.best_edge[b].is_some_and(|x| k_slack >= self.slack(x)) {
                                self.best_edge[b] = Some(k);
                            }
                        } else if self.label[w] == 0
                            && !self.best_edge[w].is_some_and(|x| k_slack >= self.slack(x))
                        {
                            self.best_edge[w] = Some(k);
                        }
                    }

                    if augmented {
                        break;
                    }
                }

                if augmented {
                    break;
                }

                match self.apply_delta(max_cardinality) {
                    DeltaType::Vertex => break,
                    DeltaType::Slack(edge) => {
                        self.allow_edge[edge] = true;
                        let (i, j, _) = self.graph.edges[edge];

                        let label = self.label[self.blossom[i]];
                        self.queue.push(if label == 0 { j } else { i });
                    }
                    DeltaType::HalfSlack(edge) => {
                        self.allow_edge[edge] = true;
                        let (i, _, _) = self.graph.edges[edge];
                        self.queue.push(i);
                    }
                    DeltaType::Blossom(blossom) => {
                        self.expand_blossom(blossom, false);
                    }
                }
            }

            if !augmented {
                break;
            }

            for b in self.graph.vertex_count()..self.graph.vertex_count() * 2 {
                if self.blossom_parent[b].is_none()
                    && self.blossom_base[b].is_some()
                    && self.label[b] == 1
                    && self.dual_var[b] == 0
                {
                    self.expand_blossom(b, true);
                }
            }
        }

        for vertex in 0..self.graph.vertex_count() {
            if let Some(mate) = self.mate[vertex].as_mut() {
                *mate = self.graph.endpoints[*mate];
            }
        }

        self.mate
    }
}

fn cannot_happen<T>() -> T {
    unreachable!("This should not happen");
}

#[derive(Clone, Copy, Debug)]
enum DeltaType {
    Vertex,
    Slack(usize),
    HalfSlack(usize),
    Blossom(usize),
}

type BlossomIterator<'a> = Box<dyn Iterator<Item = usize> + 'a>;

fn blossom_leaves(b: usize, n: usize, children: &[Option<Vec<usize>>]) -> BlossomIterator {
    if b < n {
        Box::new(once(b))
    } else {
        let b_children = children[b].as_ref().unwrap_or_else(cannot_happen);
        Box::new(b_children.iter().flat_map(move |&t| {
            if t < n {
                Box::new(once(t))
            } else {
                blossom_leaves(t, n, children)
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! graph {
        () => {
            Graph::default()
        };
        ($(($from:expr, $to:expr, $weight:expr)),+) => {
            {
                let mut graph = Graph::default();
                $(graph.add_edge($from, $to, $weight);)*
                graph
            }
        };
    }

    macro_rules! opt {
        (-) => {
            None
        };
        ($x:expr) => {
            Some($x)
        };
    }

    macro_rules! mate {
        ($($x:tt),+) => {
            vec![$(opt!($x)),+]
        };
        () => {
            Vec::new()
        };
    }

    #[test]
    fn test_empty() {
        assert_eq!(gabow_algo(&graph![], false), mate![]);
    }

    #[test]
    fn test_single_edge() {
        assert_eq!(gabow_algo(&graph![(0, 1, 1)], false), mate![1, 0]);
    }

    #[test]
    fn test_1_2() {
        let graph = graph![(1, 2, 10), (2, 3, 11)];
        assert_eq!(gabow_algo(&graph, false), mate![-, -, 3, 2]);
    }

    #[test]
    fn test_1_3() {
        let graph = graph![(1, 2, 5), (2, 3, 11), (3, 4, 5)];
        assert_eq!(gabow_algo(&graph, false), mate![-, -, 3, 2, -]);
    }

    #[test]
    fn test14_maximum_cardinality() {
        let graph = graph![(1, 2, 5), (2, 3, 11), (3, 4, 5)];
        assert_eq!(gabow_algo(&graph, true), mate![-, 2, 1, 4, 3]);
    }

    #[test]
    fn test_negative() {
        let graph = graph![(1, 2, 2), (1, 3, -2), (2, 3, 1), (2, 4, -1), (3, 4, -6)];
        assert_eq!(gabow_algo(&graph, false), mate![-, 2, 1, -, -]);
        assert_eq!(gabow_algo(&graph, true), mate![-, 3, 4, 1, 2]);
    }

    #[test]
    fn test_s_blossom() {
        let mut graph = graph![(1, 2, 8), (1, 3, 9), (2, 3, 10), (3, 4, 7)];
        assert_eq!(gabow_algo(&graph, false), mate![-, 2, 1, 4, 3]);
        graph.add_edge(1, 6, 5);
        graph.add_edge(4, 5, 6);
        assert_eq!(gabow_algo(&graph, false), mate![-, 6, 3, 2, 5, 4, 1]);
    }

    #[test]
    fn test_t_blossom() {
        let mut base_graph = graph![(1, 2, 9), (1, 3, 8), (2, 3, 10), (1, 4, 5)];
        let mut graph = base_graph.clone();
        graph.add_edge(4, 5, 4);
        graph.add_edge(1, 6, 3);
        assert_eq!(gabow_algo(&graph, false), mate![-, 6, 3, 2, 5, 4, 1]);
        base_graph.add_edge(4, 5, 3);
        let mut graph = base_graph.clone();
        graph.add_edge(1, 6, 4);
        assert_eq!(gabow_algo(&graph, false), mate![-, 6, 3, 2, 5, 4, 1]);
        let mut graph = base_graph;
        graph.add_edge(3, 6, 4);
        assert_eq!(gabow_algo(&graph, false), mate![-, 2, 1, 6, 5, 4, 3]);
    }

    #[test]
    fn test_s_nest() {
        let graph = graph![
            (1, 2, 9),
            (1, 3, 9),
            (2, 3, 10),
            (2, 4, 8),
            (3, 5, 8),
            (4, 5, 10),
            (5, 6, 6)
        ];
        assert_eq!(gabow_algo(&graph, false), mate![-, 3, 4, 1, 2, 6, 5]);
    }

    #[test]
    fn test_s_relabel_nest() {
        let graph = graph![
            (1, 2, 10),
            (1, 7, 10),
            (2, 3, 12),
            (3, 4, 20),
            (3, 5, 20),
            (4, 5, 25),
            (5, 6, 10),
            (6, 7, 10),
            (7, 8, 8)
        ];
        let expected = mate![-, 2, 1, 4, 3, 6, 5, 8, 7];
        assert_eq!(gabow_algo(&graph, false), expected);
    }

    #[test]
    fn test_s_nest_expand() {
        let graph = graph![
            (1, 2, 8),
            (1, 3, 8),
            (2, 3, 10),
            (2, 4, 12),
            (3, 5, 12),
            (4, 5, 14),
            (4, 6, 12),
            (5, 7, 12),
            (6, 7, 14),
            (7, 8, 12)
        ];
        let expected = mate![-, 2, 1, 5, 6, 3, 4, 8, 7];
        assert_eq!(gabow_algo(&graph, false), expected);
    }

    #[test]
    fn test_s_t_expand() {
        let graph = graph![
            (1, 2, 23),
            (1, 5, 22),
            (1, 6, 15),
            (2, 3, 25),
            (3, 4, 22),
            (4, 5, 25),
            (4, 8, 14),
            (5, 7, 13)
        ];
        let expected = mate![-, 6, 3, 2, 8, 7, 1, 5, 4];
        assert_eq!(gabow_algo(&graph, false), expected);
    }

    #[test]
    fn test_s_nest_t_expand() {
        let graph = graph![
            (1, 2, 19),
            (1, 3, 20),
            (1, 8, 8),
            (2, 3, 25),
            (2, 4, 18),
            (3, 5, 18),
            (4, 5, 13),
            (4, 7, 7),
            (5, 6, 7)
        ];
        let expected = mate![-, 8, 3, 2, 7, 6, 5, 4, 1];
        assert_eq!(gabow_algo(&graph, false), expected);
    }

    #[test]
    fn test_t_nasty_expand() {
        let graph = graph![
            (1, 2, 45),
            (1, 5, 45),
            (2, 3, 50),
            (3, 4, 45),
            (4, 5, 50),
            (1, 6, 30),
            (3, 9, 35),
            (4, 8, 35),
            (5, 7, 26),
            (9, 10, 5)
        ];
        let expected = mate![-, 6, 3, 2, 8, 7, 1, 5, 4, 10, 9];
        assert_eq!(gabow_algo(&graph, false), expected);
    }

    #[test]
    fn test_t_nasty_2_expand() {
        let graph = graph![
            (1, 2, 45),
            (1, 5, 45),
            (2, 3, 50),
            (3, 4, 45),
            (4, 5, 50),
            (1, 6, 30),
            (3, 9, 35),
            (4, 8, 26),
            (5, 7, 40),
            (9, 10, 5)
        ];
        let expected = mate![-, 6, 3, 2, 8, 7, 1, 5, 4, 10, 9];
        assert_eq!(gabow_algo(&graph, false), expected);
    }

    #[test]
    fn test_t_expand_least_slack() {
        let graph = graph![
            (1, 2, 45),
            (1, 5, 45),
            (2, 3, 50),
            (3, 4, 45),
            (4, 5, 50),
            (1, 6, 30),
            (3, 9, 35),
            (4, 8, 28),
            (5, 7, 26),
            (9, 10, 5)
        ];
        let expected = mate![-, 6, 3, 2, 8, 7, 1, 5, 4, 10, 9];
        assert_eq!(gabow_algo(&graph, false), expected);
    }

    #[test]
    fn test_nest_t_nasty_expand() {
        let graph = graph![
            (1, 2, 45),
            (1, 7, 45),
            (2, 3, 50),
            (3, 4, 45),
            (4, 5, 95),
            (4, 6, 94),
            (5, 6, 94),
            (6, 7, 50),
            (1, 8, 30),
            (3, 11, 35),
            (5, 9, 36),
            (7, 10, 26),
            (11, 12, 5)
        ];
        let expected = mate![-, 8, 3, 2, 6, 9, 4, 10, 1, 5, 7, 12, 11];
        assert_eq!(gabow_algo(&graph, false), expected);
    }

    #[test]
    fn test_nest_relabel_expand() {
        let graph = graph![
            (1, 2, 40),
            (1, 3, 40),
            (2, 3, 60),
            (2, 4, 55),
            (3, 5, 55),
            (4, 5, 50),
            (1, 8, 15),
            (5, 7, 30),
            (7, 6, 10),
            (8, 10, 10),
            (4, 9, 30)
        ];
        let expected = mate![-, 2, 1, 5, 9, 3, 7, 6, 10, 4, 8];
        assert_eq!(gabow_algo(&graph, false), expected);
    }
}
