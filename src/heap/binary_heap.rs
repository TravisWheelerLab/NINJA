//! A binary min-heap with reference-compatible tie behaviour.

/// A binary min-heap keyed by `K`, carrying a payload `V`.
///
/// This exists instead of [`std::collections::BinaryHeap`] because the order
/// in which equal keys are popped affects which pair neighbor joining merges
/// when Q-values tie, and the tests compare trees byte-for-byte against the
/// reference implementation. The sift-up and sift-down rules here are the
/// classic textbook ones (Weiss): sift up while the new key is strictly
/// smaller than the parent's; sift down toward the left child unless the
/// right child is strictly smaller, and only while the child is strictly
/// smaller than the hole's key.
///
/// The heap also exposes [`chop_bottom`](MinHeap::chop_bottom), which removes
/// the trailing entries of the underlying array. Those are, loosely, the
/// larger keys, and the external-memory heap uses this to spill the least
/// urgent half of its in-memory buffer to disk.
#[derive(Debug, Clone)]
pub struct MinHeap<K, V> {
    items: Vec<(K, V)>,
}

impl<K: PartialOrd + Copy, V: Copy> Default for MinHeap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialOrd + Copy, V: Copy> MinHeap<K, V> {
    /// An empty heap.
    pub fn new() -> Self {
        MinHeap { items: Vec::new() }
    }

    /// An empty heap with room for `cap` entries.
    pub fn with_capacity(cap: usize) -> Self {
        MinHeap { items: Vec::with_capacity(cap) }
    }

    /// Build a heap from unordered entries in linear time (Floyd's method),
    /// as the reference does when re-heapifying a batch.
    pub fn from_entries(entries: Vec<(K, V)>) -> Self {
        let mut h = MinHeap { items: entries };
        let n = h.items.len();
        let mut i = n / 2;
        while i > 0 {
            h.sift_down(i - 1);
            i -= 1;
        }
        h
    }

    /// Number of entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// True when there are no entries.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Remove every entry, keeping the allocation.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// The smallest entry, if any.
    #[inline]
    pub fn peek(&self) -> Option<&(K, V)> {
        self.items.first()
    }

    /// Insert an entry.
    #[inline]
    pub fn push(&mut self, key: K, value: V) {
        self.items.push((key, value));
        // Sift up while strictly smaller than the parent (1-based parent of
        // `hole` is `hole / 2`; in 0-based terms parent of i is (i - 1) / 2).
        let mut hole = self.items.len() - 1;
        while hole > 0 {
            let parent = (hole - 1) / 2;
            if key < self.items[parent].0 {
                self.items[hole] = self.items[parent];
                hole = parent;
            } else {
                break;
            }
        }
        self.items[hole] = (key, value);
    }

    /// Remove and return the smallest entry.
    pub fn pop(&mut self) -> Option<(K, V)> {
        if self.items.is_empty() {
            return None;
        }
        let last = self.items.pop().unwrap();
        if self.items.is_empty() {
            return Some(last);
        }
        let min = std::mem::replace(&mut self.items[0], last);
        self.sift_down(0);
        Some(min)
    }

    /// Remove the last `k` entries of the backing array, returning them in
    /// array order. These are leaves of the heap and therefore not among the
    /// smallest keys, but they are not the largest either.
    pub fn chop_bottom(&mut self, k: usize) -> Vec<(K, V)> {
        let k = k.min(self.items.len());
        let start = self.items.len() - k;
        self.items.drain(start..).collect()
    }

