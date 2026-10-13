import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { validateMatrix } from './lib/conformance.mjs';
import {
  validateInventory,
  validateExecution,
  summarize,
  assessmentRows,
  hash,
} from './lib/requirements.mjs';
import { renderSummary, renderBrowser } from './lib/requirements-docs.mjs';
import { kernelInputHash } from './lib/kernel-inputs.mjs';

const root = fileURLToPath(new URL('..', import.meta.url));
const read = (path) => readFileSync(join(root, path));
const json = (path) => JSON.parse(read(path));
const sourcesBytes = read('standards/sources.json');
const sources = JSON.parse(sourcesBytes);
const catalogue = json('standards/catalogue.json');
const assessments = json('standards/assessments.json');
const matrixBytes = read('tests/conformance/matrix.json');
const matrix = JSON.parse(matrixBytes);
validateMatrix(matrix);
validateInventory({ sources, sourcesBytes, catalogue, assessments, matrix, read });
const summary = summarize(catalogue, assessments, matrix);
const generated = {
  'docs/requirements-summary.md': renderSummary(catalogue, assessments, matrix, sources),
  'docs/requirements.html': renderBrowser(catalogue, assessments, matrix, sources),
};
for (const [path, value] of Object.entries(generated)) {
  if (process.argv.includes('--write-docs')) writeFileSync(join(root, path), value);
  else
    assert.equal(
      hash(read(path)),
      hash(value),
      `Generated inventory drift: ${path}; run npm run requirements:docs`,
    );
}

const reportAt = process.argv.indexOf('--conformance');
if (reportAt !== -1) {
  assert(process.argv[reportAt + 1], '--conformance needs a report path');
  const executionBytes = readFileSync(resolve(root, process.argv[reportAt + 1]));
  const execution = JSON.parse(executionBytes);
  validateExecution(
    execution,
    matrix,
    matrixBytes,
    json('dependencies.json'),
    read,
    kernelInputHash(root),
  );
  const report = {
    version: 1,
    ...summary,
    catalogueSha256: hash(read('standards/catalogue.json')),
    assessmentsSha256: hash(read('standards/assessments.json')),
    sourcesSha256: hash(sourcesBytes),
    execution: {
      sha256: hash(executionBytes),
      generatedAt: execution.generatedAt,
      kernelInputSha256: execution.kernelInputSha256,
      binarySha256: execution.binarySha256,
      platform: execution.platform,
      arch: execution.arch,
      engineVersion: execution.engineVersion,
      libraryVersion: execution.libraryVersion,
      counts: execution.counts,
    },
    assessments: assessmentRows(assessments, matrix, execution),
  };
  mkdirSync(join(root, 'target/requirements'), { recursive: true });
  writeFileSync(
    join(root, 'target/requirements/report.json'),
    JSON.stringify(report, null, 2) + '\n',
  );
  if (execution.counts.fail) process.exitCode = 1;
}
console.log(JSON.stringify(summary, null, 2));
