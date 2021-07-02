const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
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
    new HtmlWebpackPlugin({ template: "index.html" }),
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
