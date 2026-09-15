//! End-to-end driver shared by the command-line tool and library users.

use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use crate::alphabet::{Alphabet, Correction};
use crate::cluster;
use crate::distance::{DistanceCalculator, DistanceMatrix};
use crate::error::{Error, Result};
use crate::io::{fasta, phylip, resolve_duplicates, DuplicateNames, Renamed};
use crate::nj::extmem::DiskMatrix;
use crate::nj::{self, Method, NjParams, NjStats};
use crate::tree::Tree;

/// What the input file contains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputKind {
    /// A FASTA multiple sequence alignment.
    #[default]
    Alignment,
    /// A Phylip distance matrix.
    Distances,
}

/// What to produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputKind {
    /// A Newick tree.
    #[default]
    Tree,
    /// A Phylip distance matrix (alignment input only).
    Distances,
    /// A single-linkage clustering table (`cluster_id<TAB>name`).
    Clusters,
}

/// Everything needed for one run. Mirrors the command-line flags.
#[derive(Debug, Clone)]
pub struct Options {
    /// Input path; `None` reads standard input.
    pub input: Option<PathBuf>,
    /// Whether the input is an alignment or a distance matrix.
    pub input_kind: InputKind,
    /// Whether to emit a tree or a distance matrix.
    pub output_kind: OutputKind,
    /// Alphabet override; detected from the alignment when `None`.
    pub alphabet: Option<Alphabet>,
    /// Distance correction; alphabet-dependent default when `None`.
    pub correction: Option<Correction>,
    /// Which neighbor-joining engine to use.
    pub method: Method,
    /// Search tunables.
    pub nj: NjParams,
    /// Worker threads for distance computation; 0 uses every core.
    pub threads: usize,
    /// Directory for the external-memory engine's scratch files.
    pub tmp_dir: Option<PathBuf>,
    /// Memory budget in bytes, used to choose an engine under
    /// [`Method::Auto`] and to size external-memory buffers.
    pub memory_bytes: u64,
    /// Largest distance joining two sequences into one cluster, for
    /// [`OutputKind::Clusters`].
    pub cluster_cutoff: f32,
    /// Build the tree over one representative of each set of identical
    /// sequences, then re-attach the others as zero-length chains.
    pub collapse_identical: bool,
    /// What to do when two input records share a name.
    pub duplicate_names: DuplicateNames,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            input: None,
            input_kind: InputKind::Alignment,
            output_kind: OutputKind::Tree,
            alphabet: None,
            correction: None,
            method: Method::Auto,
            nj: NjParams::default(),
            threads: 0,
            tmp_dir: None,
            memory_bytes: 2 << 30,
            cluster_cutoff: 0.03,
            collapse_identical: false,
            duplicate_names: DuplicateNames::Rename,
        }
    }
}

/// Summary returned by [`run`].
#[derive(Debug, Clone)]
pub struct RunOutput {
    /// Number of taxa processed.
    pub taxa: usize,
    /// The tree, when one was built.
    pub tree: Option<Tree>,
    /// Search counters, when a tree was built.
    pub stats: Option<NjStats>,
    /// Engine actually used, when a tree was built.
    pub method_used: Option<Method>,
    /// Number of clusters, when clustering was requested.
    pub clusters: Option<usize>,
}

