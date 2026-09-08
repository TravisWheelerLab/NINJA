//! End-to-end tests of the `ninja` binary against reference outputs
//! produced by the original Java implementation (NINJA 1.2.2).
//!
//! Trees from the two implementations are compared exactly where the
//! arithmetic is integer (or float in the same order), and by splits with a
//! tolerance elsewhere. See `docs/testing.md` for how the references were
//! generated.

mod common;

use common::*;

const FIXTURES: &[&str] = &["PF08271_seed", "dna_200", "protein_120", "dna_700"];

#[test]
fn help_and_version() {
    let out = ninja_stdout(&["--help"]);
    assert!(out.contains("--in_type"));
    assert!(out.contains("--corr_type"));
    let out = ninja_stdout(&["--version"]);
    assert!(out.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn inmem_trees_match_java() {
    for f in FIXTURES {
        let path = fixture(&format!("{}.fa", f));
        let got = ninja_stdout(&["-q", "--in", path.to_str().unwrap()]);
        let want = read(&reference(&format!("{}.inmem.java.nwk", f)));
        assert_same_newick(&got, &want);
    }
}

#[test]
fn extmem_trees_match_java() {
    for f in FIXTURES {
        let path = fixture(&format!("{}.fa", f));
        let got = ninja_stdout(&["-q", "-m", "extmem", "--in", path.to_str().unwrap()]);
        let want = read(&reference(&format!("{}.extmem.java.nwk", f)));
        assert_same_newick(&got, &want);
    }
}

/// A tiny memory budget makes the resident window one block wide, so with
/// 700 taxa the matrix is flushed to disk during the build.
#[test]
fn extmem_with_disk_matches_java() {
    let path = fixture("dna_700.fa");
    let got = ninja_stdout(&["-q", "-m", "extmem", "--memory", "0.0001", "--in", path.to_str().unwrap()]);
    let want = read(&reference("dna_700.extmem.java.nwk"));
    assert_same_newick(&got, &want);
}

#[test]
fn positional_input_and_output_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("tree.nwk");
    let path = fixture("PF08271_seed.fa");
    let stdout = ninja_stdout(&["-q", path.to_str().unwrap(), "-o", out.to_str().unwrap()]);
    assert!(stdout.is_empty());
    let want = read(&reference("PF08271_seed.inmem.java.nwk"));
    assert_same_newick(&read(&out), &want);
}

#[test]
fn reads_alignment_from_stdin() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new(ninja_bin())
        .args(["-q"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let data = std::fs::read(fixture("PF08271_seed.fa")).unwrap();
    child.stdin.take().unwrap().write_all(&data).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let want = read(&reference("PF08271_seed.inmem.java.nwk"));
    assert_same_newick(&String::from_utf8(out.stdout).unwrap(), &want);
}

#[test]
fn distance_matrices_match_java() {
    for f in ["PF08271_seed", "dna_200", "protein_120"] {
        let path = fixture(&format!("{}.fa", f));
        let got = ninja_stdout(&["-q", "--out_type", "d", "--in", path.to_str().unwrap()]);
        let want = read(&reference(&format!("{}.java.phylip", f)));
        // Values are printed with six decimals by both; require identity.
        assert_phylip_close(&got, &want, 0.0);
    }
}

#[test]
fn tree_from_phylip_matches_java() {
    let path = reference("PF08271_seed.java.phylip");
    let got = ninja_stdout(&["-q", "--in_type", "d", "--in", path.to_str().unwrap()]);
    let want = read(&reference("PF08271_seed.fromphylip.java.nwk"));
    assert_same_newick(&got, &want);
}

/// Write the matrix, read it back, build with both engines, and compare to
/// Java doing the same. The written matrix itself was checked against Java
/// on the smaller fixtures (it is too large to keep for this one).
#[test]
fn phylip_round_trip_700() {
    let dir = tempfile::tempdir().unwrap();
    let phylip = dir.path().join("d.phylip");
    let path = fixture("dna_700.fa");
    ninja_stdout(&["-q", "--out_type", "d", "--in", path.to_str().unwrap(), "-o", phylip.to_str().unwrap()]);
    let got = ninja_stdout(&["-q", "--in_type", "d", "--in", phylip.to_str().unwrap()]);
    assert_same_newick(&got, &read(&reference("dna_700.fromphylip.java.nwk")));
    let got = ninja_stdout(&[
        "-q",
        "-m",
        "extmem",
        "--memory",
        "0.0001",
        "--in_type",
        "d",
        "--in",
        phylip.to_str().unwrap(),
    ]);
    assert_same_newick(&got, &read(&reference("dna_700.fromphylip.extmem.java.nwk")));
}

/// The two engines round distances differently (fixed point at 1e-8 versus
/// float at 1e-7, with float accumulation in the row sums), so they may
/// resolve near-ties differently and branch lengths drift slightly; they
/// must still agree on every branch of appreciable length.
#[test]
fn engines_agree_up_to_ties() {
    for f in FIXTURES {
        let path = fixture(&format!("{}.fa", f));
        let a = ninja_stdout(&["-q", "--in", path.to_str().unwrap()]);
        let b = ninja_stdout(&["-q", "-m", "extmem", "--in", path.to_str().unwrap()]);
        assert_trees_close(&a, &b, 1e-3, 1e-3);
    }
}

/// Sanity check on inference quality, not on the port: on simulated data
/// the tree should recover the generating tree apart from short branches.
#[test]
fn recovers_simulated_tree() {
    // dna_700 is left out: its 200-column sequences are too short for NJ
    // to recover a 700-taxon tree (that fixture exists to exercise disk paths).
    for f in ["dna_200", "protein_120"] {
        let path = fixture(&format!("{}.fa", f));
        let got = ninja_stdout(&["-q", "--in", path.to_str().unwrap()]);
        let truth = read(&fixture(&format!("{}.true.nwk", f)));
        let d = compare_trees(&got, &truth);
        let n = parse_newick(&truth).leaves.len();
        assert!(
            d.worst_mismatch() < 0.05 && d.rf() < n / 2,
            "{}: {} mismatched splits of {} taxa, longest {:.4}",
            f,
            d.rf(),
            n,
            d.worst_mismatch()
        );
    }
}

#[test]
fn alphabet_and_correction_flags() {
    let path = fixture("dna_200.fa");
    let p = path.to_str().unwrap();
    // Forcing no correction / Jukes-Cantor changes distances but still works.
    for corr in ["n", "j", "k"] {
        let out = ninja_stdout(&["-q", "--corr_type", corr, "--in", p]);
        parse_newick(&out);
    }
    // Scoredist on DNA is refused.
    let out = run_ninja(&["-q", "--corr_type", "s", "--in", p]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("scoredist"));
    // Treating DNA as protein is allowed (every residue is a valid amino acid).
    let out = ninja_stdout(&["-q", "--alph_type", "a", "--in", p]);
    parse_newick(&out);
}

#[test]
fn error_cases() {
    let out = run_ninja(&["-q", "--in", "/nonexistent/file.fa"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("nonexistent"));

    let dir = tempfile::tempdir().unwrap();
    let ragged = dir.path().join("ragged.fa");
    std::fs::write(&ragged, ">a\nACGT\n>b\nACG\n").unwrap();
    let out = run_ninja(&["-q", "--in", ragged.to_str().unwrap()]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("alignment"));

    let out = run_ninja(&["-q", "--in_type", "d", "--out_type", "d", "--in", ragged.to_str().unwrap()]);
    assert!(!out.status.success());

    let out = run_ninja(&["-q", "--method", "bogus", "--in", ragged.to_str().unwrap()]);
    assert!(!out.status.success());
}

#[test]
fn tiny_inputs() {
    let dir = tempfile::tempdir().unwrap();
    let one = dir.path().join("one.fa");
    std::fs::write(&one, ">only\nACGT\n").unwrap();
    assert_eq!(ninja_stdout(&["-q", "--in", one.to_str().unwrap()]).trim(), "only;");

    let two = dir.path().join("two.fa");
    std::fs::write(&two, ">a\nACGTACGTAC\n>b\nACGTACGTAG\n").unwrap();
    let out = ninja_stdout(&["-q", "--in", two.to_str().unwrap()]);
    let t = parse_newick(&out);
    assert_eq!(t.leaves.len(), 2);

    let three = dir.path().join("three.fa");
    std::fs::write(&three, ">a\nACGTACGTAC\n>b\nACGTACGTAG\n>c\nACGTACGAAG\n").unwrap();
    for method in ["inmem", "extmem"] {
        let out = ninja_stdout(&["-q", "-m", method, "--in", three.to_str().unwrap()]);
        let t = parse_newick(&out);
        assert_eq!(t.leaves.len(), 3);
    }
}

#[test]
fn threads_flag_gives_same_result() {
    let path = fixture("dna_200.fa");
    let p = path.to_str().unwrap();
    let a = ninja_stdout(&["-q", "-T", "1", "--in", p]);
    let b = ninja_stdout(&["-q", "-T", "8", "--in", p]);
    assert_same_newick(&a, &b);
}
