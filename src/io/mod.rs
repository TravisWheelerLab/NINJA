//! Readers and writers for alignments, distance matrices, and trees.

pub mod fasta;
pub mod names;
pub mod phylip;

pub use fasta::{read_fasta, read_fasta_from, Alignment};
pub use names::{resolve_duplicates, DuplicateNames, Renamed};
pub use phylip::{read_phylip, read_phylip_from, write_phylip, PhylipMatrix};
