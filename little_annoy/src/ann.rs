use crate::distance::{Distance, NodeImpl};
use crate::item::Item;
use crate::{random_flip, Numeric};

use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::usize;

pub struct Annoy<T: Item, D>
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

impl<T: Item, D: Distance<T>> Annoy<T, D> {
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

    pub fn add_item(&mut self, item: i64, w: &[T]) {
        let f = self._f;
        let n = self._nodes.entry(item).or_insert_with(|| D::Node::new(f));
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

    pub fn get_nns_by_vector(&mut self, v: &[T], n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>)
    where
        D: Distance<T>,
    {
        self._get_all_nns(v.to_vec(), n, search_k)
    }

    pub fn get_nns_by_item(&mut self, item: i64, n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>)
    where
        D: Distance<T>,
    {
        let m = self._nodes.get(&item).unwrap();
        let v = m.vector();
        self._get_all_nns(v.to_vec(), n, search_k)
    }

    fn _make_tree(&mut self, indices: &Vec<i64>) -> i64 {
        if indices.len() == 1 {
            return indices[0];
        }

        if indices.len() <= (self._K as usize) {
            let item = self._n_nodes;
            self._n_nodes = self._n_nodes + 1;

            let m = self._nodes.entry(item).or_insert(D::Node::new(self._f));
            m.set_descendant(indices.len());
            m.set_children(indices.clone());

            return item;
        }

        let mut children: Vec<D::Node> = Vec::default();

        indices.iter().for_each(|index| {
            if let Some(n) = self._nodes.get(&index) {
                children.push(n.clone());
            }
        });

        let children_indices = &mut [Vec::new(), Vec::new()];
        let mut m = D::Node::new(self._f);

        D::create_split(&mut children, &mut m, self._f);

        indices.iter().for_each(|index| {
            if let Some(n) = self._nodes.get(&index) {
                let side = D::side(&m, n.vector());
                children_indices[side as usize].push(*index);
            }
        });

        while children_indices[0].is_empty() || children_indices[1].is_empty() {
            children_indices[0].clear();
            children_indices[1].clear();

            indices
                .iter()
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
        self._n_nodes += 1;

        let f = self._f;
        let node = self._nodes.entry(item).or_insert_with(|| D::Node::new(f));
        node.copy(m);

        item
    }

    fn _get_all_nns(&mut self, v: Vec<T>, n: usize, mut search_k: i64) -> (Vec<i64>, Vec<f64>)
    where
        D: Distance<T>,
    {
        let mut q: BinaryHeap<(Numeric<f64>, i64)> = BinaryHeap::new();
        let v = v.as_slice();

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
            let f = self._f;
            let nd = self._nodes.entry(i).or_insert_with(|| D::Node::new(f));

            q.pop();

            if nd.descendant() == 1 && i < self._n_items {
                nns.push(i);
            } else if nd.descendant() as usize <= self._K {
                let dst = nd.children();
                nns.extend(dst);
            } else {
                let margin = D::margin(nd, v);
                let a = T::zero() + margin;
                let b = T::zero() - margin;

                q.push((Numeric(d.min(T::to_f64(&a).unwrap())), nd.children()[1]));
                q.push((Numeric(d.min(T::to_f64(&b).unwrap())), nd.children()[0]));
            }
        }

        nns.sort();

        let mut nns_dist = Vec::new();
        let mut last = -1;
        let f = self._f;

        for j in &nns {
            if *j == last {
                continue;
            }

            last = *j;
            let mut _n = self._nodes.entry(*j).or_insert_with(|| D::Node::new(f));
            let dist = D::distance(v, _n.vector(), self._f);
            nns_dist.push((dist, *j));
        }

        let m = nns_dist.len();
        let p = if n < m { n } else { m } as usize;

        nns_dist.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut distances = Vec::new();
        let mut result = Vec::new();

        for (dist, idx) in nns_dist.iter().take(p) {
            distances.push(D::normalized_distance(T::to_f64(dist).unwrap()));
            result.push(*idx)
        }

        (result, distances)
    }
}
