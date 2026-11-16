import init, { Ann } from "./pkg/little_annoy_wasm.js";

async function main() {
  await init();
  const ann = Ann.new(2);
  try {
    ann.add_item(0, [1.0, 1.0]);
    ann.add_item(1, [5.0, 5.0]);
    ann.add_item(2, [2.0, 2.0]);
    ann.add_item(3, [4.0, 4.0]);
    ann.add_item(4, [10.0, 10.0]);
    ann.build(10);
    const result = ann.get_nns_by_vector([1.0, 1.0], 10, -1);
    try {
      console.log(result.result());
      console.log(result.distance());
    } finally {
      result.free();
    }
  } finally {
    ann.free();
  }
}

main().catch(console.error);
