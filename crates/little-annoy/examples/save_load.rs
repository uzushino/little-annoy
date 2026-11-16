use little_annoy::{Annoy, Euclidean};
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::mem::DropGuard;
use std::time::{SystemTime, UNIX_EPOCH};

fn print_neighbors(index: &Annoy<f64, Euclidean>) {
    let (ids, distances) = index.get_nns_by_vector(&[1.0, 1.0], 10, -1);
    for (id, distance) in ids.into_iter().zip(distances) {
        println!("item = {id}, distance = {distance}");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut index = Annoy::<f64, Euclidean>::new(2);
    for (id, vector) in [[1.0, 1.0], [5.0, 5.0], [2.0, 2.0], [4.0, 4.0]]
        .iter()
        .enumerate()
    {
        index.add_item(id as i64, vector);
    }
    index.build(10);
    println!("Before saving:");
    print_neighbors(&index);

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = std::env::temp_dir().join(format!(
        "little-annoy-{}-{timestamp}.index",
        std::process::id()
    ));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    let path = DropGuard::new(path, |path| {
        let _ = std::fs::remove_file(path);
    });
    index.save(file)?;

    let mut restored = Annoy::<f64, Euclidean>::new(2);
    restored.load(BufReader::new(File::open(path.as_path())?))?;
    println!("After loading:");
    print_neighbors(&restored);
    Ok(())
}
