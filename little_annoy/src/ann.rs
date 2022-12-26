use crate::distance::{Distance, NodeImpl};
use crate::item::Item;
use crate::Numeric;

use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::io::BufWriter;
use std::marker::PhantomData;
use std::usize;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use async_std::task;
use lockable::*;

use rand::thread_rng;
use rand::Rng;

#[cfg(feature = "parallel_build")]
use rayon::prelude::*;

#[cfg(feature = "parallel_build")]
use rayon::iter::Either;

use bincode;

#[derive(PartialEq, PartialOrd)]
struct AnnResult<T>(T, i64);

impl<T: PartialEq> Eq for AnnResult<T> {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl<T: PartialOrd> Ord for AnnResult<T> {
    fn cmp(&self, other: &AnnResult<T>) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}


pub struct AnnoyThreadBuilder {
}
        

impl AnnoyThreadBuilder {
    pub fn build<T, D>(annoy: Arc<Mutex<&mut Annoy<T, D>>>, n_thread: usize, q: i64) 
        where T: Item + Sync + Send, D: Distance<T>, D::Node: Send + Sync {
        let n_nodes_mutex = Arc::new(Mutex::new(()));
        let shared_nodes_mutex = Arc::new(Mutex::new(()));
        let builder_mutex = Arc::new(Mutex::new(()));

        for thread_idx in 0..n_thread {
            let trees_per_thread = if q == -1 { -1 } else { (q + thread_idx as i64) / n_thread as i64};

            let mu1 = n_nodes_mutex.clone();
            let mu2 = shared_nodes_mutex.clone();
            let mu3 = builder_mutex.clone();

            let ann = annoy.clone();

            task::block_on(async {
                let mut thread_roots = Vec::new();

                loop {
                    /*
                    {
                        let ann = ann.lock().unwrap();
                        let _nodes = &ann._nodes;
                        let _roots = &ann._roots;

                        println!("thread_roots: {}, nodes: {}, roots: {}, thread: {}. tree_per_thread: {}", 
                            thread_roots.len(), _nodes.len(), _roots.len(), thread_idx, trees_per_thread);
                    } */
                    if q == -1 {
                        {
                            let ann = ann.lock().unwrap();
                            let _n_nodes = ann._n_nodes;
                            let _n_items = ann._n_items;

                            mu1.lock().unwrap();
                            if _n_nodes >= _n_items * 2 {
                                break;
                            }
                        }
                    } else {
                        if thread_roots.len() >= (trees_per_thread as usize) {
                            break;
                        }
                    }

                    let mut indices: Vec<i64> = Vec::new();
                    {
                        mu2.lock().unwrap();
                        let ann = ann.lock().unwrap();
                        let _nodes = &ann._nodes;
                        let _n_items = ann._n_items;

                        for i in 0.._n_items {
                            if let Some(n) = _nodes.get(&i) {
                                if n.descendant() >= 1 {
                                    indices.push(i)
                                }
                            }
                        }
                    }

                    {
                        let mut ann = ann.lock().unwrap();
                        let ind = ann._make_tree(true,&indices, &mu1, &mu2, &mu3);
                        thread_roots.push(ind);
                    }
                }

                {
                    mu3.lock().unwrap();
                    let mut ann = ann.lock().unwrap();
                    ann._roots.append(&mut thread_roots);
                }
            });
        }
    }
}


#[allow(non_snake_case)]
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

    pub _seed: Option<u64>,
    pub t: PhantomData<T>,
}

impl<T: Item + Sync + Send, D: Distance<T>> Annoy<T, D> {
    pub fn new(f: usize) -> Self {
        Self {
            _roots: Vec::new(),
            _nodes: HashMap::new(),
            _n_items: 0,
            _n_nodes: 0,
            _f: f,
            _K: 6,
            _seed: None,
            t: PhantomData,
        }
    }

    pub fn set_seed(&mut self, seed: u64) {
        self._seed = Some(seed);
    }

