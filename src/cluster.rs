//! Single-linkage clustering of sequences by pairwise distance.
//!
//! Two sequences belong to the same cluster when they are joined by a chain
//! of pairs each at distance at most the cutoff. That is exactly what the
//! agglomerative procedure in the C++ `cluster` branch computes (merge the
//! closest pair of clusters until the closest pair is beyond the cutoff),
//! and it is also the connected components of the graph whose edges are
//! pairs within the cutoff. The components are found with a union-find
//! while distances are computed row by row in parallel, so no matrix is
//! stored and the memory cost is linear in the number of sequences.

use rayon::prelude::*;

/// Clusters of `k` items, numbered from 0 in order of each cluster's
/// lowest-index member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clusters {
    /// Cluster id of each item.
    pub id: Vec<u32>,
    /// Members of each cluster, in ascending index order.
    pub members: Vec<Vec<usize>>,
}

impl Clusters {
    /// Number of clusters.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// True when there are no items.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

/// Rows of pairs are computed in chunks of this many rows between
/// union-find updates, bounding the edge list held at once.
const ROWS_PER_CHUNK: usize = 256;

/// Cluster `k` items with single linkage at `cutoff`.
///
/// `dist(i, j)` is called once for every pair `i < j`, in parallel across
/// rows. The cutoff is applied as `dist <= cutoff` after widening the
/// single-precision cutoff to double, as the reference did.
pub fn single_linkage(k: usize, cutoff: f32, dist: impl Fn(usize, usize) -> f64 + Sync) -> Clusters {
    let cutoff = cutoff as f64;
    let mut uf = UnionFind::new(k);
    let mut start = 0;
    while start < k {
        let end = (start + ROWS_PER_CHUNK).min(k);
        let edges: Vec<Vec<usize>> = (start..end)
            .into_par_iter()
            .map(|i| (i + 1..k).filter(|&j| dist(i, j) <= cutoff).collect())
            .collect();
        for (off, js) in edges.into_iter().enumerate() {
            let i = start + off;
            for j in js {
                uf.union(i, j);
            }
        }
        start = end;
    }
    uf.into_clusters()
}

/// Cluster from a symmetric matrix given as a lookup, without parallelism.
pub fn single_linkage_serial(k: usize, cutoff: f32, mut dist: impl FnMut(usize, usize) -> f64) -> Clusters {
    let cutoff = cutoff as f64;
    let mut uf = UnionFind::new(k);
    for i in 0..k {
        for j in (i + 1)..k {
            if dist(i, j) <= cutoff {
                uf.union(i, j);
            }
        }
    }
    uf.into_clusters()
}

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        UnionFind { parent: (0..n).collect(), rank: vec![0; n] }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb {
            return;
        }
        match self.rank[ra].cmp(&self.rank[rb]) {
            std::cmp::Ordering::Less => self.parent[ra] = rb,
            std::cmp::Ordering::Greater => self.parent[rb] = ra,
            std::cmp::Ordering::Equal => {
                self.parent[rb] = ra;
                self.rank[ra] += 1;
            }
        }
    }

    fn into_clusters(mut self) -> Clusters {
        let n = self.parent.len();
        let mut id = vec![u32::MAX; n];
        let mut root_id = vec![u32::MAX; n];
        let mut members: Vec<Vec<usize>> = Vec::new();
        for i in 0..n {
            let r = self.find(i);
            if root_id[r] == u32::MAX {
                root_id[r] = members.len() as u32;
                members.push(Vec::new());
            }
            id[i] = root_id[r];
            members[root_id[r] as usize].push(i);
        }
        Clusters { id, members }
    }
}

