//! Fact graph construction methods.
//!
//! This modules provides functions for constucting fact graphs by various algorithms.

use crate::{
    graph::{Graph, IndexMap},
    input::Document,
};

impl<T> Graph<T>
where
    T: std::ops::Add<Output = T> + Copy,
{
    /// Helper function for adding to an edge weight.
    ///
    /// Assumes that the graph contains the given verticies.
    ///
    /// # Panics
    ///
    /// Panics if the graph does not contain the verticies.
    fn add_weight(&mut self, t1: &str, t2: &str, weight: T) {
        let edge = self.get_mut(&t1, &t2).unwrap();
        *edge = match *edge {
            Some(v) => Some(v + weight),
            None => Some(weight),
        }
    }
}

fn build_language(document: &Document) -> IndexMap {
    document
        .iter()
        .map(|x| &**x)
        .flatten()
        .map(|x| &**x)
        .flatten()
        .map(|x| &***x)
        .collect()
}

/// Constructs a fact graph from a document, where edge weights are the sum of the values of each
/// term pairing in the document. The value of each paring depends on their shared tier of the
/// document heirarchy, and the values are given by the `weights` parameter.
///
/// The resulting graph is fully connected.
///
/// # Parameters
///
/// `weights` contains the weights to assign to term pairings in the same sentence, paragraph, or
/// document. It is in a array of the form [same sentence weight, same paragraph weight, same
/// document weight].
pub fn construct_hierarchial_weighed(document: &Document, weights: [f32; 4]) -> Graph<f32> {
    let [self_weight, sent_weight, para_weight, doc_weight] = weights;

    let mut graph = Graph::new(build_language(document));
    let mut doc_iter = document.iter();
    while let Some(paragraph) = doc_iter.next() {
        let mut par_iter = paragraph.iter();
        while let Some(sentence) = par_iter.next() {
            let mut sent_iter = sentence.iter();
            while let Some(term) = sent_iter.next() {
                graph.add_weight(term, term, self_weight);
                for t in sent_iter.clone() {
                    graph.add_weight(term, t, sent_weight);
                }
                for s in par_iter.clone() {
                    for t in s.iter() {
                        graph.add_weight(term, t, para_weight);
                    }
                }
                for p in doc_iter.clone() {
                    for s in p.iter() {
                        for t in s.iter() {
                            graph.add_weight(term, t, doc_weight);
                        }
                    }
                }
            }
        }
    }
    graph
}

/// Constructs a fact graph from a document, where edge weights are the number of times that term
/// pairing appears in sentences.
///
/// Pairings that do not occur are not connected.
///
/// Note that a sentence that contains mutliple instances of a pairing will count multiple times.
/// For example "cat dog dog" will add 2 to the "cat"-"dog" pairing.
pub fn construct_sentence_count(document: &Document) -> Graph<u32> {
    let mut graph = Graph::new(build_language(document));
    for paragraph in document.iter() {
        for sentence in paragraph.iter() {
            let mut sent_iter = sentence.iter();
            while let Some(term) = sent_iter.next() {
                graph.add_weight(term, term, 1);
                for t in sent_iter.clone() {
                    graph.add_weight(term, t, 1);
                }
            }
        }
    }
    graph
}

/// Constructs a fact graph from a document, where verticies are connected if the terms co-occured
/// in a sentence.
pub fn construct_sentence_link(document: &Document) -> Graph<()> {
    let mut graph = Graph::new(build_language(document));
    for paragraph in document.iter() {
        for sentence in paragraph.iter() {
            let mut sent_iter = sentence.iter();
            while let Some(term) = sent_iter.next() {
                for t in sent_iter.clone() {
                    *graph.get_mut(&term, &t).unwrap() = Some(());
                }
            }
        }
    }
    graph
}
