# Changelog

## 0.1.0 (unreleased)

First Rust release, ported from NINJA 1.2.2 (Java) with the C++ port as a
second reference.

* In-memory and external-memory neighbor-joining engines producing the same
  trees as the Java implementation on all test inputs.
* DNA and protein distances from FASTA alignments, with Jukes-Cantor,
  Kimura two-parameter, scoredist, or no correction; distance computation
  is parallel.
* Phylip distance matrices as input or output.
* From the C++ `cluster` branch: single-linkage clustering
  (`--out_type c`, `--cluster_cutoff`), Mothur's onegap distance
  (`--corr_type m`), and `--collapse_identical`, which is planned to
  become the default.
* In-memory engine about twice as fast as a direct port: sorted runs
  instead of heaps for rebuilt entries, parallel rebuilds, prefetching in
  the update loop. `--reference_order` reproduces the Java tie order.
* Library API alongside the `ninja` binary.
* Integration tests against stored Java outputs; see `docs/testing.md`.
