//! End-to-end driver shared by the command-line tool and library users.

use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

use crate::alphabet::{Alphabet, Correction};
use crate::distance::{DistanceCalculator, DistanceMatrix};
use crate::error::{Error, Result};
use crate::io::{fasta, phylip};
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
    /// [`Method::Default`] and to size external-memory buffers.
    pub memory_bytes: u64,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            input: None,
            input_kind: InputKind::Alignment,
            output_kind: OutputKind::Tree,
            alphabet: None,
            correction: None,
            method: Method::Default,
            nj: NjParams::default(),
            threads: 0,
            tmp_dir: None,
            memory_bytes: 2 << 30,
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
            let aln = match &opts.input {
                Some(p) => fasta::read_fasta(p, opts.alphabet)?,
                None => fasta::read_fasta_from(std::io::stdin().lock(), opts.alphabet)?,
            };
            if verbose >= 1 {
                eprintln!(
                    "Read {} sequences of {} columns ({} alphabet)",
                    aln.len(),
                    aln.width(),
                    aln.alphabet
                );
            }
            let calc = DistanceCalculator::new(&aln, opts.correction)?;
            let k = aln.len();

            if opts.output_kind == OutputKind::Distances {
                let names = aln.names.clone();
                write_distances(out, &names, &calc)?;
                if verbose >= 1 {
                    eprintln!("Distances written ({:.1?})", t0.elapsed());
                }
                return Ok(RunOutput { taxa: k, tree: None, stats: None, method_used: None });
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
                    build_extmem(opts, out, &names, m, k, t0, &tmp)
                }
                _ => {
                    let d = DistanceMatrix::from_calculator(&calc);
                    if verbose >= 1 {
                        eprintln!("Distances computed ({:.1?})", t0.elapsed());
                    }
                    let names = aln.names;
                    drop(aln.seqs);
                    drop(calc);
                    build_inmem(opts, out, &names, d, k, t0)
                }
            }
        }
        InputKind::Distances => {
            let path = opts
                .input
                .as_ref()
                .ok_or_else(|| Error::options("a distance matrix must be read from a file"))?;
            let p = phylip::read_phylip(path)?;
            let k = p.len();
            if verbose >= 1 {
                eprintln!("Distance file read: {} taxa", k);
            }
            let method = choose_method(opts, k);
            match method {
                Method::ExtMem => {
                    let tmp = scratch_dir(opts)?;
                    let m = DiskMatrix::from_phylip(&p, opts.memory_bytes, tmp.path())?;
                    build_extmem(opts, out, &p.names, m, k, t0, &tmp)
                }
                _ => {
                    let d = DistanceMatrix::from_phylip(&p);
                    build_inmem(opts, out, &p.names, d, k, t0)
                }
            }
        }
    }
}

fn build_inmem(
    opts: &Options,
    out: &mut dyn Write,
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
    tree.write_newick(out)?;
    out.flush()?;
    if verbose >= 1 {
        eprintln!("Tree written ({:.1?})", t0.elapsed());
    }
    Ok(RunOutput { taxa: k, tree: Some(tree), stats: Some(stats), method_used: Some(Method::InMem) })
}

fn build_extmem(
    opts: &Options,
    out: &mut dyn Write,
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
    tree.write_newick(out)?;
    out.flush()?;
    if verbose >= 1 {
        eprintln!("Tree written ({:.1?})", t0.elapsed());
    }
    Ok(RunOutput { taxa: k, tree: Some(tree), stats: Some(stats), method_used: Some(Method::ExtMem) })
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

/// Bytes the in-memory engine needs for `k` taxa: the packed triangle plus
/// per-node arrays and heap entries (about 12 bytes per pair worst case).
pub fn inmem_bytes(k: usize) -> u64 {
    let pairs = (k as u64) * (k as u64).saturating_sub(1) / 2;
    pairs * 4 + pairs * 12 / 4 + (k as u64) * 64
}

fn choose_method(opts: &Options, k: usize) -> Method {
    match opts.method {
        Method::Default => {
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
