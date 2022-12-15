use rand::Rng;
use std::cmp::Ordering;

pub mod ann;
mod distance;
mod item;

pub use distance::Angular;
pub use distance::Euclidean;
pub use distance::Hamming;
pub use distance::Manhattan;

pub use ann::Annoy;

#[derive(PartialEq)]
struct Numeric<T: item::Item>(T);

impl<T: item::Item> Eq for Numeric<T> {}

impl<T: item::Item> PartialOrd for Numeric<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: item::Item> Ord for Numeric<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}
