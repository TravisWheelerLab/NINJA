//! External-memory NINJA engine.
//!
//! Same search as the in-memory engine, with three substitutions:
//!
//! * distances are `f32` in a [`DiskMatrix`] whose older columns live on
//!   disk;
//! * each cluster-pair heap is an [`ArrayHeap`] backed by a scratch file;
//! * when the candidate list grows past a threshold, it is frozen into a
//!   [`CandidateHeap`] (also disk-backed) that is later scanned under a
//!   bound derived from the drift in row sums.

mod candidate_heap;
pub mod matrix;

use std::path::{Path, PathBuf};

pub use candidate_heap::CandidateHeap;
pub use matrix::DiskMatrix;

use matrix::RowPager;

use crate::error::{Error, Result};
use crate::heap::{ArrayHeap, ArrayHeapConfig, MinHeap};
use crate::tree::Tree;

use super::{ActiveList, NjParams, NjStats};

/// Candidate list is frozen into a heap once it holds this many entries per
/// remaining taxon (and at least `CAND_HEAP_THRESH` entries).
const COMPLEX_CANDIDATE_RATIO: usize = 40;
/// Minimum candidate count before a candidate heap is considered.
const CAND_HEAP_THRESH: usize = 50_000;
/// Hard cap on the candidate list regardless of ratio.
const SIMPLE_CANDIDATE_CAP: usize = 2_000_000;
/// A candidate heap that shrinks below this fraction of its original size
/// is dissolved back into the candidate list.
const CAND_HEAP_DECAY: f32 = 0.6;
/// Maximum number of live candidate heaps; the oldest are merged into the
/// newest when exceeded.
const MAX_CAND_HEAPS: usize = 100;
const MERGE_OLDEST: usize = 20;

/// Build a tree with the external-memory engine.
///
/// `tmp_dir` receives the scratch files (one per cluster pair plus one per
/// candidate heap); they are removed when the build finishes.
/// `memory_bytes` sizes the per-heap buffers.
pub fn build(
    names: &[String],
    m: DiskMatrix,
    params: &NjParams,
    tmp_dir: &Path,
    memory_bytes: u64,
) -> Result<(Tree, NjStats)> {
    let k = m.k;
    if names.len() != k {
        return Err(Error::invalid(format!("{} names but a {}-taxon matrix", names.len(), k)));
    }
    if params.cluster_count == 0 {
        return Err(Error::options("cluster count must be at least 1"));
    }
    // The disk-backed engine keeps the reference rebuild schedule unless a
    // ratio was given explicitly.
    let params = if params.rebuild_step_ratio.is_none() {
        NjParams { rebuild_step_ratio: Some(0.5), ..params.clone() }
    } else {
        params.clone()
    };
    let mut b = Builder::new(names, m, &params, tmp_dir, memory_bytes)?;
    b.run()?;
    Ok((b.tree, b.stats))
}

struct Builder<'a> {
    k: usize,
    params: &'a NjParams,
    m: DiskMatrix,
    /// Row sums, in double precision. The reference accumulated these in
    /// single precision, and the drift over tens of thousands of joins
    /// changed which pairs it joined.
    r: Vec<f64>,
    tree: Tree,
    redirect: Vec<i32>,
    active: ActiveList,
    tmp_dir: PathBuf,
    heap_config: ArrayHeapConfig,
    cand_heap_config: ArrayHeapConfig,

    clust_cnt: usize,
    clust_assign: Vec<u32>,
    clust_maxes: Vec<f64>,
    clusters_by_size: Vec<u32>,
    heaps: Vec<Option<ArrayHeap>>,
    /// Per-heap staging buffers for rebuilds (fast mode).
    stage: Vec<Vec<(f32, (i32, i32))>>,
    /// Scratch: active node indices for the update loop.
    rows: Vec<u32>,

    cand_d: Vec<f32>,
    cand_i: Vec<i32>,
    cand_j: Vec<i32>,
    cand_active: Vec<bool>,
    free_cands: Vec<i32>,
    last_cand: i32,
    cand_heaps: Vec<CandidateHeap>,
    using_simple: bool,

    next_internal: usize,
    new_k: usize,
    stats: NjStats,
}

