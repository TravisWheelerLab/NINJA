//! Shared helpers for integration tests: running the binary, parsing Newick,
//! and comparing trees by their splits.

#![allow(dead_code)]

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Path to the built `ninja` binary.
pub fn ninja_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ninja"))
}

/// Repository `tests/` directory.
pub fn tests_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

pub fn fixture(name: &str) -> PathBuf {
    tests_dir().join("fixtures").join(name)
}

pub fn reference(name: &str) -> PathBuf {
    tests_dir().join("reference").join(name)
}

pub fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {}", path.display(), e))
}

/// Run `ninja` with the given arguments; panics if it cannot be spawned.
pub fn run_ninja(args: &[&str]) -> Output {
    Command::new(ninja_bin()).args(args).output().expect("failed to run ninja binary")
}

/// Run `ninja`, assert success, and return stdout as a string.
pub fn ninja_stdout(args: &[&str]) -> String {
    let out = run_ninja(args);
    assert!(out.status.success(), "ninja {:?} failed:\n{}", args, String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).expect("ninja wrote non-UTF-8 output")
}

/// A parsed rooted tree: leaf names and, for every edge, the set of leaves
/// below it plus its length.
#[derive(Debug, Clone)]
pub struct ParsedTree {
    pub leaves: BTreeSet<String>,
    /// One entry per edge (excluding the root): leaves below the edge.
    pub clades: Vec<(BTreeSet<String>, f64)>,
}

/// Parse a Newick string with the grammar this tool writes: nested
/// parentheses, comma-separated children, `name:length` on leaves and
/// `):length` on internal nodes, terminated by `;`.
pub fn parse_newick(s: &str) -> ParsedTree {
    let s = s.trim().trim_end_matches(';').trim();
    let bytes = s.as_bytes();
    let mut pos = 0;
    let mut clades = Vec::new();
    let mut leaves = BTreeSet::new();
    let (root_leaves, _) = parse_node(bytes, &mut pos, &mut clades, &mut leaves);
    assert_eq!(pos, bytes.len(), "trailing text in Newick: {:?}", &s[pos..]);
    assert_eq!(root_leaves.len(), leaves.len());
    // The root's own "edge" is not a real edge; drop it.
    clades.retain(|(c, _)| c.len() < leaves.len());
    ParsedTree { leaves, clades }
}

fn parse_node(
    b: &[u8],
    pos: &mut usize,
    clades: &mut Vec<(BTreeSet<String>, f64)>,
    leaves: &mut BTreeSet<String>,
) -> (BTreeSet<String>, Option<f64>) {
    let mut below = BTreeSet::new();
    if b[*pos] == b'(' {
        *pos += 1;
        loop {
            let (child, _) = parse_node(b, pos, clades, leaves);
            below.extend(child);
            match b[*pos] {
                b',' => *pos += 1,
                b')' => {
                    *pos += 1;
                    break;
                }
                c => panic!("unexpected '{}' at {}", c as char, pos),
            }
        }
    } else {
        let start = *pos;
        while *pos < b.len() && !matches!(b[*pos], b':' | b',' | b')' | b'(') {
            *pos += 1;
        }
        let name = std::str::from_utf8(&b[start..*pos]).unwrap().to_string();
        assert!(!name.is_empty(), "empty leaf name at {}", start);
        assert!(leaves.insert(name.clone()), "duplicate leaf {}", name);
        below.insert(name);
    }
    let mut length = None;
    if *pos < b.len() && b[*pos] == b':' {
        *pos += 1;
        let start = *pos;
        while *pos < b.len() && !matches!(b[*pos], b',' | b')' | b'(') {
            *pos += 1;
        }
        length = Some(std::str::from_utf8(&b[start..*pos]).unwrap().parse().unwrap());
    }
    clades.push((below.clone(), length.unwrap_or(0.0)));
    (below, length)
}

