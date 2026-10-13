import { execFileSync } from 'node:child_process';
import { delimiter, join } from 'node:path';
import { cargoBin, root } from './rust.mjs';

// Put rustup ahead of alternative Cargo installations so rust-toolchain.toml
// controls desktop builds as well as the standalone kernel.
const cli = join(root, 'node_modules/@tauri-apps/cli/tauri.js');
execFileSync(process.execPath, [cli, ...process.argv.slice(2)], {
  cwd: root,
  stdio: 'inherit',
  env: { ...process.env, PATH: `${cargoBin}${delimiter}${process.env.PATH}` },
});
