//! Rooted binary trees produced by neighbor joining, and Newick output.

use std::io::Write;

/// Sentinel for "no length assigned" (the root).
const NO_LENGTH: f32 = f32::MAX;

/// A node in a [`Tree`]. Leaves carry a name; internal nodes carry children.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Left child index, or `None` for a leaf.
    pub left: Option<u32>,
    /// Right child index, or `None` for a leaf.
    pub right: Option<u32>,
    /// Leaf label; empty for internal nodes.
    pub name: String,
    /// Length of the branch leading to this node from its parent.
    /// `f32::MAX` for the root.
    pub length: f32,
}

impl Node {
    /// True when this node has no children.
    pub fn is_leaf(&self) -> bool {
        self.left.is_none()
    }

    /// Branch length, or `None` at the root.
    pub fn branch_length(&self) -> Option<f32> {
        if self.length == NO_LENGTH {
            None
        } else {
            Some(self.length)
        }
    }
}

/// A binary tree over `k` leaves stored as an arena of `2k - 1` nodes.
///
/// Leaves occupy indices `0..k` in input order; internal nodes follow in the
/// order they were created, so the root is always the last node.
#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    /// A forest of `k` unconnected leaves. Neighbor joining fills in the
    /// internal nodes with [`join`](Tree::join).
    pub fn leaves(names: &[String]) -> Self {
        let k = names.len();
        let mut nodes = Vec::with_capacity(2 * k.max(1) - 1);
        for n in names {
            nodes.push(Node { left: None, right: None, name: n.clone(), length: NO_LENGTH });
        }
        for _ in k..(2 * k).saturating_sub(1) {
            nodes.push(Node { left: None, right: None, name: String::new(), length: NO_LENGTH });
        }
        Tree { nodes }
    }

    /// All nodes, leaves first.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Mutable access to a node.
    pub fn node_mut(&mut self, i: usize) -> &mut Node {
        &mut self.nodes[i]
    }

    /// Number of leaves.
    pub fn num_leaves(&self) -> usize {
        self.nodes.len().div_ceil(2)
    }

    /// Index of the root node.
    pub fn root(&self) -> usize {
        self.nodes.len() - 1
    }

    /// Make `parent` the parent of `left` and `right`, assigning the two
    /// branch lengths.
    pub fn join(&mut self, parent: usize, left: usize, right: usize, len_l: f32, len_r: f32) {
        self.nodes[parent].left = Some(left as u32);
        self.nodes[parent].right = Some(right as u32);
        self.nodes[left].length = len_l;
        self.nodes[right].length = len_r;
    }

    /// Serialise as Newick with five-decimal branch lengths, ending in `;\n`.
    pub fn to_newick(&self) -> String {
        let mut out = Vec::with_capacity(self.nodes.len() * 16);
        self.write_newick(&mut out).expect("writing to a Vec cannot fail");
        String::from_utf8(out).expect("Newick output is ASCII apart from names")
    }

    /// Write Newick to a sink. Uses an explicit stack, so very unbalanced
    /// trees (hundreds of thousands of leaves) do not overflow the call stack.
    pub fn write_newick<W: Write + ?Sized>(&self, w: &mut W) -> std::io::Result<()> {
        enum Step {
            Enter(usize),
            Comma,
            Close(usize),
        }
        let mut buf = String::new();
        let mut stack = vec![Step::Enter(self.root())];
        while let Some(step) = stack.pop() {
            match step {
                Step::Enter(i) => {
                    let n = &self.nodes[i];
                    match (n.left, n.right) {
                        (Some(l), Some(r)) => {
                            buf.push('(');
                            stack.push(Step::Close(i));
                            stack.push(Step::Enter(r as usize));
                            stack.push(Step::Comma);
                            stack.push(Step::Enter(l as usize));
                        }
                        _ => {
                            buf.push_str(&n.name);
                            push_length(&mut buf, n.length);
                        }
                    }
                }
                Step::Comma => buf.push(','),
                Step::Close(i) => {
                    buf.push(')');
                    push_length(&mut buf, self.nodes[i].length);
                }
            }
            if buf.len() > 1 << 16 {
                w.write_all(buf.as_bytes())?;
                buf.clear();
            }
        }
        buf.push_str(";\n");
        w.write_all(buf.as_bytes())
    }
}

fn push_length(buf: &mut String, len: f32) {
    if len != NO_LENGTH {
        buf.push(':');
        buf.push_str(&format_len5(len));
    }
}

/// Format a branch length with five decimals, rounding half away from zero
/// on the exact decimal expansion (the reference used Java's `%.5f`, which
/// rounds HALF_UP; Rust's formatter rounds half to even).
pub fn format_len5(x: f32) -> String {
    // f32 -> f64 is exact; printing with 12 decimals is well beyond the
    // point where a tie at the sixth decimal can be affected, since a tie is
    // a dyadic rational with a terminating expansion.
    let s = format!("{:.12}", x as f64);
    let neg = s.starts_with('-');
    let digits: &str = if neg { &s[1..] } else { &s };
    let (int_part, frac) = digits.split_once('.').expect("fixed formatting has a point");
    let keep = &frac[..5];
    let next = frac.as_bytes()[5];
    let mut result: Vec<u8> = format!("{}{}", int_part, keep).into_bytes();
    if next >= b'5' {
        // Increment the decimal string.
        let mut i = result.len();
        loop {
            if i == 0 {
                result.insert(0, b'1');
                break;
            }
            i -= 1;
            if result[i] == b'9' {
                result[i] = b'0';
            } else {
                result[i] += 1;
                break;
            }
        }
    }
    let n = result.len();
    let mut out = String::with_capacity(n + 2);
    if neg {
        out.push('-');
    }
    out.push_str(std::str::from_utf8(&result[..n - 5]).unwrap());
    out.push('.');
    out.push_str(std::str::from_utf8(&result[n - 5..]).unwrap());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newick_of_small_tree() {
        let names: Vec<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        let mut t = Tree::leaves(&names);
        t.join(3, 0, 1, 0.1, 0.2);
        t.join(4, 3, 2, 0.05, 0.3);
        assert_eq!(t.to_newick(), "((a:0.10000,b:0.20000):0.05000,c:0.30000);\n");
    }

    #[test]
    fn single_leaf() {
        let t = Tree::leaves(&["only".to_string()]);
        assert_eq!(t.to_newick(), "only;\n");
    }

    #[test]
    fn length_formatting_rounds_half_up() {
        assert_eq!(format_len5(0.015625), "0.01563"); // exact tie in binary
        assert_eq!(format_len5(0.1), "0.10000");
        assert_eq!(format_len5(1.0), "1.00000");
        assert_eq!(format_len5(0.999996), "1.00000");
        assert_eq!(format_len5(0.0), "0.00000");
        assert_eq!(format_len5(-0.0000001), "-0.00000");
        assert_eq!(format_len5(12.345678), "12.34568");
    }

    #[test]
    fn deep_tree_does_not_overflow_stack() {
        let k = 200_000;
        let names: Vec<String> = (0..k).map(|i| format!("t{}", i)).collect();
        let mut t = Tree::leaves(&names);
        // Pectinate: ((((t0,t1),t2),t3),...)
        let mut prev = 0;
        for i in 1..k {
            let parent = k + i - 1;
            t.join(parent, prev, i, 0.01, 0.02);
            prev = parent;
        }
        let s = t.to_newick();
        assert!(s.starts_with("((((("));
        assert!(s.ends_with(";\n"));
    }
}
