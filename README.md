# Little Annoy

`little annoy` is written in pure Rust.

## Usage

```rust
use little_annoy::{ Annoy, Euclidean };

fn main() {
    let mut ann = Annoy::new();
    
    ann.add_item(0, [1.0, 1.0]);
    ann.add_item(1, [5.0, 5.0]);
    ann.add_item(2, [2.0, 2.0]);
    ann.add_item(3, [4.0, 4.0]);

    for z in 4..10 {
        ann.add_item(z, [10.0, 10.0]);
    }

    ann.build(100);

    let (result, distance) = ann.get_nns_by_vector::<Euclidean>([1.0, 1.0], 5, -1);
   
    for (i, id) in result.iter().enumerate() {
        println!("result = {}, distance = {}", *id, distance[i]);
    }
}
```

Link: https://github.com/uzushino/little-annoy/blob/main/examples/demo.rs


## Demo

- mnist

1. Dowloand mnist data from http://yann.lecun.com/exdb/mnist/.
2. Unzip the mnist data and extract it to `data` directory. 
3. cargo run --example mnist

## See also

spotify/annoy  
https://github.com/spotify/annoy
