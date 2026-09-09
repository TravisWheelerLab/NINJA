//! Candidate heap: a disk-backed heap of node pairs keyed by the NJ
//! criterion as it stood when the pair was inserted.
//!
//! When the candidate list grows too long, it is emptied into one of these.
//! Each entry stores `q' = (k' - 2) d - R'[i] - R'[j]` with `k'` and `R'`
//! frozen at creation. At a later iteration with `k` taxa left,
//! `q = (k - 2) d - R[i] - R[j] = (k - 2)/(k' - 2) q' + delta[i] + delta[j]`
//! where `delta[x] = (k - 2)/(k' - 2) R'[x] - R[x]`, so
//! `q >= (k - 2)/(k' - 2) q' + (two smallest deltas)` bounds every entry from
//! below, and the heap can be scanned in `q'` order until that bound exceeds
//! the best `q` seen.

use std::path::Path;

use crate::error::Result;
use crate::heap::{ArrayHeap, ArrayHeapConfig};

/// A frozen snapshot of the search state with a heap of pairs keyed by `q'`.
pub struct CandidateHeap {
    heap: ArrayHeap,
    /// Taxa remaining when the heap was created.
    pub k_prime: usize,
    /// Row sums when the heap was created, by matrix row.
    pub r_primes: Vec<f64>,
    /// Number of heap entries touching each node index.
    row_counts: Vec<u32>,
    next: Vec<i32>,
    prev: Vec<i32>,
    first: i32,
    /// `(k - 2) / (k' - 2)` from the last [`calc_deltas`](Self::calc_deltas).
    pub k_over_kprime: f64,
    /// Sum of the two smallest deltas from the last `calc_deltas`.
    pub min_delta_sum: f64,
    /// Size when the node list was built.
    pub orig_size: usize,
    /// Set when the heap has shrunk enough to be worth dissolving.
    pub expired: bool,
}

impl CandidateHeap {
    /// Create an empty candidate heap frozen at `k_prime` taxa with row sums
    /// `r`, able to hold node indices below `node_count`.
    pub fn new(
        dir: &Path,
        config: ArrayHeapConfig,
        k_prime: usize,
        r: &[f64],
        node_count: usize,
    ) -> Result<Self> {
        Ok(CandidateHeap {
            heap: ArrayHeap::new(dir, config)?,
            k_prime,
            r_primes: r.to_vec(),
            row_counts: vec![0; node_count],
            next: vec![0; node_count],
            prev: vec![0; node_count],
            first: -1,
            k_over_kprime: 1.0,
            min_delta_sum: 0.0,
            orig_size: 0,
            expired: false,
        })
    }

    /// Insert a pair with its frozen criterion value.
    pub fn insert(&mut self, i: i32, j: i32, q_prime: f32) -> Result<()> {
        self.heap.insert(i, j, q_prime, None)?;
        self.row_counts[i as usize] += 1;
        self.row_counts[j as usize] += 1;
        Ok(())
    }

    /// Link the nodes that appear in the heap, and record the size.
    pub fn build_node_list(&mut self) {
        self.orig_size = self.heap.len();
        let mut prev = -1i32;
        for i in 0..self.row_counts.len() {
            if self.row_counts[i] > 0 {
                if self.first == -1 {
                    self.first = i as i32;
                } else {
                    self.prev[i] = prev;
                    self.next[prev as usize] = i as i32;
                }
                prev = i as i32;
            }
        }
        if !self.prev.is_empty() {
            self.prev[0] = -1;
        }
        if prev >= 0 {
            self.next[prev as usize] = -1;
        }
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// True when no entries remain.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Smallest entry as `(i, j, q')`.
    pub fn peek(&self) -> Option<(i32, i32, f32)> {
        self.heap.peek()
    }

    /// Remove the smallest entry, unlinking nodes that no longer appear.
    pub fn pop(&mut self) -> Result<Option<(i32, i32, f32)>> {
        let Some((i, j, q)) = self.heap.peek() else { return Ok(None) };
        for x in [i as usize, j as usize] {
            self.row_counts[x] -= 1;
            if self.row_counts[x] == 0 {
                let (p, n) = (self.prev[x], self.next[x]);
                if n != -1 {
                    self.prev[n as usize] = p;
                }
                if p != -1 {
                    self.next[p as usize] = n;
                }
                if self.first == x as i32 {
                    self.first = n;
                }
            }
        }
        self.heap.pop()?;
        Ok(Some((i, j, q)))
    }

    /// Recompute the scaling factor and the bound on the delta terms for
    /// the current taxon count and row sums, dropping merged nodes from
    /// the node list.
    pub fn calc_deltas(&mut self, new_k: usize, redirect: &[i32], r: &[f64]) {
        self.k_over_kprime = (new_k as f64 - 2.0) / (self.k_prime as f64 - 2.0);
        let mut min1 = f64::MAX;
        let mut min2 = f64::MAX;
        let mut x = self.first;
        while x != -1 {
            let xu = x as usize;
            let rx = redirect[xu];
            if rx == -1 {
                let (p, n) = (self.prev[xu], self.next[xu]);
                if n != -1 {
                    self.prev[n as usize] = p;
                }
                if p != -1 {
                    self.next[p as usize] = n;
                }
                if self.first == x {
                    self.first = n;
                }
                x = n;
            } else {
                let delta = self.k_over_kprime * self.r_primes[rx as usize] - r[rx as usize];
                if delta < min1 {
                    min2 = min1;
                    min1 = delta;
                } else if delta < min2 {
                    min2 = delta;
                }
                x = self.next[xu];
            }
        }
        self.min_delta_sum = min1 + min2;
    }

    /// Recover the distance from a stored `q'` for a pair with rows
    /// `ri`, `rj`.
    #[inline]
    pub fn distance(&self, q_prime: f32, ri: usize, rj: usize) -> f32 {
        ((q_prime as f64 + self.r_primes[ri] + self.r_primes[rj]) / (self.k_prime as f64 - 2.0)) as f32
    }
}
