use super::{Distance, Hyperplane, split_hyperplane};
use crate::scalar::Scalar;
use rand::rngs::ThreadRng;

#[derive(Debug, Clone, Copy, Default)]
pub struct Manhattan;

impl<T: Scalar> Distance<T> for Manhattan {
    const NAME: &'static str = "manhattan";
    type ValidationError = !;
    type Split = Hyperplane<T>;

    fn validate_split(split: &Self::Split, dimensions: usize) -> bool {
        split.is_valid(dimensions)
    }
    fn margin(split: &Self::Split, vector: &[T]) -> T {
        split.margin(vector)
    }
    fn distance(x: &[T], y: &[T], dimensions: usize) -> T {
        x.iter()
            .zip(y)
            .take(dimensions)
            .fold(T::zero(), |sum, (&x, &y)| sum + (x - y).abs())
    }
    fn normalized_distance(distance: f64) -> f64 {
        distance.max(0.0)
    }
    fn create_split(vectors: &[&[T]], rng: &mut ThreadRng) -> Self::Split {
        split_hyperplane::<T, Self>(vectors, rng, false)
    }
}
