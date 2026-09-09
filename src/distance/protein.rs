//! Byte-indexed protein sequences and the scoredist correction.

use super::bl45::BL45;
use super::gaps::{self, RunState};
use crate::alphabet::{Alphabet, Correction};

/// Index used for gaps and non-standard residues.
const OTHER: u8 = 20;

/// Alignment rows as residue indices, plus the padded dissimilarity table.
#[derive(Debug, Clone)]
pub struct PackedProtein {
    width: usize,
    /// `idx[i * width ..][..width]`: 0..19 for standard residues, 20 otherwise.
    idx: Vec<u8>,
    /// 21x21 table, row-major; row and column 20 are zero.
    table: Vec<f32>,
    /// One bit per site, set for standard residues.
    words1: usize,
    valid1: Vec<u64>,
}

impl PackedProtein {
    /// Index sequences and build the padded table.
    pub fn new(seqs: &[Vec<u8>]) -> Self {
        let width = seqs.first().map_or(0, |s| s.len());
        let lookup = Alphabet::Amino.index_table();
        let words1 = width.div_ceil(64).max(1);
        let mut idx = Vec::with_capacity(seqs.len() * width);
        let mut valid1 = vec![0u64; seqs.len() * words1];
        for (i, s) in seqs.iter().enumerate() {
            for (pos, &c) in s.iter().enumerate() {
                let code = lookup[c as usize].unwrap_or(OTHER);
                idx.push(code);
                if code != OTHER {
                    valid1[i * words1 + pos / 64] |= 1u64 << (pos % 64);
                }
            }
        }
        let mut table = vec![0f32; 21 * 21];
        for a in 0..20 {
            for b in 0..20 {
                table[a * 21 + b] = BL45[a][b];
            }
        }
        PackedProtein { width, idx, table, words1, valid1 }
    }

    /// `(mismatches, comparable sites, gap openings)` for the onegap
    /// distance.
    #[inline]
    pub fn count_onegap(&self, a: usize, b: usize) -> (u32, u32, u32) {
        let w = self.width;
        let sa = &self.idx[a * w..(a + 1) * w];
        let sb = &self.idx[b * w..(b + 1) * w];
        let mut mismatches = 0u32;
        let mut sites = 0u32;
        for (&x, &y) in sa.iter().zip(sb) {
            let both = (x != OTHER) & (y != OTHER);
            sites += both as u32;
            mismatches += (both & (x != y)) as u32;
        }
        let w1 = self.words1;
        let va = &self.valid1[a * w1..(a + 1) * w1];
        let vb = &self.valid1[b * w1..(b + 1) * w1];
        let mut state = RunState::default();
        let mut openings = 0;
        for k in 0..w1 {
            let real = gaps::real_mask(k, self.width);
            openings += gaps::openings_word(va[k], vb[k], !va[k] & !vb[k] & real, &mut state);
        }
        (mismatches, sites, openings)
    }

    /// `(summed dissimilarity, comparable sites)` for a pair.
    ///
    /// The sum is accumulated in single precision in site order, as the
    /// reference did; adding the zero entries for skipped sites leaves the
    /// sum unchanged, so the loop needs no branch.
    #[inline]
    pub fn score(&self, a: usize, b: usize) -> (f32, u32) {
        let w = self.width;
        let sa = &self.idx[a * w..(a + 1) * w];
        let sb = &self.idx[b * w..(b + 1) * w];
        let mut sum = 0f32;
        let mut sites = 0u32;
        for (&x, &y) in sa.iter().zip(sb) {
            sum += self.table[x as usize * 21 + y as usize];
            sites += ((x != OTHER) & (y != OTHER)) as u32;
        }
        (sum, sites)
    }
}

/// Apply the protein correction to a summed dissimilarity.
#[inline]
pub fn correct(sum: f32, sites: u32, corr: Correction) -> f32 {
    let maxscore = corr.max_distance();
    if sites == 0 {
        return maxscore;
    }
    let mut dist = sum / sites as f32;
    if corr == Correction::ScoreDist {
        dist = if dist < 0.91 { (-1.3 * (1.0 - dist as f64).ln()) as f32 } else { maxscore };
    }
    if dist < maxscore {
        dist
    } else {
        maxscore
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_sequences_have_zero_distance() {
        let p = PackedProtein::new(&[b"ACDEFGHIK".to_vec(), b"ACDEFGHIK".to_vec()]);
        assert_eq!(p.score(0, 1), (0.0, 9));
        assert_eq!(correct(0.0, 9, Correction::ScoreDist), 0.0);
    }

    #[test]
    fn skips_gaps_and_unknowns() {
        let p = PackedProtein::new(&[b"A-XR".to_vec(), b"AR-N".to_vec()]);
        let (sum, sites) = p.score(0, 1);
        assert_eq!(sites, 2);
        assert!((sum - BL45[1][2]).abs() < 1e-7); // R vs N
    }

    #[test]
    fn saturation_gives_cap() {
        assert_eq!(correct(0.0, 0, Correction::ScoreDist), 3.0);
        assert_eq!(correct(0.95 * 4.0, 4, Correction::ScoreDist), 3.0);
        assert_eq!(correct(0.95 * 4.0, 4, Correction::None), 0.95);
    }
}
