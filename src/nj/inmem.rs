//! In-memory NINJA engine over a fixed-point distance matrix.

use crate::distance::DistanceMatrix;
use crate::error::{Error, Result};
use crate::heap::MinHeap;
use crate::tree::Tree;

use super::{ActiveList, NjParams, NjStats};

/// Branch lengths are fixed-point distances divided by this: the `1e8`
/// scale times two, since each of the two branches gets half the distance.
const LENGTH_SCALE: f32 = 200_000_000.0;

/// Heap key: fixed-point distance; payload: node indices `(i, j)`, `i < j`.
type PairHeap = MinHeap<i32, (i32, i32)>;

/// Build a tree from a distance matrix with the in-memory engine.
///
/// `names.len()` must equal `d.len()`. The matrix is consumed because it is
/// updated in place as taxa are joined.
pub fn build(names: &[String], d: DistanceMatrix, params: &NjParams) -> Result<(Tree, NjStats)> {
    let k = d.len();
    if names.len() != k {
        return Err(Error::invalid(format!("{} names but a {}x{} distance matrix", names.len(), k, k)));
    }
    if k == 0 {
        return Err(Error::invalid("no taxa"));
    }
    if params.cluster_count == 0 {
        return Err(Error::options("cluster count must be at least 1"));
    }
    let mut b = Builder::new(names, d, params);
    b.run()?;
    Ok((b.tree, b.stats))
}

struct Builder<'a> {
    k: usize,
    params: &'a NjParams,
    d: DistanceMatrix,
    /// Row sums, indexed by matrix row (`redirect[node]`).
    r: Vec<i64>,
    tree: Tree,
    /// Matrix row for each node index, or -1 once merged. A new internal
    /// node reuses the row of its left child.
    redirect: Vec<i32>,
    active: ActiveList,

    clust_cnt: usize,
    /// Cluster of each matrix row.
    clust_assign: Vec<u32>,
    /// Upper bound on the row sums in each cluster.
    clust_percentiles: Vec<i64>,
    /// Cluster ids ordered by ascending size at the last rebuild.
    clusters_by_size: Vec<u32>,
    /// One heap per unordered cluster pair `(a, b)` with `a <= b`, at
    /// `a * clust_cnt + b`.
    heaps: Vec<PairHeap>,

    cand_d: Vec<i32>,
    cand_i: Vec<i32>,
    cand_j: Vec<i32>,
    cand_active: Vec<bool>,
    free_cands: Vec<i32>,
    last_cand: i32,

    stats: NjStats,
}

impl<'a> Builder<'a> {
    fn new(names: &[String], d: DistanceMatrix, params: &'a NjParams) -> Self {
        let k = d.len();
        let total = 2 * k - 1;
        let mut r = vec![0i64; k];
        for i in 0..k {
            for j in (i + 1)..k {
                let v = d.get(i, j) as i64;
                r[i] += v;
                r[j] += v;
            }
        }
        let mut redirect = vec![-1i32; total];
        for (i, slot) in redirect.iter_mut().enumerate().take(k) {
            *slot = i as i32;
        }
        let cc = params.cluster_count;
        let mut b = Builder {
            k,
            params,
            d,
            r,
            tree: Tree::leaves(names),
            redirect,
            active: ActiveList::new(total),
            clust_cnt: cc,
            clust_assign: vec![0; k],
            clust_percentiles: vec![0; cc],
            clusters_by_size: vec![0; cc],
            heaps: (0..cc * cc).map(|_| PairHeap::new()).collect(),
            cand_d: Vec::with_capacity(10_000),
            cand_i: Vec::with_capacity(10_000),
            cand_j: Vec::with_capacity(10_000),
            cand_active: Vec::with_capacity(10_000),
            free_cands: Vec::new(),
            last_cand: -1,
            stats: NjStats::default(),
        };
        b.cluster_and_heap(k);
        b
    }

    #[inline]
    fn heap_index(&self, a: u32, b: u32) -> usize {
        let (a, b) = if a <= b { (a, b) } else { (b, a) };
        a as usize * self.clust_cnt + b as usize
    }

