//! Module containing functions needed for the clustering process and implementations of
//! clustering algorithms.
pub mod kmeans;
pub mod kmeans_lib;

use crate::{
    config::{CV_INV_THRESHOLD, SIGMA_THRESHOLD},
    graph::{Graph, IndexMap},
};
use ndarray::prelude::*;
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashMap;

/// Trait for conversions from edge type to f32
pub trait Value: Sync {
    /// converts an edge value to f32
    fn value(&self) -> f32;
}

impl Value for f32 {
    fn value(&self) -> f32 {
        *self
    }
}

impl Value for () {
    fn value(&self) -> f32 {
        1.0
    }
}

impl Value for u32 {
    fn value(&self) -> f32 {
        *self as f32
    }
}

/// Trait for clusting algorithms.
///
/// Use this trait to be generic over the clustering algorithm used.
pub trait Clustering {
    /// Takes a feature matrix and returns a clustering of it.
    fn cluster<R: Rng>(data: &Array2<f32>, clusters: usize, rng: &mut R) -> Vec<usize>;
}

/// Converts graphs into a feature matrix.
pub fn vectorize<T: Value>(graphs: &[Graph<T>]) -> Array2<f32> {
    let language: IndexMap = graphs
        .iter()
        .map(|g| g.vertices())
        .flatten()
        .fold(HashMap::new(), |mut acc, w| {
            *acc.entry(w).or_insert(0) += 1;
            acc
        })
        .into_iter()
        .filter(|&(_, v)| v > 3)
        .map(|(k, _): (String, _)| k)
        .collect();
    let dim = language.len();
    let len = (dim * (dim + 1)) / 2;
    let n = graphs.len();
    let mut res = Array2::zeros((n, len));
    res.axis_iter_mut(Axis(0))
        .into_par_iter()
        .zip(graphs)
        .for_each(|(mut row, g)| {
            g.edges().for_each(|(v1, v2, e)| {
                let v1 = language.get(&*v1);
                let v2 = language.get(&*v2);
                if let (Some(v1), Some(v2)) = (v1, v2) {
                    row[term_indices_to_edge_index(v1, v2)] = e.value();
                }
            });
        });
    res
}

fn term_indices_to_edge_index(i1: usize, i2: usize) -> usize {
    let row = std::cmp::max(i1, i2);
    let col = std::cmp::min(i1, i2);
    (row * (row + 1)) / 2 + col
}

/// Applies statistcal feature reduction methods.
pub fn trim_features(data: &Array2<f32>) -> Array2<f32> {
    let means = data.mean_axis(Axis(0)).unwrap();
    let stds = data.std_axis(Axis(0), 1.0);
    let mut mask = Array1::<usize>::zeros(means.raw_dim());
    let mut count = 1;
    for i in 0..means.len() {
        if stds[i] < SIGMA_THRESHOLD || means[i] / stds[i] < CV_INV_THRESHOLD {
            continue;
        }
        mask[i] = count;
        count += 1;
    }
    let f = mask.iter().filter(|&&e| e != 0).count();
    let mut res = Array2::<f32>::zeros((data.nrows(), f));
    for (i, m) in mask.iter().enumerate() {
        if *m > 0 {
            let mut slice = res.slice_mut(s![.., *m - 1]);
            slice.assign(&data.column(i));
        }
    }
    res
}
