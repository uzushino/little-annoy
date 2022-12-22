use little_annoy::{Annoy, Euclidean};

fn main() {
    let mut ann: Annoy<f64, Euclidean> = Annoy::new(2);

    println!("Add an item to the ANN.");
    ann.add_item(0, &[1.0, 1.0]);
    ann.add_item(1, &[5.0, 5.0]);
    ann.add_item(2, &[2.0, 2.0]);
    ann.add_item(3, &[4.0, 4.0]);

    for z in 4..5_000 {
        ann.add_item(z, &[10.0 + z as f64, 10.0 + z as f64]);
    }

    println!("Building index ...");
    ann.build(1000);

    println!("Search the nearest vector in the ANN.");
    let (result, distance) = ann.get_nns_by_vector(&[1.0, 1.0], 10, -1);
    for (i, id) in result.iter().enumerate() {
        println!("result = {}, distance = {}", *id, distance[i]);
    }
}
