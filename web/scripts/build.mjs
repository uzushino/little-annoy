import { spawnSync } from "node:child_process";
import { copyFile, mkdir, rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const dist = new URL("../dist/", import.meta.url);
await rm(dist, { recursive: true, force: true });
await mkdir(dist, { recursive: true });

const build = spawnSync(
  "wasm-pack",
  [
    "build",
    fileURLToPath(new URL("../../crates/little-annoy-wasm/", import.meta.url)),
    "--target",
    "web",
    "--out-dir",
    fileURLToPath(new URL("pkg/", dist)),
    "--out-name",
    "little_annoy_wasm",
    "--release",
    "--no-typescript",
    "--no-pack",
    "--locked",
  ],
  { stdio: "inherit" },
);
if (build.error) throw build.error;
if (build.status !== 0) process.exit(build.status ?? 1);

await Promise.all(
  ["index.html", "index.js"].map((file) =>
    copyFile(new URL(`../${file}`, import.meta.url), new URL(file, dist)),
  ),
);
console.log("Built web/dist (static HTML, JavaScript and WebAssembly).");
