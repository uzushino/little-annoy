import assert from "node:assert/strict";
import { once } from "node:events";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { createDemoServer } from "../scripts/serve.mjs";
import init, { Ann } from "../dist/pkg/little_annoy_wasm.js";

test("browser-target ES modules initialize without a bundler", async () => {
  const bytes = await readFile(
    new URL("../dist/pkg/little_annoy_wasm_bg.wasm", import.meta.url),
  );
  await init({ module_or_path: bytes });
  const index = Ann.new(2);
  try {
    index.add_item(0, [1, 1]);
    index.add_item(7, [4, 5]);
    index.build(2);
    const result = index.get_nns_by_vector([1, 1], 2, -1);
    try {
      assert.deepEqual(Array.from(result.result()), [0, 7]);
      assert.deepEqual(Array.from(result.distance()), [0, 5]);
    } finally {
      result.free();
    }
  } finally {
    index.free();
  }
});

test("static server serves modules and streaming Wasm with the correct content types", async (t) => {
  const server = createDemoServer();
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  t.after(
    () =>
      new Promise((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      ),
  );
  const base = `http://127.0.0.1:${server.address().port}`;
  const html = await fetch(base);
  assert.equal(html.status, 200);
  assert.match(await html.text(), /type="module"/);
  const module = await fetch(`${base}/pkg/little_annoy_wasm.js`);
  assert.match(module.headers.get("content-type"), /javascript/);
  assert.match(await module.text(), /export/);
  const wasm = await fetch(`${base}/pkg/little_annoy_wasm_bg.wasm`);
  assert.equal(wasm.headers.get("content-type"), "application/wasm");
  const compiled = await WebAssembly.compileStreaming(wasm);
  assert.ok(compiled instanceof WebAssembly.Module);
  assert.equal((await fetch(`${base}/package.json`)).status, 404);
});
