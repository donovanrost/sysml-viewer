import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

export const hash = (value) => createHash('sha256').update(value).digest('hex');
const statuses = ['unassessed', 'candidate', 'partial', 'unsupported'];
const kinds = [
  'clause',
  'unnumbered-heading',
  'grammar-production',
  'prose-candidate',
  'association',
  'constraint',
  'enumeration',
  'enumeration-literal',
  'generalization',
  'metaclass',
  'operation',
  'package',
  'package-import',
  'parameter',
  'property',
  'normative-artifact',
  'library-model',
  'archive-metadata',
  'schema-definition',
];
const facets = [
  'context',
  'conformance',
  'language-description',
  'textual-notation',
  'graphical-notation',
  'abstract-syntax',
  'semantics',
  'libraries',
  'interchange',
  'annex',
];

function keys(value, allowed, label) {
  assert(value && typeof value === 'object' && !Array.isArray(value), label);
  for (const key of Object.keys(value))
    assert(allowed.includes(key), `${label}: unknown field ${key}`);
}
function unique(values, label) {
  assert.equal(new Set(values).size, values.length, `Duplicate ${label}`);
}
function safePath(path) {
  assert(
    typeof path === 'string' &&
      path &&
      !path.startsWith('/') &&
      !path.includes('\\') &&
      !path.split('/').includes('..'),
    'Unsafe repository path',
  );
}

/** Verify provenance, catalogue structure and every evidence edge. No status is
 * inherited from a parent/child, an implementation filename, or test proximity. */
