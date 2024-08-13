import { spawn } from 'child_process';
import fs from 'fs/promises';

process.chdir(__dirname);

main().catch(e => {
  console.error(e);
  process.exit(1);
});

async function main() {
  await shell('wasm-pack', ['build', '--target=web']);

  const wasmBinary = await fs.readFile('./pkg/mpz_ts_wasm_bg.wasm');
  const src = `export default '${wasmBinary.toString('base64')}';\n`;
  await fs.unlink('./pkg/mpz_ts_wasm_bg.wasm');
  await fs.writeFile('./pkg/mpz_ts_wasm_base64.js', src);
  await fs.unlink('./pkg/.gitignore');
  await fs.rename('./pkg', '../srcWasm');
}

async function shell(program: string, args: string[]) {
  const child = spawn(program, args, { stdio: 'inherit' });

  await new Promise<void>((resolve, reject) => {
    child.on('exit', code => {
      if (code !== 0) {
        reject(new Error(
          `Failed shell command (code=${code}): ${[program, ...args].join(' ')}`
        ));
      } else {
        resolve();
      }
    });
  });
}
