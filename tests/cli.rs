//! End-to-end tests of the `ninja` binary against reference outputs
//! produced by the original Java implementation (NINJA 1.2.2).
//!
//! Trees from the two implementations are compared exactly where the
//! arithmetic is integer (or float in the same order), and by splits with a
//! tolerance elsewhere. See `docs/testing.md` for how the references were
//! generated.

mod common;

use std::collections::BTreeSet;

use common::*;

const FIXTURES: &[&str] = &["PF08271_seed", "dna_200", "protein_120", "dna_700"];

/// The external-memory engine keeps row sums and the criterion in double
/// precision where Java used single precision, so its trees are compared
/// with Java's by splits and lengths: disagreement only on branches up to
/// `EXTMEM_TIE_TOL` long, shared lengths within `EXTMEM_LEN_TOL`.
const EXTMEM_TIE_TOL: f64 = 1e-3;
const EXTMEM_LEN_TOL: f64 = 5e-4;

#[test]
fn help_and_version() {
    let out = ninja_stdout(&["--help"]);
    assert!(out.contains("--in_type"));
    assert!(out.contains("--corr_type"));
    let out = ninja_stdout(&["--version"]);
    assert!(out.contains(env!("CARGO_PKG_VERSION")));
}

/// In reference-order mode the in-memory engine reproduces the Java tool
/// byte for byte.
#[test]
fn inmem_trees_match_java_in_reference_order() {
    for f in FIXTURES {
        let path = fixture(&format!("{}.fa", f));
        let got = ninja_stdout(&["-q", "--reference_order", "--in", path.to_str().unwrap()]);
        let want = read(&reference(&format!("{}.inmem.java.nwk", f)));
        assert_same_newick(&got, &want);
    }
}

/// The default engine may resolve exact ties differently, so it is held to
/// the same splits and branch lengths rather than the same text.
#[test]
fn inmem_trees_match_java_by_default() {
    for f in FIXTURES {
        let path = fixture(&format!("{}.fa", f));
        let got = ninja_stdout(&["-q", "--in", path.to_str().unwrap()]);
        let want = read(&reference(&format!("{}.inmem.java.nwk", f)));
        assert_trees_close(&got, &want, 0.0, 0.0);
    }
}

#[test]
fn extmem_trees_match_java() {
    for f in FIXTURES {
        let path = fixture(&format!("{}.fa", f));
        let want = read(&reference(&format!("{}.extmem.java.nwk", f)));
        for mode in [&["--reference_order"][..], &[][..]] {
            let mut args = vec!["-q", "-m", "extmem"];
            args.extend_from_slice(mode);
            args.extend_from_slice(&["--in", path.to_str().unwrap()]);
            let got = ninja_stdout(&args);
            assert_trees_close(&got, &want, EXTMEM_TIE_TOL, EXTMEM_LEN_TOL);
        }
    }
}

/// A tiny memory budget makes the resident window one block wide, so with
/// 700 taxa the matrix is flushed to disk during the build, and the heaps'
/// runs spill to disk as well.
#[test]
fn extmem_with_disk_matches_java() {
    let path = fixture("dna_700.fa");
    let want = read(&reference("dna_700.extmem.java.nwk"));
    for mode in [&["--reference_order"][..], &[][..]] {
        let mut args = vec!["-q", "-m", "extmem", "--memory", "0.0001"];
        args.extend_from_slice(mode);
        args.extend_from_slice(&["--in", path.to_str().unwrap()]);
        let got = ninja_stdout(&args);
        assert_trees_close(&got, &want, EXTMEM_TIE_TOL, EXTMEM_LEN_TOL);
    }
}

#[test]
fn positional_input_and_output_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("tree.nwk");
    let path = fixture("PF08271_seed.fa");
    let stdout =
        ninja_stdout(&["-q", "--reference_order", path.to_str().unwrap(), "-o", out.to_str().unwrap()]);
    assert!(stdout.is_empty());
    let want = read(&reference("PF08271_seed.inmem.java.nwk"));
    assert_same_newick(&read(&out), &want);
}

