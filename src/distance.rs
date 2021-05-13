use crate::node::Node;

use crate::random_flip;
use crate::two_means;
use crate::normalize;


pub trait Distance {
    fn distance<const N: usize>(x: &[f64; N], y: [f64; N]) -> f64;

    fn create_split<const N: usize>(nodes: Vec<Node<N>>, n: &mut Node<N>);
    
    fn side<const N: usize>(n: Node<N>, y: [f64; N]) -> bool;
}

pub struct Euclidian {}

impl Euclidian {
    pub fn margin<const N: usize>(n: Node<N>, y: [f64; N]) -> f64 {
        let mut dot: f64 = n.a;

        for z in 0..N {
            dot += n.v[z as usize] * y[z as usize];
        }

        dot
    }
}

impl Distance for Euclidian {
    fn side<const N: usize>(n: Node<N>, y: [f64; N]) -> bool {
        let dot = Self::margin(n, y);
        if dot != 0.0 {
            return dot > 0.0;
        }
        random_flip()
    }

    fn distance<const N: usize>(x: &[f64; N], y: [f64; N]) -> f64 {
        let mut d = 0.0;
        for i in 0..N {
            d += ((x[i as usize] - y[i as usize])) * ((x[i as usize] - y[i as usize]));
        }
        d
    }

    fn create_split<const N: usize>(nodes: Vec<Node<N>>, n: &mut Node<N>) {
        let mut best_iv: [f64; N] = [0.0; N];
        let mut best_jv: [f64; N] = [0.0; N];

        two_means::<Euclidian, N>(nodes, &mut best_iv, &mut best_jv);

        for z in 0..N {
            n.v[z] = best_iv[z] - best_jv[z];
        }

        normalize(&mut n.v);

        n.a = 0.0;
        for z in 0..N {
            n.a += -n.v[z] * (best_iv[z] + best_jv[z]) / 2.0;
        }
    }
}
