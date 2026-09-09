//! NINJA: nearly infinite neighbor joining.
//!
//! A Rust implementation of the NINJA algorithm for large-scale
//! neighbor-joining phylogeny inference (Wheeler, WABI 2009).
//!
//! The crate exposes a library API plus a `ninja` command-line binary.
//! The pipeline is:
//!
//! 1. Read an alignment ([`io::fasta`]) or a Phylip distance matrix
//!    ([`io::phylip`]).
//! 2. Compute pairwise distances ([`distance`]), producing a
//!    [`DistanceMatrix`] of fixed-point integers (in-memory engine) or a
//!    column-blocked float matrix, partly on disk (external-memory engine).
//! 3. Run neighbor joining ([`nj`]) with the NINJA search strategy:
//!    sequences are partitioned into clusters by their row sums, each
//!    cluster pair keeps a min-heap of distances, and a small candidate list
//!    is scanned each iteration so that only pairs that can possibly minimise
//!    the NJ criterion are examined.
//! 4. Emit the tree in Newick format ([`tree::Tree::to_newick`]).
//!
//! The quickest way in is [`run`], which mirrors the command-line tool and
//! writes the tree (or matrix) to any `Write`:
//!
//! ```no_run
//! use ninja::{Options, run};
//! let opts = Options { input: Some("aln.fa".into()), ..Options::default() };
//! let mut newick = Vec::new();
//! let summary = run(&opts, &mut newick).unwrap();
//! print!("{}", String::from_utf8(newick).unwrap());
//! eprintln!("{} taxa", summary.taxa);
//! ```
//!
//! The pieces can also be used separately:
//!
//! ```no_run
//! use ninja::{io::fasta, distance::{DistanceCalculator, DistanceMatrix}, nj, NjParams};
//! let aln = fasta::read_fasta("aln.fa", None).unwrap();
//! let calc = DistanceCalculator::new(&aln, None).unwrap();
//! let matrix = DistanceMatrix::from_calculator(&calc);
//! let (tree, stats) = nj::inmem::build(&aln.names, matrix, &NjParams::default()).unwrap();
//! print!("{}", tree.to_newick());
//! eprintln!("{} candidates examined", stats.candidates_added);
//! ```

// The one `unsafe` block is a cache prefetch hint in `distance::matrix`.
#![deny(unsafe_code)]
#![warn(missing_docs)]
// Index-parallel loops over several arrays are the natural shape of this
// algorithm; iterator rewrites would obscure the correspondence with the paper.
#![allow(clippy::needless_range_loop)]

pub mod alphabet;
pub mod cluster;
pub mod distance;
pub mod error;
pub mod heap;
pub mod io;
pub mod nj;
pub mod tree;

mod pipeline;

pub use alphabet::{Alphabet, Correction};
pub use distance::DistanceMatrix;
pub use error::{Error, Result};
pub use nj::{Method, NjParams};
pub use pipeline::{run, InputKind, Options, OutputKind, RunOutput};
pub use tree::Tree;

/// Crate version string, as reported by `ninja --version`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Citation printed in the startup banner.
pub const CITATION: &str = "Wheeler, T.J. 2009. Large-scale neighbor-joining with NINJA.\n\
In S.L. Salzberg and T. Warnow (Eds.), Proceedings of\n\
the 9th Workshop on Algorithms in Bioinformatics.\n\
WABI 2009, pp. 375-389. Springer, Berlin.";