    /// Assign every active node (index below `max_index`) to a cluster by
    /// row sum, then fill one heap per cluster pair with all active pairs.
    fn cluster_and_heap(&mut self, max_index: usize) {
        let cc = self.clust_cnt;
        let max_index = max_index as i32;

        let mut max_t: i64 = 0;
        let mut min_t: i64 = i64::MAX;
        let mut i = self.active.first;
        while i < max_index {
            let ri = self.redirect[i as usize] as usize;
            max_t = max_t.max(self.r[ri]);
            min_t = min_t.min(self.r[ri]);
            i = self.active.next[i as usize];
        }
        let range = max_t - min_t;
        for c in 0..cc - 1 {
            self.clust_percentiles[c] = min_t + (c as i64 + 1) * range / cc as i64;
        }
        self.clust_percentiles[cc - 1] = max_t;

        let mut sizes = vec![0i32; cc];
        let mut i = self.active.first;
        while i < max_index {
            let ri = self.redirect[i as usize] as usize;
            for c in 0..cc {
                if self.r[ri] <= self.clust_percentiles[c] {
                    self.clust_assign[ri] = c as u32;
                    sizes[c] += 1;
                    break;
                }
            }
            i = self.active.next[i as usize];
        }

        // Order clusters by size (smallest first) via the same heap the
        // reference used, so ties order identically.
        let mut order: MinHeap<i32, u32> = MinHeap::new();
        for (c, &s) in sizes.iter().enumerate() {
            order.push(s, c as u32);
        }
        for slot in self.clusters_by_size.iter_mut() {
            *slot = order.pop().unwrap().1;
        }

        // Reset the candidate list. As in the reference, the free-slot stack
        // and stale activity flags are left alone; a stale slot reused later
        // simply re-exposes an older, still valid candidate.
        self.last_cand = -1;

        for h in self.heaps.iter_mut() {
            h.clear();
        }
        let mut i = self.active.first;
        while i < max_index {
            let ri = self.redirect[i as usize] as usize;
            let mut j = self.active.next[i as usize];
            while j < max_index {
                let rj = self.redirect[j as usize] as usize;
                let (ra, rb) = if ri < rj { (ri, rj) } else { (rj, ri) };
                let h = self.heap_index(self.clust_assign[ra], self.clust_assign[rb]);
                let dist = self.d.get(ra, rb);
                self.heaps[h].push(dist, (i, j));
                j = self.active.next[j as usize];
            }
            i = self.active.next[i as usize];
        }
        self.stats.rebuilds += 1;
    }

    fn append_candidate(&mut self, d: i32, i: i32, j: i32) -> i32 {
        let pos = match self.free_cands.pop() {
            Some(p) => p,
            None => self.last_cand + 1,
        };
        if pos > self.last_cand {
            self.last_cand = pos;
        }
        let pos_u = pos as usize;
        if pos_u == self.cand_d.len() {
            self.cand_d.push(d);
            self.cand_i.push(i);
            self.cand_j.push(j);
            self.cand_active.push(true);
        } else {
            self.cand_d[pos_u] = d;
            self.cand_i[pos_u] = i;
            self.cand_j[pos_u] = j;
            self.cand_active[pos_u] = true;
        }
        pos
    }

    /// Mark candidate `x` inactive, pulling `last_cand` back if needed.
    #[inline]
    fn deactivate(&mut self, x: i32) {
        self.cand_active[x as usize] = false;
        if x == self.last_cand {
            let mut y = x;
            while y > 0 && !self.cand_active[y as usize] {
                y -= 1;
            }
            self.last_cand = y;
        }
    }

