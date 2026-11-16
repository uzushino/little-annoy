use crate::metrics::Distance;
use crate::node::Node;
use crate::scalar::Scalar;
use std::collections::HashMap;

/// A forest of random projection trees for approximate nearest-neighbor search.
pub struct Annoy<T: Scalar, D: Distance<T>> {
    pub(crate) dimensions: usize,
    pub(crate) bucket_size: usize,
    pub(crate) item_count: usize,
    pub(crate) next_item_id: i64,
    pub(crate) next_node_id: i64,
    pub(crate) nodes: HashMap<i64, Node<T, D::Split>>,
    pub(crate) roots: Vec<i64>,
}

impl<T: Scalar, D: Distance<T>> Annoy<T, D> {
    /// Create an empty index with a positive vector dimension.
    pub fn new(dimensions: usize) -> Self {
        assert!(dimensions > 0, "dimensions must be positive");
        Self {
            dimensions,
            bucket_size: 6,
            item_count: 0,
            next_item_id: 0,
            next_node_id: 0,
            nodes: HashMap::new(),
            roots: Vec::new(),
        }
    }

    pub(crate) fn validate_vector(&self, vector: &[T]) {
        assert_eq!(vector.len(), self.dimensions, "vector dimension mismatch");
        assert!(
            vector
                .iter()
                .all(|value| value.to_f64().is_some_and(f64::is_finite)),
            "vectors must contain finite numbers"
        );
        D::validate_values(vector).unwrap_or_else(|error| panic!("{error}"));
    }

    /// Insert or replace an item and invalidate previously built trees.
    pub fn add_item(&mut self, id: i64, vector: &[T]) {
        assert!(
            (0..i64::MAX).contains(&id),
            "item ID must be nonnegative and below i64::MAX"
        );
        self.validate_vector(vector);
        self.clear_trees();
        if self.nodes.insert(id, Node::Item(vector.to_vec())).is_none() {
            self.item_count += 1;
        }
        self.next_item_id = self.next_item_id.max(id + 1);
        self.next_node_id = self.next_item_id;
    }

    pub(crate) fn clear_trees(&mut self) {
        if !self.roots.is_empty() {
            self.nodes.retain(|_, node| matches!(node, Node::Item(_)));
            self.roots.clear();
        }
        self.next_node_id = self.next_item_id;
    }

    /// Replace the forest. `-1` chooses a node budget based on the item count.
    pub fn build(&mut self, trees: i64) {
        crate::build::build(self, trees);
    }
    /// Number of coordinates in each vector.
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }
    /// Number of registered vectors, independent of sparse IDs and tree nodes.
    pub fn len(&self) -> usize {
        self.item_count
    }
    pub fn is_empty(&self) -> bool {
        self.item_count == 0
    }
    /// Number of trees in the currently built forest.
    pub fn tree_count(&self) -> usize {
        self.roots.len()
    }
    pub fn is_built(&self) -> bool {
        !self.roots.is_empty()
    }

    pub(crate) fn item_vector(&self, id: i64) -> &[T] {
        self.nodes
            .get(&id)
            .and_then(Node::item)
            .expect("unknown item ID")
    }

    pub fn get_distance(&self, first: i64, second: i64) -> f64 {
        let distance = D::distance(
            self.item_vector(first),
            self.item_vector(second),
            self.dimensions,
        );
        D::normalized_distance(distance.to_f64().unwrap())
    }
}
