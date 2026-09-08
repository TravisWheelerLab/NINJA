//! External-memory priority queue (an "array heap").
//!
//! This is the structure of Brengel, Crauser, Ferragina and Meyer ("An
//! experimental study of priority queues in external memory", 1999), as
//! adapted for NINJA. Entries are `(i, j, key)` triples with a `f32` key.
//!
//! * `h1` is an ordinary in-memory heap that receives every insert. When it
//!   reaches `2 * c_m` entries, the trailing half of its array (which cannot
//!   contain the minimum) is sorted and spilled to disk as one run.
//! * Disk runs are kept in `max_levels` levels of `num_slots` slots each; a
//!   slot at level `l` holds at most `cnt_max[l]` entries, growing by a
//!   factor of `num_slots + 1` per level. When level 0 has no free slot,
//!   every run below the first level with a free slot is merged, together
//!   with the new run, into one run at that level. Half-empty slots at a
//!   level are merged with each other first when that frees a slot.
//! * `h2` holds the head block of every non-empty run, tagged with its level
//!   and slot, so the global minimum is the smaller of the two heap tops.
//!   When a run's last in-memory entry is popped, its next block is loaded.
//!
//! During merges, entries whose `i` or `j` has been marked dead in the
//! `active` array passed by the caller are dropped, which is how expired
//! node pairs leave the structure without explicit deletion.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use super::MinHeap;
use crate::error::{Error, Result};

/// Entry payload: node pair.
pub type Pair = (i32, i32);

/// Sizing parameters of an [`ArrayHeap`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayHeapConfig {
    /// Memory the structure may use for its in-memory buffers, in bytes.
    /// Determines `c_m` (the run size) and the disk block size.
    pub memory_bytes: u64,
}

impl Default for ArrayHeapConfig {
    fn default() -> Self {
        ArrayHeapConfig { memory_bytes: 1 << 21 }
    }
}

const MAX_LEVELS: usize = 4;
const NUM_FIELDS: usize = 3;
/// Fraction of the memory budget used for the run size; the paper used 1/7,
/// NINJA stores more per entry and uses 1/85.
const C: f64 = 1.0 / 85.0;
/// Output buffering during merges, in blocks.
const OUT_BLOCKS: usize = 10;

/// Sentinel for "next key for this slot not yet fetched".
const UNFETCHED: f32 = f32::MIN_POSITIVE;

/// A disk-backed min-priority queue of `(i, j, key)` triples.
pub struct ArrayHeap {
    block_size: usize,
    c_m: usize,
    num_slots: usize,
    nodes_per_block: usize,
    fields_per_block: usize,
    cnt_max: [u64; MAX_LEVELS],
    n: usize,

    h1: MinHeap<f32, Pair>,
    /// Payload: `(i, j, level, slot)`.
    h2: MinHeap<f32, (i32, i32, u8, u16)>,

    file: File,
    /// `[level][slot]` bookkeeping.
    slot_count: Vec<Vec<u64>>,
    slot_pos: Vec<Vec<u64>>,
    on_heap: Vec<Vec<u32>>,
    buf_pos: Vec<Vec<usize>>,
    slot_buf: Vec<Vec<Vec<i32>>>,
    free_slots: Vec<VecDeque<u16>>,

    read_buf: Vec<i32>,
    out_buf: Vec<i32>,
}

impl std::fmt::Debug for ArrayHeap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArrayHeap")
            .field("n", &self.n)
            .field("c_m", &self.c_m)
            .field("num_slots", &self.num_slots)
            .field("block_size", &self.block_size)
            .finish()
    }
}

