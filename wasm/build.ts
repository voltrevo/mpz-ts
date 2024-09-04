import { spawn } from 'child_process';
import fs from 'fs/promises';
import path from 'path';
import { glob } from 'glob';
import { rollup } from 'rollup';

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

  await makeWorkerBundle();
}

async function makeWorkerBundle() {
  const srcWasmDir = path.resolve(__dirname, path.join('..', 'srcWasm'));

  const workerSrc = globMatch1(
    `${srcWasmDir}/snippets/**/workerHelpers.worker.js`,
  );

  await replaceInFile(
    workerSrc,
    `from '../../../'`,
    `from '../../../mpz_ts_wasm'`,
  );

  const lines = (await fs.readFile(workerSrc, 'utf-8')).split('\n');

  const newLines = lines.map(
    line => line.replace(`from '../../../'`, `from '../../../mpz_ts_wasm'`),
  );

  await fs.writeFile(workerSrc, newLines.join('\n'));

  const bundle = await rollup({
    input: workerSrc,
  });

  const { output } = await bundle.generate({ format: 'esm' });

  const outputStr = output.map(o => {
    if (o.type === 'chunk') {
      return o.code;
    } else {
      throw new Error('Unexpected output type');
    }
  }).join('\n');

  await fs.writeFile(
    path.resolve(workerSrc, '../workerUrl.js'),
    [
      `const outputStr = ${JSON.stringify(outputStr)};`,
      `const blob = new Blob([outputStr], { type: 'application/javascript' });`,
      `export default URL.createObjectURL(blob);`,
      '',
    ].join('\n'),
  );

  const workerHelpersSrc = globMatch1(
    `${srcWasmDir}/snippets/**/workerHelpers.js`,
  );

  await replaceInFile(
    workerHelpersSrc,
    `new URL('./workerHelpers.worker.js', import.meta.url)`,
    `workerUrl`,
  );

  const workerHelpersContent = await fs.readFile(workerHelpersSrc, 'utf-8');

  await fs.writeFile(
    workerHelpersSrc,
    [
      `import workerUrl from './workerUrl';`,
      '',
      workerHelpersContent,
      '',
    ].join('\n'),
  );
}

async function replaceInFile(
  file: string,
  pattern: string | RegExp,
  replacement: string,
) {
  const content = await fs.readFile(file, 'utf-8');
  const newContent = content.replace(pattern, replacement);
  await fs.writeFile(file, newContent);
}

function globMatch1(pattern: string) {
  const matches = glob.sync(pattern);

  if (matches.length !== 1) {
    throw new Error(`Expected exactly one match for pattern: ${pattern}`);
  }

  return matches[0];
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
