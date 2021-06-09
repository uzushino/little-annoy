use std::char::MAX;

use crate::node;
use crate::node::Node;

use crate::random_flip;

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

fn two_means<D: Distance, const N: usize>(nodes: Vec<Node<N>>) -> ([f64; N], [f64; N]) {
    let count = nodes.len();
    let i: u64 = rand::random::<u64>() % count as u64;
    let mut j : u64 = rand::random::<u64>() % (count - 1) as u64;
    j += (j >= i) as u64;
    let mut iv = nodes[i as usize].v;
    let mut jv = nodes[j as usize].v;

    let mut ic = 1.0;
    let mut jc = 1.0;
    
    for _ in 0..ITERATION_STEPS {
        let k = rand::random::<usize>() % count as usize;
        let di = ic * D::distance(iv, nodes[k].v);
        let dj = jc * D::distance(jv, nodes[k].v);

        if di < dj {
            for z in 0..N {
                iv[z] = (iv[z] * ic + nodes[k].v[z]) / (ic + 1.0);
            }

            ic += 1.0;
        } else if dj < di {
            for z in 0..N {
                jv[z] = (jv[z] * jc + nodes[k].v[z]) / (jc + 1.0);
            }

            jc += 1.0;
        }
    }

    (iv, jv)
}

pub trait Distance {
    fn distance<const N: usize>(x: [f64; N], y: [f64; N]) -> f64;

    fn create_split<const N: usize>(nodes: Vec<Node<N>>, n: &mut Node<N>);
    
    fn side<const N: usize>(n: &Node<N>, y: [f64; N]) -> bool;
}
