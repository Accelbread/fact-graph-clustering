//! An indexable trie, suitable for use as a bijective map.
//!
//! The module contains the `IndexTrie` type.

use serde::{Deserialize, Serialize};
use std::{
    cmp::{min, Ordering},
    iter::{once, FromIterator, FusedIterator},
};

/// An indexable trie, suitable for use as a bijective map.
///
/// The structure can be queried with a string to get index of that string in the set of contained
/// strings in sorted order. Additionally, the structure can be queried with an index to get the
/// corresponding string. The trie uses length metadata on each node to provide an efficient means
/// of indexing it. Each prefix node stores the largest common prefix of its children, excluding
/// prefixes of parent nodes.
///
/// As the strings are indexed in sorted order, adding a new string to the trie will increment the
/// indexes corresponding to prior strings that are lexiographically greater.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct IndexTrie {
    // As the root node doesn't have a prefix associated, we just store the top level prefix nodes.
    // No two children can start with the same character.
    roots: Vec<Node>,
    len: usize,
}

/// Represents a non-root node of the trie.
///
/// Segments of the input are stored as Vec<u8>, allowing us to ignore unicode and split across
/// unicode character boundaries. This is safe as all valid sequences of segements were from
/// inputs, and thus will be valid Strings when re-joined.
///
/// `NonLeaf` represents a prefix node, which has multiple children and a non-empty prefix. `len`
/// is the number of leaf nodes in the subtree of which that node is the root.
///
/// `Leaf` represents a leaf node, which contains the rest of the string after the sum of the
/// prefixes from it's parent nodes. It implicitly has a length of 1.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum Node {
    NonLeaf {
        prefix: Vec<u8>,
        children: Vec<Node>,
        len: usize,
    },
    Leaf {
        rest: Vec<u8>,
    },
}

impl IndexTrie {
    /// Creates a new empty IndexTrie.
    pub fn new() -> Self {
        IndexTrie {
            roots: Vec::new(),
            len: 0,
        }
    }

    /// Returns the number of strings contained.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the trie is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the corresponding value when given a string (`&str`) or index (`usize`).
    ///
    /// Returns None if the string is not in the trie or if the index is out of bounds.
    pub fn get<K: Key>(&self, k: K) -> Option<K::Value> {
        k.get(self)
    }

