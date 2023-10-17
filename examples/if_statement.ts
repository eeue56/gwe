import * as fs from "fs/promises";
async function main(): Promise<void> {
  const wasmBytes = await fs.readFile(
    `../gwe_build/examples/if_statement.wasm`
  );

  const importObject = {
    console: { log: (x: any) => console.log(x) },
  };

  const { instance } = await WebAssembly.instantiate(wasmBytes, importObject);

  // false
  (instance.exports.main as Function)(1);

  // true
  (instance.exports.main as Function)(0);
}

main();
