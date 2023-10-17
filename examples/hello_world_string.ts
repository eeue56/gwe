import * as fs from "fs/promises";
async function main(): Promise<void> {
  const wasmBytes = await fs.readFile(
    `../gwe_build/examples/hello_world_string.wasm`
  );

  const memory = new WebAssembly.Memory({ initial: 1 });

  function consoleLogString(offset: number, length: number) {
    const bytes = new Uint8Array(memory.buffer, offset, length);
    const string = new TextDecoder("utf8").decode(bytes);
    console.log(string);
  }

  const importObject = {
    console: {
      log: consoleLogString,
    },
    js: {
      mem: memory,
    },
  };

  const { instance } = await WebAssembly.instantiate(wasmBytes, importObject);

  (instance.exports.main as Function)();
}

main();
