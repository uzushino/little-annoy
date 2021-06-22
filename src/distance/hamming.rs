use crate::distance::{Distance, NodeImpl};

pub struct Hamming {}

#[derive(Debug, Clone)]
pub struct Node<const N: usize> {
    pub children: Vec<i64>,
    pub v: [f64; N],
    pub n_descendants: usize,
}

impl<T: num::Num + Copy, const N: usize> NodeImpl<T, N> for Node<N> {
    fn new() -> Self {
        Node {
            children: vec![0, 0],
            v: [0.0; N],
            n_descendants: 0,
        }
    }

    fn reset(&mut self, v: [f64; N]) {
        self.children[0] = 0;
        self.children[1] = 0;
        self.n_descendants = 1;
        self.v = v;
    }

    fn descendant(&self) -> usize {
        self.n_descendants
    }

    fn set_descendant(&mut self, other: usize) {
        self.n_descendants = other;
    }

    fn vector(&self) -> [f64; N] {
        self.v
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

impl<T: num::Num + Copy, const N: usize> Distance<T, N> for Hamming {
    type Node = Node<N>;

    fn margin(n: &Self::Node, y: [T; N]) -> f64 {
        let n_bits = 4 * 8 as u64;
        let chunk = n.v[0] as u64 / n_bits;
        let r =
            (y[chunk as usize] as i64) & (1 << (n_bits - 1 - (n.v[0] as u64 % n_bits)) != 0) as i64;
        r as f64
    }

    fn side(n: &Self::Node, y: [T; N]) -> bool {
        if Self::margin(n, y) > 0.0 {
            return true;
        }
        false
    }

    fn distance(x: [T; N], y: [T; N]) -> f64 {
        let mut dist = 0;

        for i in 0..N {
            dist += ((x[i] as u64) ^ (y[i] as u64)).count_ones();
        }

        dist as f64
    }

    fn normalized_distance(distance: f64) -> f64 {
        distance
    }

    fn create_split(nodes: Vec<Self::Node>, n: &mut Self::Node) {
        let mut cur_size = 0;
        let mut i = 0;
        for _ in 0..MAX_ITERATIONS {
            n.v[0] = (rand::random::<usize>() % N) as f64;
            cur_size = 0;

            for node in nodes.iter() {
                if Self::side(node, n.v) {
                    cur_size += 1;
                }
            }

            if cur_size > 0 && cur_size < nodes.len() {
                break;
            }

            i += 1;
        }

        if i == MAX_ITERATIONS {
            for j in 0..N {
                n.v[0] = j as f64;
                cur_size = 0;

                for node in nodes.iter() {
                    if Self::side(node, n.v) {
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