    /// Iterate over entries in array (not sorted) order.
    pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
        self.items.iter()
    }

    /// Keep only the entries for which `keep` returns true, then rebuild.
    pub fn retain(&mut self, mut keep: impl FnMut(&(K, V)) -> bool) {
        let before = self.items.len();
        self.items.retain(|e| keep(e));
        if self.items.len() != before {
            let n = self.items.len();
            let mut i = n / 2;
            while i > 0 {
                self.sift_down(i - 1);
                i -= 1;
            }
        }
    }

    fn sift_down(&mut self, mut hole: usize) {
        let n = self.items.len();
        let item = self.items[hole];
        loop {
            let mut child = 2 * hole + 1;
            if child >= n {
                break;
            }
            if child + 1 < n && self.items[child + 1].0 < self.items[child].0 {
                child += 1;
            }
            if self.items[child].0 < item.0 {
                self.items[hole] = self.items[child];
                hole = child;
            } else {
                break;
            }
        }
        self.items[hole] = item;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pops_in_sorted_order() {
        let mut h = MinHeap::new();
        let mut xs: Vec<i32> = (0..1000).map(|i| (i * 7919) % 613).collect();
        for (i, &x) in xs.iter().enumerate() {
            h.push(x, i);
        }
        xs.sort();
        for x in xs {
            assert_eq!(h.pop().unwrap().0, x);
        }
        assert!(h.pop().is_none());
    }

    #[test]
    fn from_entries_matches_pushes() {
        let entries: Vec<(i64, u32)> = (0..500u32).map(|i| ((i as i64 * 31) % 97, i)).collect();
        let mut a = MinHeap::from_entries(entries.clone());
        let mut b = MinHeap::new();
        for (k, v) in entries {
            b.push(k, v);
        }
        let mut ka = Vec::new();
        let mut kb = Vec::new();
        while let Some((k, _)) = a.pop() {
            ka.push(k);
        }
        while let Some((k, _)) = b.pop() {
            kb.push(k);
        }
        assert_eq!(ka, kb);
    }

    /// Literal transcription of the reference heap (1-based array, slot
    /// indirection, strict comparisons), used to check that `MinHeap` pops
    /// equal keys in the same order.
    struct ReferenceHeap {
        heap_array: Vec<usize>, // index 0 unused
        keys: Vec<f32>,
        vals: Vec<u32>,
        size: usize,
    }

    impl ReferenceHeap {
        fn new() -> Self {
            ReferenceHeap { heap_array: vec![0], keys: Vec::new(), vals: Vec::new(), size: 0 }
        }
        fn insert(&mut self, val: u32, key: f32) {
            let slot = self.keys.len();
            self.keys.push(key);
            self.vals.push(val);
            self.size += 1;
            if self.heap_array.len() <= self.size {
                self.heap_array.push(0);
            }
            let mut hole = self.size;
            self.heap_array[0] = slot;
            while key < self.keys[self.heap_array[hole / 2]] {
                self.heap_array[hole] = self.heap_array[hole / 2];
                hole /= 2;
            }
            self.heap_array[hole] = slot;
        }
        fn delete_min(&mut self) -> u32 {
            let min = self.vals[self.heap_array[1]];
            self.heap_array[1] = self.heap_array[self.size];
            self.size -= 1;
            self.percolate_down(1);
            min
        }
        fn percolate_down(&mut self, mut hole: usize) {
            let pos = self.heap_array[hole];
            while hole * 2 <= self.size {
                let mut child = hole * 2;
                if child != self.size
                    && self.keys[self.heap_array[child + 1]] < self.keys[self.heap_array[child]]
                {
                    child += 1;
                }
                if self.keys[self.heap_array[child]] < self.keys[pos] {
                    self.heap_array[hole] = self.heap_array[child];
                } else {
                    break;
                }
                hole = child;
            }
            self.heap_array[hole] = pos;
        }
    }

    #[test]
    fn ties_pop_in_reference_order() {
        // Mixed inserts and pops with heavily tied keys; a simple LCG keeps
        // the sequence deterministic.
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = move || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (state >> 33) as u32
        };
        let mut ours: MinHeap<f32, u32> = MinHeap::new();
        let mut reference = ReferenceHeap::new();
        for op in 0..20_000u32 {
            let r = next();
            if r % 3 == 0 && !ours.is_empty() {
                let a = ours.pop().unwrap().1;
                let b = reference.delete_min();
                assert_eq!(a, b, "diverged at operation {}", op);
            } else {
                let key = (next() % 7) as f32 * 0.25; // only seven distinct keys
                ours.push(key, op);
                reference.insert(op, key);
            }
        }
        while !ours.is_empty() {
            assert_eq!(ours.pop().unwrap().1, reference.delete_min());
        }
    }

    #[test]
    fn chop_bottom_removes_trailing_entries() {
        let mut h = MinHeap::new();
        for i in 0..10 {
            h.push(i, i);
        }
        let tail = h.chop_bottom(4);
        assert_eq!(tail.len(), 4);
        assert_eq!(h.len(), 6);
        // Remaining entries still form a valid heap and pop in order.
        let mut prev = -1;
        while let Some((k, _)) = h.pop() {
            assert!(k > prev);
            prev = k;
        }
    }
}
