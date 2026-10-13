import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { cargo, cargoBin, root } from './rust.mjs';

const suffix = process.platform === 'win32' ? '.exe' : '';
const installed = join(cargoBin, `wasm-bindgen${suffix}`);
const bindgen = process.env.WASM_BINDGEN ?? (existsSync(installed) ? installed : 'wasm-bindgen');
const version = readFileSync(join(root, 'crates/sysml-wasm/Cargo.toml'), 'utf8').match(
  /wasm-bindgen\s*=\s*"=([^"]+)"/,
)[1];
if (
  execFileSync(bindgen, ['--version'], { encoding: 'utf8' }).trim() !== `wasm-bindgen ${version}`
) {
  throw new Error(
    `Install wasm-bindgen-cli ${version}: cargo install wasm-bindgen-cli --version ${version} --locked`,
  );
}
cargo(['build', '--locked', '--release', '-p', 'sysml-wasm', '--target', 'wasm32-unknown-unknown']);
mkdirSync(join(root, 'packages/kernel-wasm'), { recursive: true });
execFileSync(
  bindgen,
  [
    '--target',
    'web',
    '--out-dir',
    join(root, 'packages/kernel-wasm'),
    join(root, 'target/wasm32-unknown-unknown/release/sysml_wasm.wasm'),
  ],
  { stdio: 'inherit' },
);
