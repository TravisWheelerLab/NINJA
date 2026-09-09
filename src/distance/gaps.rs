//! Gap-opening counts for the onegap distance, over one-bit-per-site masks.
//!
//! Given validity masks `va` and `vb` (bit set where the sequence has a
//! core-alphabet residue, least significant bit first), the onegap distance
//! needs the number of maximal runs of columns where exactly one sequence is
//! valid, after columns where neither is valid have been removed. Runs are
//! counted per side: a run of "only A valid" followed directly by a run of
//! "only B valid" is two openings.
//!
//! Removing the both-invalid columns is done without compaction: each such
//! column inherits the state of the nearest earlier column that is not
//! both-invalid, using a carry-propagating addition over each run of
//! both-invalid columns. Runs may span words, so the caller threads the
//! state of the last column through [`RunState`].

/// State carried between consecutive words.
#[derive(Debug, Clone, Copy, Default)]
pub struct RunState {
    /// The last non-both-invalid column so far was "only A valid".
    only_a: bool,
    /// The last non-both-invalid column so far was "only B valid".
    only_b: bool,
}

/// Fill each run of `both` columns with the value of the bit preceding the
/// run (`prev` supplies the bit before position 0), for a mask `x` that is
/// disjoint from `both`.
#[inline]
fn fill_runs(x: u64, both: u64, prev: bool) -> u64 {
    let start = ((x << 1) | prev as u64) & both;
    let sum = both.wrapping_add(start);
    let extra = (sum ^ both ^ start) & both;
    x | start | extra
}

/// Count gap openings in one word of a pair, updating `state`. `both` must
/// be the mask of columns where neither sequence is valid, restricted to
/// real columns (padding bits must be zero in `va`, `vb`, and `both`).
#[inline]
pub fn openings_word(va: u64, vb: u64, both: u64, state: &mut RunState) -> u32 {
    let only_a = fill_runs(va & !vb, both, state.only_a);
    let only_b = fill_runs(vb & !va, both, state.only_b);
    let rises_a = only_a & !((only_a << 1) | state.only_a as u64);
    let rises_b = only_b & !((only_b << 1) | state.only_b as u64);
    // After filling, bit 63 holds the state of the last real column.
    state.only_a = only_a >> 63 == 1;
    state.only_b = only_b >> 63 == 1;
    rises_a.count_ones() + rises_b.count_ones()
}

/// Reference implementation over per-site validity flags, one column at a
/// time, transcribed from the C++ cluster branch's state machine.
pub fn openings_scalar(valid_a: &[bool], valid_b: &[bool]) -> u32 {
    let mut in_gap = 0u8;
    let mut openings = 0;
    for (&a, &b) in valid_a.iter().zip(valid_b) {
        if a && b {
            in_gap = 0;
        } else if a && !b {
            if in_gap != 1 {
                openings += 1;
            }
            in_gap = 1;
        } else if !a && b {
            if in_gap != 2 {
                openings += 1;
            }
            in_gap = 2;
        }
    }
    openings
}

/// Onegap distance from counts, as the C++ branch computed it. A pair with
/// nothing to compare gets the cap.
#[inline]
pub fn onegap_distance(mismatches: u32, sites: u32, openings: u32, maxscore: f32) -> f32 {
    let dist = (mismatches + openings) as f32 / (sites + openings) as f32;
    if dist < maxscore {
        dist
    } else {
        maxscore
    }
}

/// Count openings over whole-sequence masks (`len` real columns).
pub fn openings_masks(va: &[u64], vb: &[u64], len: usize) -> u32 {
    let mut state = RunState::default();
    let mut total = 0;
    for (w, (&a, &b)) in va.iter().zip(vb).enumerate() {
        let real = real_mask(w, len);
        total += openings_word(a & real, b & real, !a & !b & real, &mut state);
    }
    total
}

/// Mask of real (non-padding) columns in word `w` for a sequence of `len`.
#[inline]
pub fn real_mask(w: usize, len: usize) -> u64 {
    let start = w * 64;
    if len >= start + 64 {
        u64::MAX
    } else if len <= start {
        0
    } else {
        (1u64 << (len - start)) - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn masks(valid: &[bool]) -> Vec<u64> {
        let mut m = vec![0u64; valid.len().div_ceil(64).max(1)];
        for (i, &v) in valid.iter().enumerate() {
            if v {
                m[i / 64] |= 1 << (i % 64);
            }
        }
        m
    }

    fn check(a: &[bool], b: &[bool]) {
        let got = openings_masks(&masks(a), &masks(b), a.len());
        let want = openings_scalar(a, b);
        assert_eq!(got, want, "a={:?} b={:?}", a, b);
    }

    #[test]
    fn hand_cases() {
        let t = true;
        let f = false;
        check(&[t, t, t], &[t, t, t]);
        check(&[f, t, t], &[t, t, t]); // terminal gap counts
        check(&[t, t, f], &[t, t, t]);
        check(&[t, f, f, t], &[t, t, t, t]); // one run
        check(&[t, f, t, f, t], &[t, t, t, t, t]); // two runs
        check(&[t, f, t], &[t, t, f]); // one each
        check(&[t, f, f, t], &[t, t, f, t]); // run interrupted by both-gap: one
        check(&[t, f, f, t], &[t, f, t, t]); // both-gap then A-gap: one
        check(&[f, t], &[t, f]); // adjacent opposite runs: two
        check(&[f, f], &[f, f]); // nothing
        check(&[f, t, f, t], &[t, f, t, f]); // alternating: four
    }

    #[test]
    fn random_against_scalar_across_words() {
        let mut s: u64 = 12345;
        let mut next = move || {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (s >> 33) as u32
        };
        for trial in 0..2000 {
            let len = 1 + (next() % 300) as usize;
            let p_gap = [0.05, 0.3, 0.6][trial % 3];
            let mut gen =
                || -> Vec<bool> { (0..len).map(|_| (next() % 1000) as f64 / 1000.0 >= p_gap).collect() };
            let a = gen();
            let b = gen();
            check(&a, &b);
        }
    }
}