export function validateInventory({ sources, sourcesBytes, catalogue, assessments, matrix, read }) {
  keys(sources, ['version', 'extraction', 'sources'], 'sources');
  keys(catalogue, ['version', 'sourcesSha256', 'audit', 'items'], 'catalogue');
  keys(assessments, ['version', 'scope', 'evidence', 'assessments'], 'assessments');
  assert.equal(sources.version, 1);
  assert.equal(catalogue.version, 1);
  assert.equal(assessments.version, 1);
  assert.equal(
    catalogue.sourcesSha256,
    hash(sourcesBytes),
    'Source manifest changed; re-extract catalogue',
  );
  const byId = new Map(catalogue.items.map((item) => [item.id, item]));
  unique(
    catalogue.items.map((x) => x.id),
    'source ID',
  );
  for (const [id, source] of Object.entries(sources.sources)) {
    assert(['sysml', 'kerml'].includes(source.standard));
    assert.equal(new URL(source.url).origin, 'https://www.omg.org');
    assert(/^[0-9a-f]{64}$/.test(source.sha256) && source.bytes > 0, `Unpinned source ${id}`);
  }
  for (const item of catalogue.items) {
    keys(
      item,
      [
        'id',
        'standard',
        'kind',
        'facet',
        'title',
        'section',
        'parent',
        'source',
        'page',
        'endPage',
        'decomposition',
        'line',
        'extractionNote',
        'textSha256',
        'modalTerms',
        'alsoOnPages',
        'fragment',
        'ownerFragment',
        'pdfPage',
        'ruleKind',
        'facts',
        'archivePath',
      ],
      'source unit',
    );
    assert(item.id && item.title && kinds.includes(item.kind) && facets.includes(item.facet));
    const source = sources.sources[item.source];
    assert(source && source.standard === item.standard, `Unknown source ${item.id}`);
    const clauseId = `${item.standard.toUpperCase()}-CLAUSE-${item.section}`;
    assert(byId.has(clauseId), `Missing clause ${clauseId}`);
    if (item.parent) {
      const parent = byId.get(item.parent);
      assert(
        parent?.kind === 'clause' && parent.standard === item.standard && parent.id !== item.id,
        `Invalid parent ${item.id}`,
      );
      if (item.kind === 'clause')
        assert.equal(
          item.parent,
          `${item.standard.toUpperCase()}-CLAUSE-${item.section.split('.').slice(0, -1).join('.')}`,
        );
    } else assert(item.kind === 'clause' && !item.section.includes('.'));
    if (source.pages) {
      assert(Number.isInteger(item.page) && item.page >= 1 && item.page <= source.pages);
      if (item.endPage !== undefined)
        assert(item.endPage >= item.page && item.endPage <= source.pages);
    } else if (item.source.endsWith('-xmi') || item.kind === 'schema-definition')
      assert(item.fragment, `Missing source fragment ${item.id}`);
    if (['library-model', 'archive-metadata'].includes(item.kind))
      assert(item.archivePath && /^[0-9a-f]{64}$/.test(item.textSha256));
    if (
      [
        'clause',
        'unnumbered-heading',
        'prose-candidate',
        'normative-artifact',
        'library-model',
        'archive-metadata',
        'schema-definition',
      ].includes(item.kind)
    )
      assert.equal(
        item.decomposition,
        'pending',
        'Decomposition needs an explicit reviewed-requirement schema before it can be closed',
      );
    if (item.kind === 'prose-candidate')
      assert(/^[0-9a-f]{64}$/.test(item.textSha256) && item.modalTerms.length);
  }
  for (const [source, audit] of Object.entries(catalogue.audit.companions)) {
    const items = catalogue.items.filter((i) => i.source === source);
    assert.equal(items.filter((i) => i.kind === 'normative-artifact').length, 1);
    if (audit.schemaDefinitions !== undefined)
      assert.equal(
        items.filter((i) => i.kind === 'schema-definition').length,
        audit.schemaDefinitions,
      );
    else {
      assert.equal(
        items.filter((i) => ['library-model', 'archive-metadata'].includes(i.kind)).length,
        audit.indexedFiles,
      );
      assert.equal(audit.indexedFiles + audit.packagingMetadataExcluded.length, audit.archiveFiles);
    }
  }
  for (const standard of ['sysml', 'kerml']) {
    const items = catalogue.items.filter((i) => i.standard === standard);
    const count = (kind) => items.filter((i) => i.kind === kind).length;
    const audit = catalogue.audit[standard];
    assert.equal(count('clause'), audit.outlineClauses, 'Clause inventory was truncated');
    assert.equal(audit.outlineClauses, audit.tocClauses);
    assert.equal(audit.outlineClauses, audit.locatedHeadings);
    assert.equal(audit.pdfPages, sources.sources[`${standard}-pdf`].pages);
    assert.equal(count('grammar-production'), audit.grammarProductions);
    assert.equal(count('unnumbered-heading'), audit.unnumberedHeadings);
    assert.equal(count('prose-candidate'), audit.modalPassageCandidates);
    for (const [kind, expected] of Object.entries(audit.xmi))
      assert.equal(count(kind), expected, `XMI ${kind} inventory was truncated`);
  }
  const evidence = assessments.evidence;
  for (const [id, entry] of Object.entries(evidence)) {
    keys(entry, ['path', 'symbol', 'role', 'sha256'], 'implementation evidence');
    safePath(entry.path);
    const bytes = read(entry.path);
    assert.equal(
      hash(bytes),
      entry.sha256,
      `Stale implementation evidence: ${id}; review before refreshing its hash`,
    );
    assert(entry.symbol && bytes.toString().includes(entry.symbol), `Missing symbol ${id}`);
    assert(entry.role);
  }
  const cases = new Map(matrix.cases.map((c) => [c.id, c]));
  const mappedChecks = new Set();
  const mappedCases = new Set();
  unique(
    assessments.assessments.map((a) => a.id),
    'assessment ID',
  );
  for (const a of assessments.assessments) {
    keys(
      a,
      [
        'id',
        'obligation',
        'category',
        'sources',
        'applicability',
        'priority',
        'implementation',
        'api',
        'tests',
        'limits',
      ],
      'assessment',
    );
    assert(a.id && a.obligation && a.limits && a.sources.length);
    assert(['standard', 'product', 'conformance-point'].includes(a.category));
    assert(['current-product', 'full-tool-conformance'].includes(a.applicability));
    assert(['P0', 'P1', 'P2'].includes(a.priority));
    unique(a.sources, 'assessment source');
    for (const id of a.sources) assert(byId.has(id), `Unknown source unit ${id}`);
    for (const axis of ['implementation', 'api']) {
      keys(a[axis], ['status', 'evidence', 'note'], axis);
      // The initial inventory deliberately cannot declare full coverage. Add a
      // reviewed closure model before introducing a 'qualified' enum value.
      assert(statuses.includes(a[axis].status), `Unsupported ${axis} status`);
      assert(
        a[axis].status === 'unassessed' || a[axis].evidence.length,
        `Unevidenced ${axis} claim`,
      );
      for (const id of a[axis].evidence) assert(evidence[id], `Unknown evidence ${id}`);
    }
    unique(
      a.tests.map((t) => t.case),
      'test case edge',
    );
    for (const test of a.tests) {
      keys(test, ['case', 'checks'], 'test edge');
      const c = cases.get(test.case);
      assert(c, `Unknown test case ${test.case}`);
      assert(Array.isArray(test.checks));
      assert(
        c.planned ? !test.checks.length : test.checks.length,
        'Planned tests cannot supply executable evidence',
      );
      unique(test.checks, 'check edge');
      for (const id of test.checks) {
        assert(
          c.checks.some((check) => check.id === id),
          `Unknown assertion ${test.case}/${id}`,
        );
        mappedChecks.add(`${test.case}/${id}`);
      }
      mappedCases.add(test.case);
    }
  }
  for (const c of matrix.cases) {
    assert(mappedCases.has(c.id), `Unmapped case ${c.id}`);
    for (const check of c.checks ?? [])
      assert(mappedChecks.has(`${c.id}/${check.id}`), `Unmapped assertion ${c.id}/${check.id}`);
  }
  return { byId, cases };
}

