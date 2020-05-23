#![feature(vec_remove_item)]
use fact_graph::{
    clustering::{kmeans_lib::KMeans, trim_features, vectorize, Clustering},
    config::{EdgeType, CLUSTERS, PCA_DIMS},
    graph::{self, IndexMap},
};
use rand::SeedableRng;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::{self, File},
    io::BufReader,
    process,
};

fn main() {
    type Graph = graph::Graph<EdgeType>;

    let names_to_clusters = |names: &[String]| {
        let map: IndexMap = names.iter().map(|n| n.split('-').next().unwrap()).collect();
        names
            .iter()
            .map(|n| map.get(n.split('-').next().unwrap()).unwrap())
            .collect::<Vec<usize>>()
    };

    let rng = &mut rand_pcg::Pcg64Mcg::from_entropy();

    match env::set_current_dir("workdir") {
        Ok(()) => (),
        Err(e) => error("Unable to enter workdir", e),
    }

    let files: Vec<_> = match fs::read_dir("graphs") {
        Err(e) => error("Unable to read graphs directory", e),
        Ok(dir_iter) => match dir_iter.collect() {
            Ok(files) => files,
            Err(e) => error("Error while reading graphs directory", e),
        },
    };

    let (names, graphs): (Vec<String>, Vec<_>) = files
        .par_iter()
        .map(|file| {
            let reader = BufReader::new(match File::open(file.path()) {
                Ok(f) => f,
                Err(e) => error("Error opening file", e),
            });
            let graph: Graph = match serde_json::from_reader(reader) {
                Ok(d) => d,
                Err(e) => error("Error parsing file", e),
            };
            (file.file_name().into_string().unwrap(), graph)
        })
        .unzip();

    let vectorized = vectorize(&graphs);
    drop(graphs);
    let trimmed = trim_features(&vectorized);
    drop(vectorized);

    let mut pca = petal_decomposition::Pca::new(PCA_DIMS);
    let reduced = pca.fit_transform(&trimmed).unwrap();

    let clusters = KMeans::cluster(&reduced, CLUSTERS, rng);

    names.iter().zip(&clusters).for_each(|(n, c)| {
        println!("{}: {}", n, c);
    });

    let truth: Vec<usize> = names_to_clusters(&names);

    let pred = &clusters[..];
    let truth = &truth[..];
    let num_clusters = truth.iter().max().unwrap() + 1;
    let mut pred_map = vec![0; num_clusters];

    let mut remaining_p: Vec<_> = (0..num_clusters).collect();
    let mut remaining_t: Vec<_> = (0..num_clusters).collect();
    while remaining_p.len() > 0 {
        let (p, tr_match, _): (usize, usize, usize) = remaining_p
            .iter()
            .map(|p| {
                let mut counts = HashMap::new();
                pred.iter()
                    .zip(truth)
                    .filter(|(pr, _)| *pr == p)
                    .filter(|(_, tr)| remaining_t.contains(tr))
                    .for_each(|(_, tr)| {
                        *counts.entry(tr).or_insert(0) += 1;
                    });
                let (tr, count): (usize, usize) = counts
                    .iter()
                    .map(|(&&tr, &c)| (tr, c))
                    .max_by_key(|(_, count)| (*count).clone())
                    .unwrap_or_else(|| (remaining_t[0], 0));
                (*p, tr, count)
            })
            .max_by_key(|(_, _, count)| (*count).clone())
            .expect("1");

        pred_map[p] = tr_match;
        remaining_p.remove_item(&p);
        remaining_t.remove_item(&tr_match);
    }
    let pred: Vec<_> = pred.iter().map(|p| pred_map[*p]).collect();

    let outfile = match File::create("names") {
        Ok(f) => f,
        Err(e) => error("Unable to create output file", e),
    };
    match serde_json::to_writer(outfile, &names) {
        Ok(()) => (),
        Err(e) => error("Failed to serialize data.", e),
    }

    let outfile = match File::create("true") {
        Ok(f) => f,
        Err(e) => error("Unable to create output file", e),
    };
    match serde_json::to_writer(outfile, &truth) {
        Ok(()) => (),
        Err(e) => error("Failed to serialize data.", e),
    }

    let outfile = match File::create("pred") {
        Ok(f) => f,
        Err(e) => error("Unable to create output file", e),
    };
    match serde_json::to_writer(outfile, &pred) {
        Ok(()) => (),
        Err(e) => error("Failed to serialize data.", e),
    }
}

fn error(message: &str, err: impl Error) -> ! {
    eprintln!("{}: {}", message, err);
    process::exit(1);
}
