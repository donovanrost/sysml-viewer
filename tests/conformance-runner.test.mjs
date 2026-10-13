import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { validateMatrix, assessCase } from '../tools/lib/conformance.mjs';

const matrix = () =>
  JSON.parse(readFileSync(new URL('./conformance/matrix.json', import.meta.url), 'utf8'));
const snapshot = { elements: [], references: [], diagnostics: [], connections: [] };
const caseWithGap = {
  checks: [{ id: 'target', op: 'element', name: 'Missing', metaclass: 'PartDefinition' }],
  knownFailures: [{ check: 'target', issue: 'TEST-1', reason: 'Test fixture' }],
};

test('coverage catalogue rejects empty cases, unknown operations and orphan gaps', () => {
  validateMatrix(matrix());
  for (const mutate of [
    (m) => {
      m.cases[0].checks = [];
    },
    (m) => {
      m.cases[0].checks[0].op = 'noDiagnosticTypo';
    },
    (m) => {
      m.cases[0].checks[0].stage = 'semanticTypo';
    },
    (m) => {
      m.cases[0].knownFailures = [{ check: 'absent', issue: 'X', reason: 'X' }];
    },
    (m) => {
      m.cases.push(m.cases[0]);
    },
  ]) {
    const m = matrix();
    mutate(m);
    assert.throws(() => validateMatrix(m));
  }
});

test('a known gap accepts only its named assertion failure', () => {
  assert.equal(assessCase(snapshot, caseWithGap)[0].status, 'known-gap');
  const otherFailure = {
    ...caseWithGap,
    checks: [
      ...caseWithGap.checks,
      { id: 'other', op: 'element', name: 'AnotherMissing', metaclass: 'PartDefinition' },
    ],
  };
  assert.deepEqual(
    assessCase(snapshot, otherFailure).map((c) => c.status),
    ['known-gap', 'fail'],
  );
});

test('fixed gaps and malformed runtime data require review', () => {
  const fixed = {
    ...snapshot,
    elements: [{ id: 'e', qualifiedName: 'Missing', metaclass: 'PartDefinition' }],
  };
  assert.equal(assessCase(fixed, caseWithGap)[0].status, 'unexpected-pass');
  assert.equal(assessCase({ ...snapshot, elements: [null] }, caseWithGap)[0].status, 'fail');
});

test('cardinality assertions retain exact integer strings and do not accept unknown as evaluated', () => {
  const m = matrix();
  const c = m.cases.find((c) => c.id === 'MULT-005');
  const check = c.checks.find((c) => c.id === 'large');
  const snapshot = {
    elements: [
      {
        qualifiedName: check.owner,
        cardinality: { status: 'unknown', reason: 'unsupportedBound' },
      },
    ],
    memberships: [],
    references: [],
    diagnostics: [],
    connections: [],
  };
  assert.equal(assessCase(snapshot, { checks: [check] })[0].status, 'fail');
  check.lower = 9007199254740993;
  assert.throws(() => validateMatrix(m));
});
