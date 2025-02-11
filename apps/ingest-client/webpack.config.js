const path = require("path");
const TsconfigPathsPlugin = require("tsconfig-paths-webpack-plugin");

module.exports = {
  entry: "./src/index.ts",
  devtool: "inline-source-map",
  module: {
    rules: [
      {
        test: /\.ts$/,
        use: {
          loader: "ts-loader",
          options: {
            projectReferences: true,
          },
        },
        exclude: /node_modules/,
      },
    ],
  },
  resolve: {
    extensions: [".tsx", ".ts", ".js"],
    modules: ["node_modules", path.resolve(__dirname)],
    plugins: [
      new TsconfigPathsPlugin({
        extensions: [".tsx", ".ts", ".js"],
        configFile: "src/tsconfig.json",
      }),
    ],
  },
  output: {
    filename: "bundle.js",
    path: path.resolve(__dirname, "dist"),
  },
  context: __dirname,
};
