use std::fmt::Debug;

use rand::rngs::ThreadRng;
use rand::Rng;

pub mod angular;
pub mod euclidean;
pub mod hamming;
pub mod manhattan;

pub use angular::Angular;
pub use euclidean::Euclidean;
pub use hamming::Hamming;
pub use manhattan::Manhattan;

use crate::item::Item;

const ITERATION_STEPS: usize = 200;

fn get_norm<T: Item>(v: &[T]) -> T {
    v.iter().fold(T::zero(), |acc, &x| acc + (x * x)).sqrt()
}

fn normalize<T: Item>(v: &[T]) -> Vec<T> {
    let norm = get_norm(v);
    let mut v2 = v.iter().map(|_| T::zero()).collect::<Vec<_>>();
    for z in 0..v.len() {
        v2[z] = v[z] / norm;
    }
    v2
}

fn two_means<T: Item, D: Distance<T>>(
    rng: &mut ThreadRng,
    nodes: &[&D::Node],
    f: usize,
) -> (Vec<T>, Vec<T>) {
    let count = nodes.len();
    let i: u64 = rng.gen::<u64>() % count as u64;
    let mut j: u64 = rng.gen::<u64>() % (count - 1) as u64;
    j += (j >= i) as u64;

    let mut iv = nodes[i as usize].vector().to_vec();
    let mut jv = nodes[j as usize].vector().to_vec();

    let mut ic = T::one();
    let mut jc = T::one();

    for _ in 0..ITERATION_STEPS {
        let k = rng.gen::<usize>() % count as usize;
        let di = ic * D::distance(&iv, nodes[k].vector(), f);
        let dj = jc * D::distance(&jv, nodes[k].vector(), f);
        let nk = &nodes[k].vector();

        if di < dj {
            for z in 0..f {
                let v = iv[z] * ic + nk[z];
                iv[z] = v / (ic + T::one());
            }

            ic += T::one();
        } else if dj < di {
            for z in 0..f {
                let v = jv[z] * jc + nk[z];
                jv[z] = v / (jc + T::one());
            }
            jc += T::one();
        }
    }

    (iv, jv)
}

pub trait NodeImpl<T> {
    fn new(f: usize) -> Self;

    fn reset(&mut self, w: &[T]);
    fn copy(&mut self, other: Self);

    fn descendant(&self) -> usize;
    fn set_descendant(&mut self, other: usize);

    fn vector(&self) -> &[T];
    fn set_vector(&self, _other: &[T]) {}
    fn mut_vector(&mut self) -> &mut Vec<T>;

    fn children(&self) -> Vec<i64>;
    fn set_children(&mut self, other: Vec<i64>);
}

pub trait Distance<T: Item> {
    type Node: NodeImpl<T> + Clone + Debug;

    fn distance(x: &[T], y: &[T], f: usize) -> T;

    fn create_split(nodes: &[&Self::Node], n: &mut Self::Node, f: usize, rng: &mut ThreadRng);

    fn side(n: &Self::Node, y: &[T], rng: &mut ThreadRng) -> bool;

    fn margin(n: &Self::Node, y: &[T]) -> T;

    fn normalized_distance(distance: f64) -> f64;
}
