//! Graph implementation.
//!
//! This module re-exports the chosen graph implmentation from a submodule for use by the rest of the crate.

mod adj_matrix;
mod index_trie;
pub mod lower_triangular;

#[cfg(feature = "adj_matrix")]
pub use adj_matrix::AMGraph as Graph;

pub use index_trie::IndexTrie as IndexMap;
