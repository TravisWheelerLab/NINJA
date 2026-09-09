//! Command-line front end for NINJA.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use ninja::{Alphabet, Correction, InputKind, Method, NjParams, Options, OutputKind};

/// NINJA: large-scale neighbor-joining phylogeny inference.
///
/// Reads a FASTA alignment (or a Phylip distance matrix) and writes a
/// Newick tree (or a distance matrix). Flag names follow the original Java
/// and C tools so existing scripts keep working.
#[derive(Parser, Debug)]
#[command(name = "ninja", version, about, long_about = None, disable_help_flag = true)]
struct Cli {
    /// Input file. May also be given with --in. Reads standard input if absent.
    #[arg(value_name = "FILE")]
    input_positional: Option<PathBuf>,

    /// Input file.
    #[arg(short = 'i', long = "in", value_name = "FILE")]
    input: Option<PathBuf>,

    /// Output file (default: standard output).
    #[arg(short = 'o', long = "out", value_name = "FILE")]
    output: Option<PathBuf>,

    /// Engine: 'default' picks in-memory when it fits the memory budget,
    /// 'inmem' or 'extmem' force one.
    #[arg(short = 'm', long = "method", default_value = "default", value_parser = parse_method)]
    method: Method,

    /// Input type: 'a' alignment (FASTA) or 'd' distance matrix (Phylip).
    #[arg(long = "in_type", default_value = "a", value_parser = parse_in_type, value_name = "a|d")]
    in_type: InputKind,

    /// Output type: 't' tree (Newick), 'd' distance matrix (Phylip), or
    /// 'c' single-linkage clusters (cluster id and name per line).
    #[arg(long = "out_type", default_value = "t", value_parser = parse_out_type, value_name = "t|d|c")]
    out_type: OutputKind,

    /// Build the tree over one representative of each set of identical
    /// sequences, then attach the rest as zero-length branches.
    #[arg(long = "collapse_identical")]
    collapse_identical: bool,

    /// Largest distance joining two sequences into one cluster (--out_type c).
    #[arg(long = "cluster_cutoff", default_value_t = 0.03, value_name = "DIST")]
    cluster_cutoff: f32,

    /// Alphabet: 'a' amino acid or 'd' DNA. Detected from the input by default.
    #[arg(long = "alph_type", value_parser = parse_alphabet, value_name = "a|d")]
    alph_type: Option<Alphabet>,

    /// Correction: 'n' none, 'j' Jukes-Cantor, 'k' Kimura 2-parameter (DNA),
    /// 's' scoredist (protein), 'm' Mothur onegap (either). Default: 'k' for
    /// DNA, 's' for protein.
    #[arg(long = "corr_type", value_parser = parse_correction, value_name = "n|j|k|s|m")]
    corr_type: Option<Correction>,

    /// Worker threads for distance computation (0 = all cores).
    #[arg(short = 'T', long = "threads", default_value_t = 0)]
    threads: usize,

    /// Directory for external-memory scratch files (default: system temp).
    #[arg(short = 't', long = "tmp_dir", value_name = "DIR")]
    tmp_dir: Option<PathBuf>,

    /// Memory budget in gigabytes for choosing and sizing the engine.
    /// Default: 75% of physical memory, or 2 if that cannot be determined.
    #[arg(long = "memory", value_name = "GB")]
    memory_gb: Option<f64>,

    /// Number of row-sum clusters (see the paper).
    #[arg(short = 's', long = "clust_size", default_value_t = 30)]
    clust_size: usize,

    /// Fraction of remaining taxa joined between rebuilds (see the paper).
    #[arg(short = 'r', long = "rebuild_step_ratio", default_value_t = 0.5)]
    rebuild_step_ratio: f32,

    /// Verbosity 0-3.
    #[arg(short = 'v', long = "verbose", default_value_t = 1)]
    verbose: u8,

    /// Same as --verbose 0.
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Print help.
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,
}

fn parse_method(s: &str) -> Result<Method, String> {
    s.parse()
}
fn parse_alphabet(s: &str) -> Result<Alphabet, String> {
    s.parse()
}
fn parse_correction(s: &str) -> Result<Correction, String> {
    s.parse()
}
fn parse_in_type(s: &str) -> Result<InputKind, String> {
    match s {
        "a" => Ok(InputKind::Alignment),
        "d" => Ok(InputKind::Distances),
        _ => Err(format!("unknown in_type '{}' (expected 'a' or 'd')", s)),
    }
}
fn parse_out_type(s: &str) -> Result<OutputKind, String> {
    match s {
        "t" => Ok(OutputKind::Tree),
        "d" => Ok(OutputKind::Distances),
        "c" => Ok(OutputKind::Clusters),
        _ => Err(format!("unknown out_type '{}' (expected 't', 'd', or 'c')", s)),
    }
}

/// Physical memory in bytes, from /proc/meminfo on Linux.
fn physical_memory() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/meminfo").ok()?;
    let line = text.lines().find(|l| l.starts_with("MemTotal:"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb * 1024)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let verbose = if cli.quiet { 0 } else { cli.verbose };
    if verbose >= 1 {
        eprintln!("NINJA v{} (Rust) by Travis Wheeler", ninja::VERSION);
        eprintln!("Please cite:\n{}\n", ninja::CITATION);
    }

    let input = cli.input.or(cli.input_positional);
    let memory_bytes = match cli.memory_gb {
        Some(gb) => (gb * (1u64 << 30) as f64) as u64,
        None => physical_memory().map(|b| b / 4 * 3).unwrap_or(2 << 30),
    };
    let opts = Options {
        input,
        input_kind: cli.in_type,
        output_kind: cli.out_type,
        alphabet: cli.alph_type,
        correction: cli.corr_type,
        method: cli.method,
        nj: NjParams {
            cluster_count: cli.clust_size,
            rebuild_step_ratio: cli.rebuild_step_ratio,
            verbose,
            ..NjParams::default()
        },
        threads: cli.threads,
        tmp_dir: cli.tmp_dir,
        memory_bytes,
        cluster_cutoff: cli.cluster_cutoff,
        collapse_identical: cli.collapse_identical,
    };

    let result = match &cli.output {
        Some(path) => match File::create(path) {
            Ok(f) => {
                let mut w = BufWriter::new(f);
                ninja::run(&opts, &mut w).and_then(|r| {
                    w.flush()?;
                    Ok(r)
                })
            }
            Err(e) => {
                eprintln!("ninja: cannot create {}: {}", path.display(), e);
                return ExitCode::from(2);
            }
        },
        None => {
            let stdout = io::stdout();
            let mut w = BufWriter::new(stdout.lock());
            ninja::run(&opts, &mut w).and_then(|r| {
                w.flush()?;
                Ok(r)
            })
        }
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ninja: {}", e);
            ExitCode::from(1)
        }
    }
}