/** Validate recorded execution against today's test definition and fixture bytes.
 * A planned, known-gap or stale result can never be counted as a passed check. */
export function validateExecution(report, matrix, matrixBytes, pins, read, expectedInputHash) {
  assert.equal(report.kernelInputSha256, expectedInputHash, 'Stale kernel qualification inputs');
  assert.equal(report.buildPerformed, true, 'Qualification evidence must build the native CLI');
  assert(/^[0-9a-f]{64}$/.test(report.binarySha256), 'Missing tested binary fingerprint');
  assert.equal(report.matrixSha256, hash(matrixBytes), 'Stale conformance matrix report');
  assert.equal(report.schemaVersion, 1);
  assert(report.engineVersion.includes(pins.toolkit.revision.slice(0, 12)), 'Engine pin mismatch');
  assert(
    report.libraryVersion.includes(pins.library.revision.slice(0, 12)),
    'Library pin mismatch',
  );
  unique(
    report.results.map((r) => r.id),
    'execution result',
  );
  assert.equal(report.results.length, matrix.cases.length);
  for (const c of matrix.cases) {
    const r = report.results.find((r) => r.id === c.id);
    assert(r, `Missing result ${c.id}`);
    if (c.planned) {
      assert.equal(r.status, 'planned');
      continue;
    }
    assert(['pass', 'known-gap', 'fail'].includes(r.status));
    assert.deepEqual(Object.keys(r.files).sort(), [...c.files].sort());
    for (const file of c.files)
      assert.equal(r.files[file], hash(read(file)), `Stale fixture ${file}`);
    if (r.status !== 'fail') {
      assert.equal(r.checks.length, c.checks.length);
      unique(
        r.checks.map((check) => check.id),
        'execution assertion',
      );
      for (const check of c.checks) {
        const outcome = r.checks.find((x) => x.id === check.id);
        assert(outcome && ['pass', 'known-gap'].includes(outcome.status));
        if (outcome.status === 'known-gap')
          assert(c.knownFailures?.some((g) => g.check === check.id));
      }
      assert.equal(r.status, r.checks.some((x) => x.status === 'known-gap') ? 'known-gap' : 'pass');
    }
  }
  for (const status of ['pass', 'known-gap', 'planned', 'fail'])
    assert.equal(report.counts[status], report.results.filter((r) => r.status === status).length);
}

export function summarize(catalogue, assessments, matrix) {
  return {
    sourceUnits: catalogue.items.length,
    clauses: catalogue.items.filter((i) => i.kind === 'clause').length,
    grammarProductions: catalogue.items.filter((i) => i.kind === 'grammar-production').length,
    metamodelConstraints: catalogue.items.filter((i) => i.kind === 'constraint').length,
    proseCandidates: catalogue.items.filter((i) => i.kind === 'prose-candidate').length,
    libraryModels: catalogue.items.filter((i) => i.kind === 'library-model').length,
    schemaDefinitions: catalogue.items.filter((i) => i.kind === 'schema-definition').length,
    assessments: assessments.assessments.length,
    executableCases: matrix.cases.filter((c) => !c.planned).length,
    plannedCases: matrix.cases.filter((c) => c.planned).length,
    coveragePercent: null,
    reason:
      'Clause and prose decomposition is pending; overlapping source-unit counts are not a requirements denominator.',
  };
}

export function sourceUrl(item, sources) {
  return (
    sources.sources[item.source].url +
    (item.fragment
      ? `#${encodeURIComponent(item.fragment)}`
      : item.page
        ? `#page=${item.page}`
        : '')
  );
}

export function assessmentRows(assessments, matrix, execution) {
  return assessments.assessments.map((a) => ({
    ...a,
    tests: a.tests.map((edge) => {
      const c = matrix.cases.find((x) => x.id === edge.case);
      const result = execution?.results.find((r) => r.id === edge.case);
      return {
        ...edge,
        polarity: c.polarity,
        status: c.planned ? 'planned' : (result?.status ?? 'not-run'),
        results: edge.checks.map((id) => ({
          id,
          status:
            result?.checks?.find((r) => r.id === id)?.status ??
            (result?.status === 'fail' ? 'fail' : 'not-run'),
        })),
      };
    }),
  }));
}
