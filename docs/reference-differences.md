# Differences from the Java and C implementations

This port was written from the Java source of NINJA 1.2.2. The earlier
C++ port (github.com/TravisWheelerLab/ninja-old) was a pilot and is not
maintained; its `cluster` branch supplied the features in the last section.
The search algorithm, its constants, the fixed-point and single-precision
arithmetic, and the tie behaviour of the heaps are reproduced so that
outputs match the Java tool. What follows is everything that is knowingly
different.

## Behaviour changes

* **Alphabet detection is case-insensitive.** The Java reader compared
  residues to `ACGTU` before upper-casing, so lower-case DNA was treated as
  protein. Residues are now upper-cased first.
* **Phylip parsing.** The Java reader assumed exactly six decimals per value
  (it stripped the point and appended two zeros). Values are now parsed as
  decimals with any number of digits and rounded to the eighth decimal.
* **Distance-file input to the external-memory engine** is supported (the C
  port refused it).
* **Candidate list spill to disk.** The Java code wrote candidates beyond
  eight million to a file that was only read back by a code path that is
  never enabled, so those candidates were silently lost. The list is now a
  growable vector; the two-million-entry cap that freezes the list into a
  candidate heap makes the spill unreachable in practice anyway.
* **Resident window bookkeeping.** The Java external-memory builder counted
  new columns from the first resident column rather than from the first new
  column. When a disk-backed run kept part of the input matrix resident, the
  window could overflow. This port tracks the next column directly.
* **Memory budget.** Java used the JVM heap limit to decide when to fall
  back to the external-memory engine. This port uses `--memory`, defaulting
  to three quarters of physical memory, and chooses the engine up front
  from the taxon count.
* **Threads.** Distance computation and the rebuilds of the in-memory
  engine run on all cores by default (`--threads` limits it). The join loop
  is single-threaded, as in the original.
* **Tie order and rebuild schedule.** By default the in-memory engine keeps
  each cluster pair's entries in sorted runs rather than heaps and rebuilds
  every quarter of the remaining taxa. Both change only which of two pairs
  with exactly equal `Q` is joined first. `--reference_order` restores the
  heaps and the paper's schedule and reproduces the Java output exactly.
* **Output.** The distance matrix header is the taxon count alone (Java
  wrote a leading tab); the Newick string is followed by one newline (Java
  wrote two).
* **Removed options** that were unimplemented or disabled in the original:
  `--clust_size` and `--rebuild_step_ratio` are kept; `--chop`,
  `--complex_cand_ratio`, `--cand_heap_threshold`, `--cand_heap_decay`,
  `--variable_rebuild_steps`, `--dist_in_mem` and `--disk_pages` are not
  exposed. The library's `NjParams` covers the rebuild schedule.

## The C++ `cluster` branch

The branch adds single-linkage clustering (`--out_type c`), the onegap
distance (`--corr_type m`), and an unfinished collapse of identical
sequences. All three are here, with these differences:

* **Clustering is computed as connected components** rather than by
  repeated merging on a full matrix. The result is the same partition that
  correct single linkage gives. The branch's merge loop, when it folds
  cluster `c2` into `c1`, does not update rows strictly between `c1` and
  `c2`, so it loses some distances and over-splits: on the 200-sequence DNA
  fixture at cutoff 0.1 it reports 95 clusters where single linkage gives
  87. Each of its clusters lies within one of ours on every fixture and
  cutoff tested, and the tests check that.
  Within a cluster, members are listed in input order (the branch listed
  them in merge order).
* **Onegap works for both alphabets.** On the branch it was wired into the
  SSE DNA kernel only: protein input silently got the scoredist distance,
  and DNA with `--NOSSE` silently got zero for every pair.
* **Collapsing identical sequences** was dead code on the branch (tree
  output was disabled there). Here it is `--collapse_identical`, off by
  default.
* The branch's `--print-times` flag and `-v` for version are not carried
  over; `--verbose` reports timings.
