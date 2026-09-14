//! Phylip distance-matrix reader and writer.
//!
//! The reader accepts the square format written by this tool (and FastTree),
//! and the lower-triangular format, because it only consumes the first `i`
//! numbers on row `i` and ignores the rest of the line.

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

use crate::error::{Error, Result};

/// Fixed-point scale for distances read into the in-memory engine:
/// a distance `d` is stored as `round(d * 1e8)`, then rounded to a multiple
/// of 100 so that a matrix read from a file and one computed from an
/// alignment agree to the six decimals the writer prints.
pub const SCALE: i64 = 100_000_000;

/// The lower triangle of a distance matrix, as parsed from a Phylip file.
#[derive(Debug, Clone, PartialEq)]
pub struct PhylipMatrix {
    /// Taxon names in file order.
    pub names: Vec<String>,
    /// `lower[i]` holds distances from taxon `i` to taxa `0..i`, in units of
    /// `1e-8` (see [`SCALE`]).
    pub lower: Vec<Vec<i64>>,
}

impl PhylipMatrix {
    /// Number of taxa.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// True when the matrix is empty.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Distance between `i` and `j` in `1e-8` units (`i != j`).
    pub fn get(&self, i: usize, j: usize) -> i64 {
        if i > j {
            self.lower[i][j]
        } else {
            self.lower[j][i]
        }
    }
}

/// Read a Phylip distance matrix from a file.
pub fn read_phylip(path: impl AsRef<Path>) -> Result<PhylipMatrix> {
    let path = path.as_ref();
    let f = File::open(path).map_err(|e| Error::io(path, e))?;
    read_phylip_from(BufReader::new(f)).map_err(|e| match e {
        Error::Io { path: None, source } => Error::io(path, source),
        other => other,
    })
}

/// Read a Phylip distance matrix from any reader.
pub fn read_phylip_from(reader: impl Read) -> Result<PhylipMatrix> {
    let mut r = BufReader::new(reader);
    let mut line = String::new();
    // First non-blank line holds the taxon count.
    let k: usize = loop {
        line.clear();
        if r.read_line(&mut line)? == 0 {
            return Err(Error::format("empty distance file"));
        }
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let first = t.split_whitespace().next().unwrap();
        break first.parse().map_err(|_| {
            Error::format(format!("expected the number of taxa on the first line, found '{}'", first))
        })?;
    };
    if k == 0 {
        return Err(Error::invalid("distance file declares zero taxa"));
    }

    let mut names = Vec::with_capacity(k);
    let mut lower = Vec::with_capacity(k);
    let mut row = 0;
    while row < k {
        line.clear();
        if r.read_line(&mut line)? == 0 {
            return Err(Error::format(format!(
                "too few rows in distance file: expected {}, found {}",
                k, row
            )));
        }
        let mut fields = line.split_whitespace();
        let name = match fields.next() {
            Some(n) => n.to_string(),
            None => continue, // blank line
        };
        let mut vals = Vec::with_capacity(row);
        for c in 0..row {
            let tok = fields.next().ok_or_else(|| {
                Error::format(format!(
                    "row {} ('{}') has {} distances but at least {} are required",
                    row + 1,
                    name,
                    c,
                    row
                ))
            })?;
            vals.push(parse_scaled(tok).ok_or_else(|| {
                Error::format(format!("bad distance '{}' on row {} ('{}')", tok, row + 1, name))
            })?);
        }
        names.push(name);
        lower.push(vals);
        row += 1;
    }
    Ok(PhylipMatrix { names, lower })
}

/// Parse a decimal number exactly into `1e-8` units, rounding half up at the
/// eighth decimal. Falls back to float parsing for exponent notation.
fn parse_scaled(tok: &str) -> Option<i64> {
    let b = tok.as_bytes();
    let mut i = 0;
    let mut neg = false;
    if i < b.len() && (b[i] == b'-' || b[i] == b'+') {
        neg = b[i] == b'-';
        i += 1;
    }
    let mut int_part: i64 = 0;
    let mut saw_digit = false;
    while i < b.len() && b[i].is_ascii_digit() {
        int_part = int_part.checked_mul(10)?.checked_add((b[i] - b'0') as i64)?;
        saw_digit = true;
        i += 1;
    }
    let mut frac: i64 = 0;
    let mut frac_digits = 0;
    let mut round_up = false;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            let d = (b[i] - b'0') as i64;
            if frac_digits < 8 {
                frac = frac * 10 + d;
                frac_digits += 1;
            } else if frac_digits == 8 {
                round_up = d >= 5;
                frac_digits += 1;
            }
            saw_digit = true;
            i += 1;
        }
    }
    if i != b.len() {
        // Exponent or garbage: let the float parser decide.
        let v: f64 = tok.parse().ok()?;
        return Some((v * SCALE as f64).round() as i64);
    }
    if !saw_digit {
        return None;
    }
    let mut scaled = int_part.checked_mul(SCALE)?;
    let mut f = frac;
    for _ in frac_digits.min(8)..8 {
        f *= 10;
    }
    scaled = scaled.checked_add(f)?;
    if round_up {
        scaled += 1;
    }
    Some(if neg { -scaled } else { scaled })
}

