# Testing

`cargo test` runs three layers.

## Unit tests

Inside `src/`, next to the code they cover:

* The binary heap is checked against a literal transcription of the
  original's array-based heap on long sequences of tied keys, because tie
  order decides which pair neighbor joining picks.
* The external-memory heap is checked against a shadow priority queue over
  hundreds of thousands of operations with a memory budget small enough to
  force spills and run merges at several levels.
* Both engines are given random additive trees (path distances of a random
  tree with random branch lengths) and must reproduce every split exactly,
  with rebuilds forced and, for the external-memory engine, with the matrix
  paged to disk.
* Parsers, formatters and distance formulas have small focused tests.

## Integration tests against the Java reference

`tests/cli.rs` runs the built binary on the alignments in `tests/fixtures`
and compares with outputs of the original Java NINJA (version 1.2.2) stored
in `tests/reference`:

| Fixture | Content | Purpose |
|---|---|---|
| `PF08271_seed.fa` | 58 protein sequences (Pfam seed; from the C port) | real data |
| `protein_120.fa` | 120 simulated protein sequences, 300 columns | protein path |
| `dna_200.fa` | 200 simulated DNA sequences, 600 columns | DNA path |
| `dna_700.fa` | 700 simulated DNA sequences, 200 columns | large enough to page the external-memory matrix to disk |
| `dna_200_dups.fa` | `dna_200.fa` plus seven exact copies of four of its sequences | identical-sequence collapse |

For each fixture the tests require the tree from the in-memory engine in
`--reference_order` mode, the tree from the external-memory engine, and the
written distance matrix to be identical to Java's (whitespace aside), and
the default in-memory tree to have the same splits and branch lengths. Trees built from a Phylip matrix are
compared the same way. The fixture with 700 taxa is also run with a memory
budget of 100 KB, which makes the resident window one block wide so the
matrix is flushed to disk repeatedly.

Exact agreement is possible because the in-memory search is integer
arithmetic and the external-memory search repeats the reference's
single-precision operations in the same order. It is not a promise: a tie
in `Q` broken differently, or a one-bit difference in a logarithm, would
change a tree without making it wrong. The helper `assert_trees_close` in
`tests/common/mod.rs` compares trees by their splits with a tolerance on
branch length for that situation, and is what the engine-versus-engine test
uses.

Files named `*.cpp.*` in `tests/reference` come from the C++ `cluster`
branch (built from `origin/cluster` of the C++ repository): onegap
distance matrices, which must match exactly, and cluster tables, whose
clusters must each be contained in one of ours (see
`docs/reference-differences.md` for why they are not identical).

The Java outputs were produced with `scripts/make_reference.sh`, which needs
`Ninja.jar` from <https://wheelerlab.org/software/ninja/>. The 6,000-taxon
runs mentioned in the README were done the same way but are not stored.

## Simulated data

`scripts/simulate_alignment.py` evolves sequences down a random tree and
writes the true tree alongside. `recovers_simulated_tree` checks that the
inferred tree matches the generating tree on all but short branches. That
checks the method rather than the port, so its thresholds are loose.
