//! Two-bit packed DNA and the DNA corrections.

use super::gaps::{self, RunState};
use crate::alphabet::Correction;

const SITES_PER_WORD: usize = 32;
/// Every other bit set: selects the low bit of each two-bit site.
const LOW_BITS: u64 = 0x5555_5555_5555_5555;

/// Alignment rows packed two bits per site with a per-site validity mask.
#[derive(Debug, Clone)]
pub struct PackedDna {
    words: usize,
    len: usize,
    /// `seq[i * words ..][..words]`: site codes.
    seq: Vec<u64>,
    /// `valid[i * words ..][..words]`: `01` at each site that is A/C/G/T.
    valid: Vec<u64>,
    /// One bit per site (64 per word), set where the site is A/C/G/T; used
    /// for gap-run counting.
    words1: usize,
    valid1: Vec<u64>,
}

impl PackedDna {
    /// Pack sequences. Any byte other than `A C G T` is treated as a gap.
    pub fn new(seqs: &[Vec<u8>]) -> Self {
        let n = seqs.len();
        let width = seqs.first().map_or(0, |s| s.len());
        let words = width.div_ceil(SITES_PER_WORD).max(1);
        let words1 = width.div_ceil(64).max(1);
        let mut seq = vec![0u64; n * words];
        let mut valid = vec![0u64; n * words];
        let mut valid1 = vec![0u64; n * words1];
        for (i, s) in seqs.iter().enumerate() {
            let sw = &mut seq[i * words..(i + 1) * words];
            let vw = &mut valid[i * words..(i + 1) * words];
            let v1 = &mut valid1[i * words1..(i + 1) * words1];
            for (pos, &c) in s.iter().enumerate() {
                // A=00 G=01 C=10 T=11: transitions differ only in the low bit.
                let (code, ok) = match c {
                    b'A' => (0u64, true),
                    b'G' => (1, true),
                    b'C' => (2, true),
                    b'T' => (3, true),
                    _ => (0, false),
                };
                let w = pos / SITES_PER_WORD;
                let shift = 2 * (pos % SITES_PER_WORD);
                sw[w] |= code << shift;
                if ok {
                    vw[w] |= 1u64 << shift;
                    v1[pos / 64] |= 1u64 << (pos % 64);
                }
            }
        }
        PackedDna { words, len: width, seq, valid, words1, valid1 }
    }

    /// Number of gap openings between a pair, for the onegap distance.
    #[inline]
    pub fn openings(&self, a: usize, b: usize) -> u32 {
        let w = self.words1;
        let va = &self.valid1[a * w..(a + 1) * w];
        let vb = &self.valid1[b * w..(b + 1) * w];
        let mut state = RunState::default();
        let mut total = 0;
        for k in 0..w {
            let real = gaps::real_mask(k, self.len);
            total += gaps::openings_word(va[k], vb[k], !va[k] & !vb[k] & real, &mut state);
        }
        total
    }

    /// `(transitions, transversions, comparable_sites)` for a pair.
    #[inline]
    pub fn count(&self, a: usize, b: usize) -> (u32, u32, u32) {
        let w = self.words;
        let sa = &self.seq[a * w..(a + 1) * w];
        let sb = &self.seq[b * w..(b + 1) * w];
        let va = &self.valid[a * w..(a + 1) * w];
        let vb = &self.valid[b * w..(b + 1) * w];
        let mut transitions = 0u32;
        let mut transversions = 0u32;
        let mut sites = 0u32;
        for k in 0..w {
            let x = sa[k] ^ sb[k];
            let v = va[k] & vb[k];
            let lo = x & LOW_BITS;
            let hi = (x >> 1) & LOW_BITS;
            sites += v.count_ones();
            transversions += (hi & v).count_ones();
            transitions += (lo & !hi & v).count_ones();
        }
        (transitions, transversions, sites)
    }
}

/// Apply a DNA correction to raw counts.
///
/// Mirrors the reference arithmetic: proportions are computed in single
/// precision, the logarithms in double precision, and the result is rounded
/// back to single precision and capped.
#[inline]
pub fn correct(transitions: u32, transversions: u32, sites: u32, corr: Correction) -> f32 {
    let maxscore = corr.max_distance();
    if sites == 0 {
        return maxscore;
    }
    let p = transitions as f32 / sites as f32;
    let q = transversions as f32 / sites as f32;
    let dist: f32 = if p + q == 0.0 {
        0.0
    } else {
        match corr {
            Correction::JukesCantor => (-0.75 * (1.0 - (4.0 / 3.0) * (p + q) as f64).ln()) as f32,
            Correction::Kimura2 => {
                let a = (1.0 - (2.0 * p) as f64) - q as f64;
                let b = 1.0 - (2.0 * q) as f64;
                (-0.5 * a.ln() - 0.25 * b.ln()) as f32
            }
            Correction::None => p + q,
            Correction::ScoreDist | Correction::OneGap => {
                unreachable!("{} is not handled by the DNA count correction", corr)
            }
        }
    };
    // NaN and +inf (saturated pairs) fail the comparison and get the cap.
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
    fn counts_transitions_and_transversions() {
        let p = PackedDna::new(&[b"ACGTACGT-A".to_vec(), b"GTCAACGTA-".to_vec()]);
        // A/G ti, C/T ti, G/C tv, T/A tv, then 4 identical, then two gapped.
        assert_eq!(p.count(0, 1), (2, 2, 8));
        assert_eq!(p.count(0, 0), (0, 0, 9));
    }

    #[test]
    fn crosses_word_boundaries() {
        let a: Vec<u8> = (0..100).map(|i| b"ACGT"[i % 4]).collect();
        let mut b = a.clone();
        b[0] = b'G'; // ti
        b[33] = b'A'; // pos 33 is G -> A: ti
        b[70] = b'A'; // pos 70 is C -> A: tv
        b[99] = b'-';
        let p = PackedDna::new(&[a, b]);
        assert_eq!(p.count(0, 1), (2, 1, 99));
    }

    #[test]
    fn corrections() {
        assert_eq!(correct(0, 0, 10, Correction::Kimura2), 0.0);
        assert_eq!(correct(0, 0, 0, Correction::Kimura2), 3.0);
        assert_eq!(correct(0, 0, 0, Correction::None), 1.0);
        let none = correct(1, 2, 10, Correction::None);
        assert!((none - 0.3).abs() < 1e-6);
        let jc = correct(1, 2, 10, Correction::JukesCantor);
        assert!((jc - 0.38311).abs() < 1e-4);
        let k2 = correct(1, 2, 10, Correction::Kimura2);
        assert!((k2 - 0.3831).abs() < 1e-3);
        // Saturated: 1 - 2q < 0 gives NaN, capped.
        assert_eq!(correct(0, 8, 10, Correction::Kimura2), 3.0);
    }
}
