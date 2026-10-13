import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { createHash } from 'node:crypto';

const hash = (bytes) => createHash('sha256').update(bytes).digest('hex');

/** Fingerprint source/configuration used for qualification, not a claim of binary
 * reproducibility. Vendor bytes are checked, not merely their manifest filename. */
export function kernelInputHash(root) {
  const read = (path) => readFileSync(join(root, path));
  const files = [
    'Cargo.toml',
    'Cargo.lock',
    'rust-toolchain.toml',
    '.cargo/config.toml',
    'dependencies.json',
    'vendor/SHA256SUMS.json',
    'tools/rust.mjs',
    'tools/conformance.mjs',
    'tools/lib/conformance.mjs',
    'tools/lib/kernel-inputs.mjs',
  ];
  const walk = (path) => {
    for (const entry of readdirSync(join(root, path), { withFileTypes: true })) {
      const child = `${path}/${entry.name}`;
      if (entry.isDirectory()) walk(child);
      else if (entry.isFile()) files.push(child);
      else throw new Error(`Unexpected qualification input type: ${child}`);
    }
  };
  walk('crates');
  const vendor = JSON.parse(read('vendor/SHA256SUMS.json'));
  const beforeVendor = files.length;
  walk('vendor');
  assert.deepEqual(
    files.slice(beforeVendor).sort(),
    [...Object.keys(vendor), 'vendor/SHA256SUMS.json'].sort(),
    'Vendor file inventory changed',
  );
  for (const [path, expected] of Object.entries(vendor))
    assert.equal(hash(read(path)), expected, `Vendor input changed: ${path}`);
  return hash(JSON.stringify([...new Set(files)].sort().map((path) => [path, hash(read(path))])));
}
