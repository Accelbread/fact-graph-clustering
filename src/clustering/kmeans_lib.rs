//! Kmeans using external library.

use crate::clustering::Clustering;
use ndarray::prelude::*;
use rand::Rng;

/// Kmeans using external library.
#[allow(missing_debug_implementations, missing_copy_implementations)]
pub struct KMeans;

impl Clustering for KMeans {
    fn cluster<R: Rng>(data: &Array2<f32>, clusters: usize, _rng: &mut R) -> Vec<usize> {
        let (_, clusters) = rkm::kmeans_lloyd(&data.view(), clusters);
        clusters
    }
}
