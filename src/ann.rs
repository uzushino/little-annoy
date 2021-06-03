use std::usize;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::distance::{ Euclidean, Distance };
use crate::node::Node;
use crate::{ Numeric, random_flip };

#[derive(Serialize, Deserialize, Debug)]
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
            _n_items: 0,
            _n_nodes: 0,
            _K: 6,
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
            
            let ind = self._make_tree::<Euclidean>(&indices);

            self._roots.push(ind);
        }
    }
    
    pub fn get_nns_by_vector<D>(&mut self, v: [f64; N], n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>) where D: Distance {
        self._get_all_nns::<D>(v, n, search_k) 
    }

    pub fn get_nns_by_item<D>(&mut self, item: i64, n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>) where D: Distance {
        let m = self._nodes.get(&item).unwrap();
        let v = m.v;
        self._get_all_nns::<D>(v, n, search_k) 
    }

    fn _make_tree<D>(&mut self, indices: &Vec<i64>) -> i64 where D: Distance {
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

        let children_indices = &mut [Vec::new(), Vec::new()];
        let mut m = Node::new();
        D::create_split(children, &mut m);

        for i in 0..indices.len() {
            let j = indices[i];

            if let Some(n) = self._nodes.get(&j) {
                let side = D::side(&m, n.v);
                children_indices[side as usize].push(j);
            }
        }

        while children_indices[0].len() == 0 || children_indices[1].len() == 0 {
            children_indices[0].clear();
            children_indices[1].clear();

            indices.into_iter().for_each(|j| children_indices[random_flip() as usize].push(*j));
        }

        let flip = if children_indices[0].len() > children_indices[1].len() {
            1
        } else {
            0
        };

        m.n_descendants = indices.len();

        for side in 0..2 {
            let ii = side ^ flip;
            let a = &children_indices[ii];
            m.children[ii] = self._make_tree::<D>(a);
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

    fn _get_all_nns<D>(&mut self, v: [f64; N], n: usize, mut search_k: i64) -> (Vec<i64>, Vec<f64>) where D: Distance {
        let mut q: BinaryHeap<(Numeric, i64)> = BinaryHeap::new();
        
        if search_k == -1 {
            search_k = (n as i64) * self._roots.len() as i64; 
        }

        for root in self._roots.iter() {
            q.push((Numeric(std::f64::INFINITY), *root))
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
                let margin = D::margin(nd, v);

                q.push((Numeric(d.min(0.0+margin)), nd.children[1]));
                q.push((Numeric(d.min(0.0-margin)), nd.children[0]));
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
            let dist = D::distance(v, _n.v);
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