    pub fn add_item(&mut self, item: i64, w: &[T]) {
        let f = self._f;
        let n = self._nodes.entry(item).or_insert(D::Node::new(f));
        n.reset(w);

        if item >= self._n_items {
            self._n_items = item + 1;
        }
    }

    pub fn build(&mut self, q: i64)
    where
        <D as Distance<T>>::Node: Sync + Send,
    {
        self._n_nodes = self._n_items;

        AnnoyThreadBuilder::build(Arc::new(Mutex::new(self)), 5, q);
    }

    pub fn get_nns_by_vector(&self, v: &[T], n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>)
    where
        D: Distance<T>,
    {
        self._get_all_nns(v, n, search_k)
    }

    pub fn get_nns_by_item(&self, item: i64, n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>)
    where
        D: Distance<T>,
    {
        let m = self._nodes.get(&item).unwrap();
        let v = m.vector();

        self._get_all_nns(v, n, search_k)
    }

    #[cfg(not(feature = "parallel_build"))]
    fn random_split_index(
        &self,
        m: &mut D::Node,
        indices: &[i64],
        children: &Vec<&D::Node>,
    ) -> (Vec<i64>, Vec<i64>) {
        let mut rng = thread_rng();
        D::create_split(children, m, self._f, &mut rng);

        let mut children_indices = (Vec::new(), Vec::new());

        for i in indices.iter() {
            if let Some(n) = self._nodes.get(i) {
                let side = D::side(&m, n.vector(), &mut rng);

                if side {
                    children_indices.0.push(*i);
                } else {
                    children_indices.1.push(*i);
                }
            }
        }

        while children_indices.0.is_empty() || children_indices.1.is_empty() {
            children_indices.0.clear();
            children_indices.1.clear();

            indices.iter().for_each(|j| {
                if rng.gen::<bool>() {
                    children_indices.0.push(*j);
                } else {
                    children_indices.1.push(*j);
                }
            });
        }

        children_indices
    }

    #[cfg(feature = "parallel_build")]
    fn random_split_index(
        &self,
        m: &mut D::Node,
        indices: &[i64],
        children: &Vec<&D::Node>,
    ) -> (Vec<i64>, Vec<i64>)
    where
        <D as Distance<T>>::Node: Sync,
    {
        let mut rng = thread_rng();
        D::create_split(children, m, self._f, &mut rng);

        let (mut c1, mut c2): (Vec<i64>, Vec<i64>) = indices
            .par_iter()
            .map_init(
                || rand::thread_rng(),
                |mut rng, id| {
                    if D::side(&m, self._nodes[id].vector(), &mut rng) {
                        Either::Left(*id)
                    } else {
                        Either::Right(*id)
                    }
                },
            )
            .collect();

        while c1.is_empty() || c2.is_empty() {
            c1.clear();
            c2.clear();

            indices.iter().for_each(|j| {
                if rng.gen::<bool>() {
                    c1.push(*j);
                } else {
                    c2.push(*j);
                }
            });
        }

        (c1, c2)
    }

