const rust = import("./little_annoy_wasm/pkg");

rust.then((m) => {
  console.log("wasm loaded ...")

  console.log("create ann ...")
  let ann = m.ann_new(2);

  console.log("add item ...")
  m.add_item(ann, 0, [1.0, 1.0]);
  
  console.log("build ...")
  m.build(ann, 100);
});
