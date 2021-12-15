use rand::Rng;
use rand::rngs::StdRng;
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

impl<T: item::Item + PartialOrd> PartialOrd for Numeric<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Numeric<f64> {
    fn cmp(&self, other: &Numeric<f64>) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn random_flip(rng: &mut StdRng) -> bool {
    rng.gen::<bool>()
}
