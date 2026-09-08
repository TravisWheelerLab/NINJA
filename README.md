# NINJA (Rust)

Large-scale neighbor-joining phylogeny inference. NINJA computes the exact
neighbor-joining tree for an alignment or a distance matrix while examining
only a small fraction of the taxon pairs at each step, so that inputs of
hundreds of thousands of sequences are feasible. The method is described in

> Wheeler, T.J. 2009. Large-scale neighbor-joining with NINJA. In S.L.
> Salzberg and T. Warnow (Eds.), *Proceedings of the 9th Workshop on
> Algorithms in Bioinformatics*, WABI 2009, pp. 375-389. Springer, Berlin.

This is a port of the original Java implementation
(<https://wheelerlab.org/software/ninja/>) to Rust, meant to replace both it
and the earlier C++ port (<https://github.com/TravisWheelerLab/NINJA>). On
all test inputs, up to 6,000 taxa, it produces the same trees as the Java
tool. It computes distances on all cores and builds to a single binary with
no runtime dependencies. `docs/reference-differences.md` lists what differs
from the Java tool and what was wrong in the C++ port.

## Install

With a Rust toolchain (1.75 or later):

    cargo install --path .

or build in place with `cargo build --release`; the binary is
`target/release/ninja`.

## Use

Build a tree from a FASTA alignment:

    ninja alignment.fa > tree.nwk
    ninja --in alignment.fa --out tree.nwk

Write a distance matrix instead, or start from one:

    ninja --out_type d alignment.fa > distances.phylip
    ninja --in_type d distances.phylip > tree.nwk

`ninja` detects the alphabet (DNA when every residue is `A C G T U`) and
uses the Kimura two-parameter correction for DNA and FastTree's
scoredist-like correction for protein. `--alph_type` and `--corr_type`
override these; `ninja --help` lists every option.

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
peak memory; Java ran with an 8 GB heap (16 GB for 20,000 taxa). Where the Java tool
was run, its tree was identical to ninja's.

| Taxa | ninja, in-memory, all cores | ninja, in-memory, one core | ninja, external-memory, 50 MB budget | Java NINJA 1.2.2 |
|---:|---:|---:|---:|---:|
| 6,000 DNA | 3.1 s, 0.5 GB | 4.4 s | 7.4 s, 0.14 GB | 75 s |
| 6,000 protein | 3.1 s, 0.5 GB | 8.4 s | | 19 s, 2.4 GB |
| 20,000 DNA | 37 s, 5.3 GB | 47 s | 114 s, 1.7 GB | 680 s, 15 GB |
| 50,000 DNA | 256 s, 34 GB | 329 s | | |

Distance computation is the parallel part; the neighbor-joining search is
sequential and dominates for large inputs. The in-memory engine's memory
is mostly heap entries, one per pair, created at each rebuild; the
external-memory engine trades that for disk. On the 20,000-taxon input the
C++ port took 38 s and 6.7 GB but produced a different tree from Java's.

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

MIT; see `LICENSE`.
