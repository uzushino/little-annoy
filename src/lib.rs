use std::usize;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Node<const N: usize> {
    pub children: Vec<i64>,
    pub v: [f64; N],
    pub n_descendants: usize,
    pub a: f64,
}

#[derive(PartialEq)]
struct MinFloat(f64);

impl Eq for MinFloat {}

impl PartialOrd for MinFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for MinFloat {
    fn cmp(&self, other: &MinFloat) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn random_flip() -> bool {
    rand::random::<bool>()
}

fn get_norm<const N: usize>(v: [f64; N] , f: usize) -> f64{
    let mut sq_norm = 0.0;
    
    for z in 0..f {
        sq_norm += v[z as usize] * v[z as usize];
    }

    sq_norm.sqrt()
}

fn normalize<const N: usize>(v: &mut [f64; N], f: usize) {
    let norm = get_norm(*v, f);

    for z in 0..f {
        v[z] /= norm;
    }
}

const ITERATION_STEPS: usize = 200;

fn two_means<const N: usize>(nodes: Vec<Node<N>>, f: usize, iv: &mut [f64; N], jv: &mut [f64; N]) {
    let count = nodes.len();
    let i: u64 = rand::random::<u64>() % count as u64;
    let mut j : u64 = rand::random::<u64>() % (count - 1) as u64;

    j += (j >= i) as u64;

    for d in 0..f {
        iv[d] = nodes[i as usize].v[d];
        jv[d] = nodes[j as usize].v[d];
    }

    let mut ic = 1.0;
    let mut jc = 1.0;

    for _ in 0..ITERATION_STEPS {
        let k = rand::random::<usize>() % count as usize;
        let di = ic * Euclidian::distance(iv, nodes[k].v, f);
        let dj = jc * Euclidian::distance(jv, nodes[k].v, f);
        let norm = 1.0;

        if di < dj {
            for z in 0..f {
                iv[z] = (iv[z] * ic + nodes[k].v[z] / norm) / (ic + 1.0);
            }

            ic += 1.0;
        } else if dj < di {
            for z in 0..f {
                jv[z] = (jv[z] * jc + nodes[k].v[z] / norm) / (jc + 1.0);
            }

            jc += 1.0;
        }
    }
}

trait Distance {}

struct Euclidian {}

impl Euclidian {
    pub fn margin<const N: usize>(n: Node<N>, y: [f64; N], f: usize) -> f64 {
        let mut dot: f64 = n.a;
        for z in 0..f {
            dot += n.v[z as usize] * y[z as usize];
        }
        dot
    }

    pub fn side<const N: usize>(n: Node<N>, y: [f64; N], f: usize) -> bool {
        let dot = Self::margin(n, y, f);
        if dot != 0.0 {
            return dot > 0.0;
        }
        random_flip()
    }

    pub fn distance<const N: usize>(x: &[f64; N], y: [f64; N], f: usize) -> f64 {
        let mut d = 0.0;
        for i in 0..f {
            d += ((x[i as usize] - y[i as usize])) * ((x[i as usize] - y[i as usize]));
        }
        d
    }

    pub fn create_split<const N: usize>(nodes: Vec<Node<N>>, f: usize, mut n: &mut Node<N>) {
        let mut best_iv: [f64; N] = [0.0; N];
        let mut best_jv: [f64; N] = [0.0; N];

        two_means(nodes, f, &mut best_iv, &mut best_jv);

        for z in 0..f {
            n.v[z] = best_iv[z] - best_jv[z];
        }

        normalize(&mut n.v, f);

        n.a = 0.0;
        for z in 0..f {
            n.a += -n.v[z] * (best_iv[z] + best_jv[z]) / 2.0;
        }
    }
}

impl<const N: usize> Node<N> {
    pub fn new() -> Self {
        Node {
            children: vec![0, 0],
            v: [0.0; N],
            n_descendants: 0,
            a: 0.0,
        }
    }
}

pub struct Annoy<const N: usize> {
    pub _K: usize,
    pub _n_nodes: i64,
    pub _n_items: i64,
    
    pub _nodes: HashMap<i64, Node<N>>,
    pub _roots: Vec<i64>,
}

impl<const N: usize> Annoy<N> {
    pub fn new() -> Self {
        Self {
            _roots: Vec::new(),
            _nodes: HashMap::new(),
            _n_items: 0, // // 「登録ベクトル群」に登録されているベクトルの個数
            _n_nodes: 0, // 実際に登録しているNodeの個数
            _K: 6, // TODO
        }
    }
    
    pub fn add_item(&mut self, item: i64, w: [f64; N]) {
        let mut n = self._nodes.entry(item).or_insert(Node::new());
       
        n.children[0] = 0;
        n.children[1] = 0;
        n.n_descendants = 1;
        n.v = w;
        
        if item >= self._n_items {
            self._n_items = item + 1;
        }
    }

