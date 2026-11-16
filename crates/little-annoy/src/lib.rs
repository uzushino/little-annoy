#![doc = include_str!("../README.md")]

mod build;
mod index;
mod metrics;
mod node;
mod persistence;
mod scalar;
mod search;

pub use index::Annoy;
pub use metrics::{Angular, Distance, Euclidean, Hamming, Manhattan};
pub use scalar::Scalar;
