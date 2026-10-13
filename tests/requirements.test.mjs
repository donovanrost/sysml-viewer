import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
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
  // Mapping a related clause never qualifies an untested metaclass rule.
  const untested =
    'SYSML-XMI-Systems-Ports-ConjugatedPortDefinition-validateConjugatedPortDefinitionOriginalPortDefinition';
  assert(i.catalogue.items.some((x) => x.id === untested));
  assert(!i.assessments.assessments.some((a) => a.sources.includes(untested)));
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
  i.matrix.cases.push({ id: 'TEST-PLANNED', planned: 'Synthetic future obligation' });
  i.assessments.assessments[0].tests.push({ case: 'TEST-PLANNED', checks: [] });
  const rows = assessmentRows(i.assessments, i.matrix);
  assert.equal(rows.find((a) => a.id === 'RESOLUTION-PUBLIC').tests[0].status, 'not-run');
  assert.equal(rows[0].tests.find((t) => t.case === 'TEST-PLANNED').status, 'planned');
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
  ]) {
    const report = structuredClone(fixture);
    mutate(report);
    assert.throws(() => check(report));
  }
  // Keep this boundary exercised even when the real matrix has no planned cases.
  const planned = i.matrix.cases[0];
  planned.planned = 'Synthetic future obligation';
  delete planned.files;
  delete planned.checks;
  const matrixBytes = Buffer.from(JSON.stringify(i.matrix));
  fixture.matrixSha256 = createHash('sha256').update(matrixBytes).digest('hex');
  const validatePlanned = () =>
    validateExecution(fixture, i.matrix, matrixBytes, json('dependencies.json'), read, inputHash);
  fixture.results[0] = { id: planned.id, status: 'planned' };
  fixture.counts.pass -= 1;
  fixture.counts.planned += 1;
  validatePlanned();
  fixture.results[0].status = 'pass';
  assert.throws(validatePlanned);
});
