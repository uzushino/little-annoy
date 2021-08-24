use crate::distance::{Distance, NodeImpl};
use crate::item::Item;
use serde::{Deserialize, Serialize};

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

impl<T: Item> Distance<T> for Hamming {
    type Node = Node<T>;

    fn margin(n: &Self::Node, y: &[T]) -> T {
        let n_bits = 4 * 8_u64;
        let chunk = n.v[0].to_u64().unwrap_or_default() / n_bits;
        let r = (y[chunk as usize].to_i64().unwrap())
            & (1 << (n_bits - 1 - (n.v[0].to_u64().unwrap() as u64 % n_bits)) != 0) as i64;

        T::from_i64(r).unwrap()
    }

    fn side(n: &Self::Node, y: &[T]) -> bool {
        if Self::margin(n, y) > T::zero() {
            return true;
        }
        false
    }

    fn distance(x: &[T], y: &[T], f: usize) -> T {
        let mut dist = T::zero();

        for i in 0..f {
            let v =
                ((x[i].to_u64().unwrap() as u64) ^ (y[i].to_u64().unwrap() as u64)).count_ones();
            dist += T::from_u32(v).unwrap();
        }

        dist
    }

    fn normalized_distance(distance: f64) -> f64 {
        distance
    }

    fn create_split(nodes: &mut Vec<Self::Node>, n: &mut Self::Node, f: usize) {
        let mut cur_size = 0;
        let mut i = 0;

        for _ in 0..MAX_ITERATIONS {
            let rnd = rand::random::<usize>() % f;
            n.v[0] = T::from_usize(rnd).unwrap();
            cur_size = 0;

            for node in nodes.iter() {
                if Self::side(node, &n.v) {
                    cur_size += 1;
                }
            }

            if cur_size > 0 && cur_size < nodes.len() {
                break;
            }

            i += 1;
        }

        if i == MAX_ITERATIONS {
            for j in 0..f {
                n.v[0] = T::from_usize(j).unwrap_or(T::zero());
                cur_size = 0;

                for node in nodes.iter() {
                    if Self::side(node, &n.v) {
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
