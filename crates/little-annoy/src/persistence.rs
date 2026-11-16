use crate::{Annoy, metrics::Distance, node::Node, scalar::Scalar};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{self, BufWriter, Read, Write};

const MAGIC: &[u8; 8] = b"LANN\0\0\0\x02";

#[derive(Serialize, Deserialize)]
struct Snapshot<R, N> {
    metric: String,
    dimensions: usize,
    bucket_size: usize,
    next_item: i64,
    next_node: i64,
    roots: R,
    nodes: N,
}

type OwnedSnapshot<T, S> = Snapshot<Vec<i64>, HashMap<i64, Node<T, S>>>;

impl<T: Scalar, D: Distance<T>> Annoy<T, D> {
    /// Save the current format. I/O and encoding errors are returned.
    pub fn save<W: Write>(&self, writer: W) -> io::Result<()> {
        let snapshot = Snapshot {
            metric: D::NAME.to_owned(),
            dimensions: self.dimensions,
            bucket_size: self.bucket_size,
            next_item: self.next_item_id,
            next_node: self.next_node_id,
            roots: &self.roots,
            nodes: &self.nodes,
        };
        let payload = postcard::to_stdvec(&snapshot).map_err(invalid_data)?;
        let mut writer = BufWriter::new(writer);
        writer.write_all(MAGIC)?;
        writer.write_all(&payload)?;
        writer.flush()
    }

    /// Replace the index after validating a current-format snapshot.
    /// On error the index remains unchanged. Legacy formats are not accepted.
    pub fn load<R: Read>(&mut self, mut reader: R) -> io::Result<()> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        let payload = bytes
            .strip_prefix(MAGIC)
            .ok_or_else(|| invalid_data("unsupported snapshot format"))?;
        let (snapshot, remaining): (OwnedSnapshot<T, D::Split>, _) =
            postcard::take_from_bytes(payload).map_err(invalid_data)?;
        if !remaining.is_empty() {
            return Err(invalid_data("trailing snapshot data"));
        }
        validate::<T, D>(&snapshot, self.dimensions)?;
        *self = Self {
            item_count: snapshot
                .nodes
                .values()
                .filter(|node| matches!(node, Node::Item(_)))
                .count(),
            dimensions: snapshot.dimensions,
            bucket_size: snapshot.bucket_size,
            next_item_id: snapshot.next_item,
            next_node_id: snapshot.next_node,
            roots: snapshot.roots,
            nodes: snapshot.nodes,
        };
        Ok(())
    }
}

fn validate<T: Scalar, D: Distance<T>>(
    snapshot: &OwnedSnapshot<T, D::Split>,
    dimensions: usize,
) -> io::Result<()> {
    if snapshot.metric != D::NAME {
        return Err(invalid_data("distance metric mismatch"));
    }
    if snapshot.dimensions != dimensions {
        return Err(invalid_data("vector dimension mismatch"));
    }
    let next_item = snapshot
        .nodes
        .iter()
        .filter(|(_, node)| matches!(node, Node::Item(_)))
        .map(|(&id, _)| id)
        .max()
        .unwrap_or(-1)
        .checked_add(1);
    let next_node = snapshot
        .nodes
        .keys()
        .copied()
        .max()
        .unwrap_or(-1)
        .checked_add(1);
    if snapshot.bucket_size < 2
        || next_item != Some(snapshot.next_item)
        || next_node != Some(snapshot.next_node)
        || snapshot.next_item < 0
    {
        return Err(invalid_data("invalid allocation metadata"));
    }
    for (&id, node) in &snapshot.nodes {
        if id < 0 {
            return Err(invalid_data("invalid node ID"));
        }
        let children: &[i64] = match node {
            Node::Item(vector) => {
                if vector.len() != dimensions
                    || D::validate_values(vector).is_err()
                    || vector
                        .iter()
                        .any(|value| !value.to_f64().is_some_and(f64::is_finite))
                {
                    return Err(invalid_data("invalid item vector"));
                }
                continue;
            }
            Node::Bucket(children) => {
                if children.is_empty()
                    || children.len() > snapshot.bucket_size
                    || children
                        .iter()
                        .any(|child| !matches!(snapshot.nodes.get(child), Some(Node::Item(_))))
                {
                    return Err(invalid_data("invalid leaf bucket"));
                }
                children
            }
            Node::Split { plane, children } => {
                if !D::validate_split(plane, dimensions) {
                    return Err(invalid_data("invalid split plane"));
                }
                children
            }
        };
        // Parents follow children in allocation order, which also rules out cycles.
        if id < snapshot.next_item
            || children
                .iter()
                .any(|child| *child >= id || !snapshot.nodes.contains_key(child))
        {
            return Err(invalid_data("invalid child reference"));
        }
    }
    let mut reachable = HashSet::new();
    let mut pending = snapshot.roots.clone();
    if pending.iter().any(|id| {
        !matches!(
            snapshot.nodes.get(id),
            Some(Node::Bucket(_) | Node::Split { .. })
        )
    }) {
        return Err(invalid_data("invalid root node"));
    }
    while let Some(id) = pending.pop() {
        if !reachable.insert(id) {
            continue;
        }
        match &snapshot.nodes[&id] {
            Node::Item(_) => {}
            Node::Bucket(children) => pending.extend(children),
            Node::Split { children, .. } => pending.extend(children),
        }
    }
    if if snapshot.roots.is_empty() {
        snapshot
            .nodes
            .values()
            .any(|node| !matches!(node, Node::Item(_)))
    } else {
        reachable.len() != snapshot.nodes.len()
    } {
        return Err(invalid_data("unreachable index nodes"));
    }
    Ok(())
}

fn invalid_data(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Euclidean, Hamming};

    #[test]
    fn invalid_graph_references_leave_the_current_index_intact() {
        let mut corrupt = Annoy::<f64, Euclidean>::new(1);
        corrupt.add_item(0, &[1.0]);
        corrupt.build(1);
        let root = corrupt.roots[0];
        corrupt.nodes.insert(root, Node::Bucket(vec![root]));
        let mut bytes = Vec::new();
        corrupt.save(&mut bytes).unwrap();
        let mut current = Annoy::<f64, Euclidean>::new(1);
        current.add_item(42, &[2.0]);
        current.build(1);
        assert!(current.load(bytes.as_slice()).is_err());
        assert_eq!(current.get_nns_by_item(42, 1, -1), (vec![42], vec![0.0]));
    }

    #[test]
    fn invalid_hamming_split_positions_are_rejected() {
        let mut index = Annoy::<i64, Hamming>::new(1);
        for id in 0..8 {
            index.add_item(id, &[id]);
        }
        index.build(1);
        let root = index.roots[0];
        if let Node::Split { plane, .. } = index.nodes.get_mut(&root).unwrap() {
            *plane = usize::MAX;
        } else {
            panic!("expected split root");
        }
        let mut bytes = Vec::new();
        index.save(&mut bytes).unwrap();
        assert!(
            Annoy::<i64, Hamming>::new(1)
                .load(bytes.as_slice())
                .is_err()
        );
    }
}