    /// Inserts a string into the trie.
    ///
    /// Returns true if the string was added, and false if the string was already in the trie.
    pub fn insert(&mut self, k: &str) -> bool {
        fn insert_inner(vec: &mut Vec<Node>, k: &[u8]) -> bool {
            // If k is empty, we need an empty leaf, which will be the first element of the vector.
            if k.is_empty() {
                // If the first element is an empty leaf, the string was already contained.
                if let Some(Node::Leaf { rest }) = vec.get(0) {
                    if rest.is_empty() {
                        return false;
                    }
                }
                vec.insert(0, Node::Leaf { rest: Vec::new() });
                return true;
            }
            for (i, n) in vec.iter_mut().enumerate() {
                match n {
                    Node::Leaf { rest } => {
                        // `k` is non-empty, so does not match empty leaves.
                        // This check lets us index `rest`.
                        if rest.is_empty() {
                            continue;
                        }
                        match k[0].cmp(&rest[0]) {
                            // `k` did not match, and comes afterwards.
                            Ordering::Greater => (),
                            // `k` shares a common prefix with this leaf.
                            Ordering::Equal => {
                                let c_len = common_prefix(k, rest);
                                // If we need to split the node, we need the order of the new
                                // children leaf nodes.
                                let (first, second): (&[u8], &[u8]) =
                                    match (k.len() == c_len, rest.len() == c_len) {
                                        // `k` is equal to `rest`
                                        (true, true) => {
                                            return false;
                                        }
                                        // `k` is the common prefix, and will add an empty leaf.
                                        (true, false) => (&[], &rest[c_len..]),
                                        // `rest` is the common prefix, and will add an empty leaf.
                                        (false, true) => (&[], &k[c_len..]),
                                        // Both will be non empty leaves, so need to determine
                                        // which is lexiograpically first by comparing first
                                        // non-equal character.
                                        (false, false) => {
                                            if k[c_len] < rest[c_len] {
                                                (&k[c_len..], &rest[c_len..])
                                            } else {
                                                (&rest[c_len..], &k[c_len..])
                                            }
                                        }
                                    };
                                // We need to replace the current node with a new non-leaf, and add
                                // the new leaves.
                                *n = Node::NonLeaf {
                                    #[rustfmt::skip]
                                    children: vec![
                                        Node::Leaf { rest: first.into() },
                                        Node::Leaf { rest: second.into() },
                                    ],
                                    len: 2,
                                    prefix: {
                                        // We use the same `Vec` as the old leaf to avoid having to
                                        // reallocate a buffer and copy the prefix, as that `Vec`
                                        // is otherwise no longer needed, and can be truncated to
                                        // the nessesary prefix.
                                        let mut p = std::mem::take(rest);
                                        p.truncate(c_len);
                                        p
                                    },
                                };
                                return true;
                            }
                            // k goes prior to this node.
                            // As this is the first greater node, we insert into this position and
                            // return.
                            Ordering::Less => {
                                vec.insert(i, Node::Leaf { rest: k.into() });
                                return true;
                            }
                        }
                    }
                    Node::NonLeaf {
                        prefix,
                        children,
                        len,
                    } => match k[0].cmp(&prefix[0]) {
                        // As sibling nodes will not have a common first character, we just compare
                        // the first character to see if the node is of interest.
                        // `k` comes afterwards.
                        Ordering::Greater => (),
                        // `k` will either be a child of this node or split it.
                        Ordering::Equal => {
                            let c_len = common_prefix(k, prefix);
                            // `k` is a child of this node.
                            if prefix.len() == c_len {
                                let ret = insert_inner(children, &k[c_len..]);
                                if ret {
                                    // `k` was new, so this nodes length is increased.
                                    *len += 1;
                                }
                                return ret;
                            }
                            // We need to split the node. We will create a new leaf and nonleaf
                            // node as children. The old children will be the children of the new
                            // nonleaf node.
                            let leaf = Node::Leaf {
                                rest: k[c_len..].into(),
                            };
                            let nonleaf = Node::NonLeaf {
                                prefix: prefix[c_len..].into(),
                                // Gives the children to the new node. Replaces current nodes's
                                // `children` `Vec` with a new empty one.
                                children: std::mem::take(children),
                                len: *len,
                            };
                            // Add the new child nodes in the correct order.
                            if k.len() == c_len || k[c_len] < prefix[c_len] {
                                children.push(leaf);
                                children.push(nonleaf);
                            } else {
                                children.push(nonleaf);
                                children.push(leaf);
                            };
                            // Fix up the current node.
                            prefix.truncate(c_len);
                            *len += 1;
                            return true;
                        }
                        // k goes prior to this node.
                        // As this is the first greater node, we insert into this position and
                        // return.
                        Ordering::Less => {
                            vec.insert(i, Node::Leaf { rest: k.into() });
                            return true;
                        }
                    },
                }
            }
            // All nodes were less than `k`, so new leaf is at the end of the `Vec`.
            vec.push(Node::Leaf { rest: k.into() });
            true
        }

        let ret = insert_inner(&mut self.roots, k.as_bytes());
        if ret {
            self.len += 1;
        }
        ret
    }
}

impl<'a> Extend<&'a str> for IndexTrie {
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        for s in iter {
            self.insert(s);
        }
    }
}

impl Extend<String> for IndexTrie {
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        for s in iter {
            self.insert(&s);
        }
    }
}

impl<'a> FromIterator<&'a str> for IndexTrie {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut t = IndexTrie::new();
        for s in iter {
            t.insert(s);
        }
        t
    }
}

