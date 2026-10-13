import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import {
  validateInventory,
  validateExecution,
  summarize,
  assessmentRows,
} from '../tools/lib/requirements.mjs';
import { kernelInputHash } from '../tools/lib/kernel-inputs.mjs';

const root = fileURLToPath(new URL('..', import.meta.url));
const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url));
const json = (path) => JSON.parse(read(path));
const inputs = () => ({
  sources: json('standards/sources.json'),
  sourcesBytes: read('standards/sources.json'),
  catalogue: json('standards/catalogue.json'),
  assessments: json('standards/assessments.json'),
  matrix: json('tests/conformance/matrix.json'),
  read,
});

test('inventory preserves complete extraction counts without manufacturing coverage', () => {
  const i = inputs();
  validateInventory(i);
  const summary = summarize(i.catalogue, i.assessments, i.matrix);
  assert.equal(summary.clauses, 1571);
  assert.equal(summary.grammarProductions, 620);
  assert.equal(summary.metamodelConstraints, 724);
  assert.equal(summary.libraryModels, 94);
  assert.equal(summary.schemaDefinitions, 270);
  assert.equal(summary.coveragePercent, null);
  assert(
    i.catalogue.items
      .filter((x) => x.kind === 'constraint')
      .every((x) => !i.assessments.assessments.some((a) => a.sources.includes(x.id))),
  );
});

test('dropping a source unit or its provenance fails the extraction audit', () => {
  for (const mutate of [
    (i) => {
      i.catalogue.items = i.catalogue.items.filter((x) => x.id !== 'SYSML-GRAMMAR-PackageMember');
    },
    (i) => {
      i.sourcesBytes = Buffer.from('{}');
    },
    (i) => {
      i.catalogue.items.find((x) => x.kind === 'constraint').fragment = '';
    },
  ]) {
    const i = inputs();
    mutate(i);
    assert.throws(() => validateInventory(i));
  }
});

test('unknown assertion edges, planned evidence and unmapped new checks fail closed', () => {
  for (const mutate of [
    (i) => {
      i.assessments.assessments[0].tests[0].checks = ['not-a-check'];
    },
    (i) => {
      i.assessments.assessments.find((a) => a.id === 'MEMBERSHIP-ENUMERATION').tests[0].checks = [
        'future',
      ];
    },
    (i) => {
      i.matrix.cases[0].checks.push({ id: 'unmapped', op: 'noDiagnostics' });
    },
    (i) => {
      i.assessments.assessments[0].sources = ['SYSML-CLAUSE-999'];
    },
  ]) {
    const i = inputs();
    mutate(i);
    assert.throws(() => validateInventory(i));
  }
});

test('stale implementation mappings and unsupported full-coverage claims require review', () => {
  const stale = inputs();
  stale.assessments.evidence.engine.sha256 = '0'.repeat(64);
  assert.throws(() => validateInventory(stale), /Stale implementation evidence/);
  const promotion = inputs();
  promotion.assessments.assessments[0].implementation.status = 'qualified';
  assert.throws(() => validateInventory(promotion), /Unsupported implementation status/);
});

test('a mapped case is not counted as executed and planned cases never pass by implication', () => {
  const i = inputs();
  const rows = assessmentRows(i.assessments, i.matrix);
  assert.equal(rows.find((a) => a.id === 'RESOLUTION-PUBLIC').tests[0].status, 'not-run');
  assert.equal(rows.find((a) => a.id === 'MEMBERSHIP-ENUMERATION').tests[0].status, 'planned');
});

test('execution evidence rejects stale inputs, wrong pins and a planned case reported as passed', () => {
  const i = inputs();
  // Reuse the real case outcomes as fixture data. Source/build metadata is supplied
  // explicitly for this validator test; this does not create a qualification report.
  const fixture = json('docs/evidence/conformance-macos-arm64.json');
  const inputHash = kernelInputHash(root);
  Object.assign(fixture, {
    kernelInputSha256: inputHash,
    buildPerformed: true,
    binarySha256: 'a'.repeat(64),
  });
  const check = (r) =>
    validateExecution(
      r,
      i.matrix,
      read('tests/conformance/matrix.json'),
      json('dependencies.json'),
      read,
      inputHash,
    );
  check(fixture);
  for (const mutate of [
    (r) => {
      r.kernelInputSha256 = '0'.repeat(64);
    },
    (r) => {
      r.buildPerformed = false;
    },
    (r) => {
      r.engineVersion = 'different@000000000000';
    },
    (r) => {
      r.results[0].files[Object.keys(r.results[0].files)[0]] = '0'.repeat(64);
    },
    (r) => {
      r.results.find((x) => x.status === 'planned').status = 'pass';
    },
  ]) {
    const report = structuredClone(fixture);
    mutate(report);
    assert.throws(() => check(report));
  }
});
