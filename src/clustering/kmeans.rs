//! Implementation of Kmeans using Kmeans++

use crate::clustering::Clustering;
use ndarray::prelude::*;
use ndarray_stats::DeviationExt;
use rand::{distributions::weighted::WeightedIndex, distributions::Distribution, Rng};
use rayon::prelude::*;

/// Kmeans implementation
#[allow(missing_debug_implementations, missing_copy_implementations)]
pub struct KMeans;

fn kmeans_pp<R: Rng>(data: &Array2<f32>, clusters: usize, rng: &mut R) -> Vec<Array1<f32>> {
    let mut means: Vec<Array1<f32>> = Vec::with_capacity(clusters);
    let mut min_sq_dist = Array1::from_elem(data.nrows(), f32::INFINITY);
    let init_mean = rng.gen_range(0, data.nrows());
    means.push(data.row(init_mean).to_owned());
    while means.len() < clusters {
        let new_mean = means.last().unwrap();
        ndarray::Zip::from(data.axis_iter(Axis(0)))
            .and(&mut min_sq_dist)
            .par_apply(|v, msd| {
                let new_sd = v.sq_l2_dist(new_mean).unwrap();
                if new_sd < *msd {
                    *msd = new_sd;
                }
            });
        let index = WeightedIndex::new(&min_sq_dist).unwrap().sample(rng);
        //let index = min_sq_dist
        //    .iter()
        //    .enumerate()
        //    .max_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap_or(std::cmp::Ordering::Equal))
        //    .unwrap()
        //    .0;
        means.push(data.row(index).to_owned());
    }
    means
}

impl Clustering for KMeans {
    fn cluster<R: Rng>(vectors: &Array2<f32>, mut clusters: usize, rng: &mut R) -> Vec<usize> {
        let mut cluster_map = Array1::zeros(vectors.nrows());
        clusters = std::cmp::min(clusters, vectors.nrows());
        if clusters == 0 {
            return cluster_map.to_vec();
        }
        let mut means = kmeans_pp(&vectors, clusters, rng);
        let cols = vectors.ncols();
        let rows = vectors.nrows();
        for n in 0..20 {
            println!("Iter {}", n);
            ndarray::Zip::from(vectors.axis_iter(Axis(0)))
                .and(&mut cluster_map)
                .par_apply(|v, c| {
                    *c = means
                        .iter()
                        .enumerate()
                        .map(|(i, m)| (i, v.sq_l2_dist(m).unwrap()))
                        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .unwrap()
                        .0
                });
            //(0..clusters).for_each(|c| {
            //    if !cluster_map.contains(&c) {
            //        let index = rng.gen_range(0, cluster_map.len());
            //        cluster_map[index] = c;
            //    }
            //});
            means.par_iter_mut().enumerate().for_each(|(i, m)| {
                *m = ndarray::Zip::from(vectors.axis_iter(Axis(0)))
                    .and(&cluster_map)
                    .into_par_iter()
                    .filter(|(_, c)| **c == i)
                    .map(|(v, _)| v)
                    .fold(|| Array1::zeros(cols), |s, v| s + v)
                    .reduce(|| Array1::zeros(cols), |s, sp| s + sp)
                    / (rows as f32);
            });
            println!("{:?}", cluster_map);
        }
        cluster_map.to_vec()
    }
}
