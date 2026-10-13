import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

export const root = dirname(dirname(fileURLToPath(import.meta.url)));
const suffix = process.platform === 'win32' ? '.exe' : '';
export const cargoBin = join(process.env.CARGO_HOME ?? join(homedir(), '.cargo'), 'bin');
const installedRustup = join(cargoBin, `rustup${suffix}`);
const rustup = existsSync(installedRustup) ? installedRustup : 'rustup';
const channel = readFileSync(join(root, 'rust-toolchain.toml'), 'utf8').match(
  /channel\s*=\s*"([^"]+)"/,
)[1];

export function cargo(args, options = {}) {
  return execFileSync(rustup, ['run', channel, 'cargo', ...args], {
    cwd: root,
    stdio: 'inherit',
    ...options,
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href)
  cargo(process.argv.slice(2));
