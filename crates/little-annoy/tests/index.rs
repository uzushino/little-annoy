use little_annoy::{Angular, Annoy, Euclidean, Hamming, Manhattan};

#[test]
fn sparse_ids_remain_searchable_after_rebuilding() {
    let mut ann = Annoy::<f64, Euclidean>::new(2);
    ann.add_item(7, &[1.0, 1.0]);
    ann.add_item(100, &[3.0, 3.0]);
    for trees in [4, 2, 8] {
        ann.build(trees);
        assert_eq!(ann.tree_count(), trees as usize);
        let (ids, distances) = ann.get_nns_by_vector(&[1.0, 1.0], 2, 1000);
        assert_eq!(ids, [7, 100]);
        assert_eq!(distances, [0.0, 8.0_f64.sqrt()]);
    }
}

#[test]
fn save_load_preserves_results_and_can_be_rebuilt() {
    let mut ann = Annoy::<f64, Euclidean>::new(2);
    for id in 0..40 {
        ann.add_item(id * 3, &[id as f64, 1.0]);
    }
    ann.build(8);
    let expected = ann.get_nns_by_vector(&[5.0, 1.0], 10, 10000);
    let mut bytes = Vec::new();
    ann.save(&mut bytes).unwrap();
    let mut loaded = Annoy::<f64, Euclidean>::new(2);
    loaded.load(bytes.as_slice()).unwrap();
    assert_eq!(loaded.get_nns_by_vector(&[5.0, 1.0], 10, 10000), expected);
    loaded.build(3);
    assert_eq!(loaded.get_nns_by_vector(&[5.0, 1.0], 10, 10000), expected);
}

#[test]
fn identical_vectors_produce_finite_distances() {
    let mut ann = Annoy::<f64, Euclidean>::new(2);
    for id in 0..30 {
        ann.add_item(id, &[0.0, 0.0]);
    }
    ann.build(4);
    let (ids, distances) = ann.get_nns_by_vector(&[0.0, 0.0], 30, 10000);
    assert_eq!(ids.len(), 30);
    assert!(distances.iter().all(|distance| *distance == 0.0));
}

#[test]
fn empty_and_single_item_indexes_are_searchable() {
    let mut ann = Annoy::<f64, Euclidean>::new(2);
    ann.build(-1);
    assert_eq!(ann.get_nns_by_vector(&[0.0, 0.0], 10, -1), (vec![], vec![]));
    ann.add_item(100, &[2.0, 3.0]);
    ann.build(-1);
    assert_eq!(ann.get_nns_by_item(100, 10, -1), (vec![100], vec![0.0]));
}

#[test]
fn adding_or_replacing_an_item_invalidates_built_trees() {
    let mut ann = Annoy::<f64, Euclidean>::new(1);
    ann.add_item(0, &[0.0]);
    ann.build(4);
    ann.add_item(1, &[1.0]);
    assert!(!ann.is_built());
    ann.build(3);
    ann.add_item(0, &[10.0]);
    ann.build(3);
    assert_eq!(ann.get_nns_by_item(1, 2, 100).0, [1, 0]);
}

#[test]
fn all_metrics_match_exhaustive_search() {
    macro_rules! check {
        ($metric:ty) => {{
            let mut ann = Annoy::<f64, $metric>::new(2);
            for id in 0..24 {
                ann.add_item(id, &[id as f64, (id % 5) as f64]);
            }
            ann.build(6);
            let (ids, distances) = ann.get_nns_by_item(7, 24, 10000);
            let mut expected: Vec<_> = (0..24).map(|id| (ann.get_distance(7, id), id)).collect();
            expected.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            assert_eq!(
                ids,
                expected.iter().map(|entry| entry.1).collect::<Vec<_>>()
            );
            assert_eq!(
                distances,
                expected.iter().map(|entry| entry.0).collect::<Vec<_>>()
            );
        }};
    }
    check!(Euclidean);
    check!(Angular);
    check!(Manhattan);
    check!(Hamming);
}

#[test]
fn invalid_snapshots_do_not_replace_the_index() {
    let mut ann = Annoy::<f64, Euclidean>::new(2);
    ann.add_item(0, &[1.0, 2.0]);
    ann.build(2);
    let expected = ann.get_nns_by_item(0, 1, -1);
    assert!(ann.load(&b"invalid data"[..]).is_err());
    assert!(ann.load(&b"LANN\0\0\0\x01broken"[..]).is_err());
    assert_eq!(ann.get_nns_by_item(0, 1, -1), expected);
    let mut bytes = Vec::new();
    ann.save(&mut bytes).unwrap();
    let mut wrong_dimensions = Annoy::<f64, Euclidean>::new(3);
    assert!(wrong_dimensions.load(bytes.as_slice()).is_err());
    assert!(wrong_dimensions.is_empty());
}

