use num::ToPrimitive;

use crate::distance::{normalize, two_means, Distance, NodeImpl};
use crate::random_flip;

pub struct Angular {}

#[derive(Clone)]
pub struct Node {
    pub children: Vec<i64>,
    pub v: Vec<f64>,
    pub n_descendants: usize,
    f: usize,
}

impl NodeImpl<f64> for Node {
    fn new(f: usize) -> Self {
        Node {
            children: vec![0, 0],
            v: (0..f).map(|_| 0.0).collect(),
            n_descendants: 0,
            f: f,
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

impl Distance<f64> for Angular {
    type Node = Node;

    fn margin(n: &Self::Node, y: &[f64]) -> f64 {
        let mut dot = 0.;

        for z in 0..n.f {
          dot += n.v[z] * y[z];
        }

        return dot;
    }

    fn side(n: &Self::Node, y: &[f64]) -> bool {
        let dot = Self::margin(n, y);

        if dot != 0.0 {
            return dot > 0.0;
        }

        random_flip()
    }

    fn distance(x: &[f64], y: &[f64], f: usize) -> f64 {
        let mut pp = 0.;
        let mut qq = 0.;
        let mut pq = 0.;

        for z in 0..f {
            pp += x[z] * x[z];
            qq += y[z] * y[z];
            pq += x[z] * y[z];
        }

        let ppqq = pp * qq;
        if ppqq > 0. {
            return 2.0 - 2.0 * pq / ppqq.sqrt();
        } else {
            return 2.0
        }
    }

    fn normalized_distance(distance: f64) -> f64 {
        distance.max(0.0).sqrt()
    }

    fn create_split(nodes: Vec<Self::Node>, n: &mut Self::Node, f: usize) {
        let (best_iv, best_jv) = two_means::<f64, Angular>(nodes, f);

        for z in 0..f {
            let best = best_iv[z] - best_jv[z];
            n.v[z] = best;
        }

        n.v = normalize(&n.v);
    }
}
