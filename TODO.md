# To do

Planned work, roughly in order.

- **Library use.** Make the crate a first-class library, not just the
  binary's internals: settle the public API (`DistanceCalculator`,
  `DistanceMatrix`, the two `build` functions, `Tree`, `run`), keep it
  stable across releases, document every public item with examples,
  and decide whether to offer C and Python bindings. The current
  `pub` surface exists but has not been reviewed with outside callers in
  mind.
- **From the C++ `cluster` branch:**
  - Mothur "onegap" distance, `--corr_type m`, for DNA and protein.
  - Single-linkage clustering, `--out_type c` and `--cluster_cutoff`,
    computed as connected components with union-find rather than by
    repeated merging on a full matrix.
  - Collapse identical sequences before neighbor joining and re-expand
    them as zero-length chains, behind `--collapse_identical` for now; make
    it the default once the feature set is complete.
- Verify the declared minimum Rust version (1.75) with that toolchain in CI.
- Reduce in-memory engine memory: heap entries for every pair are created
  at each rebuild (34 GB for 50,000 taxa).