    fn run(&mut self) -> Result<()> {
        let k = self.k;
        let cc = self.clust_cnt;
        let verbose = self.params.verbose;
        let total = 2 * k - 1;

        let mut next_internal = k;
        let mut new_k = k;
        let mut steps_until_rebuild = self.params.initial_rebuild_steps(k);

        let mut max_t1 = vec![-1i32; cc];
        let mut max_t2 = vec![-1i32; cc];
        let mut max_t1val = vec![i64::MIN; cc];
        let mut max_t2val = vec![i64::MIN; cc];

        while next_internal < total {
            let next_i32 = next_internal as i32;

            // Two largest row sums per cluster, for the Q lower bound.
            for c in 0..cc {
                max_t1[c] = -1;
                max_t2[c] = -1;
                max_t1val[c] = i64::MIN;
                max_t2val[c] = i64::MIN;
            }
            let mut x = self.active.first;
            while x < next_i32 {
                let rx = self.redirect[x as usize] as usize;
                let c = self.clust_assign[rx] as usize;
                let rv = self.r[rx];
                if rv > max_t2val[c] {
                    if rv > max_t1val[c] {
                        max_t2val[c] = max_t1val[c];
                        max_t1val[c] = rv;
                        max_t2[c] = max_t1[c];
                        max_t1[c] = rx as i32;
                    } else {
                        max_t2val[c] = rv;
                        max_t2[c] = rx as i32;
                    }
                }
                x = self.active.next[x as usize];
            }

            let mut min_q = i64::MAX;
            let mut min_d: i32 = 0;
            let mut min_cand: i32 = -1;
            let nk2 = new_k as i64 - 2;

            // Scan the existing candidates, dropping defunct ones.
            let mut inactive_cnt = 0i32;
            let mut x = self.last_cand;
            while x >= 0 {
                let xu = x as usize;
                if !self.cand_active[xu] {
                    inactive_cnt += 1;
                    x -= 1;
                    continue;
                }
                let ri = self.redirect[self.cand_i[xu] as usize];
                let rj = self.redirect[self.cand_j[xu] as usize];
                if ri == -1 || rj == -1 {
                    self.deactivate(x);
                    self.stats.defunct_removed += 1;
                } else {
                    let q = self.cand_d[xu] as i64 * nk2 - self.r[ri as usize] - self.r[rj as usize];
                    if q <= min_q {
                        min_cand = x;
                        min_q = q;
                        min_d = self.cand_d[xu];
                    }
                }
                x -= 1;
            }

            // Periodically return candidates that can no longer win to the
            // heaps, so the list stays short.
            let iters = self.params.candidate_iters;
            if iters > 0 && (k - new_k) % iters == 0 && steps_until_rebuild > iters / 2 {
                let mut x = self.last_cand;
                while x >= 0 {
                    let xu = x as usize;
                    if self.cand_active[xu] {
                        let ri = self.redirect[self.cand_i[xu] as usize] as usize;
                        let rj = self.redirect[self.cand_j[xu] as usize] as usize;
                        let cl_i = self.clust_assign[ri] as usize;
                        let cl_j = self.clust_assign[rj] as usize;
                        let max_t_sum = max_t1val[cl_i].wrapping_add(if cl_i == cl_j {
                            max_t2val[cl_i]
                        } else {
                            max_t1val[cl_j]
                        });
                        let q_limit = (self.cand_d[xu] as i64 * nk2).wrapping_sub(max_t_sum);
                        if q_limit > min_q {
                            self.deactivate(x);
                            let h = self.heap_index(cl_i as u32, cl_j as u32);
                            let (d, i, j) = (self.cand_d[xu], self.cand_i[xu], self.cand_j[xu]);
                            self.heaps[h].push(d, (i, j));
                        }
                    }
                    x -= 1;
                }
            }

            // Compact the candidate list when a fifth of it is dead.
            if self.last_cand > 0 && inactive_cnt as f32 > self.last_cand as f32 / 5.0 {
                let mut left = 0i32;
                let mut right = self.last_cand;
                while left < right {
                    while left < right && self.cand_active[left as usize] {
                        left += 1;
                    }
                    while right > left && !self.cand_active[right as usize] {
                        right -= 1;
                    }
                    if left < right {
                        let (l, r) = (left as usize, right as usize);
                        self.cand_d[l] = self.cand_d[r];
                        self.cand_i[l] = self.cand_i[r];
                        self.cand_j[l] = self.cand_j[r];
                        self.cand_active[r] = false;
                        self.cand_active[l] = true;
                        if min_cand == right {
                            min_cand = left;
                        }
                        left += 1;
                        right -= 1;
                    }
                }
                self.last_cand = right;
                self.free_cands.clear();
            }

            // Pull from each cluster-pair heap while the bound allows.
            for a in 0..cc {
                for b in a..cc {
                    let (ca, cb) = (self.clusters_by_size[a], self.clusters_by_size[b]);
                    let (cl_a, cl_b) = if ca < cb { (ca, cb) } else { (cb, ca) };
                    let (cl_a_u, cl_b_u) = (cl_a as usize, cl_b as usize);
                    // Empty clusters carry i64::MIN sentinels; wrapping arithmetic
                    // matches the reference (their heaps are empty anyway).
                    let max_t_sum = max_t1val[cl_a_u].wrapping_add(if cl_a == cl_b {
                        max_t2val[cl_a_u]
                    } else {
                        max_t1val[cl_b_u]
                    });
                    let h = self.heap_index(cl_a, cl_b);
                    while let Some(&(h_d, (h_i, h_j))) = self.heaps[h].peek() {
                        let ri = self.redirect[h_i as usize];
                        let rj = self.redirect[h_j as usize];
                        if ri == -1 || rj == -1 {
                            self.heaps[h].pop();
                            self.stats.defunct_removed += 1;
                            continue;
                        }
                        let mut q = h_d as i64 * nk2;
                        let q_limit = q.wrapping_sub(max_t_sum);
                        q -= self.r[ri as usize] + self.r[rj as usize];
                        if q_limit <= min_q {
                            self.heaps[h].pop();
                            let pos = self.append_candidate(h_d, h_i, h_j);
                            self.stats.candidates_added += 1;
                            if q <= min_q {
                                min_cand = pos;
                                min_q = q;
                                min_d = h_d;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }

            if min_cand < 0 {
                return Err(Error::invalid(
                    "neighbor joining found no pair to join; the distance matrix is inconsistent",
                ));
            }

            // Join the best pair.
            let best_i = self.cand_i[min_cand as usize];
            let best_j = self.cand_j[min_cand as usize];
            self.deactivate(min_cand);
            self.free_cands.push(min_cand);

            let ri = self.redirect[best_i as usize] as usize;
            let rj = self.redirect[best_j as usize] as usize;

            let (mut len_i, mut len_j) = if new_k == 2 {
                let l = min_d as f32 / LENGTH_SCALE;
                (l, l)
            } else {
                let diff = (self.r[ri] - self.r[rj]) / nk2;
                ((min_d as f32 + diff as f32) / LENGTH_SCALE, (min_d as f32 + (-diff) as f32) / LENGTH_SCALE)
            };
            // A negative length is folded into the sibling branch.
            if len_i < 0.0 {
                len_j += len_i;
                len_i = 0.0;
            } else if len_j < 0.0 {
                len_i += len_j;
                len_j = 0.0;
            }
            self.tree.join(next_internal, best_i as usize, best_j as usize, len_i, len_j);

            if verbose >= 3 {
                eprintln!(
                    "join {}: {} ({}) + {} ({}) Q={} lengths {} {}",
                    next_internal, best_i, ri, best_j, rj, min_q, len_i, len_j
                );
            }

            self.r[ri] = 0;
            self.redirect[best_i as usize] = -1;
            self.redirect[best_j as usize] = -1;
            self.active.remove(best_i as usize);
            self.active.remove(best_j as usize);

            // New distances and row sums. The new node takes over row `ri`.
            let d_ij = self.d.get(ri, rj);
            let mut x = self.active.first;
            while x < next_i32 {
                let rx = self.redirect[x as usize] as usize;
                let d_xi = self.d.get(rx, ri);
                let d_xj = self.d.get(rx, rj);
                let tmp = (d_xi + d_xj - d_ij) / 2;
                self.r[ri] += tmp as i64;
                self.r[rx] += tmp as i64 - (d_xi as i64 + d_xj as i64);
                self.d.set(rx, ri, tmp);
                x = self.active.next[x as usize];
            }

            new_k -= 1;

            if steps_until_rebuild == 0 {
                self.redirect[next_internal] = ri as i32;
                next_internal += 1;
                if verbose >= 2 {
                    eprintln!("rebuilding clusters and heaps with {} taxa left", new_k);
                }
                self.cluster_and_heap(next_internal);
                steps_until_rebuild = self.params.next_rebuild_steps(k, new_k);
            } else {
                steps_until_rebuild -= 1;
                // Move each cluster's ceiling to its (one iteration old)
                // maximum, then place the new node.
                self.clust_percentiles.copy_from_slice(&max_t1val);
                for c in 0..cc {
                    if self.r[ri] <= self.clust_percentiles[c] {
                        self.clust_assign[ri] = c as u32;
                        break;
                    }
                }
                let cl_new = self.clust_assign[ri];
                let mut x = self.active.first;
                while x < next_i32 {
                    let rx = self.redirect[x as usize] as usize;
                    let (a, b) = if rx < ri { (x, next_i32) } else { (next_i32, x) };
                    let h = self.heap_index(self.clust_assign[rx], cl_new);
                    let dist = self.d.get(rx, ri);
                    self.heaps[h].push(dist, (a, b));
                    x = self.active.next[x as usize];
                }
                self.redirect[next_internal] = ri as i32;
                next_internal += 1;
            }
        }

        if verbose >= 1 {
            eprintln!("{} candidates added", self.stats.candidates_added);
            eprintln!("{} defunct nodes removed", self.stats.defunct_removed);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matrix(vals: &[&[f64]]) -> DistanceMatrix {
        let k = vals.len();
        let mut m = DistanceMatrix::zeros(k);
        for i in 0..k {
            for j in (i + 1)..k {
                m.set(i, j, DistanceMatrix::quantize(vals[i][j]));
            }
        }
        m
    }

    fn names(k: usize) -> Vec<String> {
        (0..k).map(|i| ((b'a' + i as u8) as char).to_string()).collect()
    }

    #[test]
    fn textbook_example() {
        // Wikipedia's neighbor-joining example (five taxa).
        let d: &[&[f64]] = &[
            &[0.0, 5.0, 9.0, 9.0, 8.0],
            &[5.0, 0.0, 10.0, 10.0, 9.0],
            &[9.0, 10.0, 0.0, 8.0, 7.0],
            &[9.0, 10.0, 8.0, 0.0, 3.0],
            &[8.0, 9.0, 7.0, 3.0, 0.0],
        ];
        let params = NjParams { verbose: 0, ..Default::default() };
        let (t, _) = build(&names(5), matrix(d), &params).unwrap();
        let nw = t.to_newick();
        // a and b join first with lengths 2 and 3, then c at 4; the last edge
        // (length 2) is split evenly at the root.
        assert_eq!(
            nw,
            "((((a:2.00000,b:3.00000):3.00000,c:4.00000):2.00000,e:1.00000):1.00000,d:1.00000);\n"
        );
    }

    #[test]
    fn two_taxa() {
        let d: &[&[f64]] = &[&[0.0, 0.5], &[0.5, 0.0]];
        let params = NjParams { verbose: 0, ..Default::default() };
        let (t, _) = build(&names(2), matrix(d), &params).unwrap();
        assert_eq!(t.to_newick(), "(a:0.25000,b:0.25000);\n");
    }

    #[test]
    fn one_taxon() {
        let params = NjParams { verbose: 0, ..Default::default() };
        let (t, _) = build(&names(1), DistanceMatrix::zeros(1), &params).unwrap();
        assert_eq!(t.to_newick(), "a;\n");
    }

    #[test]
    fn additive_tree_is_recovered() {
        // Distances from a known tree: ((a:1,b:2):3,(c:4,d:5):0) style.
        let d: &[&[f64]] =
            &[&[0.0, 0.3, 0.8, 0.9], &[0.3, 0.0, 0.9, 1.0], &[0.8, 0.9, 0.0, 0.9], &[0.9, 1.0, 0.9, 0.0]];
        let params = NjParams { verbose: 0, ..Default::default() };
        let (t, _) = build(&names(4), matrix(d), &params).unwrap();
        let nw = t.to_newick();
        assert_eq!(nw, "((a:0.10000,(c:0.40000,d:0.50000):0.30000):0.10000,b:0.10000);\n");
    }

    #[test]
    fn recovers_random_additive_trees() {
        use super::super::test_util::{random_tree, tree_splits};
        for (n, seed) in [(10, 1u64), (60, 2), (300, 3), (1200, 4)] {
            let t = random_tree(n, seed);
            let d = t.distances();
            let mut m = DistanceMatrix::zeros(n);
            for i in 0..n {
                for j in (i + 1)..n {
                    m.set(i, j, DistanceMatrix::quantize(d[i][j]));
                }
            }
            let names: Vec<String> = (0..n).map(|i| i.to_string()).collect();
            // A small rebuild interval exercises the rebuild path too.
            let params = NjParams { verbose: 0, rebuild_steps: Some(n / 3 + 1), ..Default::default() };
            let (tree, _) = build(&names, m, &params).unwrap();
            assert_eq!(tree_splits(&tree), t.splits(), "n = {}", n);
        }
    }
}