impl<'a> Builder<'a> {
    fn new(
        names: &[String],
        mut m: DiskMatrix,
        params: &'a NjParams,
        tmp_dir: &Path,
        memory_bytes: u64,
    ) -> Result<Self> {
        let k = m.k;
        let total = 2 * k - 1;
        let mut redirect = vec![-1i32; total];
        for (i, slot) in redirect.iter_mut().enumerate().take(k) {
            *slot = i as i32;
        }
        let cc = params.cluster_count;
        let r = std::mem::take(&mut m.r);
        let mut b = Builder {
            k,
            params,
            m,
            r,
            tree: Tree::leaves(names),
            redirect,
            active: ActiveList::new(total),
            tmp_dir: tmp_dir.to_path_buf(),
            // About 3 MB per cluster-pair heap and 2 MB per candidate heap
            // at the reference's 2 GB budget.
            heap_config: ArrayHeapConfig { memory_bytes: (memory_bytes / 666).max(1 << 20) },
            cand_heap_config: ArrayHeapConfig { memory_bytes: (memory_bytes / 1000).max(1 << 20) },
            clust_cnt: cc,
            clust_assign: vec![0; k],
            clust_maxes: vec![0.0; cc],
            clusters_by_size: vec![0; cc],
            heaps: (0..cc * cc).map(|_| None).collect(),
            stage: (0..cc * cc).map(|_| Vec::new()).collect(),
            rows: Vec::with_capacity(k),
            cand_d: Vec::with_capacity(10_000),
            cand_i: Vec::with_capacity(10_000),
            cand_j: Vec::with_capacity(10_000),
            cand_active: Vec::with_capacity(10_000),
            free_cands: Vec::new(),
            last_cand: -1,
            cand_heaps: Vec::new(),
            using_simple: true,
            next_internal: k,
            new_k: k,
            stats: NjStats::default(),
        };
        b.cluster_and_heap(k)?;
        Ok(b)
    }

    #[inline]
    fn heap_index(&self, a: u32, b: u32) -> usize {
        let (a, b) = if a <= b { (a, b) } else { (b, a) };
        a as usize * self.clust_cnt + b as usize
    }

    /// Distance between node `x` (row `rx`) and node `node` (row `rnode`),
    /// where `pager` pages row `rnode` from disk.
    #[inline]
    fn dist(&mut self, x: usize, rx: usize, node: usize, rnode: usize, pager: &mut RowPager) -> Result<f32> {
        if node >= self.m.first_mem_col {
            Ok(self.m.mem_get(rx, node))
        } else if x >= self.m.first_mem_col {
            Ok(self.m.mem_get(rnode, x))
        } else {
            pager.get(&mut self.m, x)
        }
    }

