use crate::scalar::Scalar;
use rand::{RngExt, rngs::ThreadRng};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

mod angular;
mod euclidean;
mod hamming;
mod manhattan;

pub use angular::Angular;
pub use euclidean::Euclidean;
pub use hamming::Hamming;
pub use manhattan::Manhattan;

#[derive(Debug, Serialize, Deserialize)]
pub struct Hyperplane<T> {
    normal: Vec<T>,
    offset: T,
}

impl<T: Scalar> Hyperplane<T> {
    fn margin(&self, vector: &[T]) -> T {
        self.normal
            .iter()
            .zip(vector)
            .fold(self.offset, |sum, (&x, &y)| sum + x * y)
    }

    fn is_valid(&self, dimensions: usize) -> bool {
        self.normal.len() == dimensions
            && self.offset.to_f64().is_some_and(f64::is_finite)
            && self
                .normal
                .iter()
                .all(|value| value.to_f64().is_some_and(f64::is_finite))
    }
}

/// Distance and random split strategy used by an index.
///
/// `distance` produces the ranking value; `normalized_distance` converts it to
/// the public distance. `Split` stores only the metric's separating plane.
pub trait Distance<T: Scalar> {
    const NAME: &'static str;
    type Split: Send + Sync + Serialize + DeserializeOwned;
    /// `!` means a metric has no additional value restrictions.
    type ValidationError: std::fmt::Display;

    fn validate_values(_values: &[T]) -> Result<(), Self::ValidationError> {
        Ok(())
    }
    fn validate_split(split: &Self::Split, dimensions: usize) -> bool;
    fn distance(x: &[T], y: &[T], dimensions: usize) -> T;
    fn create_split(vectors: &[&[T]], rng: &mut ThreadRng) -> Self::Split;
    fn margin(split: &Self::Split, vector: &[T]) -> T;
    fn normalized_distance(distance: f64) -> f64;

    fn side(split: &Self::Split, vector: &[T], rng: &mut ThreadRng) -> bool {
        let margin = Self::margin(split, vector);
        if margin == T::zero() {
            rng.random_bool(0.5)
        } else {
            margin > T::zero()
        }
    }
}

fn normalize<T: Scalar>(vector: &[T]) -> Vec<T> {
    let norm = vector.iter().fold(T::zero(), |sum, &x| sum + x * x).sqrt();
    if norm == T::zero() {
        return vector.to_vec();
    }
    vector.iter().map(|&value| value / norm).collect()
}

fn two_means<T: Scalar, D: Distance<T>>(rng: &mut ThreadRng, vectors: &[&[T]]) -> (Vec<T>, Vec<T>) {
    let count = vectors.len();
    let first = rng.random_range(0..count);
    let mut second = rng.random_range(0..count - 1);
    second += usize::from(second >= first);
    let mut left = vectors[first].to_vec();
    let mut right = vectors[second].to_vec();
    let mut left_count = T::one();
    let mut right_count = T::one();
    for _ in 0..200 {
        let vector = vectors[rng.random_range(0..count)];
        let left_distance = left_count * D::distance(&left, vector, vector.len());
        let right_distance = right_count * D::distance(&right, vector, vector.len());
        let (center, weight) = if left_distance < right_distance {
            (&mut left, &mut left_count)
        } else if right_distance < left_distance {
            (&mut right, &mut right_count)
        } else {
            continue;
        };
        for (value, &coordinate) in center.iter_mut().zip(vector) {
            *value = (*value * *weight + coordinate) / (*weight + T::one());
        }
        *weight += T::one();
    }
    (left, right)
}

fn split_hyperplane<T: Scalar, D: Distance<T>>(
    vectors: &[&[T]],
    rng: &mut ThreadRng,
    through_origin: bool,
) -> Hyperplane<T> {
    let (left, right) = two_means::<T, D>(rng, vectors);
    let normal = normalize(
        &left
            .iter()
            .zip(&right)
            .map(|(&x, &y)| x - y)
            .collect::<Vec<_>>(),
    );
    let offset = if through_origin {
        T::zero()
    } else {
        normal
            .iter()
            .zip(left.iter().zip(&right))
            .fold(T::zero(), |sum, (&normal, (&x, &y))| {
                sum - normal * (x + y) / (T::one() + T::one())
            })
    };
    Hyperplane { normal, offset }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continuous_metrics_have_infallible_value_validation() {
        let result: Result<(), !> = Euclidean::validate_values(&[1.0_f64, -2.0]);
        let Ok(()) = result;
    }
    #[test]
    fn normalizing_a_zero_vector_preserves_it() {
        assert_eq!(normalize(&[0.0_f64, 0.0]), [0.0, 0.0]);
    }
}
