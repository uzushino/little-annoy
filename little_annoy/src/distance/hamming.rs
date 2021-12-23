use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::distance::{Distance, NodeImpl};
use crate::item::Item;

pub struct Hamming {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Node<T: Item> {
    pub children: Vec<i64>,
    pub v: Vec<T>,
    pub n_descendants: usize,
    pub f: usize,
}

impl<T: Item> NodeImpl<T> for Node<T> {
    fn new(f: usize) -> Self {
        Node {
            children: vec![0, 0],
            v: vec![T::zero(); f],
            n_descendants: 0,
            f,
        }
    }

    fn reset(&mut self, v: &[T]) {
        self.children[0] = 0;
        self.children[1] = 0;
        self.n_descendants = 1;
        self.v = v.to_vec();
    }

    fn descendant(&self) -> usize {
        self.n_descendants
    }

    fn set_descendant(&mut self, other: usize) {
        self.n_descendants = other;
    }

    fn vector(&self) -> &[T] {
        self.v.as_slice()
    }

    fn mut_vector(&mut self) -> &mut Vec<T> {
        &mut self.v
    }

    fn children(&self) -> Vec<i64> {
        self.children.clone()
    }

    fn set_children(&mut self, other: Vec<i64>) {
        self.children = other;
    }

    fn copy(&mut self, other: Self) {
        self.n_descendants = other.n_descendants;
        self.children = other.children;
        self.v = other.v;
    }
}

const MAX_ITERATIONS: usize = 20;

impl<T: Item + serde::Serialize> Distance<T> for Hamming {
    type Node = Node<T>;

    fn margin(n: &Self::Node, y: &[T]) -> T {
        let n_bits = 4 * 8_u64;
        let chunk = n.v[0].to_u64().unwrap_or_default() / n_bits;
        let r = (y[chunk as usize].to_i64().unwrap())
            & (1 << (n_bits - 1 - (n.v[0].to_u64().unwrap() as u64 % n_bits)) != 0) as i64;

        T::from_i64(r).unwrap()
    }

    fn side(n: &Self::Node, y: &[T], _rng: &mut StdRng) -> bool {
        Self::margin(n, y) > T::zero()
    }

    fn distance(x: &[T], y: &[T], f: usize) -> T {
        let mut dist = T::zero();

        (0..f).for_each(|i| {
            let v =
                ((x[i].to_u64().unwrap() as u64) ^ (y[i].to_u64().unwrap() as u64)).count_ones();

            dist += T::from_u32(v).unwrap();
        });

        dist
    }

    fn normalized_distance(distance: f64) -> f64 {
        distance
    }

    fn create_split(nodes: &[Self::Node], n: &mut Self::Node, f: usize, rng: &mut StdRng) {
        let mut cur_size = 0;
        let mut i = 0;

        (0..MAX_ITERATIONS).for_each(|_| {
            let rnd = rng.gen::<usize>() % f;
            n.v[0] = T::from_usize(rnd).unwrap();
            cur_size = 0;

            for node in nodes.iter() {
                if Self::side(node, &n.v, rng) {
                    cur_size += 1;
                }
            }

            if cur_size > 0 && cur_size < nodes.len() {
                return;
            }

            i += 1;
        });

        if i == MAX_ITERATIONS {
            for j in 0..f {
                n.v[0] = T::from_usize(j).unwrap_or_else(T::zero);
                cur_size = 0;

                for node in nodes.iter() {
                    if Self::side(node, &n.v, rng) {
                        cur_size += 1;
                    }
                }

                if cur_size > 0 && cur_size < nodes.len() {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let x = &[1.0, 1.0, 1.0];
        let y = &[1.0, 1.0, 0.0];
        let f = 3;

        let dist = Hamming::distance(x, y, f);

        assert_eq!(dist, 1.0);
    }
}
