use std::usize;
use std::cmp::Ordering;

mod node;
mod distance;
pub mod ann;

pub use ann::Annoy;

use distance::{ Distance, Euclidian };

#[derive(PartialEq)]
struct Numeric(f64);

impl Eq for Numeric {}

impl PartialOrd for Numeric {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Numeric {
    fn cmp(&self, other: &Numeric) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn random_flip() -> bool {
    rand::random::<bool>()
}

fn get_norm<const N: usize>(v: [f64; N]) -> f64{
    let mut sq_norm = 0.0;
    
    for z in 0..N {
        sq_norm += v[z as usize] * v[z as usize];
    }

    sq_norm.sqrt()
}

fn normalize<const N: usize>(v: &mut [f64; N]) {
    let norm = get_norm(*v);

    for z in 0..N {
        v[z] /= norm;
    }
}

const ITERATION_STEPS: usize = 200;

fn two_means<D: Distance, const N: usize>(nodes: Vec<node::Node<N>>, iv: &mut [f64; N], jv: &mut [f64; N]) {
    let count = nodes.len();
    let i: u64 = rand::random::<u64>() % count as u64;
    let mut j : u64 = rand::random::<u64>() % (count - 1) as u64;

    j += (j >= i) as u64;

    for d in 0..N {
        iv[d] = nodes[i as usize].v[d];
        jv[d] = nodes[j as usize].v[d];
    }

    let mut ic = 1.0;
    let mut jc = 1.0;

    for _ in 0..ITERATION_STEPS {
        let k = rand::random::<usize>() % count as usize;
        let di = ic * D::distance(iv, nodes[k].v);
        let dj = jc * D::distance(jv, nodes[k].v);
        let norm = 1.0;

        if di < dj {
            for z in 0..N {
                iv[z] = (iv[z] * ic + nodes[k].v[z] / norm) / (ic + 1.0);
            }

            ic += 1.0;
        } else if dj < di {
            for z in 0..N {
                jv[z] = (jv[z] * jc + nodes[k].v[z] / norm) / (jc + 1.0);
            }

            jc += 1.0;
        }
    }
}
