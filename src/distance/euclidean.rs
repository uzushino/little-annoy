pub struct Euclidean {}

impl Euclidean {
    fn margin<const N: usize>(n: &Node<N>, y: [f64; N]) -> f64 {
        let mut dot: f64 = n.a;

        for z in 0..N {
            dot += n.v[z as usize] * y[z as usize];
        }

        dot
    }
}


impl Distance for Euclidean {
    fn side<const N: usize>(n: &Node<N>, y: [f64; N]) -> bool {
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
        let (best_iv, best_jv) = two_means::<Euclidean, N>(nodes);

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

pub struct Hamming {}

impl Hamming {
    fn margin<const N: usize>(n: &Node<N>, y: [i64; N]) -> bool {
        let n_bits = 4 * 8 as i64;
        let chunk = n.v[0] as i64 / n_bits;
        (y[chunk as usize] & (1 << (n_bits - 1 - (n.v[0] as i64 % n_bits)))) != 0
    }
}

const MAX_ITERATIONS: usize = 20;

impl Distance for Hamming {
    fn side<const N: usize>(n: &Node<N>, y: [f64; N]) -> bool {
        Self::margin(n, y)
    }

    fn distance<const N: usize>(x: [f64; N], y: [f64; N]) -> f64 {
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
