use bincode;
use futures::future;
use rand::thread_rng;
use rand::Rng;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::io::BufWriter;
use std::marker::PhantomData;
use std::sync::{atomic::AtomicI64, atomic::Ordering::SeqCst, Arc, Mutex, RwLock};
use std::usize;
use tokio::runtime::Builder;

use crate::distance::{Distance, NodeImpl};
use crate::item::Item;
use crate::Numeric;

#[derive(PartialEq, PartialOrd)]
struct AnnResult<T>(T, i64);

impl<T: PartialEq> Eq for AnnResult<T> {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl<T: PartialOrd> Ord for AnnResult<T> {
    fn cmp(&self, other: &AnnResult<T>) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

pub struct AnnoyThreadBuilder<T: Item, D: Distance<T>> {
    n_nodes: AtomicI64,
    nodes: RwLock<HashMap<i64, D::Node>>,
    roots: RwLock<Vec<i64>>,
}

impl<T: Item + Sync + Send, D: Distance<T>> AnnoyThreadBuilder<T, D>
where
    D::Node: Send + Sync + 'static,
{
    pub fn new(n_nodes: i64, nodes: HashMap<i64, D::Node>, roots: Vec<i64>) -> Self {
        Self {
            n_nodes: AtomicI64::new(n_nodes),
            nodes: RwLock::new(nodes),
            roots: RwLock::new(roots),
        }
    }

    pub fn build(annoy: Arc<Mutex<&mut Annoy<T, D>>>, n_thread: usize, q: i64)
    where
        T: Item + Sync + Send + 'static,
        D: Distance<T> + 'static,
        D::Node: Send + Sync,
    {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        let (_nodes, _f, _K, _n_items, _n_nodes, _roots) = {
            let ann = annoy.lock().unwrap();
            (
                ann._nodes.clone(),
                ann._f,
                ann._K,
                ann._n_items,
                ann._n_nodes,
                ann._roots.clone(),
            )
        };

        let thread_policy = Arc::new(Self::new(_n_nodes, _nodes, _roots));
        let mut threads = vec![];

        for thread_idx in 0..n_thread {
            let trees_per_thread = if q == -1 {
                -1
            } else {
                (q + thread_idx as i64) / n_thread as i64
            };
            let thread_policy = Arc::clone(&thread_policy);

            let handle = rt.spawn(async move {
                let mut thread_roots = Vec::new();

                loop {
                    if q == -1 {
                        {
                            if thread_policy.n_nodes.load(SeqCst) >= _n_items * 2 {
                                break;
                            }
                        }
                    } else if thread_roots.len() >= (trees_per_thread as usize) {
                        break;
                    }

                    let mut indices: Vec<i64> = Vec::new();
                    {
                        let _nodes = thread_policy.nodes.read().unwrap();
                        for i in 0.._n_items {
                            if let Some(n) = _nodes.get(&i) {
                                if n.descendant() >= 1 {
                                    indices.push(i)
                                }
                            }
                        }
                    }

                    let ind = _make_tree::<D, T>(&*thread_policy, _f, _K, _n_items, true, &indices);

                    thread_roots.push(ind);
                }

                {
                    let mut _roots = thread_policy.roots.write().unwrap();
                    _roots.append(&mut thread_roots);
                }
            });

            threads.push(handle);
        }

        rt.block_on(future::join_all(threads));

        let mut ann = annoy.lock().unwrap();
        ann._n_nodes = thread_policy.n_nodes.load(SeqCst);
        ann._roots = thread_policy.roots.read().unwrap().clone();
        ann._nodes = thread_policy.nodes.read().unwrap().clone();
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

    pub t: PhantomData<T>,
}

impl<T: Item + Sync + Send + 'static, D: Distance<T>> Annoy<T, D> {
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
        let n = self._nodes.entry(item).or_insert(D::Node::new(f));
        n.reset(w);

        if item >= self._n_items {
            self._n_items = item + 1;
        }
    }

    pub fn build(&mut self, q: i64)
    where
        D: 'static,
        T: 'static,
        <D as Distance<T>>::Node: Sync + Send,
    {
        self._n_nodes = self._n_items;
        AnnoyThreadBuilder::build(Arc::new(Mutex::new(self)), 10, q);
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
        let v = m.as_slice();

        self._get_all_nns(v, n, search_k)
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
            let dist = D::distance(v, _n.as_slice(), self._f);
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
    }

    fn _get(&self, i: i64) -> &D::Node {
        &self._nodes[&i]
    }

    pub fn get_distance(self, i: i64, j: i64) -> f64 {
        let dist = D::distance(self._get(i).as_slice(), self._get(j).as_slice(), self._f);
        D::normalized_distance(dist.to_f64().unwrap_or(0.))
    }
}

fn random_split_index<T, D>(
    _nodes: &HashMap<i64, D::Node>,
    _f: usize,
    m: &mut D::Node,
    indices: &[i64],
    children: &[&D::Node],
) -> (Vec<i64>, Vec<i64>)
where
    T: Item + Sync + Send,
    D: Distance<T>,
{
    let mut rng = thread_rng();
    D::create_split(children, m, _f, &mut rng);

    let mut children_indices = (Vec::new(), Vec::new());

    for i in indices.iter() {
        if let Some(n) = _nodes.get(i) {
            let side = D::side(m, n.as_slice(), &mut rng);

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

fn _make_tree<D, T>(
    thread_policy: &AnnoyThreadBuilder<T, D>,
    _f: usize,
    _K: usize,
    _n_items: i64,
    is_root: bool,
    indices: &[i64],
) -> i64
where
    T: Item + Sync + Send,
    D: Distance<T>,
{
    if indices.len() == 1 && !is_root {
        return indices[0];
    }

    if indices.len() <= (_K as usize) && (!is_root || _n_items <= (_K as i64) || indices.len() == 1)
    {
        let item = {
            let item = thread_policy.n_nodes.load(SeqCst);
            thread_policy.n_nodes.fetch_add(1, SeqCst);
            item
        };

        {
            let mut _nodes = thread_policy.nodes.write().unwrap();
            let m = _nodes.entry(item).or_insert(D::Node::new(_f));
            
            m.set_descendant(if is_root {
                _n_items as usize
            } else {
                indices.len()
            });
           
            m.set_children(indices.to_owned());
        }

        return item;
    }

    let mut m = D::Node::new(_f);
    let children_indices = {
        let _nodes = thread_policy.nodes.read().unwrap();
        let mut children: Vec<&D::Node> = Vec::default();

        indices.iter().for_each(|index| {
            if let Some(n) = _nodes.get(index) {
                children.push(n);
            }
        });

        random_split_index::<T, D>(&_nodes, _f, &mut m, indices, &children)
    };

    let flip = (children_indices.0.len() > children_indices.1.len()) as usize;
    m.set_descendant(if is_root {
        _n_items as usize
    } else {
        indices.len()
    });

    for side in 0..2 {
        let ii = side ^ flip;
        let a = if ii == 0 {
            &children_indices.0
        } else {
            &children_indices.1
        };

        let mut v = m.children();
        v[ii] = _make_tree::<D, T>(thread_policy, _f, _K, _n_items, false, a);

        m.set_children(v);
    }

    let item = {
        let item = thread_policy.n_nodes.load(SeqCst);
        thread_policy.n_nodes.fetch_add(1, SeqCst);
        item
    };

    {
        let mut _nodes = thread_policy.nodes.write().unwrap();
        let n = _nodes.entry(item).or_insert_with(|| D::Node::new(_f));
        n.copy(m);
    }

    item
}