impl<'a> FromIterator<String> for IndexTrie {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let mut t = IndexTrie::new();
        for s in iter {
            t.insert(&s);
        }
        t
    }
}

impl<'a> IntoIterator for &'a IndexTrie {
    type Item = String;
    type IntoIter = IndexTrieIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IndexTrieIterator {
            state: vec![IterItem::Root(self, 0)],
        }
    }
}

impl Node {
    fn len(&self) -> usize {
        match self {
            Node::NonLeaf {
                prefix: _,
                children: _,
                len,
            } => *len,
            Node::Leaf { rest: _ } => 1,
        }
    }
}

/// This trait represents a type that can be used to get a value from an `IndexTrie`.
pub trait Key {
    /// The type that will get returned by the `IndexTrie` when using this type as a key.
    type Value;
    /// Returns the corrsponding value from the given `IndexTrie`.
    ///
    /// For more info, see `IndexTrie::get()`. That method should be used instead.
    fn get(self, trie: &IndexTrie) -> Option<Self::Value>;
}

impl Key for &str {
    type Value = usize;

    fn get(self, trie: &IndexTrie) -> Option<Self::Value> {
        // We start `i` at 0 and add to it until we get to the target node.
        fn get_inner(nodes: &[Node], k: &[u8], mut i: usize) -> Option<usize> {
            for n in nodes {
                match n {
                    Node::Leaf { rest } => match k.cmp(rest) {
                        Ordering::Greater => i += 1,
                        Ordering::Equal => return Some(i),
                        Ordering::Less => return None,
                    },
                    Node::NonLeaf {
                        prefix,
                        children,
                        len,
                    } => match k[..min(k.len(), prefix.len())].cmp(prefix) {
                        Ordering::Greater => i += len,
                        Ordering::Equal => return get_inner(children, &k[prefix.len()..], i),
                        Ordering::Less => return None,
                    },
                }
            }
            None
        }

        get_inner(&trie.roots, self.as_bytes(), 0)
    }
}

impl Key for usize {
    type Value = String;

    fn get(self, trie: &IndexTrie) -> Option<Self::Value> {
        // As we encounter parent nodes and the final leaf, we add their contents to `buf`, the
        // resulting string. We reduce `i` as we skip over non-matching nodes. It is thus the
        // number of remaining leaf nodes to skip over.
        fn get_inner(nodes: &[Node], mut i: usize, buf: &mut Vec<u8>) -> bool {
            for n in nodes {
                match n {
                    Node::Leaf { rest } => match i.cmp(&0) {
                        Ordering::Greater => i -= 1,
                        Ordering::Equal => {
                            buf.extend(rest);
                            return true;
                        }
                        // `i` is unsigned, and can't be less than 0.
                        Ordering::Less => unreachable!(),
                    },
                    Node::NonLeaf {
                        prefix,
                        children,
                        len,
                    } => {
                        if i >= *len {
                            i -= len;
                        } else {
                            buf.extend(prefix);
                            return get_inner(children, i, buf);
                        }
                    }
                }
            }
            // Index out of bounds
            false
        }

        let mut res = Vec::new();
        if get_inner(&trie.roots, self, &mut res) {
            // Since the buffer is a reconstructed input string, it will be valid utf8.
            Some(unsafe { String::from_utf8_unchecked(res) })
        } else {
            None
        }
    }
}

/// Returns the length of the longest common prefix shared by the given strings.
fn common_prefix(v1: &[u8], v2: &[u8]) -> usize {
    for (i, (c1, c2)) in v1.iter().zip(v2.iter()).enumerate() {
        if c1 != c2 {
            return i;
        }
    }
    // As the shorter string fully matched, its length is the result.
    min(v1.len(), v2.len())
}

/// An iterator over an `IndexTrie`.
#[derive(Clone, Debug)]
pub struct IndexTrieIterator<'a> {
    state: Vec<IterItem<'a>>,
}

