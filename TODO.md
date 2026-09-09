# To do

Planned work, roughly in order.

- **Library use.** Make the crate a first-class library, not just the
  binary's internals: settle the public API (`DistanceCalculator`,
  `DistanceMatrix`, the two `build` functions, `Tree`, `run`), keep it
  stable across releases, document every public item with examples,
  and decide whether to offer C and Python bindings. The current
  `pub` surface exists but has not been reviewed with outside callers in
  mind.
- Make `--collapse_identical` the default once the feature set is
  complete (it is off so that output matches the Java tool exactly).
- The minimum Rust version (1.85) is set by clap 4.6; CI builds with it.
- Performance pass once CI is green and outputs match the Java tool: profile
  the search (the sequential part), the rebuild that pushes every pair onto
  the heaps, and the cache behaviour of the triangular matrix updates.
- Reduce in-memory engine memory: heap entries for every pair are created
  at each rebuild (34 GB for 50,000 taxa).
