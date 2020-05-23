//! Configuration options

#[allow(unused_imports)]
use crate::{
    construct::{construct_hierarchial_weighed, construct_sentence_count},
    graph::Graph,
    input::Document,
};

/// Number of clusters in dataset
pub const CLUSTERS: usize = 32;
/// Number of dimensions to keep with PCA
pub const PCA_DIMS: usize = 163;
/// Minimum standard deviation to keep feature
pub const SIGMA_THRESHOLD: f32 = 0.4;
/// Minimum CV^-1 to keep feature
pub const CV_INV_THRESHOLD: f32 = 0.2;

/// Type used for graph edges
pub type EdgeType = f32;
/// Graph construction method
pub fn construct_method(d: &Document) -> Graph<EdgeType> {
    construct_hierarchial_weighed(d, [0.0, 1.0, 0.5, 0.0])
}