/// Unrooted bipartitions of a tree, each represented by the side not
/// containing the alphabetically first leaf, keeping only non-trivial
/// splits (2 <= size <= n - 2). Lengths of edges inducing the same split
/// (the two root edges) are summed.
pub fn splits(t: &ParsedTree) -> HashMap<BTreeSet<String>, f64> {
    let n = t.leaves.len();
    let anchor = t.leaves.iter().next().cloned().unwrap();
    let mut out = HashMap::new();
    for (clade, len) in &t.clades {
        if clade.len() < 2 || clade.len() > n - 2 {
            continue;
        }
        let key: BTreeSet<String> = if clade.contains(&anchor) {
            t.leaves.difference(clade).cloned().collect()
        } else {
            clade.clone()
        };
        *out.entry(key).or_insert(0.0) += len;
    }
    out
}

/// Summary of how two trees differ.
#[derive(Debug, Default)]
pub struct TreeDiff {
    /// Lengths (in `a`) of splits in `a` but not `b`.
    pub only_a: Vec<f64>,
    /// Lengths (in `b`) of splits in `b` but not `a`.
    pub only_b: Vec<f64>,
    /// Largest absolute length difference over shared splits.
    pub max_len_diff: f64,
    /// Number of shared splits.
    pub shared: usize,
}

impl TreeDiff {
    /// Robinson-Foulds distance.
    pub fn rf(&self) -> usize {
        self.only_a.len() + self.only_b.len()
    }
    /// Length of the longest branch among the mismatched splits.
    pub fn worst_mismatch(&self) -> f64 {
        self.only_a.iter().chain(self.only_b.iter()).cloned().fold(0.0, f64::max)
    }
}

pub fn compare_trees(a: &str, b: &str) -> TreeDiff {
    let ta = parse_newick(a);
    let tb = parse_newick(b);
    assert_eq!(ta.leaves, tb.leaves, "trees have different leaf sets");
    let sa = splits(&ta);
    let sb = splits(&tb);
    let mut d = TreeDiff::default();
    for (k, la) in &sa {
        match sb.get(k) {
            Some(lb) => {
                d.shared += 1;
                d.max_len_diff = d.max_len_diff.max((la - lb).abs());
            }
            None => d.only_a.push(*la),
        }
    }
    for (k, lb) in &sb {
        if !sa.contains_key(k) {
            d.only_b.push(*lb);
        }
    }
    d
}

/// Assert two trees agree except possibly around branches no longer than
/// `tie_len`, and that shared branch lengths agree within `len_tol`.
pub fn assert_trees_close(a: &str, b: &str, tie_len: f64, len_tol: f64) {
    let d = compare_trees(a, b);
    assert!(
        d.worst_mismatch() <= tie_len,
        "trees differ on {} splits, longest mismatched branch {:.5} (allowed {:.5})",
        d.rf(),
        d.worst_mismatch(),
        tie_len
    );
    assert!(
        d.max_len_diff <= len_tol,
        "shared branch lengths differ by up to {:.6} (allowed {:.6})",
        d.max_len_diff,
        len_tol
    );
}

/// Assert two Newick strings are identical apart from surrounding whitespace.
pub fn assert_same_newick(a: &str, b: &str) {
    assert_eq!(a.trim(), b.trim(), "Newick strings differ");
}

/// Parse a square Phylip matrix into `(names, rows)`.
pub fn parse_phylip(s: &str) -> (Vec<String>, Vec<Vec<f64>>) {
    let mut lines = s.lines().filter(|l| !l.trim().is_empty());
    let k: usize = lines.next().unwrap().trim().parse().unwrap();
    let mut names = Vec::new();
    let mut rows = Vec::new();
    for line in lines.take(k) {
        let mut f = line.split_whitespace();
        names.push(f.next().unwrap().to_string());
        rows.push(f.map(|x| x.parse::<f64>().unwrap()).collect());
    }
    assert_eq!(names.len(), k);
    (names, rows)
}

/// Assert two Phylip matrices have the same names and values within `tol`.
pub fn assert_phylip_close(a: &str, b: &str, tol: f64) {
    let (na, va) = parse_phylip(a);
    let (nb, vb) = parse_phylip(b);
    assert_eq!(na, nb);
    let mut worst = 0.0f64;
    for (ra, rb) in va.iter().zip(&vb) {
        assert_eq!(ra.len(), rb.len());
        for (x, y) in ra.iter().zip(rb) {
            worst = worst.max((x - y).abs());
        }
    }
    assert!(worst <= tol, "distance matrices differ by up to {}", worst);
}
