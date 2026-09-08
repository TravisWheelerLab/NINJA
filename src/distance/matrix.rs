//! Fixed-point distance matrix for the in-memory engine.

use rayon::prelude::*;

use super::DistanceCalculator;
use crate::io::phylip::{PhylipMatrix, SCALE};

/// The strict upper triangle of a symmetric distance matrix, stored as
/// fixed-point `i32` values in units of `1e-8`, rounded to a multiple of 100.
///
/// Row `i` holds the distances to `i+1..k`, packed contiguously. The
/// in-memory engine mutates entries in place as nodes are merged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistanceMatrix {
    k: usize,
    data: Vec<i32>,
}

impl DistanceMatrix {
    /// Fixed-point scale of the stored values.
    pub const SCALE: i64 = SCALE;

    /// Convert a real-valued distance to the stored fixed-point form.
    ///
    /// This is `100 * trunc((d * 1e8 + 50) / 100)`: round half up to the
    /// nearest `1e-6`, which is the precision the Phylip writer prints, so a
    /// tree built from an alignment and one built from the written matrix
    /// see identical distances.
    #[inline]
    pub fn quantize(d: f64) -> i32 {
        100 * (((SCALE as f64 * d) + 50.0) / 100.0) as i32
    }

    /// An all-zero matrix for `k` taxa.
    pub fn zeros(k: usize) -> Self {
        DistanceMatrix { k, data: vec![0; Self::tri_len(k)] }
    }

    fn tri_len(k: usize) -> usize {
        k * k.saturating_sub(1) / 2
    }

    /// Compute every pairwise distance from an alignment, in parallel.
    ///
    /// Rows are distributed across the current rayon thread pool.
    pub fn from_calculator(calc: &DistanceCalculator) -> Self {
        let k = calc.len();
        let mut m = Self::zeros(k);
        // Split `data` into per-row slices of decreasing length so rows can
        // be filled independently.
        let mut rows: Vec<&mut [i32]> = Vec::with_capacity(k);
        let mut rest: &mut [i32] = &mut m.data;
        for i in 0..k {
            let (row, tail) = rest.split_at_mut(k - i - 1);
            rows.push(row);
            rest = tail;
        }
        rows.into_par_iter().enumerate().for_each(|(i, row)| {
            for (off, cell) in row.iter_mut().enumerate() {
                let j = i + 1 + off;
                *cell = Self::quantize(calc.calc(i, j));
            }
        });
        m
    }

    /// Build from a parsed Phylip matrix (already in `1e-8` units).
    pub fn from_phylip(p: &PhylipMatrix) -> Self {
        let k = p.len();
        let mut m = Self::zeros(k);
        for i in 0..k {
            for j in (i + 1)..k {
                let v = p.lower[j][i];
                // Match the quantisation applied to computed distances.
                let v = 100 * ((v + 50) / 100);
                m.data[Self::index(k, i, j)] = v.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
            }
        }
        m
    }

    /// Number of taxa.
    #[inline]
    pub fn len(&self) -> usize {
        self.k
    }

    /// True when there are no taxa.
    pub fn is_empty(&self) -> bool {
        self.k == 0
    }

    #[inline]
    fn index(k: usize, i: usize, j: usize) -> usize {
        debug_assert!(i < j && j < k);
        i * (2 * k - i - 1) / 2 + (j - i - 1)
    }

    /// Distance between `i` and `j` (`i != j`), in fixed-point units.
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> i32 {
        let (a, b) = if i < j { (i, j) } else { (j, i) };
        self.data[Self::index(self.k, a, b)]
    }

    /// Overwrite the distance between `i` and `j` (`i != j`).
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, v: i32) {
        let (a, b) = if i < j { (i, j) } else { (j, i) };
        let k = self.k;
        self.data[Self::index(k, a, b)] = v;
    }

    /// Distance between `i` and `j` as a real number.
    pub fn get_f64(&self, i: usize, j: usize) -> f64 {
        self.get(i, j) as f64 / SCALE as f64
    }

    /// The packed upper triangle.
    pub fn as_slice(&self) -> &[i32] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantize_rounds_to_hundreds() {
        assert_eq!(DistanceMatrix::quantize(0.705596), 70_559_600);
        assert_eq!(DistanceMatrix::quantize(0.7055964), 70_559_600);
        assert_eq!(DistanceMatrix::quantize(0.7055965), 70_559_700);
        assert_eq!(DistanceMatrix::quantize(0.0), 0);
        assert_eq!(DistanceMatrix::quantize(3.0), 300_000_000);
    }

    #[test]
    fn indexing_round_trips() {
        let k = 7;
        let mut m = DistanceMatrix::zeros(k);
        let mut v = 1;
        for i in 0..k {
            for j in (i + 1)..k {
                m.set(i, j, v);
                v += 1;
            }
        }
        assert_eq!(m.as_slice().len(), 21);
        let mut v = 1;
        for i in 0..k {
            for j in (i + 1)..k {
                assert_eq!(m.get(i, j), v);
                assert_eq!(m.get(j, i), v);
                v += 1;
            }
        }
    }
}
