# Little Annoy

`little annoy` is written in pure Rust.

## Usage

```rust
use little_annoy::{Annoy, Euclidean};

fn main() {
    let mut ann: Annoy<f64, Euclidean> = Annoy::new(2);

    ann.add_item(0, &[1.0, 1.0]);
    ann.add_item(1, &[5.0, 5.0]);
    ann.add_item(2, &[2.0, 2.0]);
    ann.add_item(3, &[4.0, 4.0]);

    for z in 4..1_000 {
        ann.add_item(z, &[10.0, 10.0]);
    }

    ann.build(1000);

    let (result, distance) = ann.get_nns_by_vector(&[1.0, 1.0], 10, -1);
    for (i, id) in result.iter().enumerate() {
        println!("result = {}, distance = {}", *id, distance[i]);
    }
}
```

Link: https://github.com/uzushino/little-annoy/blob/main/little_annoy/examples/demo.rs


## Demo

```bash

$ cargo run --example demo

```

## See also

spotify/annoy  
https://github.com/spotify/annoy
