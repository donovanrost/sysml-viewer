import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
} from 'node:fs';
import { dirname, join } from 'node:path';
import { tmpdir } from 'node:os';

const hash = (bytes) => createHash('sha256').update(bytes).digest('hex');
const safe = (path) =>
  typeof path === 'string' &&
  path &&
  !path.startsWith('/') &&
  !path.includes('\\') &&
  !path.split('/').some((p) => p === '..' || p === '.' || p === '');

/** Original vendor hashes remain immutable. Every local delta has a pinned
 * patch and is reversed in a temporary tree to verify the original bytes. */
export function verifyVendor(root, { apply = false } = {}) {
  const read = (p) => readFileSync(join(root, p));
  const hashes = JSON.parse(read('vendor/SHA256SUMS.json'));
  const pins = JSON.parse(read('dependencies.json'));
  const manifest = JSON.parse(read('patches/manifest.json'));
  assert.equal(manifest.version, 1);
  assert.equal(manifest.toolkitRevision, pins.toolkit.revision, 'Patches require revision review');
  const inputs = ['patches/manifest.json'];
  const patched = new Set();
  for (const entry of manifest.patches) {
    assert(safe(entry.file) && entry.file.startsWith('vendor/sysml-toolkit/'));
    assert(safe(entry.patch) && entry.patch.startsWith('patches/'));
    assert(!patched.has(entry.file), 'Overlapping patches require a new reviewed combined patch');
    patched.add(entry.file);
    assert(entry.id && entry.reason);
    const addition = entry.baseSha256 === null;
    if (addition) {
      assert(!Object.hasOwn(hashes, entry.file), 'An added file must be absent from upstream');
    } else {
      assert(Object.hasOwn(hashes, entry.file), 'Existing-file patch requires an upstream base');
      assert.equal(
        hashes[entry.file],
        entry.baseSha256,
        'Patch base differs from upstream manifest',
      );
    }
    assert.equal(hash(read(entry.patch)), entry.sha256, 'Patch bytes changed');
    const patch = join(root, entry.patch);
    const numstat = execFileSync('git', ['apply', '--numstat', patch], {
      cwd: root,
      encoding: 'utf8',
    })
      .trim()
      .split('\n');
    assert.equal(numstat.length, 1, 'One patch must touch exactly one declared file');
    assert.equal(numstat[0].split('\t')[2], entry.file, 'Unexpected patch target');
    const present = existsSync(join(root, entry.file));
    if (apply && (addition ? !present : present && hash(read(entry.file)) === entry.baseSha256)) {
      execFileSync('git', ['apply', '--check', patch], { cwd: root });
      execFileSync('git', ['apply', patch], { cwd: root });
    }
    assert(existsSync(join(root, entry.file)), `Patch not applied: ${entry.file}`);
    assert.equal(
      hash(read(entry.file)),
      entry.resultSha256,
      `Patch not applied or vendor changed: ${entry.file}`,
    );
    const temporary = mkdtempSync(join(tmpdir(), 'sysml-patch-'));
    try {
      const target = join(temporary, entry.file);
      mkdirSync(dirname(target), { recursive: true });
      copyFileSync(join(root, entry.file), target);
      execFileSync('git', ['apply', '--reverse', patch], { cwd: temporary, stdio: 'pipe' });
      if (addition) {
        assert(!existsSync(target), 'Reversing an added-file patch must remove the file');
      } else {
        assert.equal(
          hash(readFileSync(target)),
          entry.baseSha256,
          'Patch does not reconstruct upstream bytes',
        );
      }
    } finally {
      rmSync(temporary, { recursive: true, force: true });
    }
    hashes[entry.file] = entry.resultSha256;
    inputs.push(entry.patch);
  }
  const actual = [];
  const walk = (path) => {
    for (const entry of readdirSync(join(root, path), { withFileTypes: true })) {
      const child = `${path}/${entry.name}`;
      if (entry.isDirectory()) walk(child);
      else {
        assert(entry.isFile(), `Unexpected vendor file type: ${child}`);
        actual.push(child);
      }
    }
  };
  walk('vendor/sysml-toolkit');
  walk('vendor/sysml-library');
  assert.deepEqual(actual.sort(), Object.keys(hashes).sort(), 'Vendor inventory changed');
  for (const [path, expected] of Object.entries(hashes))
    assert.equal(hash(read(path)), expected, `Modified vendor file: ${path}`);
  return { hashes, inputs };
}
