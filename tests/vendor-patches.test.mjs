import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
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
test('vendor verification preserves exact bytes with Windows Git line-ending settings', (t) => {
  const f = additionFixture(t);
  const module = new URL('../tools/lib/vendor.mjs', import.meta.url).href;
  execFileSync(
    process.execPath,
    [
      '--input-type=module',
      '-e',
      `import { verifyVendor } from ${JSON.stringify(module)};
       verifyVendor(${JSON.stringify(f.root)}, { apply: true });
       verifyVendor(${JSON.stringify(f.root)});`,
    ],
    {
      env: {
        ...process.env,
        GIT_CONFIG_COUNT: '2',
        GIT_CONFIG_KEY_0: 'core.autocrlf',
        GIT_CONFIG_VALUE_0: 'true',
        GIT_CONFIG_KEY_1: 'core.eol',
        GIT_CONFIG_VALUE_1: 'crlf',
      },
      stdio: 'pipe',
    },
  );
  assert.equal(readFileSync(join(f.root, f.file), 'utf8'), f.after);
  assert.equal(readFileSync(join(f.root, f.added.file), 'utf8'), f.added.after);
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

function additionFixture(t) {
  const f = fixture(t);
  const file = 'vendor/sysml-toolkit/new/module.rs';
  const patch = 'patches/add.patch';
  const after = 'pub fn added() {}\n';
  const delta = `diff --git a/${file} b/${file}\nnew file mode 100644\n--- /dev/null\n+++ b/${file}\n@@ -0,0 +1 @@\n+${after}`;
  const entry = {
    id: 'ADDITION',
    reason: 'New independently maintained module',
    file,
    patch,
    sha256: hash(delta),
    baseSha256: null,
    resultSha256: hash(after),
  };
  f.manifest.patches.push(entry);
  f.write(patch, delta);
  f.write('patches/manifest.json', f.manifest);
  return { ...f, added: { file, patch, after, delta, entry } };
}

test('new-file patches verify absence of an upstream base and enter the effective inventory', (t) => {
  const f = additionFixture(t);
  const before = readFileSync(join(f.root, 'vendor/SHA256SUMS.json'));
  const result = verifyVendor(f.root, { apply: true });
  assert.equal(readFileSync(join(f.root, f.added.file), 'utf8'), f.added.after);
  assert.equal(result.hashes[f.added.file], hash(f.added.after));
  assert(result.inputs.includes(f.added.patch));
  assert.deepEqual(readFileSync(join(f.root, 'vendor/SHA256SUMS.json')), before);
  assert.deepEqual(verifyVendor(f.root, { apply: true }), result);
  rmSync(join(f.root, f.added.file));
  assert.throws(() => verifyVendor(f.root), /Patch not applied/);
  verifyVendor(f.root, { apply: true });
});

test('new-file patches cannot conceal edits by refreshing only the result hash', (t) => {
  const f = additionFixture(t);
  verifyVendor(f.root, { apply: true });
  const changed = f.added.after + '// unrecorded\n';
  f.write(f.added.file, changed);
  assert.throws(() => verifyVendor(f.root), /vendor changed/);
  f.added.entry.resultSha256 = hash(changed);
  f.write('patches/manifest.json', f.manifest);
  assert.throws(() => verifyVendor(f.root), /patch does not apply/);
});

test('null is required for additions and forbidden for an upstream file', (t) => {
  const f = additionFixture(t);
  delete f.added.entry.baseSha256;
  f.write('patches/manifest.json', f.manifest);
  assert.throws(() => verifyVendor(f.root, { apply: true }), /requires an upstream base/);
  f.added.entry.baseSha256 = hash('');
  f.write('patches/manifest.json', f.manifest);
  assert.throws(() => verifyVendor(f.root), /requires an upstream base/);
  f.added.entry.baseSha256 = null;
  f.manifest.patches[0].baseSha256 = null;
  f.write('patches/manifest.json', f.manifest);
  assert.throws(() => verifyVendor(f.root), /must be absent from upstream/);
});

test('an addition claim must reverse to absence and touch only its declared path', (t) => {
  const f = additionFixture(t);
  verifyVendor(f.root, { apply: true });
  const before = 'pub fn old() {}\n';
  const modification = `--- a/${f.added.file}\n+++ b/${f.added.file}\n@@ -1 +1 @@\n-${before}+${f.added.after}`;
  f.write(f.added.patch, modification);
  f.added.entry.sha256 = hash(modification);
  f.write('patches/manifest.json', f.manifest);
  assert.throws(() => verifyVendor(f.root), /must remove the file/);
  const twoFiles = f.added.delta + f.delta;
  f.write(f.added.patch, twoFiles);
  f.added.entry.sha256 = hash(twoFiles);
  f.write('patches/manifest.json', f.manifest);
  assert.throws(() => verifyVendor(f.root), /exactly one declared file/);
});
