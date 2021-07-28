const rust = import("./little_annoy_wasm/pkg");

rust.then((m) => {
  let ann = m.Ann.new(2);
  
  ann.add_item(0, [1.0, 1.0]);
  ann.add_item(1, [5.0, 5.0]);
  ann.add_item(2, [2.0, 2.0]);
  ann.add_item(3, [4.0, 4.0]);
  ann.add_item(4, [10.0, 10.0]);

  ann.build(1000);

  let r = ann.get_nns_by_vector([1.0, 1.0], 10, -1);

  console.log(r.result())
  console.log(r.distance())
});
