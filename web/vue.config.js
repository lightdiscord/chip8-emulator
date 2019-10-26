const path = require('path');
const WasmPack = require('@wasm-tool/wasm-pack-plugin');

module.exports = {
    lintOnSave: false,
    configureWebpack: {
        plugins: [
            new WasmPack({
                crateDirectory: path.resolve(__dirname, '..', 'wasm')
            })
        ]
    }
}
