//! Neighbor joining with the NINJA search strategy.
//!
//! Plain neighbor joining examines every pair at every iteration, which is
//! cubic in the number of taxa. NINJA avoids most of that work:
//!
//! * Taxa are grouped into a small number of clusters by their row sum
//!   `R[i] = sum_j D[i][j]`. For a pair drawn from clusters `(a, b)`, the NJ
//!   criterion `Q = (n - 2) D - R[i] - R[j]` is bounded below by
//!   `(n - 2) D - maxR[a] - maxR[b]`, so a min-heap of `D` for each cluster
//!   pair can be scanned in order and abandoned as soon as the bound exceeds
//!   the best `Q` seen so far.
//! * Pairs popped from the heaps go on a candidate list that is re-scanned
//!   cheaply each iteration; every `candidate_iters` iterations, candidates
//!   whose bound no longer competes are pushed back into the heaps.
//! * Row sums drift as taxa are merged, so the clustering and heaps are
//!   rebuilt after a fixed fraction of the remaining taxa have been joined.
//!
//! The in-memory engine ([`inmem`]) keeps the distance matrix as fixed-point
//! integers and, since it consumes each cluster pair's entries in distance
//! order with lazy deletion, keeps the bulk inserted at a rebuild as a
//! sorted run rather than a heap. The external-memory engine ([`extmem`])
//! stores the matrix as columns of floats, most of them on disk, and
//! replaces each in-memory heap with a disk-backed one.

pub mod extmem;
pub mod inmem;

use std::fmt;
use std::str::FromStr;

/// Which neighbor-joining engine to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Use the in-memory engine when the matrix fits the memory budget,
    /// otherwise the external-memory engine.
    Auto,
    /// Force the in-memory engine.
    InMem,
    /// Force the external-memory engine.
    ExtMem,
}

impl FromStr for Method {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            // "default" is what the Java and C++ tools called this.
            "auto" | "default" => Ok(Method::Auto),
            "inmem" => Ok(Method::InMem),
            "extmem" => Ok(Method::ExtMem),
            _ => Err(format!("unknown method '{}' (expected 'auto', 'inmem', or 'extmem')", s)),
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Method::Auto => "auto",
            Method::InMem => "inmem",
            Method::ExtMem => "extmem",
        })
    }
}

/// Tunables of the NINJA search. The defaults are those of the reference
/// implementation and the paper.
#[derive(Debug, Clone, PartialEq)]
pub struct NjParams {
    /// Number of row-sum clusters (heaps are kept per cluster pair).
    pub cluster_count: usize,
    /// Fraction of the remaining taxa to join before rebuilding clusters and
    /// heaps. Ignored when `rebuild_steps` is set. `None` selects 0.5, the
    /// reference value, in reference-order mode and 0.25 otherwise, where
    /// a rebuild is cheap enough that the smaller heaps pay for it.
    pub rebuild_step_ratio: Option<f32>,
    /// Fixed number of joins between rebuilds, if given.
    pub rebuild_steps: Option<usize>,
    /// When true, the rebuild interval is based on the original taxon count
    /// rather than the remaining count.
    pub rebuild_steps_constant: bool,
    /// How often (in joins) stale candidates are returned to the heaps.
    pub candidate_iters: usize,
    /// Verbosity: 0 silent, 1 progress, 2 statistics, 3 per-join trace.
    pub verbose: u8,
    /// Resolve ties between equal distances exactly as the original Java
    /// implementation did, at some cost in speed (in-memory engine only).
    pub reference_order: bool,
}

impl Default for NjParams {
    fn default() -> Self {
        NjParams {
            cluster_count: 30,
            rebuild_step_ratio: None,
            rebuild_steps: None,
            rebuild_steps_constant: false,
            candidate_iters: 50,
            verbose: 1,
            reference_order: false,
        }
    }
}

impl NjParams {
    /// The rebuild ratio in effect.
    pub fn effective_rebuild_ratio(&self) -> f32 {
        match self.rebuild_step_ratio {
            Some(r) => r,
            None if self.reference_order => 0.5,
            None => 0.25,
        }
    }

    /// Initial number of joins before the first rebuild: the configured
    /// ratio of `k`, or `k` itself (never rebuild) when that is under 500.
    pub(crate) fn initial_rebuild_steps(&self, k: usize) -> usize {
        let steps = match self.rebuild_steps {
            Some(s) => s,
            None => (k as f32 * self.effective_rebuild_ratio()) as usize,
        };
        if steps < 500 {
            k
        } else {
            steps
        }
    }

    /// Joins until the next rebuild, given the remaining taxon count `new_k`.
    pub(crate) fn next_rebuild_steps(&self, k: usize, new_k: usize) -> usize {
        if new_k < 200 {
            new_k
        } else if let Some(s) = self.rebuild_steps {
            s
        } else if self.rebuild_steps_constant {
            (k as f32 * self.effective_rebuild_ratio()) as usize
        } else {
            (new_k as f32 * self.effective_rebuild_ratio()) as usize
        }
    }
}

/// Counters reported after a run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NjStats {
    /// Pairs moved from heaps to the candidate list.
    pub candidates_added: u64,
    /// Heap or candidate entries discarded because a member had been merged.
    pub defunct_removed: u64,
    /// Number of cluster/heap rebuilds performed.
    pub rebuilds: u32,
}

/// Shared bookkeeping: the doubly linked list of live node indices.
///
/// Node indices run over `0..2k-1`; leaves first, then internal nodes in
/// creation order. A node is "active" once created and until merged. The
/// list is threaded through all indices up front, and callers bound their
/// walks by the next internal node index, so not-yet-created internal nodes
/// are never visited.
#[derive(Debug, Clone)]
pub(crate) struct ActiveList {
    pub next: Vec<i32>,
    pub prev: Vec<i32>,
    pub first: i32,
}

