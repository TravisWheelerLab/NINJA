//! Duplicate record names.
//!
//! Newick and Phylip files identify taxa by name, so a name that appears
//! twice in the input makes the output ambiguous. [`resolve_duplicates`]
//! either renames the later occurrences or rejects the input.

use std::collections::HashSet;

use crate::error::{Error, Result};

/// What to do when two input records share a name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DuplicateNames {
    /// Append `_2`, `_3`, ... to the second and later occurrences and
    /// report each rename.
    #[default]
    Rename,
    /// Refuse the input, naming the duplicates.
    Error,
}

/// One record renamed by [`resolve_duplicates`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Renamed {
    /// Position of the record in input order, starting at 0.
    pub index: usize,
    /// The name as read.
    pub from: String,
    /// The name the output will carry.
    pub to: String,
}

/// Make every name unique according to `policy`.
///
/// Under [`DuplicateNames::Rename`] the first occurrence keeps its name and
/// each later one gets the smallest suffix `_2`, `_3`, ... that collides
/// with no name in the input and no name already assigned. The returned
/// list describes every change, in input order. Under
/// [`DuplicateNames::Error`] any repeated name is an error and `names` is
/// left unchanged.
pub fn resolve_duplicates(names: &mut [String], policy: DuplicateNames) -> Result<Vec<Renamed>> {
    let original: HashSet<&str> = names.iter().map(String::as_str).collect();
    if original.len() == names.len() {
        return Ok(Vec::new());
    }
    if policy == DuplicateNames::Error {
        let mut seen: HashSet<&str> = HashSet::with_capacity(names.len());
        let mut repeated: Vec<&str> = Vec::new();
        for name in names.iter() {
            if !seen.insert(name) && !repeated.contains(&name.as_str()) {
                repeated.push(name);
            }
        }
        const SHOWN: usize = 10;
        let (noun, verb) = if repeated.len() == 1 { ("name", "appears") } else { ("names", "appear") };
        let more = repeated.len().saturating_sub(SHOWN);
        let tail = if more > 0 { format!(" and {} more", more) } else { String::new() };
        return Err(Error::invalid(format!(
            "{} {} {} more than once in the input: {}{}",
            repeated.len(),
            noun,
            verb,
            repeated.iter().take(SHOWN).cloned().collect::<Vec<_>>().join(", "),
            tail
        )));
    }

    let original: HashSet<String> = names.iter().cloned().collect();
    let mut assigned: HashSet<String> = HashSet::with_capacity(names.len());
    let mut renamed = Vec::new();
    for (index, name) in names.iter_mut().enumerate() {
        if assigned.insert(name.clone()) {
            continue;
        }
        let mut k = 2;
        let to = loop {
            let candidate = format!("{}_{}", name, k);
            if !original.contains(&candidate) && !assigned.contains(&candidate) {
                break candidate;
            }
            k += 1;
        };
        assigned.insert(to.clone());
        renamed.push(Renamed { index, from: name.clone(), to: to.clone() });
        *name = to;
    }
    Ok(renamed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn unique_names_are_untouched() {
        let mut n = names(&["a", "b", "c"]);
        let r = resolve_duplicates(&mut n, DuplicateNames::Rename).unwrap();
        assert!(r.is_empty());
        assert_eq!(n, names(&["a", "b", "c"]));
        assert!(resolve_duplicates(&mut n, DuplicateNames::Error).unwrap().is_empty());
    }

    #[test]
    fn later_occurrences_get_suffixes() {
        let mut n = names(&["a", "b", "a", "a", "b"]);
        let r = resolve_duplicates(&mut n, DuplicateNames::Rename).unwrap();
        assert_eq!(n, names(&["a", "b", "a_2", "a_3", "b_2"]));
        let got: Vec<(usize, &str, &str)> =
            r.iter().map(|x| (x.index, x.from.as_str(), x.to.as_str())).collect();
        assert_eq!(got, vec![(2, "a", "a_2"), (3, "a", "a_3"), (4, "b", "b_2")]);
    }

    #[test]
    fn suffix_skips_names_present_in_the_input() {
        let mut n = names(&["a", "a", "a_2", "a_2"]);
        resolve_duplicates(&mut n, DuplicateNames::Rename).unwrap();
        assert_eq!(n, names(&["a", "a_3", "a_2", "a_2_2"]));
    }

    #[test]
    fn error_policy_names_each_duplicate_once() {
        let mut n = names(&["a", "b", "a", "b", "a"]);
        let err = resolve_duplicates(&mut n, DuplicateNames::Error).unwrap_err().to_string();
        assert!(err.contains("2 names appear more than once"), "{err}");
        assert!(err.ends_with("a, b"), "{err}");
        assert_eq!(n, names(&["a", "b", "a", "b", "a"]));
    }
}
