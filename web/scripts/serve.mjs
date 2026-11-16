import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const assets = new Map([
  ["/", ["index.html", "text/html; charset=utf-8"]],
  ["/index.html", ["index.html", "text/html; charset=utf-8"]],
  ["/index.js", ["index.js", "text/javascript; charset=utf-8"]],
  [
    "/pkg/little_annoy_wasm.js",
    ["pkg/little_annoy_wasm.js", "text/javascript; charset=utf-8"],
  ],
  [
    "/pkg/little_annoy_wasm_bg.wasm",
    ["pkg/little_annoy_wasm_bg.wasm", "application/wasm"],
  ],
]);

export function createDemoServer() {
  return createServer(async (request, response) => {
    if (request.method !== "GET" && request.method !== "HEAD") {
      response.writeHead(405, { Allow: "GET, HEAD" }).end();
      return;
    }
    const asset = assets.get(new URL(request.url, "http://localhost").pathname);
    if (!asset) {
      response.writeHead(404).end("Not found");
      return;
    }
    try {
      const [file, contentType] = asset;
      const body = await readFile(new URL(`../dist/${file}`, import.meta.url));
      response.writeHead(200, {
        "Content-Type": contentType,
        "Content-Length": body.byteLength,
        "Cache-Control": "no-store",
      });
      response.end(request.method === "HEAD" ? undefined : body);
    } catch (error) {
      response
        .writeHead(error.code === "ENOENT" ? 404 : 500)
        .end("Could not read asset");
    }
  });
}

if (resolve(process.argv[1] ?? "") === fileURLToPath(import.meta.url)) {
  const server = createDemoServer();
  server.on("error", (error) => {
    console.error(error);
    process.exitCode = 1;
  });
  server.listen(Number(process.env.PORT ?? 8080), "127.0.0.1", () => {
    console.log(`Demo: http://127.0.0.1:${server.address().port}`);
  });
}
