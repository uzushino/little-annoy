# Little Annoy

**Approximate nearest-neighbor search in Rust and WebAssembly.**

Build a forest of random projection trees, then search for nearby vectors by
item ID or by vector. Little Annoy supports native applications and a small
browser demo built with standard ES modules.

[Quickstart](#quickstart) · [Rust API](crates/little-annoy/README.md) · [Browser demo](#browser-demo) · [Development](#development)

- **Four distance metrics:** Euclidean, angular, Manhattan and Hamming.
- **Native parallel builds:** scoped threads share immutable input and build their own trees.
- **Read-only queries:** searches borrow the index without copying its storage.
- **Persistence:** versioned postcard snapshots, validated before replacing an index.
- **Small browser setup:** `wasm-pack`, ES modules and a local static server.

## Quickstart

### Requirements

| Tool      | Version                     | Used for                            |
| --------- | --------------------------- | ----------------------------------- |
| Rust      | 1.100+ stable, Edition 2024 | Library, examples and Wasm          |
| Node.js   | 22.15+                      | Browser demo tooling and tests      |
| pnpm      | 12.18.0                     | JavaScript dependencies and scripts |
| wasm-pack | Current release             | Building the Wasm bindings          |

The Rust toolchain is selected by `rust-toolchain.toml`; pnpm is pinned in
`web/package.json`. Use the `cargo` and `rustc` proxies installed by rustup.
If Homebrew Rust is earlier in your PATH, put `~/.cargo/bin` first.

From the repository root:

```sh
cargo run --release -p little-annoy --example demo
```

## Rust usage

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
```

IDs may be sparse. Vectors must match the index dimension and contain finite
numbers. Adding or replacing an item invalidates the forest; call `build` again
before searching. `build(-1)` chooses a node budget based on the item count.

| Metric      | Distance                                                               |
| ----------- | ---------------------------------------------------------------------- |
| `Euclidean` | Euclidean distance                                                     |
| `Angular`   | `sqrt(2 - 2 * cosine_similarity)`                                      |
| `Manhattan` | Sum of absolute coordinate differences                                 |
| `Hamming`   | Number of differing bits across packed nonnegative integer coordinates |

For continuous metrics, use `f32` or `f64`. `search_k = -1` uses
`n * tree_count`; a larger search budget trades query time for recall.

See the [API guide](crates/little-annoy/README.md) for lifecycle rules, snapshot
loading and the available accessors. The [examples](crates/little-annoy/examples)
include a save/load round trip and perceptual image-hash matching.

## Browser demo

Install the requirements above, then run:

```sh
cargo install wasm-pack --locked
cd web
pnpm install --frozen-lockfile --ignore-scripts
pnpm run build
pnpm run serve
```

Open [http://127.0.0.1:8080](http://127.0.0.1:8080) and check the browser console
for the neighbor IDs and distances. Set `PORT` to use a different port.

The build runs `wasm-pack --target web` and writes a static site to `web/dist/`.
The browser loads the generated ES module and initializes the Wasm directly.
Webpack and other JavaScript bundlers are not required. JavaScript has a single
development dependency: Prettier.

After editing Rust or JavaScript, run `pnpm run build` again and refresh the page.
The demo also shows how to release Wasm objects with `.free()`.

## Project layout

```text
crates/
  little-annoy/       Rust library, examples, benchmarks and regression tests
  little-annoy-wasm/  JavaScript bindings and boundary validation
web/
  index.html         Browser demo
  index.js           Native ES module entry point
  scripts/           Wasm build and static server
  tests/             Browser-target and Node.js Wasm integration tests
```

The Rust library separates item storage (`index`), tree construction (`build`),
traversal and ranking (`search`), snapshots (`persistence`) and distance
strategies (`metrics`). Typed `Item`, `Bucket` and `Split` nodes encode the tree
structure; each distance strategy defines its own split type.

Rust 1.100's `!` type marks infallible value validation. `DropGuard` restores the
registered items if a build worker panics. Browser Wasm builds run sequentially.

## Development

Rust checks, from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
```

Web checks, from `web/`:

```sh
pnpm run format:check
pnpm run build
pnpm run test:browser
pnpm run test:wasm
pnpm audit
```

`test:browser` exercises the browser-target ES module and checks static asset
serving, including the Wasm content type. `test:wasm` exercises the Node.js
bindings. Both use Node's built-in test runner. [CI](.github/workflows/ci.yml)
runs Rust checks and builds and tests the demo from the frozen pnpm lockfile.

Snapshots use the current postcard format. Previous bincode/node-map files and
old internal APIs are intentionally unsupported. `save` and `load` return
`std::io::Result<()>`.

## License

[Apache-2.0](LICENSE). Inspired by [Spotify Annoy](https://github.com/spotify/annoy).
