//! Priority queue of node pairs used by the in-memory engine.
//!
//! Entries are consumed in increasing distance order with lazy deletion:
//! a pair whose node has been joined is discarded when it reaches the
//! front. Most entries never move after a rebuild, so the bulk that a
//! rebuild inserts is kept as a sorted run read through a cursor, and only
//! entries added after the rebuild go into a binary heap. Popping a stale
//! entry from the run costs a cursor increment instead of a sift-down.
//!
//! In reference-order mode every entry goes through the heap, which
//! reproduces the original implementation's order among equal keys.

use super::MinHeap;

/// Heap key: fixed-point distance; payload: node indices `(i, j)`.
pub type Entry = (i32, (i32, i32));

/// A queue of pairs ordered by distance.
#[derive(Debug, Clone)]
pub struct PairQueue {
    run: Vec<Entry>,
    cursor: usize,
    heap: MinHeap<i32, (i32, i32)>,
    reference_order: bool,
}

impl PairQueue {
    /// An empty queue. With `reference_order`, bulk fills also go through
    /// the heap so that ties resolve as in the reference implementation.
    pub fn new(reference_order: bool) -> Self {
        PairQueue { run: Vec::new(), cursor: 0, heap: MinHeap::new(), reference_order }
    }

    /// Remove every entry. The run's memory is released (a rebuild
    /// supplies a new one); the heap keeps its allocation.
    pub fn clear(&mut self) {
        self.run = Vec::new();
        self.cursor = 0;
        self.heap.clear();
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.run.len() - self.cursor + self.heap.len()
    }

    /// True when no entries remain.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Insert one entry.
    #[inline]
    pub fn push(&mut self, key: i32, pair: (i32, i32)) {
        self.heap.push(key, pair);
    }

    /// Insert many entries at once (a rebuild). In reference-order mode
    /// they are pushed one by one, in the given order; otherwise they are
    /// sorted into the run.
    pub fn fill(&mut self, mut entries: Vec<Entry>) {
        if self.reference_order {
            for (k, p) in entries {
                self.heap.push(k, p);
            }
        } else {
            entries.sort_unstable_by_key(|e| e.0);
            self.run = entries;
            self.cursor = 0;
        }
    }

    /// The smallest entry. On a tie between the run and the heap, the run
    /// entry (the older one) comes first.
    #[inline]
    pub fn peek(&self) -> Option<Entry> {
        let from_run = self.run.get(self.cursor).copied();
        match (from_run, self.heap.peek()) {
            (None, None) => None,
            (Some(r), None) => Some(r),
            (None, Some(&(k, p))) => Some((k, p)),
            (Some(r), Some(&(k, p))) => {
                if r.0 <= k {
                    Some(r)
                } else {
                    Some((k, p))
                }
            }
        }
    }

    /// Remove the smallest entry.
    #[inline]
    pub fn pop(&mut self) -> Option<Entry> {
        let from_run = self.run.get(self.cursor).copied();
        match (from_run, self.heap.peek()) {
            (None, None) => None,
            (Some(r), None) => {
                self.cursor += 1;
                Some(r)
            }
            (None, Some(_)) => self.heap.pop(),
            (Some(r), Some(&(k, _))) => {
                if r.0 <= k {
                    self.cursor += 1;
                    Some(r)
                } else {
                    self.heap.pop()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_run_and_heap_in_order() {
        for reference in [false, true] {
            let mut q = PairQueue::new(reference);
            let bulk: Vec<Entry> = (0..1000).map(|i| ((i * 7919) % 613, (i, 0))).collect();
            q.fill(bulk.clone());
            for i in 0..300 {
                q.push((i * 31) % 613, (i, 1));
            }
            let mut keys: Vec<i32> =
                bulk.iter().map(|e| e.0).chain((0..300).map(|i| (i * 31) % 613)).collect();
            keys.sort();
            assert_eq!(q.len(), keys.len());
            for k in keys {
                assert_eq!(q.peek().unwrap().0, k);
                assert_eq!(q.pop().unwrap().0, k);
            }
            assert!(q.is_empty());
            assert!(q.pop().is_none());
        }
    }
}