/// Write the cluster table: one `id<TAB>name` line per item, grouped by
/// cluster.
pub fn write_table<W: std::io::Write + ?Sized>(
    w: &mut W,
    clusters: &Clusters,
    names: &[String],
) -> std::io::Result<()> {
    for (c, members) in clusters.members.iter().enumerate() {
        for &m in members {
            writeln!(w, "{}\t{}", c, names[m])?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plain single linkage on a full matrix: merge the closest pair of
    /// clusters, folding the merged row and column into the survivor, until
    /// the closest pair is beyond the cutoff.
    fn naive(k: usize, cutoff: f32, d: &[Vec<f64>]) -> Vec<Vec<usize>> {
        let cutoff = cutoff as f64;
        let mut dm = d.to_vec();
        let mut alive: Vec<bool> = vec![true; k];
        let mut members: Vec<Vec<usize>> = (0..k).map(|i| vec![i]).collect();
        loop {
            let mut best = (f64::MAX, 0, 0);
            for i in 0..k {
                if !alive[i] {
                    continue;
                }
                for j in (i + 1)..k {
                    if alive[j] && dm[i][j] < best.0 {
                        best = (dm[i][j], i, j);
                    }
                }
            }
            let (bd, i, j) = best;
            if bd > cutoff {
                break;
            }
            for x in 0..k {
                if x != i && x != j {
                    let m = dm[i][x].min(dm[j][x]);
                    dm[i][x] = m;
                    dm[x][i] = m;
                }
            }
            alive[j] = false;
            let moved = std::mem::take(&mut members[j]);
            members[i].extend(moved);
        }
        members
            .into_iter()
            .filter(|c| !c.is_empty())
            .map(|mut c| {
                c.sort_unstable();
                c
            })
            .collect()
    }

    /// Literal transcription of the C++ cluster branch's merge loop. Its
    /// update skips rows strictly between the two merged clusters, so it
    /// misses merges and over-splits; it is kept to document that.
    fn cpp_branch(k: usize, cutoff: f32, d: &[Vec<f64>]) -> Vec<Vec<usize>> {
        let mut distances: Vec<Vec<f64>> = (0..k).map(|i| (i + 1..k).map(|j| d[i][j]).collect()).collect();
        let mut clusters: Vec<Vec<usize>> = (0..k).map(|i| vec![i]).collect();
        let mut min_dist = vec![0usize; k];
        for i in 0..k.saturating_sub(1) {
            for j in 0..(k - i - 1) {
                if distances[i][j] < distances[i][min_dist[i]] {
                    min_dist[i] = j;
                }
            }
        }
        for _ in 0..k.saturating_sub(1) {
            let mut c1 = 0;
            for i in 0..k - 1 {
                if distances[i][min_dist[i]] < distances[c1][min_dist[c1]] {
                    c1 = i;
                }
            }
            let c2 = min_dist[c1] + c1 + 1;
            if distances[c1][min_dist[c1]] > cutoff as f64 {
                break;
            }
            for c2col in 0..(k - c2 - 1) {
                let c1col = c2col + (c2 - c1);
                if distances[c1][c1col] > distances[c2][c2col] {
                    distances[c1][c1col] = distances[c2][c2col];
                }
                distances[c2][c2col] = 3.0;
            }
            for i in 0..c1 {
                let (c1col, c2col) = (c1 - i - 1, c2 - i - 1);
                if distances[i][c1col] > distances[i][c2col] {
                    distances[i][c1col] = distances[i][c2col];
                }
            }
            for i in 0..c2 {
                distances[i][c2 - i - 1] = 3.0;
            }
            let moved = std::mem::take(&mut clusters[c2]);
            clusters[c1].extend(moved);
            for i in 0..c1 {
                if min_dist[i] == c2 - i - 1 {
                    min_dist[i] = c1 - i - 1;
                }
            }
            for j in 0..(k - c1 - 1) {
                if distances[c1][j] < distances[c1][min_dist[c1]] {
                    min_dist[c1] = j;
                }
            }
        }
        clusters
            .into_iter()
            .filter(|c| !c.is_empty())
            .map(|mut c| {
                c.sort_unstable();
                c
            })
            .collect()
    }

    #[test]
    fn matches_reference_merge_loop() {
        let mut s: u64 = 99;
        let mut next = move || {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (s >> 33) as u32
        };
        for trial in 0..200 {
            let k = 2 + (next() % 40) as usize;
            let mut d = vec![vec![0.0f64; k]; k];
            for i in 0..k {
                for j in (i + 1)..k {
                    // Coarse values so that ties at the cutoff occur.
                    let v = (next() % 20) as f64 * 0.05;
                    d[i][j] = v;
                    d[j][i] = v;
                }
            }
            let cutoff = [0.1f32, 0.25, 0.5][trial % 3];
            let want = naive(k, cutoff, &d);
            let got = single_linkage(k, cutoff, |i, j| d[i][j]);
            assert_eq!(got.members, want, "k={} cutoff={}", k, cutoff);
            let serial = single_linkage_serial(k, cutoff, |i, j| d[i][j]);
            assert_eq!(serial, got);
            // The C++ branch's clusters each lie within one of ours.
            for c in cpp_branch(k, cutoff, &d) {
                let id = got.id[c[0]];
                assert!(c.iter().all(|&m| got.id[m] == id), "C++ cluster {:?} spans several of ours", c);
            }
            for (c, members) in got.members.iter().enumerate() {
                for &m in members {
                    assert_eq!(got.id[m], c as u32);
                }
            }
        }
    }

    #[test]
    fn singletons_and_one_cluster() {
        let c = single_linkage(4, 0.5, |_, _| 1.0);
        assert_eq!(c.len(), 4);
        let c = single_linkage(4, 0.5, |_, _| 0.5);
        assert_eq!(c.len(), 1);
        assert_eq!(c.members[0], vec![0, 1, 2, 3]);
        let c = single_linkage(1, 0.5, |_, _| 0.0);
        assert_eq!(c.len(), 1);
    }
}
