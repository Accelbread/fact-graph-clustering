//! fact_graph
//!
//! Code for the fact graph research project.
//!
//! Provides tools for clustering documents using graph methods.

#![feature(vec_remove_item)]

#![warn(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_numeric_casts,
    unreachable_pub,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]

#[macro_use]
mod macros;

pub mod clustering;
pub mod construct;
pub mod graph;
pub mod input;
pub mod config;
