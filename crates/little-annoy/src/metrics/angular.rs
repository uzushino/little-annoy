use super::{Distance, Hyperplane, split_hyperplane};
use crate::scalar::Scalar;
use rand::rngs::ThreadRng;

#[derive(Debug, Clone, Copy, Default)]
pub struct Angular;

impl<T: Scalar> Distance<T> for Angular {
    const NAME: &'static str = "angular";
    type ValidationError = !;
    type Split = Hyperplane<T>;

    fn validate_split(split: &Self::Split, dimensions: usize) -> bool {
        split.is_valid(dimensions)
    }
    fn margin(split: &Self::Split, vector: &[T]) -> T {
        split.margin(vector)
    }
    fn distance(x: &[T], y: &[T], dimensions: usize) -> T {
        let (mut xx, mut yy, mut xy) = (T::zero(), T::zero(), T::zero());
        for (&x, &y) in x.iter().zip(y).take(dimensions) {
            xx += x * x;
            yy += y * y;
            xy += x * y;
        }
        let two = T::one() + T::one();
        if xx * yy > T::zero() {
            two - two * xy / (xx * yy).sqrt()
        } else {
            two
        }
    }
    fn normalized_distance(distance: f64) -> f64 {
        distance.max(0.0).sqrt()
    }
    fn create_split(vectors: &[&[T]], rng: &mut ThreadRng) -> Self::Split {
        split_hyperplane::<T, Self>(vectors, rng, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parallel_vectors_have_zero_distance() {
        assert_eq!(Angular::distance(&[1.0, 2.0], &[2.0, 4.0], 2), 0.0);
    }
}
