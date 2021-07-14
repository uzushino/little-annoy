use crate::distance::{Distance, NodeImpl};
use crate::{random_flip, Numeric};
use serde::Serialize;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::usize;

#[derive(Debug, Serialize)]
pub struct Annoy<T: num::Num, D>
where
    D: Distance<T>,
{
    pub _f: usize,
    pub _K: usize,
    pub _n_nodes: i64,
    pub _n_items: i64,

    pub _nodes: HashMap<i64, D::Node>,
    pub _roots: Vec<i64>,

    pub t: PhantomData<T>,
}

impl<T: num::Num + Copy, D: Distance<T>> Annoy<T, D> {
    pub fn new(f: usize) -> Self {
        Self {
            _roots: Vec::new(),
            _nodes: HashMap::new(),
            _n_items: 0,
            _n_nodes: 0,
            _f: f,
            _K: 6,
            t: PhantomData,
        }
    }

    pub fn add_item(&mut self, item: i64, w: Vec<T>) {
        let n = self._nodes.entry(item).or_insert(D::Node::new());
        n.reset(w);

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
                    if n.descendant() >= 1 {
                        indices.push(i)
                    }
                }
            }

            let ind = self._make_tree(&indices);

            self._roots.push(ind);
        }
    }

    pub fn get_nns_by_vector(&mut self, v: Vec<T>, n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>)
    where
        D: Distance<T>,
    {
        self._get_all_nns(v, n, search_k)
    }

    pub fn get_nns_by_item(&mut self, item: i64, n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>)
    where
        D: Distance<T>,
    {
        let m = self._nodes.get(&item).unwrap();
        let v = m.vector();

        self._get_all_nns(v, n, search_k)
    }

    fn _make_tree(&mut self, indices: &Vec<i64>) -> i64 {
        if indices.len() == 1 {
            return indices[0];
        }

        if indices.len() <= (self._K as usize) {
            let item = self._n_nodes;
            self._n_nodes = self._n_nodes + 1;

            let m = self._nodes.entry(item).or_insert(D::Node::new());
            m.set_descendant(indices.len());
            m.set_children(indices.clone());

            return item;
        }

        let mut children: Vec<D::Node> = Vec::default();
        indices.iter().for_each(|j| {
            if let Some(n) = self._nodes.get(&j) {
                children.push(n.clone());
            }
        });

        let children_indices = &mut [Vec::new(), Vec::new()];
        let mut m = D::Node::new();

        D::create_split(children, &mut m, self._f);

        for i in 0..indices.len() {
            let j = indices[i];

            if let Some(n) = self._nodes.get(&j) {
                let side = D::side(&m, n.vector());
                children_indices[side as usize].push(j);
            }
        }

        while children_indices[0].len() == 0 || children_indices[1].len() == 0 {
            children_indices[0].clear();
            children_indices[1].clear();

            indices
                .into_iter()
                .for_each(|j| children_indices[random_flip() as usize].push(*j));
        }

        let flip = if children_indices[0].len() > children_indices[1].len() {
            1
        } else {
            0
        };

        m.set_descendant(indices.len());

        for side in 0..2 {
            let ii = side ^ flip;
            let a = &children_indices[ii];
            let mut v = m.children();
            v[ii] = self._make_tree(a);
            m.set_children(v);
        }

        let item = self._n_nodes;
        self._n_nodes = self._n_nodes + 1;

        let e = self._nodes.entry(item).or_insert(D::Node::new());
        e.copy(m);

        return item;
    }

    fn _get_all_nns(&mut self, v: Vec<T>, n: usize, mut search_k: i64) -> (Vec<i64>, Vec<f64>)
    where
        D: Distance<T>,
    {
        let mut q: BinaryHeap<(Numeric<f64>, i64)> = BinaryHeap::new();

        if search_k == -1 {
            search_k = (n as i64) * self._roots.len() as i64;
        }

        for root in self._roots.iter() {
            q.push((Numeric(0.0), *root))
        }

        let mut nns: Vec<i64> = Vec::new();
        while nns.len() < (search_k as usize) && !q.is_empty() {
            let top = q.peek().unwrap();
            let d = top.0 .0;
            let i = top.1;

            let nd = self._nodes.entry(i).or_insert(D::Node::new());
            q.pop();

            if nd.descendant() == 1 && i < self._n_items {
                nns.push(i);
            } else if nd.descendant() as usize <= self._K {
                let dst = nd.children();
                nns.extend(dst);
            } else {
                let margin = D::margin(nd, v);

                q.push((Numeric(d.min(0.0 + margin)), nd.children()[1]));
                q.push((Numeric(d.min(0.0 - margin)), nd.children()[0]));
            }
        }

        nns.sort();

        let mut nns_dist = Vec::new();
        let mut last = -1;

        for i in 0..nns.len() {
            let j = nns[i];
            if j == last {
                continue;
            }

            last = j;
            let mut _n = self._nodes.entry(j).or_insert(D::Node::new());
            let dist = D::distance(v, _n.vector(), self._f);
            nns_dist.push((dist, j));
        }

        let m = nns_dist.len();
        let p = if n < m { n } else { m } as usize;

        nns_dist.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut distances = Vec::new();
        let mut result = Vec::new();

        for (dist, idx) in nns_dist.iter().take(p) {
            distances.push(D::normalized_distance(*dist));
            result.push(*idx)
        }

        (result, distances)
    }
}