/// Write a square Phylip distance matrix.
///
/// `dist(i, j)` must return the distance between taxa `i` and `j` for
/// `i != j`; it is called for every ordered pair, from several threads, so
/// it should be a cheap lookup. Rows are formatted in parallel, a block at a
/// time, and written in order. Values are printed with six decimals,
/// formatted exactly as the reference implementation did.
pub fn write_phylip<W: Write>(
    mut w: W,
    names: &[String],
    dist: impl Fn(usize, usize) -> f64 + Sync,
) -> std::io::Result<()> {
    use rayon::prelude::*;
    let k = names.len();
    writeln!(w, "{}", k)?;
    const ROWS_PER_BLOCK: usize = 256;
    let mut start = 0;
    while start < k {
        let end = (start + ROWS_PER_BLOCK).min(k);
        let rows: Vec<String> = (start..end)
            .into_par_iter()
            .map(|i| {
                let mut buf = String::with_capacity(k * 9 + names[i].len() + 2);
                buf.push_str(&names[i]);
                for j in 0..k {
                    buf.push(' ');
                    if i == j {
                        buf.push_str("0.000000");
                    } else {
                        push_fixed6(&mut buf, dist(i, j));
                    }
                }
                buf.push('\n');
                buf
            })
            .collect();
        for row in &rows {
            w.write_all(row.as_bytes())?;
        }
        start = end;
    }
    w.flush()
}

/// Format `val` with six decimals the way the reference did: truncate the
/// integer part after adding half a unit in the last place, then print the
/// fraction rounded to six digits.
pub fn push_fixed6(buf: &mut String, val: f64) {
    use std::fmt::Write as _;
    let int_part = (val + 0.0000005) as i64;
    let _ = write!(buf, "{}.", int_part);
    let frac = val - int_part as f64;
    if frac < 0.0 {
        buf.push_str("000000");
    } else {
        let mut shift = 10.0;
        while (frac + 0.0000005) * shift < 1.0 && shift < 1_000_000.0 {
            buf.push('0');
            shift *= 10.0;
        }
        let _ = write!(buf, "{}", (0.5 + frac * 1_000_000.0) as i64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_scaled_exact() {
        assert_eq!(parse_scaled("0.705596"), Some(70_559_600));
        assert_eq!(parse_scaled("1"), Some(100_000_000));
        assert_eq!(parse_scaled("2.5"), Some(250_000_000));
        assert_eq!(parse_scaled("-0.000001"), Some(-100));
        assert_eq!(parse_scaled("0.123456789"), Some(12_345_679));
        assert_eq!(parse_scaled("1e-2"), Some(1_000_000));
        assert_eq!(parse_scaled("abc"), None);
        assert_eq!(parse_scaled(""), None);
    }

    #[test]
    fn reads_square_and_lower() {
        let square = "3\na 0.0 0.5 0.25\nb 0.5 0.0 0.75\nc 0.25 0.75 0.0\n";
        let lower = "  3\na\nb 0.5\nc 0.25 0.75\n";
        let s = read_phylip_from(square.as_bytes()).unwrap();
        let l = read_phylip_from(lower.as_bytes()).unwrap();
        assert_eq!(s, l);
        assert_eq!(s.get(0, 2), 25_000_000);
        assert_eq!(s.get(2, 1), 75_000_000);
    }

    #[test]
    fn fixed6_formatting() {
        let mut s = String::new();
        push_fixed6(&mut s, 0.705596);
        assert_eq!(s, "0.705596");
        s.clear();
        push_fixed6(&mut s, 1.9999996);
        assert_eq!(s, "2.000000");
        s.clear();
        push_fixed6(&mut s, 0.00999996);
        assert_eq!(s, "0.010000");
        s.clear();
        push_fixed6(&mut s, 3.0);
        assert_eq!(s, "3.000000");
    }

    #[test]
    fn write_round_trip() {
        let names = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let d = [[0.0, 0.5, 0.25], [0.5, 0.0, 0.75], [0.25, 0.75, 0.0]];
        let mut out = Vec::new();
        write_phylip(&mut out, &names, |i, j| d[i][j]).unwrap();
        let m = read_phylip_from(out.as_slice()).unwrap();
        assert_eq!(m.names, names);
        assert_eq!(m.get(1, 2), 75_000_000);
    }
}