    fn _make_tree(&mut self, is_root: bool, indices: &[i64], nodes_mutex: &Mutex<()>, shared_mutex: &Mutex<()>, mu3: &Mutex<()>) -> i64
    where
        <D as Distance<T>>::Node: Sync,
    {
        if indices.len() == 1 && is_root {
            return indices[0];
        }

        let mut m = D::Node::new(self._f);
        let c = indices.len();
        
        if c <= (self._K as usize) && (!is_root || self._n_items <= (self._K as i64) || indices.len() == 1) {
            let item = {
                nodes_mutex.lock();
                let item = self._n_nodes;
                self._n_nodes += 1;
                item
            };

            {
                shared_mutex.lock();

                if let Some(m) = self._nodes.get_mut(&item) {
                    m.set_descendant(if is_root { self._n_items as usize } else { indices.len() });
                    m.set_children(indices.to_owned());
                }
            }
            
            return item;
        }

        let children_indices= {
            shared_mutex.lock();
            let mut children: Vec<&D::Node> = Vec::default();
            indices.iter().for_each(|index| {
                if let Some(n) = self._nodes.get(index) {
                    children.push(n);
                }
            });
        
            self.random_split_index(&mut m, indices, &children)
        };

        let flip = if children_indices.0.len() > children_indices.1.len() {
            1
        } else {
            0
        };

        m.set_descendant(if is_root { self._n_items as usize } else { indices.len() });

        for side in 0..2 {
            let ii = side ^ flip;
            let a = if ii == 0 {
                &children_indices.0
            } else {
                &children_indices.1
            };

            let mut v = m.children();
            v[ii] = self._make_tree(is_root, a, nodes_mutex, shared_mutex, mu3);

            m.set_children(v);
        }

        let item = {
            nodes_mutex.lock();
            let item = self._n_nodes;
            self._n_nodes += 1;
            item
        };

        {
            shared_mutex.lock();

            let f = self._f;
            let node = self._nodes.entry(item).or_insert_with(|| D::Node::new(f));
            node.copy(m);
        }

        item
    }

    fn _get_all_nns(&self, v: &[T], n: usize, mut search_k: i64) -> (Vec<i64>, Vec<f64>)
    where
        D: Distance<T>,
    {
        let mut nodes = self._nodes.clone();
        let mut q: BinaryHeap<(Numeric<T>, i64)> = BinaryHeap::new();
        let f = self._f;

        if search_k == -1 {
            search_k = (n as i64) * self._roots.len() as i64;
        }

        for root in self._roots.iter() {
            q.push((Numeric(T::zero()), *root))
        }

        let mut nns: Vec<i64> = Vec::new();
        while nns.len() < (search_k as usize) && !q.is_empty() {
            let top = q.peek().unwrap();
            let d: T = top.0 .0;
            let i = top.1;
            let nd = nodes.entry(i).or_insert_with(|| D::Node::new(f));

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

                let a = Numeric(a);
                let b = Numeric(b);

                q.push((Numeric(d).min(a), nd.children()[1]));
                q.push((Numeric(d).min(b), nd.children()[0]));
            }
        }

        nns.sort_unstable();

        let mut nns_dist: BinaryHeap<AnnResult<T>> = BinaryHeap::new();
        let mut last = -1;

        for j in &nns {
            if *j == last {
                continue;
            }

            last = *j;
            let mut _n = nodes.entry(*j).or_insert_with(|| D::Node::new(f));
            let dist = D::distance(v, _n.vector(), self._f);
            nns_dist.push(AnnResult(dist, *j));
        }

        let nns_dist = nns_dist.into_sorted_vec();
        let m = nns_dist.len();
        let p = if n < m { n } else { m };

        let mut distances = Vec::new();
        let mut result = Vec::new();

        for AnnResult(dist, idx) in nns_dist.iter().take(p) {
            distances.push(D::normalized_distance(T::to_f64(dist).unwrap()));
            result.push(*idx)
        }

        (result, distances)
    }

    /*
    pub fn save<W>(&self, w: W)
    where
        W: std::io::Write,
    {
        let mut f = BufWriter::new(w);
        bincode::serialize_into(&mut f, &self._nodes).unwrap();
    }

    pub fn load<R>(&mut self, reader: R) -> bool
    where
        R: std::io::BufRead,
    {
        let mut m = -1;

        self._nodes = bincode::deserialize_from(reader).unwrap();
        self._roots = Vec::default();

        for (i, node) in self._nodes.iter() {
            let k = node.descendant() as i64;

            if m == -1 || k == m {
                self._roots.push(*i);
                m = k;
            } else {
                break;
            }
        }

        self._n_items = m;

        true
    } */

    fn _get(&self, i: i64) -> &D::Node {
        &self._nodes[&i]
    }

    pub fn get_distance(self, i: i64, j: i64) -> f64 {
        let dist = D::distance(self._get(i).vector(), self._get(j).vector(), self._f);
        D::normalized_distance(dist.to_f64().unwrap_or(0.))
    }
}
