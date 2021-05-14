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
    let mut iv =  [0.0; N];
    let mut jv =  [0.0; N];
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

    (iv, jv)
}

pub trait Distance {
    fn distance<const N: usize>(x: [f64; N], y: [f64; N]) -> f64;

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

    fn distance<const N: usize>(x: [f64; N], y: [f64; N]) -> f64 {
        let mut d = 0.0;
        for i in 0..N {
            d += ((x[i as usize] - y[i as usize])) * ((x[i as usize] - y[i as usize]));
        }
        d
    }

    fn create_split<const N: usize>(nodes: Vec<Node<N>>, n: &mut Node<N>) {
        let (best_iv, best_jv) = two_means::<Euclidian, N>(nodes);

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