    fn cluster_and_heap(&mut self, max_index: usize) -> Result<()> {
        let cc = self.clust_cnt;
        let max_index_i = max_index as i32;
        self.free_cands.clear();
        self.last_cand = -1;
        self.cand_heaps.clear();

        let mut max_t = 0f64;
        let mut min_t = f64::MAX;
        let mut i = self.active.first;
        while i < max_index_i {
            let ri = self.redirect[i as usize] as usize;
            if self.r[ri] > max_t {
                max_t = self.r[ri];
            }
            if self.r[ri] < min_t {
                min_t = self.r[ri];
            }
            i = self.active.next[i as usize];
        }
        let range = max_t - min_t;
        for c in 0..cc - 1 {
            self.clust_maxes[c] = min_t + (c + 1) as f64 * range / cc as f64;
        }
        self.clust_maxes[cc - 1] = max_t;
        let mut sizes = vec![0i32; cc];
        let mut i = self.active.first;
        while i < max_index_i {
            let ri = self.redirect[i as usize] as usize;
            for c in 0..cc {
                if self.r[ri] <= self.clust_maxes[c] {
                    self.clust_assign[ri] = c as u32;
                    sizes[c] += 1;
                    break;
                }
            }
            i = self.active.next[i as usize];
        }
        let mut order: MinHeap<i32, u32> = MinHeap::new();
        for (c, &s) in sizes.iter().enumerate() {
            order.push(s, c as u32);
        }
        for slot in self.clusters_by_size.iter_mut() {
            *slot = order.pop().unwrap().1;
        }

        let reference = self.params.reference_order;
        for a in 0..cc {
            for b in a..cc {
                let h = a * cc + b;
                match &mut self.heaps[h] {
                    Some(heap) => heap.clear(),
                    None => {
                        let mut heap = ArrayHeap::new(&self.tmp_dir, self.heap_config)?;
                        heap.set_reference_order(reference);
                        self.heaps[h] = Some(heap);
                    }
                }
            }
        }

        // Every active pair goes to its cluster pair's heap. In fast mode
        // pairs are staged per heap and, once a run's worth has
        // accumulated, sorted and written as a disk run directly; the
        // reference mode inserts them one at a time through the heap's
        // in-memory stage, which fixes its tie order.
        let run_size = self.heaps[0].as_ref().unwrap().run_size();
        let mut i = self.active.first;
        while i < max_index_i {
            let ri = self.redirect[i as usize] as usize;
            let mut pager = RowPager::new(ri, self.m.mem_cols);
            let mut j = self.active.next[i as usize];
            while j < max_index_i {
                let rj = self.redirect[j as usize] as usize;
                let h = self.heap_index(self.clust_assign[ri], self.clust_assign[rj]);
                let d = if (j as usize) >= self.m.first_mem_col {
                    self.m.mem_get(ri, j as usize)
                } else {
                    pager.get(&mut self.m, j as usize)?
                };
                let redirect = &self.redirect;
                if reference {
                    self.heaps[h].as_mut().unwrap().insert(i, j, d, Some(redirect))?;
                } else {
                    self.stage[h].push((d, (i, j)));
                    if self.stage[h].len() == run_size {
                        let mut run = std::mem::take(&mut self.stage[h]);
                        run.sort_unstable_by(|a, b| {
                            a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        self.heaps[h].as_mut().unwrap().insert_sorted(&run, Some(redirect))?;
                        run.clear();
                        self.stage[h] = run;
                    }
                }
                j = self.active.next[j as usize];
            }
            i = self.active.next[i as usize];
        }
        if !reference {
            for h in 0..cc * cc {
                if !self.stage[h].is_empty() {
                    let mut run = std::mem::take(&mut self.stage[h]);
                    run.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                    let redirect = &self.redirect;
                    self.heaps[h].as_mut().unwrap().insert_sorted(&run, Some(redirect))?;
                    run.clear();
                    self.stage[h] = run;
                }
            }
        }
        self.stats.rebuilds += 1;
        Ok(())
    }

    fn append_simple(&mut self, d: f32, i: i32, j: i32) -> i32 {
        let pos = match self.free_cands.pop() {
            Some(p) => p,
            None => self.last_cand + 1,
        };
        let pu = pos as usize;
        if pu == self.cand_d.len() {
            self.cand_d.push(d);
            self.cand_i.push(i);
            self.cand_j.push(j);
            self.cand_active.push(true);
        } else {
            self.cand_d[pu] = d;
            self.cand_i[pu] = i;
            self.cand_j[pu] = j;
            self.cand_active[pu] = true;
        }
        if pos > self.last_cand {
            self.last_cand = pos;
        }
        pos
    }

    fn remove_candidate(&mut self, x: i32) {
        self.free_cands.push(x);
        self.deactivate(x);
    }

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

    #[inline]
    fn q_prime(&self, d: f32, i: i32, j: i32) -> f32 {
        let ri = self.redirect[i as usize] as usize;
        let rj = self.redirect[j as usize] as usize;
        (d as f64 * (self.new_k as f64 - 2.0) - self.r[ri] - self.r[rj]) as f32
    }

    /// Add a candidate: to the simple list, or, when that has grown too
    /// long, freeze the list into a new candidate heap.
    fn append_candidate(&mut self, d: f32, i: i32, j: i32) -> Result<()> {
        if !self.using_simple {
            let q = self.q_prime(d, i, j);
            let last = self.cand_heaps.len() - 1;
            return self.cand_heaps[last].insert(i, j, q);
        }
        let cand_cnt =
            (self.last_cand + 1) as usize - self.free_cands.len().min((self.last_cand + 1) as usize);
        let freeze = cand_cnt >= SIMPLE_CANDIDATE_CAP
            || (cand_cnt >= CAND_HEAP_THRESH && cand_cnt / self.new_k > COMPLEX_CANDIDATE_RATIO);
        if !freeze {
            self.append_simple(d, i, j);
            return Ok(());
        }

        self.using_simple = false;
        let mut heap = CandidateHeap::new(
            &self.tmp_dir,
            self.cand_heap_config,
            self.new_k,
            &self.r,
            self.next_internal + 1,
        )?;
        for x in 0..=self.last_cand as usize {
            if self.cand_active[x]
                && self.redirect[self.cand_i[x] as usize] != -1
                && self.redirect[self.cand_j[x] as usize] != -1
            {
                let q = self.q_prime(self.cand_d[x], self.cand_i[x], self.cand_j[x]);
                heap.insert(self.cand_i[x], self.cand_j[x], q)?;
            }
        }
        let q = self.q_prime(d, i, j);
        heap.insert(i, j, q)?;
        self.last_cand = -1;
        self.free_cands.clear();

        if self.cand_heaps.len() == MAX_CAND_HEAPS {
            for _ in 0..MERGE_OLDEST {
                let mut old = self.cand_heaps.remove(0);
                while let Some((i, j, qp)) = old.peek() {
                    let ri = self.redirect[i as usize];
                    let rj = self.redirect[j as usize];
                    if ri != -1 && rj != -1 {
                        let d = old.distance(qp, ri as usize, rj as usize);
                        let q = self.q_prime(d, i, j);
                        heap.insert(i, j, q)?;
                    }
                    old.pop()?;
                }
            }
        }
        self.cand_heaps.push(heap);
        if self.params.verbose >= 2 {
            eprintln!(
                "froze candidate list into a heap with {} taxa left ({} heaps live)",
                self.new_k,
                self.cand_heaps.len()
            );
        }
        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        let k = self.k;
        let cc = self.clust_cnt;
        let verbose = self.params.verbose;
        let total = 2 * k - 1;
        let mut steps_until_rebuild = self.params.initial_rebuild_steps(k);

        let mut max_t1val = vec![f64::MIN_POSITIVE; cc];
        let mut max_t2val = vec![f64::MIN_POSITIVE; cc];
        let mut horiz: Vec<f32> = Vec::new();

        while self.next_internal < total {
            let next_i32 = self.next_internal as i32;
            let new_k = self.new_k;
            let nk2 = new_k as f64 - 2.0;
            self.using_simple = true;

            for c in 0..cc {
                max_t1val[c] = f64::MIN_POSITIVE;
                max_t2val[c] = f64::MIN_POSITIVE;
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
                    } else {
                        max_t2val[c] = rv;
                    }
                }
                x = self.active.next[x as usize];
            }

            let mut min_q = f64::MAX;
            let mut min_d = f32::MIN_POSITIVE;
            let mut min_i: i32 = -1;
            let mut min_j: i32 = -1;

            // Simple candidates.
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
                    let q = self.cand_d[xu] as f64 * nk2 - self.r[ri as usize] - self.r[rj as usize];
                    if q <= min_q {
                        min_i = self.cand_i[xu];
                        min_j = self.cand_j[xu];
                        min_q = q;
                        min_d = self.cand_d[xu];
                    }
                }
                x -= 1;
            }