    pub fn build(&mut self, q: i64) {
        self._n_nodes = self._n_items;

        loop {
            if q == -1 && self._n_nodes >= self._n_items * 2 {
                break;
            }
            if q != -1 && self._roots.len() >= (q as usize) {
                break;
            }
            
            let mut indices: Vec<i64> = Vec::new();
            
            for i in 0..self._n_items {
                if let Some(n) = self._nodes.get(&i) {
                    if n.n_descendants >= 1 {
                        indices.push(i)
                    }
                }
            }
            
            let ind = self._make_tree(&indices);

            self._roots.push(ind);
        }
    }
    
    pub fn get_nns_by_vector(&mut self, v: &[f64; N], n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>) {
        self._get_all_nns(v, n, search_k) 
    }

    pub fn get_nns_by_item(&mut self, item: i64, n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>) {
        let m = self._nodes.get(&item).unwrap();
        let v = m.v;
        self._get_all_nns(&v, n, search_k) 
    }

    fn _make_tree(&mut self, indices: &Vec<i64>) -> i64 {
        if indices.len() == 1 {
            return indices[0];
        }
            
        if indices.len() <= (self._K as usize) {
            let item = self._n_nodes;
            self._n_nodes = self._n_nodes + 1;
    
            let m = self._nodes.entry(item).or_insert(Node::new());
            m.n_descendants = indices.len();
            m.children = indices.clone();
            
            return item;
        }

        let mut children: Vec<Node<N>> = Vec::default();
        indices.iter().for_each(|j| {
            if let Some(n) = self._nodes.get(&j) {
                children.push(n.clone());
            }
        });
        let children_indicies = &mut [Vec::new(), Vec::new()];
        let mut m = Node::new();

        Euclidian::create_split(children, N, &mut m);

        for i in 0..indices.len() {
            let j = indices[i];

            if let Some(n) = self._nodes.get(&j) {
                let side = Euclidian::side(m.clone(), n.v, N);
                children_indicies[side as usize].push(j);
            }
        }

        while children_indicies[0].len() == 0 || children_indicies[1].len() == 0 {
            children_indicies[0].clear();
            children_indicies[1].clear();

            for i in 0..indices.len() {
                let j = indices[i];
                children_indicies[random_flip() as usize].push(j)
            }
        }

        let flip = if children_indicies[0].len() > children_indicies[1].len() {
            1
        } else {
            0
        };

        m.n_descendants = indices.len();

        for side in 0..2 {
            let ii = side ^ flip;
            let a = &children_indicies[ii];
            m.children[ii] = self._make_tree(a);
        }
        
        let item = self._n_nodes;
        self._n_nodes = self._n_nodes + 1;
        
        let mut e = self._nodes.entry(item).or_insert(Node::new());
        e.n_descendants = m.n_descendants;
        e.children = m.children;
        e.v = m.v;
        e.a = m.a;

        return item;
    }

    fn _get_all_nns(&mut self, v: &[f64; N], n: usize, k: i64) -> (Vec<i64>, Vec<f64>) {
        let mut q: BinaryHeap<(MinFloat, i64)> = BinaryHeap::new();
        let mut search_k = k.clone();

        if search_k == -1 {
            search_k = (n as i64) * self._roots.len() as i64; 
        }

        for i in 0..self._roots.len() {
            let _a = self._roots[i];
            q.push((MinFloat(std::f64::INFINITY), self._roots[i]))
        }

        let mut nns: Vec<i64> =  Vec::new();
        while nns.len() < (search_k as usize) && !q.is_empty() {
            let top = q.peek().unwrap();
            let d = top.0.0;
            let i = top.1;
            
            let nd = self._nodes.entry(i).or_insert(Node::new());
            q.pop();

            if nd.n_descendants == 1 && i < self._n_items {
                nns.push(i);
            } else if nd.n_descendants <= self._K {
                let dst = nd.children.clone();
                nns.extend(dst);
            } else {
                let margin = Euclidian::margin(nd.clone(), v.clone(), N);

                q.push((MinFloat(d.min(0.0 + margin)), nd.children[1]));
                q.push((MinFloat(d.min(0.0 - margin)), nd.children[0]));
            }
        }

        nns.sort();

        let mut nns_dist: Vec<(f64, i64)> = Vec::new();
        let mut last = -1;

        for i in 0..nns.len() {
            let j = nns[i];
            if j == last {
                continue;
            }

            last = j;
            let mut _n = self._nodes.entry(j).or_insert(Node::new());
            let dist = Euclidian::distance(v, _n.v, N);
            nns_dist.push((dist, j));
        }

        let m = nns_dist.len();

        let p = if n < m {
            n
        } else {
            m
        } as usize;

        nns_dist.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut distances: Vec<f64> = Vec::new();
        let mut result = Vec::new();

        for i in 0..p {
            distances.push(nns_dist[i].0);
            result.push(nns_dist[i].1)
        }

        (result, distances)
    }
}
