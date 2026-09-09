# Differences from the Java and C implementations

This port was written from the Java source of NINJA 1.2.2, with the C++
port (github.com/TravisWheelerLab/NINJA) used as a second reference. The
search algorithm, its constants, the fixed-point and single-precision
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
* **Threads.** Distance computation runs on all cores by default
  (`--threads` limits it). Everything after that is single-threaded, as in
  the original.
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

## Bugs in the C++ port that are not reproduced

That repository's README says it has a couple of small bugs. These are the
ones found by reading the C++ against the Java source.

* Row sums were 32-bit integers (64-bit in Java). At the fixed-point scale
  used, a few hundred taxa overflow them, fewer when sequences are divergent.
* Without `--alph_type`, every input was treated as DNA: the alphabet
  detection loop could never run because the default had already been
  set to DNA.
* The non-SIMD protein distance remapped residues in place and then indexed
  the BLOSUM table with the remapped bytes, producing zero and negative
  distances. The SIMD protein path returned the negated distance.
* The Kimura correction on the non-SIMD DNA path used a square root where a
  logarithm belongs.
* In the external-memory heap: the scratch file was read while closed in
  one merge routine; a `qsort` call had its element count and element size
  swapped, and its comparator sorted in the wrong direction; row-sum extremes
  used for clustering were stored in integers, collapsing every cluster
  boundary to zero when sums were below one.
* In the external-memory builder: distances paged from disk for the second
  joined node were read into one buffer and consumed from another, never
  written; candidates spilled to disk were converted to integers by value
  rather than by bit pattern; the spill file was opened with an invalid mode
  in a directory that was never created; a variable-length array of up to
  `2n` floats was placed on the stack.
* Heaps used `std::push_heap`/`pop_heap`, whose handling of equal keys
  differs from the original heap, so trees can differ from Java's when `Q`
  values tie.

## What the C++ port contributed

Its SSE kernels for DNA and protein distances motivated packing sequences
once and computing each pair from machine words. The DNA kernel here is a
different formulation of the same idea (two-bit codes chosen so that XOR
separates transitions from transversions, then population counts) that
needs no intrinsics and vectorises on any target. The clustered-alphabet
protein kernel of the C++ port approximated BLOSUM62 over residue classes
and was not equivalent to the Java distance; it was not carried over.
