use serde::{Deserialize, Serialize};

use crate::distance::{normalize, two_means, Distance, NodeImpl};
use crate::random_flip;
use crate::float::Float;

pub struct Angular {}

#[derive(Clone, Serialize, Deserialize)]
pub struct Node<T: Float> {
    pub children: Vec<i64>,
    pub v: Vec<T>,
    pub n_descendants: usize,
    f: usize,
}

impl<T: Float> NodeImpl<T> for Node<T> {
    fn new(f: usize) -> Self {
        Node {
            children: vec![0, 0],
            v: (0..f).map(|_| T::zero()).collect(),
            n_descendants: 0,
            f: f,
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

impl<T: Float> Distance<T> for Angular {
    type Node = Node<T>;

    fn margin(n: &Self::Node, y: &[T]) -> T {
        let mut dot = T::zero();

        for z in 0..n.f {
            dot += n.v[z] * y[z];
        }

        dot
    }

    fn side(n: &Self::Node, y: &[T]) -> bool {
        let dot = Self::margin(n, y);

        if dot != T::zero() {
            return dot > T::zero();
        }

        random_flip()
    }

    fn distance(x: &[T], y: &[T], f: usize) -> T {
        let mut pp = T::zero();
        let mut qq = T::zero();
        let mut pq = T::zero();

        for z in 0..f {
            pp += x[z] * x[z];
            qq += y[z] * y[z];
            pq += x[z] * y[z];
        }

        let ppqq = pp * qq;

        let make_distance = || {
            let two = T::from_f32(2.0).unwrap();
            if ppqq > T::zero() {
                return two - two * pq / ppqq.sqrt();
            } else {
                return T::from_f32(2.0).unwrap();
            }
        };

        make_distance()
    }

    fn normalized_distance(distance: f64) -> f64 {
        distance.max(0.0).sqrt()
    }

    fn create_split(nodes: Vec<Self::Node>, n: &mut Self::Node, f: usize) {
        let (best_iv, best_jv) = two_means::<T, Angular>(nodes, f);

        for z in 0..f {
            let best = best_iv[z] - best_jv[z];
            n.v[z] = best;
        }

        n.v = normalize(&n.v);
    }
}
