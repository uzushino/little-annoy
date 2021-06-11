use crate::distance::{ two_means, normalize, Distance};
use crate::random_flip;

use serde::{Serialize, Deserialize};

pub struct Hamming {}

pub struct Node<const N: usize> {
    pub children: Vec<i64>,
    pub v: [u64; N],
}

impl Hamming {
    fn margin<const N: usize>(n: &Node<N>, y: [u64; N]) -> bool {
        let n_bits = 4 * 8 as u64;
        let chunk = n.v[0] as u64 / n_bits;
        (y[chunk as usize] & (1 << (n_bits - 1 - (n.v[0] as u64 % n_bits)))) != 0
    }
}

const MAX_ITERATIONS: usize = 20;

impl Distance for Hamming {
    fn side<const N: usize>(n: &Node<N>, y: [u64; N]) -> bool {
        Self::margin(n, y)
    }

    fn distance<const N: usize>(x: [u64; N], y: [u64; N]) -> f64 {
        let mut dist = 0;

        for i in 0..N {
            dist += ((x[i] as u64) ^ (y[i] as u64)).count_ones();
        }

        dist as f64
    }

    fn create_split<const N: usize>(nodes: Vec<Node<N>>, n: &mut Node<N>) {
        let mut cur_size = 0;
        let mut idx = 0;
        for i in 0..MAX_ITERATIONS {
            n.v[0] = rand::random::<f64>() % N;
            cur_size = 0;

            for node in nodes.iter() {
                if Self::margin(node, n.v) {
                    cur_size += 1;
                }
            }

            if cur_size > 0 && cur_size < nodes.len() {
                break
            }

            i = idx;
        }

        if idx == MAX_ITERATIONS {
            let jdx = 0;
            for j in 0..N {
                n.v[0] = j;
                cur_size = 0 ;

                for node in nodes.iter() {
                    if Self::margin(node, n.v) {
                        cur_size += 1;
                    }
                }

                if cur_size > 0 && cur_size < nodes.len() {
                    break
                }
            }
        }
    }
}
