import * as fs from "fs/promises";
async function main(): Promise<void> {
  const wasmBytes = await fs.readFile(
    `../gwe_build/examples/hello_world_console_log.wasm`
  );

  const importObject = {
    console: { log: (x: any) => console.log(x) },
  };

  const { instance } = await WebAssembly.instantiate(wasmBytes, importObject);

  (instance.exports.main as Function)();
}

main();
