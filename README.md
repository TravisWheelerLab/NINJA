# NINJA (Rust)

A Numerically Impressive Neighbor-Joining Algorithm, NINJA computes an exact
neighbor-joining tree given an alignment or a distance matrix while examining
only a small fraction of the taxon pairs at each step, so that inputs of
hundreds of thousands of sequences are feasible. The core method is 
described in

> Wheeler, T.J. 2009. Large-scale neighbor-joining with NINJA. In S.L.
> Salzberg and T. Warnow (Eds.), *Proceedings of the 9th Workshop on
> Algorithms in Bioinformatics*, WABI 2009, pp. 375-389. Springer, Berlin.

This code base is a port of the original Java implementation
(<https://wheelerlab.org/software/ninja/>) to Rust, with additional 
improvements to sequence distance computation. 

## Install

From crates.io, with a Rust toolchain (1.85 or later):

    cargo install ninja-phylo

which installs a binary named `ninja`. Or build from a checkout with
`cargo build --release`; the binary is `target/release/ninja`.

## Use

Build a tree from a FASTA alignment:

    ninja alignment.fa > tree.nwk
    ninja --in alignment.fa --out tree.nwk

Write a distance matrix instead, or start from one:

    ninja --out_type d alignment.fa > distances.phylip
    ninja --in_type d distances.phylip > tree.nwk

Cluster sequences instead of building a tree (single linkage: two
sequences share a cluster when a chain of pairs within the cutoff connects
them):

    ninja --out_type c --cluster_cutoff 0.03 alignment.fa > clusters.tsv

The output has one `cluster_id<TAB>name` line per sequence, clusters
numbered from 0 in order of their first member. Clustering runs in
parallel and stores no matrix, so its memory use is linear in the number
of sequences.

`ninja` detects the alphabet (DNA when every residue is `A C G T U`) and
uses the Kimura two-parameter correction for DNA and FastTree's
scoredist-like correction for protein. `--alph_type` and `--corr_type`
override these. `--corr_type m` selects Mothur's "onegap" distance,
`(mismatches + gap openings) / (compared columns + gap openings)`, for
either alphabet; a run of columns gapped in only one sequence counts as one
opening, terminal gaps included. `ninja --help` lists every option.

`--collapse_identical` builds the tree over one representative of each set
of identical sequences and attaches the rest as zero-length branches. It
saves work when an alignment has many duplicates. It is off for now so
that output matches the Java tool exactly, and is planned to become the
default.

### Engines and memory

Two engines implement the same search. The in-memory engine keeps the
distance matrix as fixed-point integers (`2n^2` bytes for `n` taxa) and is
the faster one. The external-memory engine keeps a window of the matrix in
memory and pages the rest to disk, and replaces each of its priority queues
with a disk-backed one, so it is bounded by disk rather than memory.

By default `ninja` picks the in-memory engine when the matrix fits in
`--memory` (three quarters of physical memory unless given, in gigabytes)
and the external-memory engine otherwise. Force one with `-m inmem` or
`-m extmem`. The external-memory engine writes its scratch files under
`--tmp_dir` (the system temporary directory by default); on a cluster,
point it at a local disk.

### Performance

Simulated alignments of 300 columns on a 192-core machine. Wall time and
peak memory; Java ran with an 8 GB heap (16 GB for 20,000 taxa). Every
in-memory tree below has the same splits and branch lengths as the Java
tool's, and with `--reference_order` (31.7 s for 20,000 taxa) the Newick
text is identical too; the external-memory engine keeps its sums in
double precision and agrees with Java's to within rounding.

| Taxa | ninja, in-memory, all cores | ninja, in-memory, one core | ninja, external-memory, 50 MB budget | Java NINJA 1.2.2 |
|---:|---:|---:|---:|---:|
| 6,000 DNA | 1.4 s, 0.25 GB | 3.6 s | 5.4 s, 0.17 GB | 75 s |
| 6,000 protein | 1.3 s, 0.22 GB | 7.6 s | | 19 s, 2.4 GB |
| 20,000 DNA | 14.6 s, 4.1 GB | 40 s | 68 s, 0.30 GB | 680 s, 15 GB |
| 50,000 DNA | 92 s, 26 GB | | | |
| 100,000 DNA | 417 s, 104 GB | | 1,737 s, 2.3 GB (4 GB budget) | |

With a 2 GB budget the external-memory engine takes 54 s and 0.64 GB for
20,000 taxa. Distance computation and rebuilds are the parallel parts;
the join loop is sequential and dominates for large inputs. Memory in the
in-memory engine is mostly the queue entries, one per pair; the
external-memory engine trades that for disk.

## Library

The crate exposes the pieces separately: `io::fasta` and `io::phylip`
readers and writers, `distance::DistanceCalculator` for pairwise distances,
`nj::inmem::build` and `nj::extmem::build` for the search, and `tree::Tree`
for Newick output. `ninja::run` does what the binary does. See
`examples/library.rs` and the API documentation (`cargo doc --open`).

## Layout

    src/
      alphabet.rs      alphabets and corrections
      distance/        packed sequences and pair distances
      io/              FASTA, Phylip, and Newick
      heap/            binary heap; external-memory array heap
      nj/inmem.rs      in-memory engine
      nj/extmem/       external-memory engine, disk matrix, candidate heaps
      tree.rs          tree arena and Newick output
      pipeline.rs      the end-to-end driver behind `ninja::run`
      bin/ninja.rs     command-line front end
    tests/             integration tests, fixtures, Java reference outputs
    docs/              algorithm notes, testing, differences from Java/C
    scripts/           alignment simulator, reference and benchmark scripts

## Testing

`cargo test` runs unit tests, tests that both engines recover random
additive trees exactly, and integration tests that compare the binary's
output with stored outputs of the Java implementation. `docs/testing.md`
has the details.

## License

BSD 3-clause; see `LICENSE`.