/// Rebuild schedules only change which of two tied pairs is joined first;
/// that can move a branch length by one unit in the last printed digit.
#[test]
fn rebuild_ratio_does_not_change_the_tree() {
    let path = fixture("dna_700.fa");
    let p = path.to_str().unwrap();
    let base = ninja_stdout(&["-q", "--in", p]);
    for r in ["0.1", "0.5", "0.9"] {
        let got = ninja_stdout(&["-q", "-r", r, "--in", p]);
        assert_trees_close(&got, &base, 0.0, 1.5e-5);
    }
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
    assert_trees_close(&String::from_utf8(out.stdout).unwrap(), &want, 0.0, 0.0);
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
    let got = ninja_stdout(&["-q", "--reference_order", "--in_type", "d", "--in", path.to_str().unwrap()]);
    let want = read(&reference("PF08271_seed.fromphylip.java.nwk"));
    assert_same_newick(&got, &want);
    let got = ninja_stdout(&["-q", "--in_type", "d", "--in", path.to_str().unwrap()]);
    assert_trees_close(&got, &want, 0.0, 0.0);
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
    let got = ninja_stdout(&["-q", "--reference_order", "--in_type", "d", "--in", phylip.to_str().unwrap()]);
    assert_same_newick(&got, &read(&reference("dna_700.fromphylip.java.nwk")));
    let want = read(&reference("dna_700.fromphylip.extmem.java.nwk"));
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
    assert_trees_close(&got, &want, EXTMEM_TIE_TOL, EXTMEM_LEN_TOL);
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

// ---- features from the C++ `cluster` branch ----

#[test]
fn onegap_distances_match_cpp_branch() {
    // dna_700 was checked the same way; its 4 MB matrix is not kept.
    let path = fixture("dna_200.fa");
    let got = ninja_stdout(&["-q", "--out_type", "d", "--corr_type", "m", "--in", path.to_str().unwrap()]);
    let want = read(&reference("dna_200.onegap.cpp.phylip"));
    assert_phylip_close(&got, &want, 0.0);
}

#[test]
fn onegap_accepts_protein() {
    let path = fixture("protein_120.fa");
    let out = ninja_stdout(&["-q", "--out_type", "d", "--corr_type", "m", "--in", path.to_str().unwrap()]);
    let (_, rows) = parse_phylip(&out);
    assert!(rows.iter().flatten().all(|&v| (0.0..=1.0).contains(&v)));
}

fn read_clusters(text: &str) -> Vec<std::collections::BTreeSet<String>> {
    let mut by_id: std::collections::BTreeMap<u32, std::collections::BTreeSet<String>> = Default::default();
    for line in text.lines().filter(|l| !l.is_empty()) {
        let (id, name) = line.split_once('\t').expect("id<TAB>name");
        by_id.entry(id.parse().unwrap()).or_default().insert(name.to_string());
    }
    by_id.into_values().collect()
}

/// The C++ branch's merge loop skips some updates and over-splits, so its
/// clusters must each lie inside one of ours; at the lowest cutoff, where
/// the skipped updates do not matter on these data, the partitions agree.
#[test]
fn clusters_refine_cpp_branch_tables() {
    for f in ["dna_200", "dna_700"] {
        let path = fixture(&format!("{}.fa", f));
        for cutoff in ["0.03", "0.1", "0.3"] {
            let got = read_clusters(&ninja_stdout(&[
                "-q",
                "--out_type",
                "c",
                "--cluster_cutoff",
                cutoff,
                "--in",
                path.to_str().unwrap(),
            ]));
            let want = read_clusters(&read(&reference(&format!("{}.clusters_{}.cpp.tsv", f, cutoff))));
            let total: usize = got.iter().map(|c| c.len()).sum();
            assert_eq!(total, want.iter().map(|c| c.len()).sum::<usize>());
            for c in &want {
                assert!(got.iter().any(|g| c.is_subset(g)), "{} {}: C++ cluster {:?} split", f, cutoff, c);
            }
            assert!(got.len() <= want.len());
            if cutoff == "0.03" {
                assert_eq!(got, want, "{} at cutoff {}", f, cutoff);
            }
        }
    }
}

#[test]
fn clusters_from_phylip_input_and_numbering() {
    let dir = tempfile::tempdir().unwrap();
    let phylip = dir.path().join("d.phylip");
    let path = fixture("dna_200.fa");
    ninja_stdout(&["-q", "--out_type", "d", "--in", path.to_str().unwrap(), "-o", phylip.to_str().unwrap()]);
    let a =
        ninja_stdout(&["-q", "--out_type", "c", "--cluster_cutoff", "0.1", "--in", path.to_str().unwrap()]);
    let b = ninja_stdout(&[
        "-q",
        "--out_type",
        "c",
        "--cluster_cutoff",
        "0.1",
        "--in_type",
        "d",
        "--in",
        phylip.to_str().unwrap(),
    ]);
    assert_eq!(a, b);
    // Ids start at 0, appear in order of first member, and every input
    // sequence appears exactly once.
    let mut seen_ids = Vec::new();
    for line in a.lines() {
        let id: u32 = line.split('\t').next().unwrap().parse().unwrap();
        if seen_ids.last() != Some(&id) {
            seen_ids.push(id);
        }
    }
    assert_eq!(seen_ids, (0..seen_ids.len() as u32).collect::<Vec<_>>());
    assert_eq!(a.lines().count(), 200);
}

/// With duplicates collapsed, the tree must agree with the uncollapsed one
/// on every branch of non-zero length; only the arrangement of the
/// zero-length branches among identical sequences may differ. Branch
/// lengths shift slightly, because the NJ length formula depends on the
/// taxon count and row sums, which the duplicates change.
#[test]
fn collapse_identical_preserves_tree() {
    let path = fixture("dna_200_dups.fa");
    let p = path.to_str().unwrap();
    for method in ["inmem", "extmem"] {
        let plain = ninja_stdout(&["-q", "-m", method, "--in", p]);
        let collapsed = ninja_stdout(&["-q", "-m", method, "--collapse_identical", "--in", p]);
        assert_ne!(plain, collapsed);
        assert_trees_close(&plain, &collapsed, 1e-9, 5e-3);
        let t = parse_newick(&collapsed);
        assert_eq!(t.leaves.len(), 207);
        assert!(collapsed.contains("seq133_dup1:0.00000"));
    }
    // Without duplicates the flag changes nothing.
    let path = fixture("dna_200.fa");
    let p = path.to_str().unwrap();
    assert_eq!(ninja_stdout(&["-q", "--in", p]), ninja_stdout(&["-q", "--collapse_identical", "--in", p]));
}

#[test]
fn duplicate_names_are_renamed_and_reported() {
    let f = common::fixture("dna_dup_names.fa");
    let out = common::run_ninja(&["--in", f.to_str().unwrap(), "-q"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(stderr.contains("3 records shared a name"), "{stderr}");
    for line in ["record 3: seqA -> seqA_2", "record 5: seqB -> seqB_2", "record 6: seqA -> seqA_3"] {
        assert!(stderr.contains(line), "missing {line:?} in:\n{stderr}");
    }
    let tree = common::parse_newick(&String::from_utf8_lossy(&out.stdout));
    let want: BTreeSet<String> = ["seqA", "seqB", "seqA_2", "seqC", "seqB_2", "seqA_3", "seqD", "seqE"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(tree.leaves, want);
}

#[test]
fn duplicate_names_error_policy_refuses_input() {
    let f = common::fixture("dna_dup_names.fa");
    let out = common::run_ninja(&["--in", f.to_str().unwrap(), "--duplicate_names", "error"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("2 names appear more than once in the input: seqA, seqB"), "{stderr}");
}

#[test]
fn duplicate_names_in_a_distance_matrix_are_renamed() {
    let f = common::fixture("dup_names.phylip");
    let out = common::run_ninja(&["--in_type", "d", "--in", f.to_str().unwrap(), "-q"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(stderr.contains("record 3: a -> a_2"), "{stderr}");
    let tree = common::parse_newick(&String::from_utf8_lossy(&out.stdout));
    let want: BTreeSet<String> = ["a", "b", "a_2", "c"].iter().map(|s| s.to_string()).collect();
    assert_eq!(tree.leaves, want);
}

#[test]
fn names_with_hash_draw_a_warning_for_tree_output() {
    let f = common::fixture("dna_hash_names.fa");
    let out = common::run_ninja(&["--in", f.to_str().unwrap(), "-q"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(stderr.contains("8 of 8 sequence names contain '#'"), "{stderr}");
    assert!(stderr.contains("extended Newick"), "{stderr}");
    // Distance output is not a tree, so no warning.
    let out = common::run_ninja(&["--in", f.to_str().unwrap(), "-q", "--out_type", "d"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert!(!stderr.contains("contain '#'"), "{stderr}");
}
