use super::Distance;
use crate::scalar::Scalar;
use rand::{RngExt, rngs::ThreadRng};

#[derive(Debug, Clone, Copy, Default)]
pub struct Hamming;

impl<T: Scalar> Distance<T> for Hamming {
    const NAME: &'static str = "hamming";
    type ValidationError = &'static str;
    type Split = usize;

    fn validate_values(values: &[T]) -> Result<(), Self::ValidationError> {
        if values
            .iter()
            .all(|value| value.to_u64().and_then(T::from_u64) == Some(*value))
        {
            Ok(())
        } else {
            Err("Hamming values must be nonnegative integers representable as u64")
        }
    }
    fn validate_split(bit: &usize, dimensions: usize) -> bool {
        *bit / (u64::BITS as usize) < dimensions
    }
    fn margin(bit: &usize, vector: &[T]) -> T {
        let chunk = bit / u64::BITS as usize;
        let mask = 1_u64 << (bit % u64::BITS as usize);
        if vector[chunk].to_u64().unwrap() & mask != 0 {
            T::one()
        } else {
            -T::one()
        }
    }
    fn distance(x: &[T], y: &[T], dimensions: usize) -> T {
        x.iter()
            .zip(y)
            .take(dimensions)
            .fold(T::zero(), |sum, (&x, &y)| {
                sum + T::from_u32((x.to_u64().unwrap() ^ y.to_u64().unwrap()).count_ones()).unwrap()
            })
    }
    fn normalized_distance(distance: f64) -> f64 {
        distance
    }
    fn create_split(vectors: &[&[T]], rng: &mut ThreadRng) -> usize {
        let bits = vectors[0].len() * u64::BITS as usize;
        let divides = |bit: &usize| {
            let first = Self::margin(bit, vectors[0]);
            vectors[1..]
                .iter()
                .any(|vector| Self::margin(bit, vector) != first)
        };
        for _ in 0..20 {
            let bit = rng.random_range(0..bits);
            if divides(&bit) {
                return bit;
            }
        }
        (0..bits).find(divides).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_bit_is_read_from_the_correct_chunk() {
        let mut rng = rand::rng();
        assert!(Hamming::side(&67, &[0_i64, 8], &mut rng));
        assert!(!Hamming::side(&67, &[8_i64, 0], &mut rng));
        assert!(!Hamming::side(&67, &[0_i64, 1], &mut rng));
    }
    #[test]
    fn split_finds_high_bits_in_later_coordinates() {
        let first = [0_i64, 0];
        let second = [0_i64, 1 << 40];
        let mut rng = rand::rng();
        let bit = Hamming::create_split(&[&first, &second], &mut rng);
        assert_ne!(
            Hamming::side(&bit, &first, &mut rng),
            Hamming::side(&bit, &second, &mut rng)
        );
    }
    #[test]
    fn counts_differing_bits() {
        assert_eq!(Hamming::distance(&[1_i64, 1, 1], &[1, 1, 0], 3), 1);
    }
}
