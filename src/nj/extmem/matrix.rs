//! Column-blocked float distance matrix, partly on disk.
//!
//! The external-memory engine works on a `K x (2K - 2)` matrix of `f32`:
//! the first `K` columns are the input distances and each later column
//! holds the distances from one internal node to every other node alive when
//! it was created. Only the last `mem_cols` columns are resident; whenever
//! the resident window fills, it is appended to the on-disk rows and the
//! window advances. Rows are indexed by matrix row (an internal node reuses
//! the row of its left child); columns are indexed by node index.
//!
//! The row of a node written at flush time also gets its full column written
//! out as a row, so that a later lookup of `D(x, node)` for two old nodes
//! can read `disk[row(node)][x]`.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use rayon::prelude::*;

use crate::distance::DistanceCalculator;
use crate::error::{Error, Result};

/// Disk block size in floats; the resident window is a multiple of this.
pub const PAGE_BLOCK: usize = 1024;

/// The engine's view of the distance matrix.
pub struct DiskMatrix {
    /// Number of taxa.
    pub k: usize,
    /// Floats per on-disk row: `2K - 2`.
    pub row_len: usize,
    /// Width of the resident window.
    pub mem_cols: usize,
    /// Resident window, `k * mem_cols`, row-major.
    mem: Vec<f32>,
    /// On-disk rows, or `None` when the whole matrix is resident.
    disk: Option<File>,
    /// First column held in the resident window.
    pub first_mem_col: usize,
    /// Initial row sums.
    pub r: Vec<f32>,
}

impl std::fmt::Debug for DiskMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiskMatrix")
            .field("k", &self.k)
            .field("row_len", &self.row_len)
            .field("mem_cols", &self.mem_cols)
            .field("first_mem_col", &self.first_mem_col)
            .field("on_disk", &self.disk.is_some())
            .finish()
    }
}

/// Round a distance to seven decimals as the reference did:
/// `round(d * 1e7) / 1e7` in single precision, ties toward +infinity.
#[inline]
pub fn round7(d: f64) -> f32 {
    let f = (1e7 * d) as f32;
    let r = (f as f64 + 0.5).floor();
    r as f32 / 1e7f32
}

impl DiskMatrix {
    /// Choose the resident window width for `k` taxa under a memory budget.
    ///
    /// A tenth of the budget goes to the window (the heaps and candidate
    /// structures need the rest), rounded down to whole blocks, at least
    /// one block, and never more than the full width.
    pub fn window_width(k: usize, memory_bytes: u64) -> usize {
        let full = 2 * k - 2;
        let max_bytes = memory_bytes / 10;
        let cols = (max_bytes / (4 * k as u64)) as usize;
        let blocks = (cols / PAGE_BLOCK).max(1);
        (blocks * PAGE_BLOCK).min(full)
    }

    fn allocate(k: usize, memory_bytes: u64, tmp_dir: &Path) -> Result<Self> {
        if k < 2 {
            return Err(Error::invalid("the external-memory engine needs at least two taxa"));
        }
        let row_len = 2 * k - 2;
        let mem_cols = Self::window_width(k, memory_bytes);
        let disk = if mem_cols >= row_len {
            None
        } else {
            let f = tempfile::tempfile_in(tmp_dir).map_err(|e| Error::io(tmp_dir, e))?;
            f.set_len(k as u64 * row_len as u64 * 4)?;
            Some(f)
        };
        Ok(DiskMatrix {
            k,
            row_len,
            mem_cols,
            mem: vec![0.0; k * mem_cols],
            disk,
            first_mem_col: 0,
            r: vec![0.0; k],
        })
    }

    /// Columns `0..n` of every input row go to disk; the rest stay
    /// resident. Leaves room for half the window's worth of new columns
    /// before the first flush.
    fn cols_to_disk(&self) -> usize {
        if self.mem_cols >= self.row_len {
            return 0;
        }
        let n = self.row_len - self.mem_cols / 2;
        (n / PAGE_BLOCK * PAGE_BLOCK).min(self.k)
    }

