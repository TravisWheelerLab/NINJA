//! Pairwise evolutionary distances from an alignment.
//!
//! The calculator packs each sequence once so that a pair distance is a tight
//! loop over machine words:
//!
//! * DNA is stored two bits per site (`A=00, G=01, C=10, T=11`) plus a
//!   one-bit-per-site validity mask, 32 sites to a `u64`. XOR of two packed
//!   sequences yields `01` exactly for transitions (A<->G, C<->T) and a set
//!   high bit for transversions, so transitions, transversions, and the
//!   number of comparable sites are three population counts. This replaces
//!   the hand-written SSE shuffle kernel of the C port with code that
//!   vectorises on any target.
//! * Protein is stored one byte per site as an index into a 21x21 BLOSUM45
//!   dissimilarity table whose row and column 20 (any non-standard residue)
//!   are zero, so the inner loop is a branch-free gather and accumulate.
//!
//! The arithmetic (single-precision accumulation, the order of promotions in
//! the correction formulas) follows the reference implementation so that
//! distances agree with it to the printed precision.

mod bl45;
mod dna;
mod matrix;
mod protein;

pub use matrix::DistanceMatrix;

use crate::alphabet::{Alphabet, Correction};
use crate::error::{Error, Result};
use crate::io::fasta::Alignment;

/// Computes corrected distances between any two sequences of an alignment.
#[derive(Debug, Clone)]
pub struct DistanceCalculator {
    alphabet: Alphabet,
    correction: Correction,
    packed: Packed,
    n: usize,
}

#[derive(Debug, Clone)]
enum Packed {
    Dna(dna::PackedDna),
    Amino(protein::PackedProtein),
}

impl DistanceCalculator {
    /// Pack an alignment for distance computation.
    ///
    /// `correction` defaults to Kimura two-parameter for DNA and scoredist
    /// for protein. A correction that does not apply to the alphabet is an
    /// error.
    pub fn new(aln: &Alignment, correction: Option<Correction>) -> Result<Self> {
        let alphabet = aln.alphabet;
        let correction = correction.unwrap_or_else(|| Correction::default_for(alphabet));
        if !correction.applies_to(alphabet) {
            return Err(Error::options(format!(
                "correction '{}' cannot be used with the {} alphabet",
                correction, alphabet
            )));
        }
        let packed = match alphabet {
            Alphabet::Dna => Packed::Dna(dna::PackedDna::new(&aln.seqs)),
            Alphabet::Amino => Packed::Amino(protein::PackedProtein::new(&aln.seqs)),
        };
        Ok(DistanceCalculator { alphabet, correction, packed, n: aln.len() })
    }

    /// Number of sequences.
    pub fn len(&self) -> usize {
        self.n
    }

    /// True when there are no sequences.
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// The alphabet the calculator was built for.
    pub fn alphabet(&self) -> Alphabet {
        self.alphabet
    }

    /// The correction in use.
    pub fn correction(&self) -> Correction {
        self.correction
    }

    /// Corrected distance between sequences `a` and `b`.
    ///
    /// Pairs with no comparable sites get the correction's maximum distance
    /// (1 with no correction, 3 otherwise), as do pairs whose correction
    /// formula is undefined (saturated divergence).
    #[inline]
    pub fn calc(&self, a: usize, b: usize) -> f64 {
        match &self.packed {
            Packed::Dna(p) => {
                let (transitions, transversions, sites) = p.count(a, b);
                dna::correct(transitions, transversions, sites, self.correction) as f64
            }
            Packed::Amino(p) => {
                let (sum, sites) = p.score(a, b);
                protein::correct(sum, sites, self.correction) as f64
            }
        }
    }
}
