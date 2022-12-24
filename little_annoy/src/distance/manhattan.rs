use rand::rngs::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::distance::{normalize, two_means, Distance, NodeImpl};
pub struct Manhattan {}

#[derive(Debug, Clone)]
pub struct Node {
    pub children: Vec<i64>,
    pub v: Vec<f64>,
    pub n_descendants: usize,
    pub a: f64,
}

impl NodeImpl<f64> for Node {
    fn new(f: usize) -> Self {
        Node {
            children: vec![0, 0],
            v: vec![0.0; f],
            n_descendants: 0,
            a: 0.0,
        }
    }

    fn reset(&mut self, v: &[f64]) {
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

    fn vector(&self) -> &[f64] {
        self.v.as_slice()
    }

    fn mut_vector(&mut self) -> &mut Vec<f64> {
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
        self.a = other.a;
    }
}

impl Distance<f64> for Manhattan {
    type Node = Node;

    #[inline]
    fn margin(n: &Self::Node, y: &[f64]) -> f64 {
        let mut dot: f64 = n.a;

        for (z, _y) in y.iter().enumerate() {
            dot += n.v[z] * y[z];
        }

        dot
    }

    #[inline]
    fn side(n: &Self::Node, y: &[f64], rng: &mut ThreadRng) -> bool {
        let dot = Self::margin(n, y);

        if dot != 0.0 {
            return dot > 0.0;
        }

        rng.gen()
    }

    #[inline]
    fn distance(x: &[f64], y: &[f64], f: usize) -> f64 {
        let mut d = 0.0;

        for i in 0..f {
            d += (x[i] - y[i]).abs();
        }

        d
    }

    #[inline]
    fn normalized_distance(distance: f64) -> f64 {
        distance.max(0.0)
    }

    #[inline]
    fn create_split(nodes: &[&Self::Node], n: &mut Self::Node, f: usize, rng: &mut ThreadRng) {
        let (best_iv, best_jv) = two_means::<f64, Manhattan>(rng, nodes, f);

        for z in 0..f {
            n.v[z] = best_iv[z] - best_jv[z];
        }

        n.v = normalize(&n.v);
        n.a = 0.0;

        for z in 0..f {
            n.a += -n.v[z] * (best_iv[z] + best_jv[z]) / 2.0;
        }
    }
}