    /// Fill the matrix from a distance calculator, computing rows in
    /// parallel.
    pub fn from_calculator(calc: &DistanceCalculator, memory_bytes: u64, tmp_dir: &Path) -> Result<Self> {
        let k = calc.len();
        let mut m = Self::allocate(k, memory_bytes, tmp_dir)?;
        let to_disk = m.cols_to_disk();
        m.first_mem_col = to_disk;
        let mem_cols = m.mem_cols;

        // Resident part and row sums, row-parallel. Each row sums its own
        // distances in column order, as the reference did.
        let rows: Vec<(f32, Vec<f32>)> = (0..k)
            .into_par_iter()
            .map(|row| {
                let mut sum = 0f32;
                let mut disk_part = Vec::with_capacity(to_disk);
                for col in 0..k {
                    let d = if col == row { 0.0 } else { round7(calc.calc(row, col)) };
                    sum += d;
                    if col < to_disk {
                        disk_part.push(d);
                    }
                }
                (sum, disk_part)
            })
            .collect();
        // Second pass for the resident columns (kept separate so the disk
        // part above is the only per-row allocation of size `to_disk`).
        m.mem.par_chunks_mut(mem_cols).enumerate().for_each(|(row, chunk)| {
            for col in to_disk..k {
                let d = if col == row { 0.0 } else { round7(calc.calc(row, col)) };
                chunk[col - to_disk] = d;
            }
        });
        for (row, (sum, disk_part)) in rows.into_iter().enumerate() {
            m.r[row] = sum;
            if to_disk > 0 {
                m.write_disk(row, 0, &disk_part)?;
            }
        }
        Ok(m)
    }

    /// Fill the matrix from parsed Phylip rows (`lower[i][j]` for `j < i`,
    /// in `1e-8` units) with the given memory budget.
    pub fn from_phylip(
        p: &crate::io::phylip::PhylipMatrix,
        memory_bytes: u64,
        tmp_dir: &Path,
    ) -> Result<Self> {
        let k = p.len();
        let mut m = Self::allocate(k, memory_bytes, tmp_dir)?;
        let to_disk = m.cols_to_disk();
        m.first_mem_col = to_disk;
        let mut disk_part = vec![0f32; to_disk];
        for row in 0..k {
            let mut sum = 0f32;
            for col in 0..k {
                // The reference parsed the decimal text straight to single
                // precision; the correctly rounded double of the exact value
                // narrows to the same float in all but pathological cases.
                let d = if col == row { 0.0 } else { (p.get(row, col) as f64 / 1e8) as f32 };
                sum += d;
                if col < to_disk {
                    disk_part[col] = d;
                } else {
                    m.mem[row * m.mem_cols + col - to_disk] = d;
                }
            }
            m.r[row] = sum;
            if to_disk > 0 {
                m.write_disk(row, 0, &disk_part)?;
            }
        }
        Ok(m)
    }

    /// True when part of the matrix lives on disk.
    pub fn uses_disk(&self) -> bool {
        self.disk.is_some()
    }

    /// Resident entry at `(row, col)`; `col >= first_mem_col`.
    #[inline]
    pub fn mem_get(&self, row: usize, col: usize) -> f32 {
        debug_assert!(col >= self.first_mem_col);
        self.mem[row * self.mem_cols + (col - self.first_mem_col)]
    }

    /// Set the resident entry at `(row, col)`; `col >= first_mem_col`.
    #[inline]
    pub fn mem_set(&mut self, row: usize, col: usize, v: f32) {
        debug_assert!(col >= self.first_mem_col);
        let idx = row * self.mem_cols + (col - self.first_mem_col);
        self.mem[idx] = v;
    }

    /// The resident row slice.
    #[inline]
    pub fn mem_row(&self, row: usize) -> &[f32] {
        &self.mem[row * self.mem_cols..(row + 1) * self.mem_cols]
    }

    /// Read `buf.len()` floats of on-disk row `row` starting at `col`.
    pub fn read_disk(&mut self, row: usize, col: usize, buf: &mut [f32]) -> Result<()> {
        let f = self
            .disk
            .as_mut()
            .ok_or_else(|| Error::invalid("distance matrix is fully resident; no disk to read"))?;
        let pos = 4 * (self.row_len as u64 * row as u64 + col as u64);
        f.seek(SeekFrom::Start(pos))?;
        let mut bytes = vec![0u8; buf.len() * 4];
        f.read_exact(&mut bytes)?;
        for (v, c) in buf.iter_mut().zip(bytes.chunks_exact(4)) {
            *v = f32::from_le_bytes([c[0], c[1], c[2], c[3]]);
        }
        Ok(())
    }