impl ArrayHeap {
    /// Create an empty heap whose scratch file lives in `dir`.
    ///
    /// The scratch file is unnamed and removed when the heap is dropped.
    pub fn new(dir: &Path, config: ArrayHeapConfig) -> Result<Self> {
        let mem = config.memory_bytes;
        let block_size = if mem > 1 << 23 {
            4096
        } else if mem > 1 << 22 {
            2048
        } else {
            1024
        };
        let c_m = ((C * mem as f64) as u64).max(16) as usize;
        let num_slots = (c_m / block_size).saturating_sub(1).max(1);
        let nodes_per_block = block_size / NUM_FIELDS;
        let fields_per_block = nodes_per_block * NUM_FIELDS;
        let mut cnt_max = [0u64; MAX_LEVELS];
        cnt_max[0] = c_m as u64;
        for l in 1..MAX_LEVELS {
            cnt_max[l] = cnt_max[l - 1] * (num_slots as u64 + 1);
        }
        let file = tempfile::tempfile_in(dir).map_err(|e| Error::io(dir, e))?;
        let mut h = ArrayHeap {
            block_size,
            c_m,
            num_slots,
            nodes_per_block,
            fields_per_block,
            cnt_max,
            n: 0,
            h1: MinHeap::with_capacity(1000),
            h2: MinHeap::with_capacity(1000),
            file,
            slot_count: vec![vec![0; num_slots]; MAX_LEVELS],
            slot_pos: vec![vec![0; num_slots]; MAX_LEVELS],
            on_heap: vec![vec![0; num_slots]; MAX_LEVELS],
            buf_pos: vec![vec![0; num_slots]; MAX_LEVELS],
            slot_buf: vec![vec![Vec::new(); num_slots]; MAX_LEVELS],
            free_slots: Vec::new(),
            read_buf: vec![0; fields_per_block],
            out_buf: Vec::new(),
        };
        h.clear();
        Ok(h)
    }

    /// Remove every entry. Disk space is reused, not released.
    pub fn clear(&mut self) {
        self.n = 0;
        self.h1.clear();
        self.h2.clear();
        for l in 0..MAX_LEVELS {
            for s in 0..self.num_slots {
                self.slot_count[l][s] = 0;
                self.slot_pos[l][s] = 0;
                self.on_heap[l][s] = 0;
                self.buf_pos[l][s] = 0;
            }
        }
        self.free_slots = (0..MAX_LEVELS).map(|_| (0..self.num_slots as u16).collect()).collect();
    }

    /// Number of live entries.
    pub fn len(&self) -> usize {
        self.n
    }

    /// True when no entries remain.
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// Run size (entries spilled per disk run).
    pub fn run_size(&self) -> usize {
        self.c_m
    }

    /// Insert an entry.
    ///
    /// `active` marks dead node indices with -1; entries touching a dead
    /// node are discarded whenever runs are merged.
    pub fn insert(&mut self, i: i32, j: i32, key: f32, active: Option<&[i32]>) -> Result<()> {
        self.h1.push(key, (i, j));
        self.n += 1;
        if self.h1.len() < 2 * self.c_m {
            return Ok(());
        }
        // Spill the trailing half of the heap array: it holds none of the
        // smallest keys, so those stay resident.
        let chopped = self.h1.chop_bottom(self.c_m);
        let mut sorter = MinHeap::from_entries(chopped);
        let mut run: Vec<(f32, Pair)> = Vec::with_capacity(self.c_m);
        while let Some(e) = sorter.pop() {
            run.push(e);
        }

        let mut target = 0;
        while self.free_slots[target].is_empty() {
            if self.merge_slots(target, active)? {
                break;
            }
            target += 1;
            if target == MAX_LEVELS {
                return Err(Error::invalid(
                    "external-memory heap exhausted its disk levels; increase the memory budget",
                ));
            }
        }
        let slot = if target == 0 { self.store(0, &run)? } else { self.merge_levels(target, &run, active)? };
        self.load(target, slot)?;
        Ok(())
    }

    /// The smallest entry as `(i, j, key)`, if any.
    pub fn peek(&self) -> Option<(i32, i32, f32)> {
        match (self.h1.peek(), self.h2.peek()) {
            (None, None) => None,
            (Some(&(k, (i, j))), None) => Some((i, j, k)),
            (None, Some(&(k, (i, j, _, _)))) => Some((i, j, k)),
            (Some(&(k1, (i1, j1))), Some(&(k2, (i2, j2, _, _)))) => {
                if k1 <= k2 {
                    Some((i1, j1, k1))
                } else {
                    Some((i2, j2, k2))
                }
            }
        }
    }

