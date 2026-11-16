use image_hasher::HasherConfig;
use little_annoy::{Annoy, Hamming};

fn main() {
    let image1 = image::open("data/a.png").expect("Could not open data/a.png");
    let image2 = image::open("data/b.png").expect("Could not open data/b.png");
    let hasher = HasherConfig::new().to_hasher();
    let hash1 = hasher.hash_image(&image1);
    let hash2 = hasher.hash_image(&image2);
    let v1: Vec<i64> = hash1
        .as_bytes()
        .iter()
        .map(|&byte| i64::from(byte))
        .collect();
    let v2: Vec<i64> = hash2
        .as_bytes()
        .iter()
        .map(|&byte| i64::from(byte))
        .collect();
    let mut ann = Annoy::<i64, Hamming>::new(v1.len());
    ann.add_item(1, &v1);
    ann.add_item(2, &v1);
    ann.add_item(3, &v1);
    ann.add_item(4, &v2);
    ann.build(10);
    let (result, distance) = ann.get_nns_by_vector(&v2, 4, -1);
    println!("result: {:?}, distance: {:?}", result, distance);
}
