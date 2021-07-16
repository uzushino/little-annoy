use crate::distance::{Distance, NodeImpl};
use num::ToPrimitive;
use serde::{Deserialize, Serialize};

pub struct Hamming {}

#[derive(Debug, Clone)]
pub struct Node {
    pub children: Vec<i64>,
    pub v: Vec<u64>,
    pub n_descendants: usize,
    pub f: usize,
}

impl NodeImpl<u64> for Node {
    fn new(f: usize) -> Self {
        Node {
            children: vec![0, 0],
            v: vec![0; f],
            n_descendants: 0,
            f
        }
    }

    fn reset(&mut self, v: &[u64]) {
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

    fn vector(&self) -> &[u64] {
        self.v.as_slice()
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

impl Distance<u64> for Hamming {
    type Node = Node;

    fn margin(n: &Self::Node, y: &[u64]) -> f64 {
        let n_bits = 4 * 8_u64;
        let chunk = n.v[0].to_u64().unwrap_or_default() / n_bits;
        let r = (y[chunk as usize].to_i64().unwrap())
            & (1 << (n_bits - 1 - (n.v[0].to_u64().unwrap() as u64 % n_bits)) != 0) as i64;
        r as f64
    }

    fn side(n: &Self::Node, y: &[u64]) -> bool {
        if Self::margin(n, y) > 0.0 {
            return true;
        }
        false
    }

    fn distance(x: &[u64], y: &[u64], f: usize) -> f64 {
        let mut dist = 0;

        for i in 0..f {
            dist +=
                ((x[i].to_u64().unwrap() as u64) ^ (y[i].to_u64().unwrap() as u64)).count_ones();
        }

        dist as f64
    }

    fn normalized_distance(distance: f64) -> f64 {
        distance
    }

    fn create_split(nodes: Vec<Self::Node>, n: &mut Self::Node, f: usize) {
        let mut cur_size = 0;
        let mut i = 0;
        for _ in 0..MAX_ITERATIONS {
            n.v[0] = (rand::random::<usize>() % f) as u64;
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
                n.v[0] = j as u64;

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
