import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
import { cargo, root } from './rust.mjs';
import { validateMatrix, assessCase, matrixMarkdown } from './lib/conformance.mjs';
import { kernelInputHash } from './lib/kernel-inputs.mjs';

const matrix = JSON.parse(readFileSync(join(root, 'tests/conformance/matrix.json'), 'utf8'));
validateMatrix(matrix);
const markdown = matrixMarkdown(matrix);
const doc = join(root, 'docs/conformance.md');
if (process.argv.includes('--write-docs')) {
  writeFileSync(doc, markdown);
  process.exit(0);
}
assert.equal(readFileSync(doc, 'utf8'), markdown, 'Run npm run conformance:docs');
const hash = (bytes) => createHash('sha256').update(bytes).digest('hex');
const provenance = JSON.parse(
  readFileSync(join(root, 'tests/conformance/upstream/provenance.json'), 'utf8'),
);
const pins = JSON.parse(readFileSync(join(root, 'dependencies.json'), 'utf8'));
assert.equal(provenance.revision, pins.library.revision);
for (const file of provenance.files)
  assert.equal(
    hash(readFileSync(join(root, file.path))),
    file.sha256,
    `Modified upstream fixture ${file.path}`,
  );
if (process.argv.includes('--check-docs')) process.exit(0);
const kernelInputSha256 = kernelInputHash(root);
if (!process.argv.includes('--no-build'))
  cargo(['build', '--locked', '--release', '-p', 'sysml-cli']);
const binary = join(
  root,
  `target/release/sysml-inspect${process.platform === 'win32' ? '.exe' : ''}`,
);
assert(existsSync(binary), 'Build sysml-cli first');
const results = [];
let metadata;
for (const [index, c] of matrix.cases.entries()) {
  if (c.planned) {
    results.push({ id: c.id, status: 'planned', reason: c.planned });
    continue;
  }
  const started = performance.now();
  const documents = c.files.map((file) => ({
    id: file,
    name: file.split('/').at(-1),
    text: readFileSync(join(root, file), 'utf8'),
  }));
  const files = Object.fromEntries(documents.map((d) => [d.id, hash(d.text)]));
  try {
    const snapshot = JSON.parse(
      execFileSync(binary, ['--request-json'], {
        input: JSON.stringify({ revision: index + 1, documents }),
        encoding: 'utf8',
        timeout: 30_000,
        maxBuffer: 50_000_000,
        stdio: ['pipe', 'pipe', 'pipe'],
      }),
    );
    metadata ??= {
      schemaVersion: snapshot.schemaVersion,
      engineVersion: snapshot.engineVersion,
      libraryVersion: snapshot.libraryVersion,
    };
    assert.deepEqual(
      {
        schemaVersion: snapshot.schemaVersion,
        engineVersion: snapshot.engineVersion,
        libraryVersion: snapshot.libraryVersion,
      },
      metadata,
    );
    const checks = assessCase(snapshot, c);
    const status = checks.some((check) => ['fail', 'unexpected-pass'].includes(check.status))
      ? 'fail'
      : checks.some((check) => check.status === 'known-gap')
        ? 'known-gap'
        : 'pass';
    results.push({
      id: c.id,
      status,
      files,
      elapsedMs: Math.round(performance.now() - started),
      checks,
    });
    console.log(`${status.padEnd(10)} ${c.id} (${results.at(-1).elapsedMs} ms)`);
    for (const check of checks.filter((check) =>
      ['fail', 'unexpected-pass'].includes(check.status),
    ))
      console.error(`  ${check.id}: ${check.message ?? check.status}`);
  } catch (error) {
    results.push({
      id: c.id,
      status: 'fail',
      files,
      elapsedMs: Math.round(performance.now() - started),
      message: String(error),
    });
    console.error(`fail       ${c.id}: ${error}`);
  }
}
mkdirSync(join(root, 'target/conformance'), { recursive: true });
const counts = Object.fromEntries(
  ['pass', 'known-gap', 'planned', 'fail'].map((s) => [
    s,
    results.filter((r) => r.status === s).length,
  ]),
);
writeFileSync(
  join(root, 'target/conformance/report.json'),
  JSON.stringify(
    {
      generatedAt: new Date().toISOString(),
      platform: process.platform,
      arch: process.arch,
      kernelInputSha256,
      buildPerformed: !process.argv.includes('--no-build'),
      binarySha256: hash(readFileSync(binary)),
      ...metadata,
      matrixSha256: hash(readFileSync(join(root, 'tests/conformance/matrix.json'))),
      counts,
      results,
    },
    null,
    2,
  ) + '\n',
);
console.log(JSON.stringify(counts));
assert.equal(
  kernelInputHash(root),
  kernelInputSha256,
  'Qualification inputs changed during execution',
);
if (counts.fail) process.exitCode = 1;
