# How NINJA builds a tree

This is a summary of the method described in Wheeler (2009), "Large-scale
neighbor-joining with NINJA", written from the point of view of this
implementation. Section headings name the modules that carry each part.

## Neighbor joining

Given `n` taxa and a distance matrix `D`, neighbor joining repeatedly joins
the pair `(i, j)` that minimises

    Q(i, j) = (n - 2) D[i][j] - R[i] - R[j]

where `R[i]` is the sum of row `i`. The joined pair becomes a new taxon `u`
with

    D[u][x] = (D[i][x] + D[j][x] - D[i][j]) / 2

and branch lengths `(D[i][j] + (R[i] - R[j]) / (n - 2)) / 2` for `i` and the
mirror image for `j`. Row sums are updated incrementally rather than
recomputed. A naive implementation scans all `n^2` pairs at each of the `n`
iterations.

## Pruning the search (`nj`)

NINJA keeps the exact NJ result but avoids looking at most pairs.

**Clusters.** Taxa are split into `cluster_count` (default 30) clusters by
their row sum, using evenly spaced thresholds between the smallest and
largest sum. For a pair drawn from clusters `a` and `b`,

    Q(i, j) >= (n - 2) D[i][j] - maxR[a] - maxR[b]

where `maxR` is the largest row sum in a cluster (the two largest when
`a == b`). Because the right-hand side depends on the pair only through
`D[i][j]`, each cluster pair keeps a min-heap of its distances, and the heap
can be scanned in order and abandoned as soon as the bound exceeds the best
`Q` found so far.

**Candidates.** Pairs pulled from a heap are not put back. They join a
candidate list that is rescanned in full each iteration, which is cheap
because it is small. Every `candidate_iters` (default 50) iterations, any
candidate whose bound could not beat the current best is returned to its
heap. Candidates that mention a taxon that has since been joined are dropped
lazily when encountered, and the list is compacted when a fifth of it is
dead.

**Rebuilds.** Row sums shrink as taxa are merged, so the cluster thresholds
and heaps go stale. After a fraction `rebuild_step_ratio` (default 0.5) of
the remaining taxa have been joined, the engine rebuilds clusters and heaps
from the current state. Between rebuilds, the new node is placed in a cluster by comparing
its row sum to each cluster's most recent maximum, and its distances are
pushed onto the appropriate heaps. Inputs under 500 taxa never rebuild.

**Ties.** When several pairs share the minimum `Q`, the pair chosen depends
on the order in which candidates and heaps are scanned and on the heap's
behaviour for equal keys. Which of the tied pairs is joined does not affect
correctness (all are valid NJ choices) but does change the output. This
implementation's heap (`heap::MinHeap`) uses the same sift rules as the
original, so the same tree comes out.

## The in-memory engine (`nj::inmem`)

Distances are stored as 32-bit fixed-point integers in units of `1e-8`,
rounded to a multiple of 100 so that a matrix computed from an alignment and
one read back from the six-decimal Phylip output are identical. Row sums and
`Q` values are 64-bit integers, so the search involves no floating point at
all and its result is bit-for-bit reproducible. Memory is `2n^2` bytes for
the matrix plus heap entries.

## The external-memory engine (`nj::extmem`)

For inputs whose matrix does not fit in memory, three things change.

**Matrix layout** (`nj::extmem::matrix`). The matrix has `n` rows and
`2n - 2` columns of `f32`: the input distances, then one column per internal
node holding its distances to everything alive when it was created. Only a
window of the most recent columns is resident; when it fills, it is appended
to the on-disk rows and the window advances. A node's column is also written
out as a full row at that point, so the distance between two old nodes can
always be read from the row of the newer one. Rows are indexed by matrix row
(an internal node reuses the row of its left child), columns by node index.

**Cluster-pair heaps** (`heap::ArrayHeap`) become external-memory priority
queues after Brengel, Crauser, Ferragina and Meyer (1999). Inserts go to an
in-memory heap; when it reaches twice its run size, the trailing half of its
array (leaves of the heap, so none of the smallest keys) is sorted and
written to disk as a run. Runs live in four levels of slots; when a level is
full, every run below the first level with a free slot is merged into one
run at that level, and half-empty runs at a level are merged with each other
first. The head block of every run sits in a second in-memory heap tagged
with its level and slot, so the global minimum is the smaller of two heap
tops. During merges, entries that mention a joined taxon are discarded, which
is how expired pairs leave the structure.

**Candidate heaps** (`nj::extmem::CandidateHeap`). When the candidate list
grows past a threshold (50,000 entries and 40 per remaining taxon, or two
million), it is frozen into a disk-backed heap keyed by `Q` as it stood at
that moment, `Q'`. At a later iteration with `k` taxa left,

    Q = (k - 2)/(k' - 2) * Q' + delta[i] + delta[j],
    delta[x] = (k - 2)/(k' - 2) * R'[x] - R[x]

so scanning the frozen heap in `Q'` order and stopping when
`(k - 2)/(k' - 2) Q' + (two smallest deltas)` exceeds the best `Q` is exact.
A frozen heap that has shrunk to 60% of its original size is dissolved back
into the candidate list; at most 100 exist at once.

Because this engine uses single-precision floats, its branch lengths can
differ from the in-memory engine's in the fourth decimal place and it may
resolve near-ties differently.

## Distances (`distance`)

DNA sequences are packed two bits per site with a one-bit validity mask, so
that XOR of two sequences gives `01` exactly for transitions and a set high
bit for transversions, and the three counts a pair needs are population
counts over 64-bit words. Any symbol other than `A C G T` (after `U` is
mapped to `T`) is a gap for this purpose. The corrections are Jukes-Cantor,
Kimura two-parameter (the default), or none; a pair whose correction is
undefined (saturated) gets the cap of 3 (1 with no correction).

Protein distances follow FastTree: the mean over comparable sites of a
BLOSUM45-derived dissimilarity, then the scoredist-like correction
`-1.3 ln(1 - d)` for `d < 0.91` and the cap of 3 otherwise. Sites where
either residue is not one of the twenty standard amino acids are skipped.

Distance computation is row-parallel across all cores.
