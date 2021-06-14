use little_annoy::{Annoy, Hamming};

fn main() {
    let mut ann: Annoy<Hamming, 3> = Annoy::new();

    ann.add_item(0, [0.0, 1.0, 1.0]);
    ann.add_item(1, [1.0, 0.0, 1.0]);
    ann.add_item(2, [0.0, 0.0, 1.0]);

    ann.build(100);

    let (result, distance) = ann.get_nns_by_vector([1.0, 0.0, 1.0], 5, -1);

    for (i, id) in result.iter().enumerate() {
        println!("result = {}, distance = {}", *id, distance[i]);
    }
}
