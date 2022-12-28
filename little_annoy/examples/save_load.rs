use little_annoy::{Annoy, Euclidean};

fn print_distance(ann: &mut Annoy<f64, Euclidean>) {
    let (result, distance) = ann.get_nns_by_vector(&[1.0, 1.0], 10, -1);
    for (i, id) in result.iter().enumerate() {
        println!("result = {}, distance = {}", *id, distance[i]);
    }
}

fn main() {
       let mut ann: Annoy<f64, Euclidean> = Annoy::new(2);

       ann.add_item(0, &[1.0, 1.0]);
       ann.add_item(1, &[5.0, 5.0]);
       ann.add_item(2, &[2.0, 2.0]);
       ann.add_item(3, &[4.0, 4.0]);
       ann.build(1000);

       println!("Print distance.");
       print_distance(&mut ann);

       let file = std::fs::File::create("/tmp/hoge.db").expect("Could not create temp file");
       println!("Save nodes.");
       let _ = ann.save(file);

       println!();

       println!("load nodes.");
       let mut ann: Annoy<f64, Euclidean> = Annoy::new(2);

       let bin = std::fs::File::open("/tmp/hoge.db").unwrap();
       let _ = ann.load(std::io::BufReader::new(bin));

       print_distance(&mut ann);
}
