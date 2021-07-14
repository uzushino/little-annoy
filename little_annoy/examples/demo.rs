use little_annoy::{Annoy, Euclidean};

fn main() {
    let mut ann: Annoy<f64, Euclidean> = Annoy::new(2);

    println!("Add an item to the ANN.");
    ann.add_item(0, vec![1.0, 1.0]);
    ann.add_item(1, vec![5.0, 5.0]);
    ann.add_item(2, vec![2.0, 2.0]);
    ann.add_item(3, vec![4.0, 4.0]);

    for z in 4..1_000 {
        ann.add_item(z, vec![10.0, 10.0]);
    }

    println!("Building index ...");
    ann.build(1000);

    println!("Search the nearest vector in the ANN.");
    let (result, distance) = ann.get_nns_by_vector(vec![1.0, 1.0], 10, -1);
    for (i, id) in result.iter().enumerate() {
        println!("result = {}, distance = {}", *id, distance[i]);
    }
}
