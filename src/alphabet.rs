//! Sequence alphabets and multiple-substitution corrections.

use std::fmt;
use std::str::FromStr;

/// The kind of residues in an alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Alphabet {
    /// Nucleotides. Only `A`, `C`, `G`, `T` count toward distances; `U` is
    /// converted to `T` on input.
    Dna,
    /// The twenty standard amino acids. Any other symbol (gap, `X`, `B`,
    /// `Z`, ...) is ignored when computing distances.
    Amino,
}

impl Alphabet {
    /// Symbols that make up the core alphabet, in the index order used by the
    /// distance tables (`AGCT` for DNA so that transitions are index pairs
    /// {0,1} and {2,3}).
    pub fn symbols(self) -> &'static [u8] {
        match self {
            Alphabet::Dna => b"AGCT",
            Alphabet::Amino => b"ARNDCQEGHILKMFPSTWYV",
        }
    }

    /// Lookup table from ASCII byte to core-alphabet index, or `None`.
    pub fn index_table(self) -> [Option<u8>; 256] {
        let mut t = [None; 256];
        for (i, &c) in self.symbols().iter().enumerate() {
            t[c as usize] = Some(i as u8);
        }
        t
    }
}

impl FromStr for Alphabet {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "a" | "amino" | "protein" => Ok(Alphabet::Amino),
            "d" | "dna" | "nucleotide" => Ok(Alphabet::Dna),
            _ => Err(format!("unknown alphabet '{}' (expected 'a' or 'd')", s)),
        }
    }
}

impl fmt::Display for Alphabet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Alphabet::Dna => "dna",
            Alphabet::Amino => "amino",
        })
    }
}

/// Correction applied to a raw pairwise dissimilarity to estimate
/// evolutionary distance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Correction {
    /// Raw proportion of differing sites (DNA) or mean BLOSUM45 dissimilarity
    /// (protein), capped at 1.
    None,
    /// Jukes-Cantor: `d = -3/4 ln(1 - 4/3 p)`. DNA only.
    JukesCantor,
    /// Kimura two-parameter: `d = -1/2 ln(1 - 2P - Q) - 1/4 ln(1 - 2Q)`
    /// with `P` the transition and `Q` the transversion proportion. DNA only.
    Kimura2,
    /// FastTree's scoredist-like correction for proteins:
    /// `d = -1.3 ln(1 - s)` for `s < 0.91`, else the cap of 3.
    ScoreDist,
}

impl Correction {
    /// The correction NINJA uses when none is requested: Kimura for DNA,
    /// scoredist for protein.
    pub fn default_for(alphabet: Alphabet) -> Self {
        match alphabet {
            Alphabet::Dna => Correction::Kimura2,
            Alphabet::Amino => Correction::ScoreDist,
        }
    }

    /// Whether this correction can be applied to the given alphabet.
    pub fn applies_to(self, alphabet: Alphabet) -> bool {
        matches!(
            (self, alphabet),
            (Correction::None, _)
                | (Correction::JukesCantor | Correction::Kimura2, Alphabet::Dna)
                | (Correction::ScoreDist, Alphabet::Amino)
        )
    }

    /// Largest distance that can be reported: 1 without correction,
    /// otherwise 3.
    pub fn max_distance(self) -> f32 {
        match self {
            Correction::None => 1.0,
            _ => 3.0,
        }
    }
}

impl FromStr for Correction {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "n" | "none" => Ok(Correction::None),
            "j" | "jc" | "jukes-cantor" => Ok(Correction::JukesCantor),
            "k" | "k2p" | "kimura" => Ok(Correction::Kimura2),
            "s" | "scoredist" => Ok(Correction::ScoreDist),
            _ => Err(format!("unknown correction '{}' (expected 'n', 'j', 'k', or 's')", s)),
        }
    }
}

impl fmt::Display for Correction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Correction::None => "none",
            Correction::JukesCantor => "jukes-cantor",
            Correction::Kimura2 => "kimura2",
            Correction::ScoreDist => "scoredist",
        })
    }
}
