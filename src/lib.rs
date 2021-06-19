use std::cmp::Ordering;

pub mod ann;
mod distance;

pub use distance::Euclidean;
//pub use distance::Hamming;

pub use ann::Annoy;

#[derive(PartialEq)]
struct Numeric<T: num::Num>(T);

impl<T: num::Num> Eq for Numeric<T> {}

impl<T: num::Num + PartialOrd> PartialOrd for Numeric<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Numeric<f64> {
    fn cmp(&self, other: &Numeric<f64>) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn random_flip() -> bool {
    rand::random::<bool>()
}