/// Run the full pipeline, writing the tree or distance matrix to `out`.
pub fn run(opts: &Options, out: &mut dyn Write) -> Result<RunOutput> {
    let verbose = opts.nj.verbose;
    if opts.threads > 0 {
        // Ignore the error if a global pool already exists (library use).
        let _ = rayon::ThreadPoolBuilder::new().num_threads(opts.threads).build_global();
    }
    if opts.output_kind == OutputKind::Distances && opts.input_kind == InputKind::Distances {
        return Err(Error::options("input and output are both distance matrices; nothing to do"));
    }

    let t0 = Instant::now();
    match opts.input_kind {
        InputKind::Alignment => {
            let mut aln = match &opts.input {
                Some(p) => fasta::read_fasta(p, opts.alphabet)?,
                None => fasta::read_fasta_from(std::io::stdin().lock(), opts.alphabet)?,
            };
            warn_renamed(&resolve_duplicates(&mut aln.names, opts.duplicate_names)?);
            if opts.output_kind == OutputKind::Tree {
                warn_hash_names(&aln.names);
            }
            if verbose >= 1 {
                eprintln!(
                    "Read {} sequences of {} columns ({} alphabet)",
                    aln.len(),
                    aln.width(),
                    aln.alphabet
                );
            }
            // Tree building may run on one representative per identical group.
            let (aln, groups, all_names) = if opts.collapse_identical && opts.output_kind == OutputKind::Tree
            {
                let groups = aln.duplicate_groups();
                if groups.len() < aln.len() {
                    if verbose >= 1 {
                        eprintln!(
                            "Collapsed {} identical sequences into {} representatives",
                            aln.len(),
                            groups.len()
                        );
                    }
                    let reps = aln.representatives(&groups);
                    let all_names = aln.names;
                    (reps, Some(groups), Some(all_names))
                } else {
                    (aln, None, None)
                }
            } else {
                (aln, None, None)
            };
            let calc = DistanceCalculator::new(&aln, opts.correction)?;
            let k = aln.len();

            if opts.output_kind == OutputKind::Distances {
                let names = aln.names.clone();
                write_distances(out, &names, &calc)?;
                if verbose >= 1 {
                    eprintln!("Distances written ({:.1?})", t0.elapsed());
                }
                return Ok(RunOutput { taxa: k, tree: None, stats: None, method_used: None, clusters: None });
            }
            if opts.output_kind == OutputKind::Clusters {
                let clusters = cluster::single_linkage(k, opts.cluster_cutoff, |i, j| calc.calc(i, j));
                return write_clusters(opts, out, &aln.names, clusters, t0);
            }

            let method = choose_method(opts, k);
            match method {
                Method::ExtMem => {
                    let tmp = scratch_dir(opts)?;
                    let m = DiskMatrix::from_calculator(&calc, opts.memory_bytes, tmp.path())?;
                    if verbose >= 1 {
                        eprintln!("Distances computed ({:.1?})", t0.elapsed());
                    }
                    let names = aln.names;
                    drop(aln.seqs);
                    drop(calc);
                    let mut r = build_extmem(opts, &names, m, k, t0, &tmp)?;
                    finish_tree(opts, out, &mut r, groups.as_deref(), all_names.as_deref(), t0)?;
                    Ok(r)
                }
                _ => {
                    let d = DistanceMatrix::from_calculator(&calc);
                    if verbose >= 1 {
                        eprintln!("Distances computed ({:.1?})", t0.elapsed());
                    }
                    let names = aln.names;
                    drop(aln.seqs);
                    drop(calc);
                    let mut r = build_inmem(opts, &names, d, k, t0)?;
                    finish_tree(opts, out, &mut r, groups.as_deref(), all_names.as_deref(), t0)?;
                    Ok(r)
                }
            }
        }
        InputKind::Distances => {
            let path = opts
                .input
                .as_ref()
                .ok_or_else(|| Error::options("a distance matrix must be read from a file"))?;
            let mut p = phylip::read_phylip(path)?;
            warn_renamed(&resolve_duplicates(&mut p.names, opts.duplicate_names)?);
            if opts.output_kind == OutputKind::Tree {
                warn_hash_names(&p.names);
            }
            let k = p.len();
            if verbose >= 1 {
                eprintln!("Distance file read: {} taxa", k);
            }
            if opts.output_kind == OutputKind::Clusters {
                let clusters = cluster::single_linkage(k, opts.cluster_cutoff, |i, j| {
                    p.get(i, j) as f64 / phylip::SCALE as f64
                });
                return write_clusters(opts, out, &p.names, clusters, t0);
            }
            let method = choose_method(opts, k);
            match method {
                Method::ExtMem => {
                    let tmp = scratch_dir(opts)?;
                    let m = DiskMatrix::from_phylip(&p, opts.memory_bytes, tmp.path())?;
                    let mut r = build_extmem(opts, &p.names, m, k, t0, &tmp)?;
                    finish_tree(opts, out, &mut r, None, None, t0)?;
                    Ok(r)
                }
                _ => {
                    let d = DistanceMatrix::from_phylip(&p);
                    let mut r = build_inmem(opts, &p.names, d, k, t0)?;
                    finish_tree(opts, out, &mut r, None, None, t0)?;
                    Ok(r)
                }
            }
        }
    }
}

fn build_inmem(
    opts: &Options,
    names: &[String],
    d: DistanceMatrix,
    k: usize,
    t0: Instant,
) -> Result<RunOutput> {
    let verbose = opts.nj.verbose;
    if verbose >= 1 {
        eprintln!("Building tree (in-memory engine)");
    }
    let (tree, stats) = nj::inmem::build(names, d, &opts.nj)?;
    if verbose >= 1 {
        eprintln!("Tree built ({:.1?})", t0.elapsed());
    }
    Ok(RunOutput {
        taxa: k,
        tree: Some(tree),
        stats: Some(stats),
        method_used: Some(Method::InMem),
        clusters: None,
    })
}

/// Expand collapsed duplicates if any, write the tree, and record the
/// final taxon count.
fn finish_tree(
    opts: &Options,
    out: &mut dyn Write,
    r: &mut RunOutput,
    groups: Option<&[Vec<usize>]>,
    all_names: Option<&[String]>,
    t0: Instant,
) -> Result<()> {
    if let (Some(groups), Some(all_names)) = (groups, all_names) {
        let expanded = r.tree.as_ref().unwrap().expand_duplicates(groups, all_names);
        r.tree = Some(expanded);
        r.taxa = all_names.len();
    }
    r.tree.as_ref().unwrap().write_newick(out)?;
    out.flush()?;
    if opts.nj.verbose >= 1 {
        eprintln!("Tree written ({:.1?})", t0.elapsed());
    }
    Ok(())
}

