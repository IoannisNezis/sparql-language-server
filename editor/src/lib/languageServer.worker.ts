import { init_language_server } from "qlue-ls";

import { BrowserMessageReader, BrowserMessageWriter } from "vscode-languageserver/browser";

// Connection Language-Client <-> Worker
const editorReader = new BrowserMessageReader(self);
const editorWriter = new BrowserMessageWriter(self);


// Connection Worker <-> Language Server(WASM)
const wasmInputStream = new TransformStream();
const wasmOutputStream = new TransformStream();
const wasmReader = wasmOutputStream.readable.getReader();
const wasmWriter = wasmInputStream.writable.getWriter();


// Initialize & start language server
const server = init_language_server(wasmOutputStream.writable.getWriter());
server.listen(wasmInputStream.readable.getReader());


// Language Client -> Language Server
editorReader.listen((data) => {
        // console.log(data);
        wasmWriter.write(JSON.stringify(data));
});

// Forward Language Server -> Language Client
(async () => {
        while (true) {
                const { value, done } = await wasmReader.read();
                if (done) break;
                // console.log(JSON.parse(value));
                editorWriter.write(JSON.parse(value));
        }
})();


console.log("hello from worker");
self.postMessage({ type: "ready" });
export { }
