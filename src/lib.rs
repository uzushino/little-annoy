use std::cmp::Ordering;

mod node;
mod distance;
pub mod ann;

pub use ann::Annoy;

#[derive(PartialEq)]
struct Numeric(f64);

impl Eq for Numeric {}

impl PartialOrd for Numeric {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Numeric {
    fn cmp(&self, other: &Numeric) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn random_flip() -> bool {
    rand::random::<bool>()
}
