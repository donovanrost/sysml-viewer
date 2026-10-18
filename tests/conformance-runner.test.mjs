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

test('negative diagnostic assertions select the exact rule without suppressing other findings', () => {
  const source = { documentId: 'model', start: { byteOffset: 10 }, end: { byteOffset: 20 } };
  const base = {
    ...snapshot,
    elements: [{ id: 'owner', qualifiedName: 'P::View', source }],
    diagnostics: [
      { stage: 'semantic', code: 'cycle', severity: 'error', message: 'cycle', source },
      { stage: 'semantic', code: 'rendering', severity: 'warning', message: 'candidate', source },
    ],
  };
  const check = {
    id: 'no-rendering-error',
    op: 'noDiagnostics',
    owner: 'P::View',
    stage: 'semantic',
    code: 'rendering',
    severity: 'error',
    messageIncludes: 'rendering',
  };
  assert.equal(assessCase(base, { checks: [check] })[0].status, 'pass');
  assert.equal(
    assessCase(base, { checks: [{ ...check, code: 'cycle', messageIncludes: 'cycle' }] })[0].status,
    'fail',
  );
  const withError = {
    ...base,
    diagnostics: [
      ...base.diagnostics,
      {
        stage: 'semantic',
        code: 'rendering',
        severity: 'error',
        message: 'extra rendering',
        source,
      },
    ],
  };
  assert.equal(assessCase(withError, { checks: [check] })[0].status, 'fail');
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

test('state query assertions reject partial unknowns and inconsistent binding identities', () => {
  const m = matrix();
  const c = m.cases.find((c) => c.id === 'STATE-009');
  const complete = c.checks.find((c) => c.op === 'stateSubactions');
  complete.status = 'unknown';
  complete.reason = 'invalidSyntax';
  assert.throws(() => validateMatrix(m));
  const check = {
    id: 'state',
    op: 'stateSubactions',
    owner: 'P::S',
    status: 'complete',
    actions: [{ kind: 'entry', target: 'P::S::a', metaclass: 'ActionUsage' }],
  };
  const value = {
    ...snapshot,
    elements: [
      {
        id: 's',
        qualifiedName: 'P::S',
        ownedMemberships: ['m'],
        stateSubactions: {
          status: 'complete',
          actions: [{ kind: 'entry', membershipId: 'm', actionId: 'a' }],
        },
      },
      { id: 'a', qualifiedName: 'P::S::a', metaclass: 'ActionUsage', parentId: 's' },
    ],
    memberships: [
      {
        id: 'm',
        metaclass: 'StateSubactionMembership',
        namespaceId: 's',
        member: { status: 'resolved', targetId: 'a' },
      },
    ],
  };
  assert.equal(assessCase(value, { checks: [check] })[0].status, 'pass');
  value.memberships[0].member.targetId = 'another-action';
  assert.equal(assessCase(value, { checks: [check] })[0].status, 'fail');
});

test('membership assertions retain requested metaclass and declaration order', () => {
  const binding = (name) => ({
    name,
    shortName: null,
    namespace: 'P::T',
    target: name ? `P::T::${name}` : null,
    status: 'resolved',
    visibility: 'public',
    alias: false,
  });
  const check = {
    id: 'inputs',
    op: 'memberships',
    owner: 'P::T',
    mode: 'owned',
    metaclass: 'ParameterMembership',
    ordered: true,
    bindings: [binding(null), binding('payload')],
  };
  const value = {
    ...snapshot,
    elements: [
      { id: 't', qualifiedName: 'P::T', membersAvailable: true, ownedMemberships: ['s', 'p', 'n'] },
      { id: 'source', qualifiedName: null },
      { id: 'payload', qualifiedName: 'P::T::payload' },
    ],
    memberships: [
      ...[
        ['s', null, 'source'],
        ['p', 'payload', 'payload'],
      ].map(([id, name, targetId]) => ({
        id,
        name,
        shortName: null,
        namespaceId: 't',
        metaclass: 'ParameterMembership',
        member: { status: 'resolved', targetId },
        visibility: 'public',
        isAlias: false,
      })),
      { id: 'n', namespaceId: 't', metaclass: 'OwningMembership' },
    ],
  };
  assert.equal(assessCase(value, { checks: [check] })[0].status, 'pass');
  value.elements[0].ownedMemberships = ['p', 's', 'n'];
  assert.equal(assessCase(value, { checks: [check] })[0].status, 'fail');
  value.elements[0].ownedMemberships = ['s', 'p', 'n'];
  value.memberships[1].metaclass = 'FeatureMembership';
  assert.equal(assessCase(value, { checks: [check] })[0].status, 'fail');
  for (const invalid of [{ metaclass: '' }, { ordered: 'true' }]) {
    const m = matrix();
    m.cases[0].checks.push({ ...check, ...invalid });
    assert.throws(() => validateMatrix(m));
  }
});
