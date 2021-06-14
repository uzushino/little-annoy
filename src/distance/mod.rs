// use crate::node::Node;

pub mod euclidean;
pub mod hamming;

pub use euclidean::Euclidean;
pub use hamming::Hamming;

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

fn two_means<D: Distance<N>, const N: usize>(nodes: Vec<D::Node>) -> ([f64; N], [f64; N]) {
    let count = nodes.len();
    let i: u64 = rand::random::<u64>() % count as u64;
    let mut j : u64 = rand::random::<u64>() % (count - 1) as u64;
    j += (j >= i) as u64;
    let mut iv = nodes[i as usize].vector();
    let mut jv = nodes[j as usize].vector();

    let mut ic = 1.0;
    let mut jc = 1.0;
    
    for _ in 0..ITERATION_STEPS {
        let k = rand::random::<usize>() % count as usize;
        let di = ic * D::distance(iv, nodes[k].vector());
        let dj = jc * D::distance(jv, nodes[k].vector());

        if di < dj {
            for z in 0..N {
                iv[z] = (iv[z] * ic + nodes[k].vector()[z]) / (ic + 1.0);
            }

            ic += 1.0;
        } else if dj < di {
            for z in 0..N {
                jv[z] = (jv[z] * jc + nodes[k].vector()[z]) / (jc + 1.0);
            }

            jc += 1.0;
        }
    }

    (iv, jv)
}

pub trait NodeImpl<const N: usize> {
    fn new() -> Self;

    fn reset(&mut self, w: [f64; N]);
    fn copy(&mut self, other: Self);

    fn descendant(&self) -> usize;
    fn set_descendant(&mut self, other: usize);
   
    fn vector(&self) -> [f64; N];
    fn set_vector(&self, _other: [f64; N]) {}
    
    fn children(&self) -> Vec<i64>; 
    fn set_children(&mut self, other: Vec<i64>);
}

pub trait Distance<const N: usize> {
    type Node: NodeImpl<N> + Clone;

    fn distance(x: [f64; N], y: [f64; N]) -> f64;

    fn create_split(nodes: Vec<Self::Node>, n: &mut Self::Node);
    
    fn side(n: &Self::Node, y: [f64; N]) -> bool;
    
    fn margin(n: &Self::Node, y: [f64; N]) -> f64 ;
}
