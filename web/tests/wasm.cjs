const assert = require("node:assert/strict");
const { test } = require("node:test");
const {
  Ann,
} = require("../../crates/little-annoy-wasm/pkg-node/little_annoy_wasm.js");

test("search, rebuilding and single-item indexes work in WebAssembly", () => {
  const ann = Ann.new(2);
  try {
    ann.add_item(100, [1, 1]);
    ann.build(-1);
    let result = ann.get_nns_by_vector([1, 1], 10, -1);
    try {
      assert.deepEqual(Array.from(result.result()), [100]);
      assert.deepEqual(Array.from(result.distance()), [0]);
    } finally {
      result.free();
    }
    ann.add_item(7, [4, 5]);
    for (const trees of [4, 2]) {
      ann.build(trees);
      result = ann.get_nns_by_vector([1, 1], 10, -1);
      try {
        assert.deepEqual(Array.from(result.result()), [100, 7]);
        assert.deepEqual(Array.from(result.distance()), [0, 5]);
      } finally {
        result.free();
      }
    }
  } finally {
    ann.free();
  }
});

test("invalid JavaScript inputs throw errors before entering the index", () => {
  assert.throws(() => Ann.new(0), /dimensions/);
  const ann = Ann.new(2);
  try {
    assert.throws(() => ann.add_item(-1, [0, 0]), /item ID/);
    assert.throws(() => ann.add_item(0, [0]), /dimension/);
    assert.throws(() => ann.add_item(0, [NaN, 0]), /finite/);
    assert.throws(() => ann.build(0), /tree count/);
    assert.throws(() => ann.get_nns_by_vector([0, 0], -1, -1), /nonnegative/);
    assert.throws(() => ann.get_nns_by_vector([0, 0], 1, -2), /search_k/);
  } finally {
    ann.free();
  }
});
