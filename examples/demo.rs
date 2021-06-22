use little_annoy::{Annoy, Euclidean};

fn main() {
    let mut ann: Annoy<f64, Euclidean, 2> = Annoy::new();

    ann.add_item(0, [1.0, 1.0]);
    ann.add_item(1, [5.0, 5.0]);
    ann.add_item(2, [2.0, 2.0]);
    ann.add_item(3, [4.0, 4.0]);

    for z in 4..10 {
        ann.add_item(z, [10.0, 10.0]);
    }

    ann.build(100);

    let (result, distance) = ann.get_nns_by_vector([1.0, 1.0], 5, -1);

    for (i, id) in result.iter().enumerate() {
        println!("result = {}, distance = {}", *id, distance[i]);
    }
}