            // Return hopeless candidates to the cluster-pair heaps.
            let iters = self.params.candidate_iters;
            if iters > 0 && (k - new_k) % iters == 0 && steps_until_rebuild > iters / 2 {
                let mut x = self.last_cand;
                while x >= 0 {
                    let xu = x as usize;
                    if self.cand_active[xu] {
                        let ri = self.redirect[self.cand_i[xu] as usize] as usize;
                        let rj = self.redirect[self.cand_j[xu] as usize] as usize;
                        let (cl_i, cl_j) = (self.clust_assign[ri], self.clust_assign[rj]);
                        let max_t_sum = max_t1val[cl_i as usize]
                            + if cl_i == cl_j { max_t2val[cl_i as usize] } else { max_t1val[cl_j as usize] };
                        let q_limit = self.cand_d[xu] as f64 * nk2 - max_t_sum;
                        if q_limit > min_q {
                            self.remove_candidate(x);
                            let h = self.heap_index(cl_i, cl_j);
                            let (d, i, j) = (self.cand_d[xu], self.cand_i[xu], self.cand_j[xu]);
                            let redirect = &self.redirect;
                            self.heaps[h].as_mut().unwrap().insert(i, j, d, Some(redirect))?;
                        }
                    }
                    x -= 1;
                }
            }

