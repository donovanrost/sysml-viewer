import assert from 'node:assert/strict';

const ops = {
  element: ['name', 'metaclass'],
  reference: ['owner', 'kind', 'spelling', 'status', 'target'],
  members: ['owner', 'mode', 'names', 'complete'],
  multiplicity: ['owner', 'value'],
  diagnostic: ['stage', 'code', 'severity'],
  noDiagnostics: ['stage'],
  source: ['owner', 'text'],
  connection: ['name', 'chains', 'statuses'],
};

function keys(object, allowed, context) {
  assert(object && typeof object === 'object' && !Array.isArray(object), context);
  for (const key of Object.keys(object))
    assert(allowed.includes(key), `${context}: unknown field ${key}`);
}

export function validateMatrix(matrix) {
  keys(matrix, ['version', 'standards', 'areas', 'cases'], 'matrix');
  assert.equal(matrix.version, 1);
  const areas = new Set();
  for (const area of matrix.areas) {
    keys(area, ['id', 'title', 'clauses', 'uncovered'], 'area');
    assert(area.id && !areas.has(area.id), `Duplicate/empty area ${area.id}`);
    assert(area.title && area.clauses.length && area.uncovered);
    for (const clause of area.clauses) {
      keys(clause, ['standard', 'section'], 'clause');
      assert(matrix.standards[clause.standard] && clause.section);
    }
    areas.add(area.id);
  }
  const cases = new Set();
  for (const c of matrix.cases) {
    keys(
      c,
      ['id', 'area', 'purpose', 'polarity', 'files', 'checks', 'knownFailures', 'planned'],
      'case',
    );
    assert(c.id && !cases.has(c.id), `Duplicate/empty case ${c.id}`);
    cases.add(c.id);
    assert(areas.has(c.area) && c.purpose);
    assert(['positive', 'negative', 'robustness'].includes(c.polarity));
    if (c.planned) {
      assert(
        typeof c.planned === 'string' &&
          !c.files?.length &&
          !c.checks?.length &&
          !c.knownFailures?.length,
      );
      continue;
    }
    assert(
      c.files?.length && c.checks?.length,
      `${c.id}: executable case must have files and checks`,
    );
    for (const file of c.files)
      assert(typeof file === 'string' && /^tests\//.test(file) && !file.split('/').includes('..'));
    const checkIds = new Set();
    for (const check of c.checks) {
      assert(Object.hasOwn(ops, check.op), `${c.id}: unknown operation ${check.op}`);
      keys(check, ['id', 'op', ...ops[check.op]], 'check');
      assert(check.id && !checkIds.has(check.id), `Duplicate/empty check ${check.id}`);
      checkIds.add(check.id);
      if (Object.hasOwn(check, 'stage'))
        assert(
          ['syntax', 'structure', 'resolution', 'semantic', 'capability'].includes(check.stage),
          'Unknown diagnostic stage',
        );
      if (Object.hasOwn(check, 'severity')) assert(['warning', 'error'].includes(check.severity));
      if (Object.hasOwn(check, 'code')) assert(typeof check.code === 'string' && check.code.length);
      if (Object.hasOwn(check, 'complete')) assert.equal(typeof check.complete, 'boolean');
      if (check.op === 'reference') {
        assert(
          check.owner &&
            check.kind &&
            ['resolved', 'unresolved', 'ambiguous', 'unsupported'].includes(check.status),
        );
        assert(
          [
            'typing',
            'specialization',
            'subsetting',
            'redefinition',
            'import',
            'reference',
            'other',
          ].includes(check.kind),
        );
        assert(check.status !== 'resolved' || check.target, 'Resolved reference needs a target');
      }
      if (check.op === 'members')
        assert(
          check.owner &&
            ['owned', 'effective'].includes(check.mode) &&
            Array.isArray(check.names) &&
            check.names.every((n) => typeof n === 'string'),
        );
      if (check.op === 'diagnostic')
        assert(check.stage || check.code, 'Diagnostic must identify a stage or code');
      if (check.op === 'element') assert(check.name && check.metaclass);
      if (check.op === 'multiplicity') assert(check.owner && Object.hasOwn(check, 'value'));
      if (check.op === 'source') assert(check.owner && typeof check.text === 'string');
      if (check.op === 'connection')
        assert(
          check.name && check.chains?.length && check.statuses?.length === check.chains.length,
        );
    }
    const gaps = new Set();
    for (const gap of c.knownFailures ?? []) {
      keys(gap, ['check', 'issue', 'reason'], 'known failure');
      assert(checkIds.has(gap.check) && !gaps.has(gap.check) && gap.issue && gap.reason);
      gaps.add(gap.check);
    }
  }
  for (const area of areas)
    assert(
      matrix.cases.some((c) => c.area === area),
      `Area ${area} has no cases`,
    );
}

export function checkSnapshot(snapshot, check) {
  const byId = new Map(snapshot.elements.map((e) => [e.id, e]));
  const find = (name) => {
    const matches = snapshot.elements.filter((e) => e.qualifiedName === name);
    assert.equal(matches.length, 1, `Expected exactly one declaration named ${name}`);
    return matches[0];
  };
  switch (check.op) {
    case 'element':
      assert.equal(find(check.name).metaclass, check.metaclass);
      break;
    case 'reference': {
      const owner = find(check.owner);
      const refs = snapshot.references.filter(
        (r) =>
          r.ownerId === owner.id &&
          r.kind === check.kind &&
          (check.spelling === undefined || r.spelling === check.spelling),
      );
      assert.equal(refs.length, 1, `Expected one ${check.kind} reference from ${check.owner}`);
      assert.equal(refs[0].resolution.status, check.status);
      if (check.status === 'resolved')
        assert.equal(byId.get(refs[0].resolution.targetId)?.qualifiedName, check.target);
      break;
    }
    case 'members': {
      const owner = find(check.owner);
      assert(owner.membersAvailable, 'Member query unavailable');
      const ids = check.mode === 'owned' ? owner.ownedMembers : owner.effectiveMembers;
      const members = ids.map((id) => {
        assert(byId.has(id), `Dangling member ${id}`);
        return byId.get(id);
      });
      assert.deepEqual(
        members
          .filter((e) => !e.library)
          .map((e) => e.qualifiedName)
          .sort(),
        [...check.names].sort(),
      );
      if (check.complete !== undefined) assert.equal(owner.inheritanceComplete, check.complete);
      break;
    }
    case 'multiplicity':
      assert.equal(find(check.owner).declaredMultiplicity, check.value);
      break;
    case 'diagnostic':
      assert(
        snapshot.diagnostics.some(
          (d) =>
            (!check.stage || d.stage === check.stage) &&
            (!check.code || d.code === check.code) &&
            (!check.severity || d.severity === check.severity),
        ),
        `Missing diagnostic ${JSON.stringify(check)}`,
      );
      break;
    case 'noDiagnostics':
      assert.deepEqual(
        snapshot.diagnostics.filter((d) => !check.stage || d.stage === check.stage),
        [],
        'Unexpected diagnostics',
      );
      break;
    case 'source': {
      const range = find(check.owner).source;
      assert(range, 'No declaration range');
      const doc = snapshot.documents.find((d) => d.id === range.documentId);
      assert.equal(doc.text.slice(range.start.utf16Offset, range.end.utf16Offset), check.text);
      assert.equal(
        Buffer.from(doc.text).subarray(range.start.byteOffset, range.end.byteOffset).toString(),
        check.text,
      );
      break;
    }
    case 'connection': {
      const id = find(check.name).id;
      const connections = snapshot.connections.filter((c) => c.elementId === id);
      assert.equal(connections.length, 1);
      assert.deepEqual(
        connections[0].ends.map((e) => e.chain.map((id) => byId.get(id)?.qualifiedName)),
        check.chains,
      );
      assert.deepEqual(
        connections[0].ends.map((e) => e.resolution.status),
        check.statuses,
      );
      break;
    }
    default:
      throw new Error(`Unknown operation ${check.op}`);
  }
}

/** A known gap is tied to exactly one assertion. Crashes, new failures, and
 * unexpected passes are never accepted as an expected gap. */
export function assessCase(snapshot, c) {
  const known = new Map((c.knownFailures ?? []).map((g) => [g.check, g]));
  return c.checks.map((check) => {
    try {
      checkSnapshot(snapshot, check);
      return { id: check.id, status: known.has(check.id) ? 'unexpected-pass' : 'pass' };
    } catch (error) {
      // Only assertion mismatches qualify. TypeError, malformed wire data and
      // other harness/runtime failures must fail the run even on an expected gap.
      return {
        id: check.id,
        status:
          known.has(check.id) && error instanceof assert.AssertionError ? 'known-gap' : 'fail',
        issue: known.get(check.id)?.issue,
        message: String(error),
      };
    }
  });
}

export function matrixMarkdown(matrix) {
  const lines = [
    '# Executable standards coverage',
    '',
    '<!-- Generated by npm run conformance:docs. Edit tests/conformance/matrix.json. -->',
    '',
    'These are bounded acceptance claims, not a percentage of SysML conformance.',
    '“Covered” means the listed assertions pass. A known gap must still fail its',
    'named assertion; new failures or unexpected passes fail the run. Planned cases',
    'are not executed and never counted as passing. Each area lists remaining scope.',
    '',
    'Run `npm run conformance`. Results, engine/library pins, fixture hashes and',
    'elapsed process times are written to `target/conformance/report.json`.',
    'Each executable case gets a fresh native process and a 30-second deadline.',
    'Clause references identify the source of a test, not complete coverage of that clause.',
    '',
    ...Object.entries(matrix.standards).map(
      ([name, value]) => `- [${name} ${value.document}](${value.url})`,
    ),
    '',
    '| Case | Area / clauses | Kind | Expected result | Assertion scope |',
    '| --- | --- | --- | --- | --- |',
  ];
  for (const c of matrix.cases) {
    const area = matrix.areas.find((a) => a.id === c.area);
    const clauses = area.clauses.map((r) => `${r.standard} §${r.section}`).join(', ');
    const status = c.planned
      ? 'Planned'
      : c.knownFailures?.length
        ? `Known gap: ${c.knownFailures.map((g) => g.issue).join(', ')}`
        : 'Covered';
    lines.push(
      `| ${c.id} | ${area.title}: ${clauses} | ${c.polarity} | ${status} | ${c.purpose.replaceAll('|', '\\|')} |`,
    );
  }
  lines.push('', '## Coverage limits', '');
  for (const area of matrix.areas) lines.push(`- **${area.title}:** ${area.uncovered}`);
  lines.push('', '## Known failures and planned work', '');
  for (const c of matrix.cases) {
    if (c.planned) lines.push(`- **${c.id}:** ${c.planned}`);
    for (const gap of c.knownFailures ?? [])
      lines.push(`- **${gap.issue} / ${c.id} / ${gap.check}:** ${gap.reason}`);
  }
  lines.push(
    '',
    'The authored fixtures are in `tests/conformance/fixtures`. Official examples are',
    'copied unchanged from the pinned release; see `tests/conformance/upstream/provenance.json`',
    'and its README. The corpus probe does not execute the Java pilot implementation.',
    '',
  );
  return lines.join('\n');
}
