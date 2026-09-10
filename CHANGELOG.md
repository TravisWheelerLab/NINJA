# Changelog

## 0.1.0 (2026-09-09)

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
* External-memory engine: rebuilds write sorted runs to disk directly,
  spills use the standard sort, level merges use a k-way heap, and the
  disk matrix is filled in one pass. 20,000 taxa at a 50 MB budget: 114 s
  and 1.7 GB before, 68 s and 0.3 GB after. Row sums and the criterion are
  double precision, which brings this engine's trees into agreement with
  the exact in-memory engine on every split at 20,000 taxa.
* `--method auto` names the engine choice that the Java and C++ tools
  called `default`; `default` is still accepted.
* Library API alongside the `ninja` binary.
* Integration tests against stored Java outputs; see `docs/testing.md`.