    /// Remove the smallest entry.
    pub fn pop(&mut self) -> Result<Option<(i32, i32, f32)>> {
        let from_h2 = match (self.h1.peek(), self.h2.peek()) {
            (None, None) => return Ok(None),
            (Some(_), None) => false,
            (None, Some(_)) => true,
            (Some(&(k1, _)), Some(&(k2, _))) => k1 > k2,
        };
        self.n -= 1;
        if !from_h2 {
            let (k, (i, j)) = self.h1.pop().unwrap();
            return Ok(Some((i, j, k)));
        }
        let (k, (i, j, level, slot)) = self.h2.pop().unwrap();
        let (l, s) = (level as usize, slot as usize);
        self.on_heap[l][s] -= 1;
        if self.on_heap[l][s] == 0 {
            self.load(l, s)?;
        }
        Ok(Some((i, j, k)))
    }

    fn byte_pos(&self, level: usize, slot: usize) -> u64 {
        let mut pos = 0u64;
        for l in 0..level {
            pos += self.num_slots as u64 * self.cnt_max[l] * NUM_FIELDS as u64 * 4;
        }
        pos + slot as u64 * self.cnt_max[level] * NUM_FIELDS as u64 * 4
    }

    fn read_ints(&mut self, pos: u64, out: &mut [i32]) -> Result<usize> {
        self.file.seek(SeekFrom::Start(pos))?;
        let mut bytes = vec![0u8; out.len() * 4];
        let mut got = 0;
        while got < bytes.len() {
            let r = self.file.read(&mut bytes[got..])?;
            if r == 0 {
                break;
            }
            got += r;
        }
        for (k, chunk) in bytes[..got / 4 * 4].chunks_exact(4).enumerate() {
            out[k] = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        Ok(got / 4)
    }

    fn write_ints(&mut self, pos: u64, data: &[i32]) -> Result<()> {
        self.file.seek(SeekFrom::Start(pos))?;
        let mut bytes = Vec::with_capacity(data.len() * 4);
        for v in data {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        self.file.write_all(&bytes)?;
        Ok(())
    }

    /// Write a sorted run into a free slot at `level`; returns the slot.
    fn store(&mut self, level: usize, run: &[(f32, Pair)]) -> Result<usize> {
        let slot = self.free_slots[level]
            .pop_front()
            .ok_or_else(|| Error::invalid("array heap: no free slot to store into"))?
            as usize;
        let mut data = Vec::with_capacity(run.len() * NUM_FIELDS);
        for &(k, (i, j)) in run {
            data.push(i);
            data.push(j);
            data.push(k.to_bits() as i32);
        }
        let pos = self.byte_pos(level, slot);
        self.write_ints(pos, &data)?;
        self.slot_pos[level][slot] = 0;
        self.on_heap[level][slot] = 0;
        self.slot_count[level][slot] = run.len() as u64;
        Ok(slot)
    }

    /// Move the next block of a run onto `h2`, or free the slot if the run
    /// is exhausted.
    fn load(&mut self, level: usize, slot: usize) -> Result<()> {
        let count = self.slot_count[level][slot];
        if self.slot_pos[level][slot] == count {
            self.slot_count[level][slot] = 0;
            self.slot_pos[level][slot] = 0;
            self.on_heap[level][slot] = 0;
            self.free_slots[level].push_back(slot as u16);
            return Ok(());
        }
        let pos = self.byte_pos(level, slot) + 4 * NUM_FIELDS as u64 * self.slot_pos[level][slot];
        let mut buf = std::mem::take(&mut self.read_buf);
        let got = self.read_ints(pos, &mut buf)?;
        if got == 0 {
            self.read_buf = buf;
            return Err(Error::invalid("array heap: unexpected end of scratch file"));
        }
        let end = (self.slot_pos[level][slot] + self.nodes_per_block as u64).min(count);
        self.on_heap[level][slot] = (end - self.slot_pos[level][slot]) as u32;
        let mut k = 0;
        while self.slot_pos[level][slot] < end {
            let i = buf[k];
            let j = buf[k + 1];
            let key = f32::from_bits(buf[k + 2] as u32);
            k += NUM_FIELDS;
            self.h2.push(key, (i, j, level as u8, slot as u16));
            self.slot_pos[level][slot] += 1;
        }
        self.read_buf = buf;
        Ok(())
    }

    /// Rebuild `h2` without any entries from the given `(level, slot)` set.
    fn drop_from_h2(&mut self, mut drop: impl FnMut(u8, u16) -> bool) {
        self.h2.retain(|&(_, (_, _, l, s))| !drop(l, s));
    }

    /// Fetch the head key of run `(level, slot)`, refilling its block buffer
    /// from disk and skipping dead entries. Returns `None` when exhausted.
    /// Decrements `n` for every dead entry dropped.
    fn head_key(&mut self, level: usize, slot: usize, active: Option<&[i32]>) -> Result<Option<f32>> {
        loop {
            if self.slot_pos[level][slot] == self.slot_count[level][slot] {
                return Ok(None);
            }
            if self.buf_pos[level][slot] >= self.fields_per_block {
                let base = self.byte_pos(level, slot);
                let pos = base + NUM_FIELDS as u64 * 4 * self.slot_pos[level][slot];
                let mut buf = std::mem::take(&mut self.slot_buf[level][slot]);
                buf.resize(self.fields_per_block, 0);
                self.read_ints(pos, &mut buf)?;
                self.slot_buf[level][slot] = buf;
                self.buf_pos[level][slot] = 0;
            }
            let bp = self.buf_pos[level][slot];
            let buf = &self.slot_buf[level][slot];
            let (i, j) = (buf[bp], buf[bp + 1]);
            let dead = match active {
                Some(a) => a[i as usize] == -1 || a[j as usize] == -1,
                None => false,
            };
            if dead {
                self.buf_pos[level][slot] += NUM_FIELDS;
                self.slot_pos[level][slot] += 1;
                self.n -= 1;
                continue;
            }
            return Ok(Some(f32::from_bits(buf[bp + 2] as u32)));
        }
    }

    /// Take the head triple of run `(level, slot)` (after `head_key`).
    fn take_head(&mut self, level: usize, slot: usize) -> [i32; 3] {
        let bp = self.buf_pos[level][slot];
        let buf = &self.slot_buf[level][slot];
        let t = [buf[bp], buf[bp + 1], buf[bp + 2]];
        self.buf_pos[level][slot] += NUM_FIELDS;
        self.slot_pos[level][slot] += 1;
        t
    }

    /// Merge every run below `target`, plus `run`, into a free slot at
    /// `target`. Returns the slot.
    fn merge_levels(&mut self, target: usize, run: &[(f32, Pair)], active: Option<&[i32]>) -> Result<usize> {
        let target_slot = self.free_slots[target]
            .pop_front()
            .ok_or_else(|| Error::invalid("array heap: no free slot at merge level"))?
            as usize;
        let mut out_pos = self.byte_pos(target, target_slot);

        // Rewind each lower run to include the entries currently on h2, and
        // drop those from h2.
        let mut heads: Vec<Vec<f32>> = vec![vec![UNFETCHED; self.num_slots]; target];
        for level in 0..target {
            for slot in 0..self.num_slots {
                self.slot_pos[level][slot] -= self.on_heap[level][slot] as u64;
                self.buf_pos[level][slot] = self.fields_per_block;
                self.on_heap[level][slot] = 0;
            }
        }
        self.drop_from_h2(|l, _| (l as usize) < target);

        let mut out = std::mem::take(&mut self.out_buf);
        out.clear();
        let out_cap = OUT_BLOCKS * self.fields_per_block;
        let mut input_pos = 0;
        let mut new_cnt = 0u64;
        // Number of lower runs not yet exhausted, maintained lazily: we
        // simply loop until no source yields an entry.
        loop {
            let mut min_key = f32::MAX;
            let mut min_src: Option<(usize, usize)> = None; // None = input run
            let mut have = false;
            if input_pos < run.len() {
                min_key = run[input_pos].0;
                have = true;
            }
            for level in 0..target {
                for slot in 0..self.num_slots {
                    if heads[level][slot] == UNFETCHED {
                        match self.head_key(level, slot, active)? {
                            Some(k) => heads[level][slot] = k,
                            None => continue,
                        }
                    }
                    let k = heads[level][slot];
                    if !have || k < min_key {
                        min_key = k;
                        min_src = Some((level, slot));
                        have = true;
                    }
                }
            }
            if !have {
                break;
            }
            match min_src {
                None => {
                    let (k, (i, j)) = run[input_pos];
                    out.push(i);
                    out.push(j);
                    out.push(k.to_bits() as i32);
                    input_pos += 1;
                }
                Some((level, slot)) => {
                    let t = self.take_head(level, slot);
                    out.extend_from_slice(&t);
                    heads[level][slot] = UNFETCHED;
                }
            }
            new_cnt += 1;
            if out.len() == out_cap {
                self.write_ints(out_pos, &out)?;
                out_pos += out.len() as u64 * 4;
                out.clear();
            }
        }
        if !out.is_empty() {
            self.write_ints(out_pos, &out)?;
            out.clear();
        }
        self.out_buf = out;

        for level in 0..target {
            self.free_slots[level] = (0..self.num_slots as u16).collect();
            for slot in 0..self.num_slots {
                self.on_heap[level][slot] = 0;
                self.slot_count[level][slot] = 0;
                self.slot_pos[level][slot] = 0;
            }
        }
        self.slot_count[target][target_slot] = new_cnt;
        self.on_heap[target][target_slot] = 0;
        self.slot_pos[target][target_slot] = 0;
        Ok(target_slot)
    }

    /// Merge the smallest half-empty runs at `level` into one run when at
    /// least two exist; returns whether a merge happened.
    fn merge_slots(&mut self, level: usize, active: Option<&[i32]>) -> Result<bool> {
        if level == MAX_LEVELS - 1 {
            return Err(Error::invalid(
                "external-memory heap needs to merge runs at its top level, which is not supported; \
                 increase the memory budget",
            ));
        }
        let half = self.cnt_max[level] / 2;
        let mut cands: Vec<(u64, usize)> = Vec::new();
        for slot in 0..self.num_slots {
            if self.on_heap[level][slot] > 0 {
                let remaining = self.slot_count[level][slot] - self.slot_pos[level][slot]
                    + self.on_heap[level][slot] as u64;
                if remaining <= half {
                    cands.push((remaining, slot));
                }
            }
        }
        if cands.len() < 2 {
            return Ok(false);
        }
        cands.sort_by_key(|&(r, _)| r);
        let mut summed = cands[0].0 + cands[1].0;
        let mut take = 2;
        while take < cands.len() && summed + cands[take].0 <= self.cnt_max[level] {
            summed += cands[take].0;
            take += 1;
        }
        let slots: Vec<usize> = cands[..take].iter().map(|&(_, s)| s).collect();

        let mut heads = vec![UNFETCHED; self.num_slots];
        for &s in &slots {
            self.slot_pos[level][s] -= self.on_heap[level][s] as u64;
            self.buf_pos[level][s] = self.fields_per_block;
        }
        let mut merged: Vec<(f32, Pair)> = Vec::with_capacity(summed as usize);
        loop {
            let mut min_key = f32::MAX;
            let mut min_slot = None;
            for &s in &slots {
                if heads[s] == UNFETCHED {
                    match self.head_key(level, s, active)? {
                        Some(k) => heads[s] = k,
                        None => continue,
                    }
                }
                if min_slot.is_none() || heads[s] < min_key {
                    min_key = heads[s];
                    min_slot = Some(s);
                }
            }
            let Some(s) = min_slot else { break };
            let t = self.take_head(level, s);
            merged.push((f32::from_bits(t[2] as u32), (t[0], t[1])));
            heads[s] = UNFETCHED;
        }

        self.drop_from_h2(|l, s| l as usize == level && slots.contains(&(s as usize)));
        for &s in &slots {
            self.free_slots[level].push_back(s as u16);
            self.on_heap[level][s] = 0;
            self.slot_pos[level][s] = 0;
            self.slot_count[level][s] = 0;
        }
        let loc = self.store(level, &merged)?;
        self.load(level, loc)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lcg(state: &mut u64) -> u32 {
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*state >> 33) as u32
    }

    #[test]
    fn pops_sorted_across_disk_levels() {
        let dir = tempfile::tempdir().unwrap();
        // A tiny memory budget forces many spills and level merges.
        let mut h = ArrayHeap::new(dir.path(), ArrayHeapConfig { memory_bytes: 1 << 20 }).unwrap();
        assert!(h.run_size() < 20_000);
        let mut state = 7u64;
        let n = 400_000;
        let mut keys = Vec::with_capacity(n);
        for i in 0..n {
            let k = (lcg(&mut state) % 100_000) as f32 * 1e-3;
            h.insert(i as i32, 0, k, None).unwrap();
            keys.push(k);
        }
        assert_eq!(h.len(), n);
        keys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for (idx, &k) in keys.iter().enumerate() {
            let (_, _, got) = h.pop().unwrap().unwrap();
            assert_eq!(got, k, "mismatch at pop {}", idx);
        }
        assert!(h.is_empty());
        assert!(h.pop().unwrap().is_none());
    }

    #[test]
    fn interleaved_inserts_and_pops() {
        use std::cmp::Reverse;
        use std::collections::BinaryHeap;
        let dir = tempfile::tempdir().unwrap();
        let mut h = ArrayHeap::new(dir.path(), ArrayHeapConfig { memory_bytes: 1 << 20 }).unwrap();
        let mut state = 99u64;
        let mut shadow: BinaryHeap<Reverse<u32>> = BinaryHeap::new();
        for i in 0..300_000 {
            if lcg(&mut state) % 4 == 0 && !shadow.is_empty() {
                let want = shadow.pop().unwrap().0;
                let (_, _, got) = h.pop().unwrap().unwrap();
                assert_eq!(got, want as f32);
            } else {
                let k = lcg(&mut state) % 5000;
                h.insert(i, i + 1, k as f32, None).unwrap();
                shadow.push(Reverse(k));
            }
        }
        assert_eq!(h.len(), shadow.len());
    }

    #[test]
    fn dead_entries_are_dropped_during_merges() {
        let dir = tempfile::tempdir().unwrap();
        let mut h = ArrayHeap::new(dir.path(), ArrayHeapConfig { memory_bytes: 1 << 20 }).unwrap();
        let n = 100_000i32;
        // Node 1 dies after the first half of the inserts.
        let mut active = vec![0i32; 3];
        for i in 0..n {
            let j = if i % 2 == 0 { 1 } else { 2 };
            let k = ((i * 7919) % 10007) as f32;
            h.insert(0, j, k, Some(&active)).unwrap();
            if i == n / 2 {
                active[1] = -1;
            }
        }
        // Everything still pops in order; entries with j == 1 may or may not
        // have been dropped depending on when merges ran.
        let mut prev = -1.0f32;
        let mut popped = 0;
        while let Some((_, _, k)) = h.pop().unwrap() {
            assert!(k >= prev);
            prev = k;
            popped += 1;
        }
        assert!(popped <= n as usize);
        assert!(popped > (n / 2) as usize);
    }
}
