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
- More speed in the in-memory engine. Profile for 20,000 taxa (about
  15 s): heap pushes of each new node's distances 6 s (4 s of it the
  sift-up), the update loop 4 s, pulls 2 s, rebuilds 1 s. Radix-sorted
  batches in place of the post-rebuild heaps would cut the pushes; the
  update loop is bound by cache misses on the triangular matrix.
- Reduce in-memory engine memory: the queues hold an entry for every pair
  (about 12 bytes each).