            // Compact.
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
                        left += 1;
                        right -= 1;
                    }
                }
                self.last_cand = right;
                self.free_cands.clear();
            }

            // Candidate heaps, newest first.
            let mut expired_exists = false;
            for c in (0..self.cand_heaps.len()).rev() {
                self.cand_heaps[c].calc_deltas(new_k, &self.redirect, &self.r);
                while let Some((i, j, qp)) = self.cand_heaps[c].peek() {
                    let bound =
                        self.cand_heaps[c].k_over_kprime * qp as f64 + self.cand_heaps[c].min_delta_sum;
                    if bound >= min_q {
                        break;
                    }
                    self.cand_heaps[c].pop()?;
                    let ri = self.redirect[i as usize];
                    let rj = self.redirect[j as usize];
                    if ri == -1 || rj == -1 {
                        self.stats.defunct_removed += 1;
                        continue;
                    }
                    let d = self.cand_heaps[c].distance(qp, ri as usize, rj as usize);
                    let q = d as f64 * nk2 - self.r[ri as usize] - self.r[rj as usize];
                    self.append_candidate(d, i, j)?;
                    if q <= min_q {
                        min_i = i;
                        min_j = j;
                        min_q = q;
                        min_d = d;
                    }
                }
                let h = &mut self.cand_heaps[c];
                if (h.len() as f32) < h.orig_size as f32 * CAND_HEAP_DECAY {
                    h.expired = true;
                    expired_exists = true;
                }
            }
            if expired_exists {
                for c in (0..self.cand_heaps.len()).rev() {
                    if !self.cand_heaps[c].expired {
                        continue;
                    }
                    let mut h = self.cand_heaps.remove(c);
                    while let Some((i, j, qp)) = h.peek() {
                        let ri = self.redirect[i as usize];
                        let rj = self.redirect[j as usize];
                        if ri != -1 && rj != -1 {
                            let d = h.distance(qp, ri as usize, rj as usize);
                            self.append_candidate(d, i, j)?;
                        }
                        h.pop()?;
                    }
                    if verbose >= 2 {
                        eprintln!(
                            "dissolved a candidate heap (frozen at {} taxa) with {} left",
                            h.k_prime, new_k
                        );
                    }
                }
            }

            // Cluster-pair heaps.
            for a in 0..cc {
                for b in a..cc {
                    let (ca, cb) = (self.clusters_by_size[a], self.clusters_by_size[b]);
                    let (cl_a, cl_b) = if ca < cb { (ca, cb) } else { (cb, ca) };
                    let max_t_sum = max_t1val[cl_a as usize]
                        + if cl_a == cl_b { max_t2val[cl_a as usize] } else { max_t1val[cl_b as usize] };
                    let h = self.heap_index(cl_a, cl_b);
                    while let Some((h_i, h_j, h_d)) = self.heaps[h].as_ref().unwrap().peek() {
                        let ri = self.redirect[h_i as usize];
                        let rj = self.redirect[h_j as usize];
                        if ri == -1 || rj == -1 {
                            self.heaps[h].as_mut().unwrap().pop()?;
                            self.stats.defunct_removed += 1;
                            continue;
                        }
                        let mut q = h_d as f64 * nk2;
                        let q_limit = q - max_t_sum;
                        if q_limit > min_q {
                            break;
                        }
                        self.heaps[h].as_mut().unwrap().pop()?;
                        self.append_candidate(h_d, h_i, h_j)?;
                        self.stats.candidates_added += 1;
                        q -= self.r[ri as usize] + self.r[rj as usize];
                        if q <= min_q {
                            min_i = h_i;
                            min_j = h_j;
                            min_q = q;
                            min_d = h_d;
                        }
                    }
                }
            }
            if !self.using_simple {
                let last = self.cand_heaps.len() - 1;
                self.cand_heaps[last].build_node_list();
            }

            if min_i < 0 {
                return Err(Error::invalid(
                    "neighbor joining found no pair to join; the distance matrix is inconsistent",
                ));
            }

            // Join.
            let (mi, mj) = (min_i as usize, min_j as usize);
            let ri = self.redirect[mi] as usize;
            let rj = self.redirect[mj] as usize;
            let min_d = min_d as f64;
            let (mut len_i, mut len_j) = if new_k == 2 {
                (min_d / 2.0, min_d / 2.0)
            } else {
                (
                    (min_d + (self.r[ri] - self.r[rj]) / nk2) / 2.0,
                    (min_d + (self.r[rj] - self.r[ri]) / nk2) / 2.0,
                )
            };
            if len_i < 0.0 {
                len_j += len_i;
                len_i = 0.0;
            } else if len_j < 0.0 {
                len_i += len_j;
                len_j = 0.0;
            }
            let ni = self.next_internal;
            self.tree.join(ni, mi, mj, len_i as f32, len_j as f32);
            if verbose >= 3 {
                eprintln!(
                    "join {}: {} ({}) + {} ({}) Q={} lengths {} {}",
                    ni, mi, ri, mj, rj, min_q, len_i, len_j
                );
            }
            self.redirect[mi] = -1;
            self.redirect[mj] = -1;
            self.active.remove(mi);
            self.active.remove(mj);
            self.r[ri] = 0.0;

            // D(i, j), then new distances to every active node.
            let first = self.m.first_mem_col;
            let d_ij = if mi >= first {
                self.m.mem_get(rj, mi)
            } else if mj >= first {
                self.m.mem_get(ri, mj)
            } else {
                self.m.read_disk_one(ri, mj)?
            };
            let mut pager_i = RowPager::new(ri, self.m.mem_cols);
            let mut pager_j = RowPager::new(rj, self.m.mem_cols);
            self.rows.clear();
            let mut x = self.active.first;
            while x < next_i32 {
                self.rows.push(x as u32);
                x = self.active.next[x as usize];
            }
            for idx in 0..self.rows.len() {
                let xu = self.rows[idx] as usize;
                let rx = self.redirect[xu] as usize;
                let d_xi = self.dist(xu, rx, mi, ri, &mut pager_i)?;
                let d_xj = self.dist(xu, rx, mj, rj, &mut pager_j)?;
                let tmp = (d_xi + d_xj - d_ij) / 2.0;
                self.r[ri] += tmp as f64;
                self.r[rx] += tmp as f64 - (d_xi as f64 + d_xj as f64);
                self.m.mem_set(rx, ni, tmp);
                if xu >= first {
                    self.m.mem_set(ri, xu, tmp);
                }
            }
            self.redirect[ni] = ri as i32;

            // Flush the resident window when it is full and more columns
            // are still to come.
            let window_full =
                self.m.uses_disk() && ni + 1 == first + self.m.mem_cols && ni + 1 < self.m.row_len;
            if window_full {
                let mut rows = Vec::new();
                let mut x = self.active.first;
                while x < next_i32 {
                    rows.push(self.redirect[x as usize] as usize);
                    x = self.active.next[x as usize];
                }
                self.m.flush_rows(rows.into_iter())?;
                horiz.clear();
                horiz.resize(ni, 0.0);
                for col in first..=ni {
                    let ry = self.redirect[col];
                    if ry == -1 {
                        continue;
                    }
                    for v in horiz.iter_mut() {
                        *v = 0.0;
                    }
                    let mut x = self.active.first;
                    while x < next_i32 {
                        let rx = self.redirect[x as usize] as usize;
                        horiz[x as usize] = self.m.mem_get(rx, col);
                        x = self.active.next[x as usize];
                    }
                    self.m.write_disk(ry as usize, 0, &horiz)?;
                }
            }

            self.new_k -= 1;
            let new_k = self.new_k;

            if steps_until_rebuild == 0 {
                self.next_internal += 1;
                if verbose >= 2 {
                    eprintln!("rebuilding clusters and heaps with {} taxa left", new_k);
                }
                let mi = self.next_internal;
                self.cluster_and_heap(mi)?;
                steps_until_rebuild = self.params.next_rebuild_steps(k, new_k);
            } else {
                steps_until_rebuild -= 1;
                self.clust_maxes.copy_from_slice(&max_t1val);
                for c in 0..cc {
                    if self.r[ri] <= self.clust_maxes[c] {
                        self.clust_assign[ri] = c as u32;
                        break;
                    }
                }
                let cl_new = self.clust_assign[ri];
                let mut x = self.active.first;
                while x < next_i32 {
                    let rx = self.redirect[x as usize] as usize;
                    let d = self.m.mem_get(rx, ni);
                    let h = self.heap_index(cl_new, self.clust_assign[rx]);
                    let redirect = &self.redirect;
                    self.heaps[h].as_mut().unwrap().insert(next_i32, x, d, Some(redirect))?;
                    x = self.active.next[x as usize];
                }
                self.next_internal += 1;
            }

            if window_full {
                self.m.first_mem_col = ni + 1;
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
    use super::super::test_util::{random_tree, tree_splits};
    use super::*;
    use crate::io::phylip::PhylipMatrix;

    #[test]
    fn recovers_random_additive_trees_on_disk() {
        for (n, seed, memory) in [(10, 1u64, 1u64 << 30), (60, 2, 1 << 30), (700, 3, 1), (1200, 4, 1 << 16)] {
            let t = random_tree(n, seed);
            let d = t.distances();
            let mut lower = Vec::with_capacity(n);
            for i in 0..n {
                lower.push((0..i).map(|j| (d[i][j] * 1e8).round() as i64).collect::<Vec<_>>());
            }
            let names: Vec<String> = (0..n).map(|i| i.to_string()).collect();
            let p = PhylipMatrix { names: names.clone(), lower };
            let dir = tempfile::tempdir().unwrap();
            let m = DiskMatrix::from_phylip(&p, memory, dir.path()).unwrap();
            assert_eq!(m.uses_disk(), n > 513, "n = {}", n);
            let params = NjParams { verbose: 0, rebuild_steps: Some(n / 3 + 1), ..Default::default() };
            let (tree, _) = build(&names, m, &params, dir.path(), memory.max(1 << 20)).unwrap();
            assert_eq!(tree_splits(&tree), t.splits(), "n = {}", n);
        }
    }
}
