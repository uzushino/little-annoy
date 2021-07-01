const rust = import("./little_annoy_wasm/pkg");

rust.then((m) => {
  window.eucridian = m.eucridian;
});
