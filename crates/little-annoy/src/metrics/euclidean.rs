use super::{Distance, Hyperplane, split_hyperplane};
use crate::scalar::Scalar;
use rand::rngs::ThreadRng;

#[derive(Debug, Clone, Copy, Default)]
pub struct Euclidean;

impl<T: Scalar> Distance<T> for Euclidean {
    const NAME: &'static str = "euclidean";
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
            .fold(T::zero(), |sum, (&x, &y)| {
                let delta = x - y;
                sum + delta * delta
            })
    }
    fn normalized_distance(distance: f64) -> f64 {
        distance.max(0.0).sqrt()
    }
    fn create_split(vectors: &[&[T]], rng: &mut ThreadRng) -> Self::Split {
        split_hyperplane::<T, Self>(vectors, rng, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn squared_distance() {
        assert_eq!(Euclidean::distance(&[1.0, 2.0], &[2.0, 4.0], 2), 5.0);
    }
    #[test]
    fn positive_margin_selects_the_positive_side() {
        let plane = Hyperplane {
            normal: vec![2.0, 4.0],
            offset: 0.0,
        };
        assert!(Euclidean::side(&plane, &[1.0, 2.0], &mut rand::rng()));
    }
}