#[derive(Clone, Debug)]
enum IterItem<'a> {
    Root(&'a IndexTrie, usize),
    Prefix(&'a Node, usize),
}

impl Iterator for IndexTrieIterator<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        // Get current node list and index.
        let (cur, i): (&[Node], &mut usize) = match self.state.last_mut() {
            // No state left; we've exhusted the root node.
            None => return None,
            Some(item) => match item {
                IterItem::Root(IndexTrie { roots, len: _ }, i) => (roots, i),
                IterItem::Prefix(
                    Node::NonLeaf {
                        prefix: _,
                        children,
                        len: _,
                    },
                    i,
                ) => (children, i),
                IterItem::Prefix(Node::Leaf { rest: _ }, _) => unreachable!(),
            },
        };
        match cur.get(*i) {
            // This node list has run out, return to the previous level.
            None => {
                self.state.pop();
                self.next()
            }
            Some(n) => {
                // Increment index for next visit to the list.
                *i += 1;
                match n {
                    Node::NonLeaf {
                        prefix: _,
                        children: _,
                        len: _,
                    } => {
                        // Decend into node.
                        self.state.push(IterItem::Prefix(n, 0));
                        self.next()
                    }
                    Node::Leaf { rest } => {
                        // Reconstruct string from parent node prefixes
                        let res: Vec<u8> = self
                            .state
                            .iter()
                            // First element is root node.
                            .skip(1)
                            .map(|x| match x {
                                IterItem::Prefix(
                                    Node::NonLeaf {
                                        prefix,
                                        children: _,
                                        len: _,
                                    },
                                    _,
                                ) => prefix,
                                // Only first element can be root, and none are leaves.
                                _ => unreachable!(),
                            })
                            .chain(once(rest))
                            .flatten()
                            .cloned()
                            .collect();
                        // Since `res` is a reconstructed input string, it will be valid utf8.
                        Some(unsafe { String::from_utf8_unchecked(res) })
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem: usize = self
            .state
            .clone()
            .iter()
            // Map each node in state to the children not yet traversed.
            .map(|x| match x {
                IterItem::Root(IndexTrie { roots, len: _ }, i) => &roots[*i..],
                IterItem::Prefix(
                    Node::NonLeaf {
                        prefix: _,
                        children,
                        len: _,
                    },
                    i,
                ) => &children[*i..],
                IterItem::Prefix(Node::Leaf { rest: _ }, _) => unreachable!(),
            })
            .flatten()
            .map(Node::len)
            .sum();
        (rem, Some(rem))
    }
}

impl FusedIterator for IndexTrieIterator<'_> {}

impl ExactSizeIterator for IndexTrieIterator<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    const STRINGS: [&str; 7] = ["aaa", "aaa", "aaaaa", "aaaab", "aabb", "aacb", "aacee"];

    fn test_trie() -> IndexTrie {
        let mut t = IndexTrie::new();
        for s in &STRINGS {
            t.insert(s);
        }
        t
    }

    fn expected_contents() -> Vec<&'static str> {
        let mut entries = STRINGS.to_vec();
        entries.sort();
        entries.dedup();
        entries
    }

    #[test]
    fn len() {
        let t = test_trie();
        let c = expected_contents();
        assert_eq!(t.len(), c.len());
    }

    #[test]
    fn get_by_str() {
        let t = test_trie();
        let c = expected_contents();
        for (i, &s) in c.iter().enumerate() {
            assert_eq!(t.get(s), Some(i));
        }
    }

    #[test]
    fn get_by_index() {
        let t = test_trie();
        let c = expected_contents();
        for (i, &s) in c.iter().enumerate() {
            assert_eq!(t.get(i), Some(s.to_string()));
        }
    }

    #[test]
    fn iter() {
        let t = test_trie();
        let c = expected_contents();
        assert_eq!(t.into_iter().eq(c.iter().map(|s| s.to_string())), true);
    }

    #[test]
    fn iter_len() {
        let t = test_trie();
        let len = t.len();
        let mut iter = t.into_iter();
        for i in 0..len {
            assert_eq!(iter.len(), len - i);
            iter.next();
        }
    }
}