#[test]
fn save_reports_buffer_flush_errors() {
    struct FailingWriter;
    impl std::io::Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("write failed"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let ann = Annoy::<f64, Euclidean>::new(2);
    assert!(ann.save(FailingWriter).is_err());
}

#[test]
fn zero_results_and_zero_search_budget_are_empty() {
    let mut ann = Annoy::<f64, Euclidean>::new(2);
    ann.add_item(0, &[1.0, 2.0]);
    ann.build(2);
    assert_eq!(ann.get_nns_by_item(0, 0, -1), (vec![], vec![]));
    assert_eq!(ann.get_nns_by_item(0, 1, 0), (vec![], vec![]));
}

#[test]
#[should_panic(expected = "vector dimension mismatch")]
fn rejects_wrong_vector_dimensions() {
    Annoy::<f64, Euclidean>::new(2).add_item(0, &[1.0]);
}

#[test]
#[should_panic(expected = "vectors must contain finite numbers")]
fn rejects_nonfinite_vectors() {
    Annoy::<f64, Euclidean>::new(2).add_item(0, &[f64::NAN, 1.0]);
}

#[test]
#[should_panic(expected = "item ID must be nonnegative")]
fn rejects_negative_item_ids() {
    Annoy::<f64, Euclidean>::new(2).add_item(-1, &[1.0, 2.0]);
}

#[test]
#[should_panic(expected = "tree count must be positive or -1")]
fn rejects_invalid_tree_counts() {
    Annoy::<f64, Euclidean>::new(2).build(0);
}

#[test]
#[should_panic(expected = "Hamming values must be nonnegative integers")]
fn rejects_fractional_hamming_values() {
    Annoy::<f64, Hamming>::new(1).add_item(0, &[0.5]);
}

#[test]
#[should_panic(expected = "Hamming values must be nonnegative integers")]
fn rejects_negative_hamming_values() {
    Annoy::<i64, Hamming>::new(1).add_item(0, &[-1]);
}

#[test]
fn hamming_search_prioritizes_the_matching_high_bit() {
    let mut ann = Annoy::<i64, Hamming>::new(1);
    for id in 0..16 {
        ann.add_item(id, &[if id < 8 { 0 } else { 1 << 40 }]);
    }
    ann.build(1);
    let (ids, distances) = ann.get_nns_by_vector(&[1 << 40], 1, 1);
    assert_eq!(ids.len(), 1);
    assert!(ids[0] >= 8);
    assert_eq!(distances, [0.0]);
}

#[test]
fn snapshot_rejects_a_different_distance_metric() {
    let mut ann = Annoy::<f64, Angular>::new(2);
    ann.add_item(0, &[1.0, 2.0]);
    ann.build(1);
    let mut bytes = Vec::new();
    ann.save(&mut bytes).unwrap();
    let mut loaded = Annoy::<f64, Hamming>::new(2);
    assert!(loaded.load(bytes.as_slice()).is_err());
    assert!(loaded.is_empty());
}

#[test]
fn item_count_is_independent_of_sparse_ids_and_tree_nodes() {
    let mut ann = Annoy::<f64, Euclidean>::new(1);
    assert!(ann.is_empty());
    ann.add_item(100, &[1.0]);
    ann.add_item(100, &[2.0]);
    assert_eq!(ann.len(), 1);
    ann.add_item(10, &[3.0]);
    ann.build(4);
    assert_eq!(ann.len(), 2);
    assert_eq!(ann.dimensions(), 1);
    assert_eq!(ann.tree_count(), 4);
}

#[test]
fn truncated_and_trailing_snapshot_data_are_rejected() {
    let mut index = Annoy::<f64, Euclidean>::new(1);
    index.add_item(0, &[1.0]);
    index.build(2);
    let mut bytes = Vec::new();
    index.save(&mut bytes).unwrap();
    assert!(index.load(&bytes[..bytes.len() - 1]).is_err());
    assert_eq!(index.get_nns_by_item(0, 1, -1), (vec![0], vec![0.0]));
    bytes.push(0);
    assert!(index.load(bytes.as_slice()).is_err());
    assert_eq!(index.len(), 1);
}

#[test]
fn hamming_split_indices_are_independent_of_the_scalar_range() {
    let mut index = Annoy::<i8, Hamming>::new(3);
    for id in 0..8 {
        index.add_item(id, &[0, 0, if id < 4 { 0 } else { 2 }]);
    }
    index.build(1);
    let (ids, distances) = index.get_nns_by_vector(&[0, 0, 2], 1, 1);
    assert!(ids[0] >= 4);
    assert_eq!(distances, [0.0]);
}

#[test]
fn every_metric_round_trips_its_split_type() {
    macro_rules! check {
        ($metric:ty) => {{
            let mut index = Annoy::<f64, $metric>::new(2);
            for id in 0..20 {
                index.add_item(id * 3, &[id as f64, (id % 5) as f64]);
            }
            index.build(4);
            let expected = index.get_nns_by_item(21, 20, 10000);
            let mut bytes = Vec::new();
            index.save(&mut bytes).unwrap();
            let mut loaded = Annoy::<f64, $metric>::new(2);
            loaded.load(bytes.as_slice()).unwrap();
            assert_eq!(loaded.get_nns_by_item(21, 20, 10000), expected);
        }};
    }
    check!(Euclidean);
    check!(Angular);
    check!(Manhattan);
    check!(Hamming);
}
