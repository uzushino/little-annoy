use crate::{Annoy, metrics::Distance, node::Node, scalar::Scalar};
use std::collections::HashMap;
use std::mem::DropGuard;
use std::sync::atomic::{AtomicI64, Ordering::Relaxed};

struct ForestBuilder<T, S> {
    next_node_id: AtomicI64,
    items: HashMap<i64, Node<T, S>>,
}

struct BuiltTrees<T, S> {
    roots: Vec<i64>,
    nodes: HashMap<i64, Node<T, S>>,
}

pub(crate) fn build<T: Scalar, D: Distance<T>>(index: &mut Annoy<T, D>, trees: i64) {
    assert!(
        trees == -1 || trees > 0,
        "tree count must be positive or -1"
    );
    index.clear_trees();
    let mut indices: Vec<_> = index.nodes.keys().copied().collect();
    indices.sort_unstable();
    if indices.is_empty() {
        return;
    }
    let bucket_size = index.bucket_size;
    let builder = ForestBuilder {
        next_node_id: AtomicI64::new(index.next_item_id),
        items: std::mem::take(&mut index.nodes),
    };
    // Workers borrow immutable items and own their output. Unwinding drops
    // partial trees and restores the original data without shared locks.
    let builder = DropGuard::new(builder, |builder| {
        index.nodes = builder.items;
        index.next_node_id = index.next_item_id;
    });
    #[cfg(not(target_arch = "wasm32"))]
    let forest = {
        let workers = if trees == -1 {
            1
        } else {
            std::thread::available_parallelism()
                .map_or(1, usize::from)
                .min(trees as usize)
        };
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..workers)
                .map(|worker| {
                    let count = if trees == -1 {
                        -1
                    } else {
                        trees / workers as i64 + i64::from((worker as i64) < trees % workers as i64)
                    };
                    let builder = &*builder;
                    let indices = &indices;
                    scope.spawn(move || builder.build_trees::<D>(bucket_size, indices, count))
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .unwrap_or_else(|error| std::panic::resume_unwind(error))
                })
                .collect::<Vec<_>>()
        })
    };
    #[cfg(target_arch = "wasm32")]
    let forest = vec![builder.build_trees::<D>(bucket_size, &indices, trees)];
    let builder = DropGuard::dismiss(builder);
    index.next_node_id = builder.next_node_id.load(Relaxed);
    index.nodes = builder.items;
    for tree in forest {
        index.roots.extend(tree.roots);
        index.nodes.extend(tree.nodes);
    }
}

impl<T: Scalar, S> ForestBuilder<T, S> {
    fn build_trees<D: Distance<T, Split = S>>(
        &self,
        bucket_size: usize,
        indices: &[i64],
        trees: i64,
    ) -> BuiltTrees<T, S> {
        let mut forest = BuiltTrees {
            roots: Vec::new(),
            nodes: HashMap::new(),
        };
        let limit = self
            .next_node_id
            .load(Relaxed)
            .saturating_add(indices.len() as i64);
        while if trees == -1 {
            forest.roots.is_empty() || self.next_node_id.load(Relaxed) < limit
        } else {
            forest.roots.len() < trees as usize
        } {
            forest
                .roots
                .push(self.make_tree::<D>(bucket_size, true, indices, &mut forest.nodes));
        }
        forest
    }

    fn make_tree<D: Distance<T, Split = S>>(
        &self,
        bucket_size: usize,
        is_root: bool,
        indices: &[i64],
        nodes: &mut HashMap<i64, Node<T, S>>,
    ) -> i64 {
        if indices.len() == 1 && !is_root {
            return indices[0];
        }
        let node = if indices.len() <= bucket_size {
            Node::Bucket(indices.to_vec())
        } else {
            let mut rng = rand::rng();
            let vectors: Vec<_> = indices
                .iter()
                .map(|id| self.items[id].item().unwrap())
                .collect();
            let plane = D::create_split(&vectors, &mut rng);
            let (mut right, mut left): (Vec<_>, Vec<_>) = indices
                .iter()
                .copied()
                .partition(|id| D::side(&plane, self.items[id].item().unwrap(), &mut rng));
            // Duplicate vectors cannot define a useful separating plane.
            if left.is_empty() || right.is_empty() {
                left = indices[..indices.len() / 2].to_vec();
                right = indices[indices.len() / 2..].to_vec();
            }
            let children = [
                self.make_tree::<D>(bucket_size, false, &left, nodes),
                self.make_tree::<D>(bucket_size, false, &right, nodes),
            ];
            Node::Split { plane, children }
        };
        let id = self
            .next_node_id
            .try_update(Relaxed, Relaxed, |id| id.checked_add(1))
            .expect("node ID capacity exhausted");
        nodes.insert(id, node);
        id
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::Euclidean;
    use rand::rngs::ThreadRng;

    struct FailingSplit;
    impl Distance<f64> for FailingSplit {
        const NAME: &'static str = "failing-split";
        type ValidationError = !;
        type Split = <Euclidean as Distance<f64>>::Split;
        fn validate_split(_: &Self::Split, _: usize) -> bool {
            true
        }
        fn distance(x: &[f64], y: &[f64], f: usize) -> f64 {
            Euclidean::distance(x, y, f)
        }
        fn create_split(_: &[&[f64]], _: &mut ThreadRng) -> Self::Split {
            panic!("worker failed");
        }
        fn margin(plane: &Self::Split, vector: &[f64]) -> f64 {
            Euclidean::margin(plane, vector)
        }
        fn normalized_distance(distance: f64) -> f64 {
            <Euclidean as Distance<f64>>::normalized_distance(distance)
        }
    }
    #[test]
    fn worker_panics_restore_items_and_discard_partial_trees() {
        let mut index = Annoy::<f64, FailingSplit>::new(1);
        for id in 0..10 {
            index.add_item(id, &[id as f64]);
        }
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| index.build(2)));
        assert!(failed.is_err());
        assert_eq!(index.nodes.len(), 10);
        assert_eq!(index.next_node_id, index.next_item_id);
        assert!(!index.is_built());
        assert_eq!(index.get_distance(0, 9), 9.0);
    }
}
