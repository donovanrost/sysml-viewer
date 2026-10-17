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

test('diagnostic assertions cannot be satisfied by another owner or another document', () => {
  const source = (documentId, start, end) => ({
    documentId,
    start: { byteOffset: start },
    end: { byteOffset: end },
  });
  const check = {
    id: 'rule',
    op: 'diagnostic',
    owner: 'P::Expected',
    stage: 'semantic',
    code: 'rule',
  };
  const absent = { id: 'clean', op: 'noDiagnostics', owner: check.owner, stage: check.stage };
  const base = {
    ...snapshot,
    elements: [{ id: 'owner', qualifiedName: 'P::Expected', source: source('model', 10, 30) }],
  };
  for (const location of [
    source('peer', 10, 30),
    source('model', 31, 40),
    source('model', 9, 20),
    source('model', 20, 31),
  ]) {
    const withPeer = {
      ...base,
      diagnostics: [{ stage: 'semantic', code: 'rule', source: location }],
    };
    assert.equal(assessCase(withPeer, { checks: [check] })[0].status, 'fail');
    assert.equal(assessCase(withPeer, { checks: [absent] })[0].status, 'pass');
  }
  const withFinding = {
    ...base,
    diagnostics: [{ stage: 'semantic', code: 'rule', source: source('model', 10, 30) }],
  };
  assert.equal(assessCase(withFinding, { checks: [check] })[0].status, 'pass');
  assert.equal(assessCase(withFinding, { checks: [absent] })[0].status, 'fail');
  assert.equal(
    assessCase(base, { checks: [{ op: 'noDiagnostics', id: 'missing', owner: 'P::Missing' }] })[0]
      .status,
    'fail',
  );
});

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

test('import assertions reject unknown as empty and require contextual visibility', () => {
  const m = matrix();
  const c = m.cases.find((c) => c.id === 'IMP-020');
  const check = c.checks.find((c) => c.id === 'empty');
  const s = {
    ...snapshot,
    elements: [
      {
        qualifiedName: check.owner,
        membersAvailable: true,
        importedMemberships: { status: 'unknown', reason: 'incompleteProvider' },
      },
    ],
    memberships: [],
  };
  assert.equal(assessCase(s, { checks: [check] })[0].status, 'fail');
  for (const mutate of [
    (check) => {
      check.status = 'unknown';
      check.reason = 'incompleteProvider';
    },
    (check) => {
      check.reason = 'workLimit';
    },
    (check) => {
      delete check.bindings[0].contextVisibility;
    },
  ]) {
    const value = matrix();
    mutate(value.cases.find((c) => c.id === 'IMP-020').checks.find((c) => c.id === 'bindings'));
    assert.throws(() => validateMatrix(value));
  }
});