    /// Read a single on-disk entry.
    pub fn read_disk_one(&mut self, row: usize, col: usize) -> Result<f32> {
        let mut b = [0f32; 1];
        self.read_disk(row, col, &mut b)?;
        Ok(b[0])
    }

    /// Write floats to on-disk row `row` starting at `col`.
    pub fn write_disk(&mut self, row: usize, col: usize, data: &[f32]) -> Result<()> {
        let f = self
            .disk
            .as_mut()
            .ok_or_else(|| Error::invalid("distance matrix is fully resident; no disk to write"))?;
        let pos = 4 * (self.row_len as u64 * row as u64 + col as u64);
        f.seek(SeekFrom::Start(pos))?;
        let mut bytes = Vec::with_capacity(data.len() * 4);
        for v in data {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        f.write_all(&bytes)?;
        Ok(())
    }

    /// Append the resident window of every listed row to disk.
    pub fn flush_rows(&mut self, rows: impl Iterator<Item = usize>) -> Result<()> {
        let first = self.first_mem_col;
        let width = self.mem_cols;
        let mut buf = vec![0f32; width];
        for row in rows {
            buf.copy_from_slice(&self.mem[row * width..(row + 1) * width]);
            self.write_disk(row, first, &buf)?;
        }
        Ok(())
    }
}

/// A sliding read buffer over one on-disk row, so a sequential scan of old
/// columns reads the file a block at a time.
pub struct RowPager {
    row: usize,
    start: usize,
    buf: Vec<f32>,
    valid: usize,
}

impl RowPager {
    /// A pager for `row` with `width` floats per page.
    pub fn new(row: usize, width: usize) -> Self {
        RowPager { row, start: usize::MAX, buf: vec![0.0; width], valid: 0 }
    }

    /// Entry at column `col` of the pager's row (must be an on-disk column).
    #[inline]
    pub fn get(&mut self, m: &mut DiskMatrix, col: usize) -> Result<f32> {
        if self.start == usize::MAX || col < self.start || col >= self.start + self.valid {
            let width = self.buf.len();
            let start = col / width * width;
            let n = width.min(m.row_len - start);
            m.read_disk(self.row, start, &mut self.buf[..n])?;
            self.start = start;
            self.valid = n;
        }
        Ok(self.buf[col - self.start])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round7_matches_reference_rounding() {
        assert_eq!(round7(0.0), 0.0);
        assert_eq!(round7(0.12345675), 0.1234568);
        assert_eq!(round7(0.12345674), 0.1234567);
        assert_eq!(round7(3.0), 3.0);
    }

    #[test]
    fn window_width_bounds() {
        assert_eq!(DiskMatrix::window_width(10, 1 << 30), 18);
        assert_eq!(DiskMatrix::window_width(100_000, 1 << 30), PAGE_BLOCK);
        assert_eq!(DiskMatrix::window_width(5_000, 1 << 30), 5 * PAGE_BLOCK);
    }

    #[test]
    fn disk_round_trip_and_pager() {
        let dir = tempfile::tempdir().unwrap();
        // A tiny budget gives a one-block window, which is narrower than the
        // matrix once there are more than 513 taxa.
        let k = 600;
        let mut m = DiskMatrix::allocate(k, 1, dir.path()).unwrap();
        assert!(m.uses_disk());
        assert_eq!(m.mem_cols, PAGE_BLOCK);
        let data: Vec<f32> = (0..m.row_len).map(|c| c as f32 * 0.5).collect();
        m.write_disk(3, 0, &data).unwrap();
        assert_eq!(m.read_disk_one(3, 17).unwrap(), 8.5);
        let mut pager = RowPager::new(3, 8);
        for c in (0..m.row_len).rev() {
            assert_eq!(pager.get(&mut m, c).unwrap(), c as f32 * 0.5);
        }
    }

    #[test]
    fn small_matrices_stay_resident() {
        let dir = tempfile::tempdir().unwrap();
        let m = DiskMatrix::allocate(40, 1, dir.path()).unwrap();
        assert!(!m.uses_disk());
        assert_eq!(m.mem_cols, 78);
        assert_eq!(m.cols_to_disk(), 0);
    }
}
