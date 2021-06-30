const path = require("path");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  entry: {
    app: "./index.js",
  },
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "[name].js",
  },
  plugins: [
    new CleanWebpackPlugin(),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "./little_annoy_wasm/"),
      outDir: path.resolve(__dirname, "./little_annoy_wasm/pkg/"),
    }),
  ],
  experiments: {
    asyncWebAssembly: true
  },
  mode: "development",
};
