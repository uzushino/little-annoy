use little_annoy::{Annoy, Euclidean};
use std::path::Path;
use std::io::Read;

fn main() {
    let mut ann: Annoy<f64, Euclidean> = Annoy::new(2);

    ann.add_item(0, &[1.0, 1.0]);
    ann.add_item(1, &[5.0, 5.0]);
    ann.add_item(2, &[2.0, 2.0]);
    ann.add_item(3, &[4.0, 4.0]);
    ann.build(1000);

    let mut file = std::fs::File::create("/tmp/hoge.db").expect("Could not create temp file");
    println!("Save nodes.");
    let _ = ann.save(file);

    println!("load nodes.");
    let mut ann: Annoy<f64, Euclidean> = Annoy::new(2);
    let _ = ann.load("/tmp/hoge.db");

    let (result, distance) = ann.get_nns_by_vector(&[1.0, 1.0], 10, -1);
    for (i, id) in result.iter().enumerate() {
        println!("result = {}, distance = {}", *id, distance[i]);
    }
}
