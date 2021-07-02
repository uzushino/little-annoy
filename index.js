const rust = import("./little_annoy_wasm/pkg");

rust.then((m) => {
  console.log("wasm loaded ...")
  window.eucridian = m.eucridian;
});
