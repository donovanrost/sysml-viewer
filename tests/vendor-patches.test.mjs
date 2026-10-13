import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { createHash } from 'node:crypto';
import { verifyVendor } from '../tools/lib/vendor.mjs';
const hash = (s) => createHash('sha256').update(s).digest('hex');
function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), 'sysml-vendor-test-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const write = (path, value) => {
    mkdirSync(dirname(join(root, path)), { recursive: true });
    writeFileSync(join(root, path), typeof value === 'string' ? value : JSON.stringify(value));
  };
  const file = 'vendor/sysml-toolkit/example.rs';
  const patch = 'patches/fix.patch';
  const before = 'const VALUE: bool = false;\n';
  const after = 'const VALUE: bool = true;\n';
  const delta = `--- a/${file}\n+++ b/${file}\n@@ -1 +1 @@\n-${before}+${after}`;
  const manifest = {
    version: 1,
    toolkitRevision: 'pinned',
    patches: [
      {
        id: 'TEST',
        reason: 'Regression fixture',
        file,
        patch,
        sha256: hash(delta),
        baseSha256: hash(before),
        resultSha256: hash(after),
      },
    ],
  };
  write('dependencies.json', { toolkit: { revision: 'pinned' } });
  write('vendor/SHA256SUMS.json', {
    [file]: hash(before),
    'vendor/sysml-library/source.sysml': hash('package Library;\n'),
  });
  write('vendor/sysml-library/source.sysml', 'package Library;\n');
  write(file, before);
  write(patch, delta);
  write('patches/manifest.json', manifest);
  return { root, write, file, patch, before, after, delta, manifest };
}
test('vendor patches apply reproducibly and preserve the original source manifest', (t) => {
  const f = fixture(t);
  assert.throws(() => verifyVendor(f.root), /Patch not applied/);
  verifyVendor(f.root, { apply: true });
  verifyVendor(f.root, { apply: true });
  assert.equal(readFileSync(join(f.root, f.file), 'utf8'), f.after);
  assert.equal(
    JSON.parse(readFileSync(join(f.root, 'vendor/SHA256SUMS.json')))[f.file],
    hash(f.before),
  );
});
test('vendor checks reject patch edits, source edits and unrecorded files', (t) => {
  const f = fixture(t);
  verifyVendor(f.root, { apply: true });
  f.write(f.patch, f.delta + '\n');
  assert.throws(() => verifyVendor(f.root), /Patch bytes changed/);
  f.write(f.patch, f.delta);
  f.write(f.file, f.after + '// unrecorded\n');
  assert.throws(() => verifyVendor(f.root), /vendor changed/);
  f.write(f.file, f.after);
  f.write('vendor/sysml-toolkit/extra.rs', '');
  assert.throws(() => verifyVendor(f.root), /Vendor inventory changed/);
});
test('refreshing a result hash cannot conceal unpatched edits or a changed base revision', (t) => {
  const f = fixture(t);
  verifyVendor(f.root, { apply: true });
  const changed = f.after + '// unrecorded\n';
  f.write(f.file, changed);
  f.manifest.patches[0].resultSha256 = hash(changed);
  f.write('patches/manifest.json', f.manifest);
  assert.throws(
    () => verifyVendor(f.root),
    /does not reconstruct upstream bytes|patch does not apply/,
  );
  f.manifest.toolkitRevision = 'different';
  f.write('patches/manifest.json', f.manifest);
  assert.throws(() => verifyVendor(f.root), /revision review/);
});
