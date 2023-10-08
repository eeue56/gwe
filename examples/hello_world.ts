import * as fs from "fs/promises";
async function main(): Promise<void> {
  const wasmBytes = await fs.readFile(`../gwe_build/examples/hello_world.wasm`);

  const { instance } = await WebAssembly.instantiate(wasmBytes, {});

  console.log((instance.exports.helloWorld as Function)());
}

main();
