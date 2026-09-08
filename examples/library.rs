//! Use ninja as a library: read an alignment, compute distances, build a
//! tree, and print a few facts about it.
//!
//! Run with `cargo run --release --example library -- tests/fixtures/dna_200.fa`.

use std::env;

use ninja::distance::{DistanceCalculator, DistanceMatrix};
use ninja::io::fasta;
use ninja::{nj, NjParams};

fn main() {
    let path = env::args().nth(1).expect("usage: library <alignment.fa>");
    let aln = fasta::read_fasta(&path, None).expect("read alignment");
    println!("{} sequences, {} columns, {} alphabet", aln.len(), aln.width(), aln.alphabet);

    let calc = DistanceCalculator::new(&aln, None).expect("distance calculator");
    println!("correction: {}", calc.correction());
    println!("d(0, 1) = {:.6}", calc.calc(0, 1));

    let matrix = DistanceMatrix::from_calculator(&calc);
    let params = NjParams { verbose: 0, ..NjParams::default() };
    let (tree, stats) = nj::inmem::build(&aln.names, matrix, &params).expect("neighbor joining");

    let longest = tree
        .nodes()
        .iter()
        .filter_map(|n| n.branch_length().map(|l| (l, n.name.clone())))
        .fold((0.0f32, String::new()), |best, cur| if cur.0 > best.0 { cur } else { best });
    println!("{} candidates examined, {} rebuilds", stats.candidates_added, stats.rebuilds);
    println!("longest branch: {:.5} (leaf '{}')", longest.0, longest.1);
    print!("{}", tree.to_newick());
}
