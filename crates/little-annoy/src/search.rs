use crate::Annoy;
use crate::metrics::Distance;
use crate::node::Node;
use crate::scalar::Scalar;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

impl<T: Scalar, D: Distance<T>> Annoy<T, D> {
    pub fn get_nns_by_vector(&self, v: &[T], n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>) {
        self.validate_vector(v);
        self.get_all_nns(v, n, search_k)
    }

    pub fn get_nns_by_item(&self, item: i64, n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>) {
        self.get_all_nns(self.item_vector(item), n, search_k)
    }

    fn get_all_nns(&self, v: &[T], n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>) {
        assert!(search_k >= -1, "search_k must be nonnegative or -1");
        if n == 0 || search_k == 0 || self.roots.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let budget = if search_k == -1 {
            n.saturating_mul(self.roots.len())
        } else {
            search_k as usize
        };
        let mut queue = BinaryHeap::new();
        for &root in &self.roots {
            queue.push((Numeric(T::zero()), root));
        }
        let mut candidates = Vec::new();
        while candidates.len() < budget {
            let Some((priority, id)) = queue.pop() else {
                break;
            };
            match &self.nodes[&id] {
                Node::Item(_) => candidates.push(id),
                Node::Bucket(items) => candidates.extend(items),
                Node::Split { plane, children } => {
                    let margin = D::margin(plane, v);
                    queue.push((priority.min(Numeric(margin)), children[1]));
                    queue.push((priority.min(Numeric(-margin)), children[0]));
                }
            }
        }
        candidates.sort_unstable();
        candidates.dedup();
        let mut distances: Vec<_> = candidates
            .into_iter()
            .map(|id| {
                let distance = D::distance(v, self.item_vector(id), self.dimensions);
                (Numeric(distance), id)
            })
            .collect();
        distances.sort_unstable();
        let mut result = Vec::with_capacity(n.min(distances.len()));
        let mut normalized = Vec::with_capacity(result.capacity());
        for (distance, id) in distances.into_iter().take(n) {
            result.push(id);
            normalized.push(D::normalized_distance(distance.0.to_f64().unwrap()));
        }
        (result, normalized)
    }
}

#[derive(Clone, Copy)]
struct Numeric<T: Scalar>(T);

impl<T: Scalar> PartialEq for Numeric<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<T: Scalar> Eq for Numeric<T> {}

impl<T: Scalar> PartialOrd for Numeric<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Scalar> Ord for Numeric<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or_else(|| {
            self.0
                .partial_cmp(&self.0)
                .is_none()
                .cmp(&other.0.partial_cmp(&other.0).is_none())
        })
    }
}