fn build_extmem(
    opts: &Options,
    names: &[String],
    m: DiskMatrix,
    k: usize,
    t0: Instant,
    tmp: &tempfile::TempDir,
) -> Result<RunOutput> {
    let verbose = opts.nj.verbose;
    if verbose >= 1 {
        eprintln!(
            "Building tree (external-memory engine; scratch files in {}, matrix {})",
            tmp.path().display(),
            if m.uses_disk() { "partly on disk" } else { "resident" }
        );
    }
    let (tree, stats) = nj::extmem::build(names, m, &opts.nj, tmp.path(), opts.memory_bytes)?;
    if verbose >= 1 {
        eprintln!("Tree built ({:.1?})", t0.elapsed());
    }
    Ok(RunOutput {
        taxa: k,
        tree: Some(tree),
        stats: Some(stats),
        method_used: Some(Method::ExtMem),
        clusters: None,
    })
}

fn write_clusters(
    opts: &Options,
    out: &mut dyn Write,
    names: &[String],
    clusters: cluster::Clusters,
    t0: Instant,
) -> Result<RunOutput> {
    let mut w = std::io::BufWriter::new(out);
    cluster::write_table(&mut w, &clusters, names)?;
    w.flush()?;
    if opts.nj.verbose >= 1 {
        eprintln!(
            "{} sequences in {} clusters at cutoff {} ({:.1?})",
            names.len(),
            clusters.len(),
            opts.cluster_cutoff,
            t0.elapsed()
        );
    }
    Ok(RunOutput {
        taxa: names.len(),
        tree: None,
        stats: None,
        method_used: None,
        clusters: Some(clusters.len()),
    })
}

/// A private scratch directory, removed when dropped.
fn scratch_dir(opts: &Options) -> Result<tempfile::TempDir> {
    let parent = opts.tmp_dir.clone().unwrap_or_else(std::env::temp_dir);
    tempfile::Builder::new().prefix("ninja_").tempdir_in(&parent).map_err(|e| Error::io(parent, e))
}

fn write_distances(out: &mut dyn Write, names: &[String], calc: &DistanceCalculator) -> Result<()> {
    // Compute in parallel, then stream the square matrix.
    let d = DistanceMatrix::from_calculator(calc);
    let mut w = std::io::BufWriter::new(out);
    phylip::write_phylip(&mut w, names, |i, j| d.get_f64(i, j))?;
    Ok(())
}

/// Bytes the in-memory engine needs for `k` taxa: 4 per pair for the
/// packed triangle, 12 per pair for the queue entries of a rebuild, and
/// room for the entries added between rebuilds; about 20 bytes per pair
/// measured at 20,000 and 50,000 taxa, rounded up.
pub fn inmem_bytes(k: usize) -> u64 {
    let pairs = (k as u64) * (k as u64).saturating_sub(1) / 2;
    pairs * 22 + (k as u64) * 128
}

fn choose_method(opts: &Options, k: usize) -> Method {
    match opts.method {
        Method::Auto => {
            if inmem_bytes(k) <= opts.memory_bytes {
                Method::InMem
            } else {
                if opts.nj.verbose >= 1 {
                    eprintln!(
                        "{} taxa exceed the in-memory budget of {} MB; using the external-memory engine",
                        k,
                        opts.memory_bytes >> 20
                    );
                }
                Method::ExtMem
            }
        }
        m => m,
    }
}

/// Report renamed records on standard error at every verbosity, since the
/// output no longer carries the names as read.
/// Warn when names contain `#`. Readers of extended Newick (IcyTree,
/// Dendroscope, SplitsTree) take `name#tag` as a reticulation node and merge
/// every leaf that shares the tag, so such a tree displays as a network.
fn warn_hash_names(names: &[String]) {
    let n = names.iter().filter(|s| s.contains('#')).count();
    if n == 0 {
        return;
    }
    eprintln!(
        "warning: {} of {} sequence names contain '#'. Tree viewers that read extended Newick \
         (IcyTree, Dendroscope, SplitsTree) treat \"name#tag\" as a reticulation node and merge \
         every leaf sharing the tag, so this tree will display there as a network with cycles. \
         Plain Newick readers (FigTree, ape, ETE) show it correctly. Rename the sequences before \
         building the tree if the file must open in one of those viewers.",
        n,
        names.len()
    );
}

fn warn_renamed(renamed: &[Renamed]) {
    if renamed.is_empty() {
        return;
    }
    const SHOWN: usize = 20;
    let (noun, verb) = if renamed.len() == 1 { ("record", "was") } else { ("records", "were") };
    eprintln!(
        "warning: {} {} shared a name with an earlier record and {} renamed \
         (--duplicate_names error refuses such input):",
        renamed.len(),
        noun,
        verb
    );
    for r in renamed.iter().take(SHOWN) {
        eprintln!("  record {}: {} -> {}", r.index + 1, r.from, r.to);
    }
    if renamed.len() > SHOWN {
        eprintln!("  ... and {} more", renamed.len() - SHOWN);
    }
}