impl ActiveList {
    pub fn new(total: usize) -> Self {
        ActiveList {
            next: (0..total as i32).map(|i| i + 1).collect(),
            prev: (0..total as i32).map(|i| i - 1).collect(),
            first: 0,
        }
    }

    /// Unlink node `i`.
    #[inline]
    pub fn remove(&mut self, i: usize) {
        let prev = self.prev[i];
        let next = self.next[i];
        if (next as usize) < self.prev.len() {
            self.prev[next as usize] = prev;
        }
        if prev == -1 {
            self.first = next;
        } else {
            self.next[prev as usize] = next;
        }
    }
}

#[cfg(test)]
pub(crate) mod test_util {
    //! Random additive trees for engine tests: neighbor joining recovers a
    //! tree exactly from its own path distances.

    use std::collections::BTreeSet;

    pub struct Lcg(pub u64);
    impl Lcg {
        pub fn next(&mut self) -> u32 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (self.0 >> 33) as u32
        }
        pub fn unit(&mut self) -> f64 {
            self.next() as f64 / u32::MAX as f64
        }
    }

    /// A random rooted binary tree over `n` leaves (`children[i]` for
    /// internal nodes, `branch[i]` for every node except the root) built by
    /// repeatedly joining two random subtrees.
    pub struct RandomTree {
        pub n: usize,
        pub children: Vec<Option<(usize, usize)>>,
        pub branch: Vec<f64>,
    }

    pub fn random_tree(n: usize, seed: u64) -> RandomTree {
        let mut rng = Lcg(seed);
        let total = 2 * n - 1;
        let mut children = vec![None; total];
        let mut branch = vec![0.0; total];
        for b in branch.iter_mut().take(total - 1) {
            *b = 0.01 + 0.2 * rng.unit();
        }
        let mut roots: Vec<usize> = (0..n).collect();
        let mut next = n;
        while roots.len() > 1 {
            let i = rng.next() as usize % roots.len();
            let a = roots.swap_remove(i);
            let j = rng.next() as usize % roots.len();
            let b = roots.swap_remove(j);
            children[next] = Some((a, b));
            roots.push(next);
            next += 1;
        }
        RandomTree { n, children, branch }
    }

    impl RandomTree {
        /// Leaf-to-leaf path lengths.
        pub fn distances(&self) -> Vec<Vec<f64>> {
            let total = self.children.len();
            let mut parent = vec![usize::MAX; total];
            for (p, c) in self.children.iter().enumerate() {
                if let Some((a, b)) = c {
                    parent[*a] = p;
                    parent[*b] = p;
                }
            }
            let depth_to_root = |mut x: usize| {
                let mut path = vec![x];
                while parent[x] != usize::MAX {
                    x = parent[x];
                    path.push(x);
                }
                path
            };
            let mut d = vec![vec![0.0; self.n]; self.n];
            for i in 0..self.n {
                let pi = depth_to_root(i);
                for j in (i + 1)..self.n {
                    let pj = depth_to_root(j);
                    let set: std::collections::HashSet<usize> = pj.iter().cloned().collect();
                    let lca = *pi.iter().find(|x| set.contains(x)).unwrap();
                    let mut sum = 0.0;
                    for &x in pi.iter().take_while(|&&x| x != lca) {
                        sum += self.branch[x];
                    }
                    for &x in pj.iter().take_while(|&&x| x != lca) {
                        sum += self.branch[x];
                    }
                    d[i][j] = sum;
                    d[j][i] = sum;
                }
            }
            d
        }

        /// Non-trivial splits, each as the side not containing leaf 0.
        pub fn splits(&self) -> BTreeSet<Vec<usize>> {
            let mut out = BTreeSet::new();
            for node in 0..self.children.len() - 1 {
                let mut leaves = Vec::new();
                collect(self, node, &mut leaves);
                canonical(self.n, leaves, &mut out);
            }
            out
        }
    }

    fn collect(t: &RandomTree, node: usize, out: &mut Vec<usize>) {
        match t.children[node] {
            Some((a, b)) => {
                collect(t, a, out);
                collect(t, b, out);
            }
            None => out.push(node),
        }
    }

    pub fn canonical(n: usize, mut leaves: Vec<usize>, out: &mut BTreeSet<Vec<usize>>) {
        if leaves.len() < 2 || leaves.len() > n - 2 {
            return;
        }
        leaves.sort_unstable();
        if leaves[0] == 0 {
            let set: BTreeSet<usize> = leaves.into_iter().collect();
            leaves = (0..n).filter(|x| !set.contains(x)).collect();
        }
        out.insert(leaves);
    }

    /// Non-trivial splits of a built [`crate::tree::Tree`] whose leaves are
    /// named by their index.
    pub fn tree_splits(t: &crate::tree::Tree) -> BTreeSet<Vec<usize>> {
        let n = t.num_leaves();
        let nodes = t.nodes();
        let mut out = BTreeSet::new();
        for i in 0..nodes.len() - 1 {
            let mut leaves = Vec::new();
            let mut stack = vec![i];
            while let Some(x) = stack.pop() {
                match (nodes[x].left, nodes[x].right) {
                    (Some(l), Some(r)) => {
                        stack.push(l as usize);
                        stack.push(r as usize);
                    }
                    _ => leaves.push(nodes[x].name.parse().unwrap()),
                }
            }
            canonical(n, leaves, &mut out);
        }
        out
    }
}
