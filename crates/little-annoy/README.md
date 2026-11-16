# Little Annoy

An approximate nearest-neighbor index written in Rust, with Euclidean, angular,
Manhattan and Hamming distances and a WebAssembly binding.

Requires the latest stable Rust (minimum **1.100**, Edition 2024). The workspace
uses `rust-toolchain.toml`; run the `cargo` and `rustc` proxies installed by rustup.
If Homebrew Rust appears first in your PATH, put `~/.cargo/bin` before it.

```rust
use little_annoy::{Annoy, Euclidean};

let mut index = Annoy::<f64, Euclidean>::new(2);
index.add_item(0, &[1.0, 1.0]);
index.add_item(7, &[5.0, 5.0]);
index.add_item(42, &[2.0, 2.0]);
index.build(10);

let (ids, distances) = index.get_nns_by_vector(&[1.0, 1.0], 2, -1);
assert_eq!(ids, [0, 42]);
assert_eq!(distances, [0.0, 2.0_f64.sqrt()]);
assert_eq!(index.len(), 3);
```

IDs can be sparse nonnegative integers below `i64::MAX`. Vectors must match the
index dimension and contain finite numbers. Hamming values must additionally be
nonnegative integers representable as `u64`; each coordinate is a packed bit word.
For continuous metrics, prefer `f32` or `f64`.

`build(trees)` replaces the existing forest. `build(-1)` chooses a node budget
based on the item count. Adding or replacing an item invalidates the forest;
build again before searching. Queries return empty results before a build.
`search_k = -1` uses `n * tree_count`; larger budgets improve recall at the cost of
query time. Results are ordered by distance, then item ID when distances tie.

Use `dimensions()`, `len()`, `is_empty()`, `tree_count()` and `is_built()` to
inspect an index. Storage and construction details are private.

Save and load errors are returned to the caller. Loading validates a snapshot
before changing the current index; the receiving dimension must match.

```rust
# use little_annoy::{Annoy, Euclidean};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut index = Annoy::<f64, Euclidean>::new(2);
index.add_item(0, &[1.0, 2.0]);
index.build(4);
let mut bytes = Vec::new();
index.save(&mut bytes)?;

let mut restored = Annoy::<f64, Euclidean>::new(2);
restored.load(bytes.as_slice())?;
assert_eq!(restored.get_nns_by_item(0, 1, -1), (vec![0], vec![0.0]));
# Ok(())
# }
```

Snapshots use a versioned postcard format and include the metric, dimensions,
roots and allocation counters. Old bincode/node-map files are not supported.
Both `save(writer)` and `load(reader)` return `std::io::Result<()>`.

This refactor intentionally breaks compatibility with the previous internal
fields, `ann::AnnoyThreadBuilder`, `try_load` and boolean `load` APIs. Use the
index accessors and handle the result of `load` with `?` or `unwrap`.
