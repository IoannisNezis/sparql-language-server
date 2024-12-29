import { BrowserMessageReader, BrowserMessageWriter } from "vscode-languageserver/browser";
import { init_language_server } from "qlue-ls";

// connectrion languageclient <-> this worker
const editorReader = new BrowserMessageReader(self);
const editorWriter = new BrowserMessageWriter(self);

// connectrion this worker <-> Language Server (WASM)
const wasmInputStream = new TransformStream();
const wasmOutputStream = new TransformStream();
const wasmReader = wasmOutputStream.readable.getReader();
const wasmWriter = wasmInputStream.writable.getWriter();

const server = init_language_server(wasmOutputStream.writable.getWriter());

// Forward Language Server -> Language Client
(async () => {
        while (true) {
                const { value, done } = await wasmReader.read();
                if (done) break;
                console.log(JSON.parse(value));
                editorWriter.write(JSON.parse(value));
        }
})();

// Forward Language CLient -> Language Server
editorReader.listen((data) => {
        console.log(data);
        wasmWriter.write(JSON.stringify(data));

});

// start Language Server 
server.listen(wasmInputStream.readable.getReader());

export { }
