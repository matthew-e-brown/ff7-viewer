const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');


const dist = path.resolve(__dirname, './dist');
const static = path.resolve(__dirname, './www');


/** @type {import('webpack').Configuration} */
const config = {
    mode: 'production',
    entry: {
        bootstrap: './bootstrap.js',
    },
    output: {
        path: dist,
        filename: '[name].js',  // no hash so we can reference easily from HTML
    },
    devServer: {
        static: {
            directory: dist,
        }
    },
    plugins: [
        new CopyWebpackPlugin({
            patterns: [
                { from: static, to: dist },
            ],
        }),
        new WasmPackPlugin({
            crateDirectory: __dirname,
        }),
    ],
    experiments: {
        asyncWebAssembly: true,
    }
};

module.exports = config;
