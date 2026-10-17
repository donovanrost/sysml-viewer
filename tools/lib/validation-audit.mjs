import assert from 'node:assert/strict';
import { hash, sourceUrl } from './requirements.mjs';

export const validationScanRoots = [
  'vendor/sysml-toolkit/crates/sysmlv2-syntax/src',
  'vendor/sysml-toolkit/crates/sysmlv2-model/src',
  'crates/sysml-kernel/src',
];

const registries = [
  {
    path: 'vendor/sysml-toolkit/crates/sysmlv2-syntax/src/check.rs',
    symbol: 'IMPLEMENTED_NORMATIVE_VALIDATION_RULES',
  },
  {
    path: 'vendor/sysml-toolkit/crates/sysmlv2-model/src/check.rs',
    symbol: 'IMPLEMENTED_NORMATIVE_SEMANTIC_RULES',
  },
];

/** Search evidence only. A name in source (including a comment or private test)
 * is not proof that a validator runs, implements the rule, or reaches the API.
 * Likewise an absent name does not establish an absent implementation. */
export function buildValidationAudit({ catalogue, assessments, matrix, paths, read }) {
  const rules = catalogue.items
    .filter((item) => item.kind === 'constraint' && item.ruleKind === 'validate')
    .sort((a, b) => a.title.localeCompare(b.title, 'en'));
  const names = new Set(rules.map((rule) => rule.title));
  assert.equal(names.size, rules.length, 'Duplicate normative validation rule name');
  const files = [...paths].sort();
  assert.equal(new Set(files).size, files.length, 'Duplicate validation scan path');
  const inputs = files.map((path) => {
    assert(
      validationScanRoots.some((root) => path.startsWith(`${root}/`)) &&
        path.endsWith('.rs') &&
        !path.split('/').includes('..'),
      `Unexpected validation scan path: ${path}`,
    );
    const bytes = read(path);
    return { path, sha256: hash(bytes), text: bytes.toString() };
  });
  const mentions = new Map();
  for (const input of inputs) {
    for (const [index, line] of input.text.split('\n').entries()) {
      for (const name of new Set(line.match(/\bvalidate[A-Z][A-Za-z0-9]*/g) ?? [])) {
        const sites = mentions.get(name) ?? [];
        sites.push({ path: input.path, line: index + 1 });
        mentions.set(name, sites);
      }
    }
  }
  const registered = new Map();
  for (const registry of registries) {
    const input = inputs.find((file) => file.path === registry.path);
    assert(input, `Missing validation registry: ${registry.path}`);
    const pattern = new RegExp(`pub const ${registry.symbol}: &\\[&str\\] = &\\[([\\s\\S]*?)\\];`);
    const match = input.text.match(pattern);
    assert(match, `Cannot read validation registry: ${registry.symbol}`);
    const entries = [...match[1].matchAll(/"(validate[A-Z][A-Za-z0-9]*)"/g)];
    assert(entries.length, `Empty validation registry: ${registry.symbol}`);
    const unique = new Set();
    for (const [, name] of entries) {
      assert(names.has(name), `Non-normative registry entry: ${name}`);
      assert(!unique.has(name), `Duplicate registry entry: ${name}`);
      unique.add(name);
      const sites = registered.get(name) ?? [];
      sites.push(registry);
      registered.set(name, sites);
    }
  }
  const cases = new Map(matrix.cases.map((entry) => [entry.id, entry]));
  const rows = rules.map((rule) => {
    // Only explicit source edges count. Parent clauses, similarly named tests,
    // and string matches in assertion messages never create an assessment edge.
    const mapped = assessments.assessments
      .filter((entry) => entry.sources.includes(rule.id))
      .map((entry) => ({
        id: entry.id,
        obligation: entry.obligation,
        implementation: entry.implementation.status,
        api: entry.api.status,
        limits: entry.limits,
        tests: entry.tests.map((edge) => {
          const test = cases.get(edge.case);
          assert(test, `Missing audit test: ${edge.case}`);
          for (const check of edge.checks)
            assert(
              test.checks?.some((item) => item.id === check),
              `Missing audit assertion: ${edge.case}/${check}`,
            );
          return {
            case: edge.case,
            checks: edge.checks,
            polarity: test.polarity,
            status: test.planned ? 'planned' : 'mapped-not-execution-evidence',
          };
        }),
      }));
    const sites = mentions.get(rule.title) ?? [];
    return {
      sourceId: rule.id,
      name: rule.title,
      standard: rule.standard,
      owner: rule.ownerFragment,
      registrations: registered.get(rule.title) ?? [],
      mentions: sites,
      assessments: mapped,
      nextAction: mapped.length
        ? 'complete-obligation-review'
        : sites.length
          ? 'review-code-candidate'
          : 'locate-or-implement',
    };
  });
  return {
    version: 1,
    scope: 'Named validate constraints in the pinned SysML and KerML metamodels only.',
    limitations: [
      'Source mentions include comments and private tests; they are navigation aids, not implementation or execution proof.',
      'Registry declarations do not certify behavior. Missing names do not prove missing implementations.',
      'Only direct assessment source edges are listed. Each applies to its stated obligation, not necessarily the whole rule.',
      'Mapped tests are not execution results. Use the separately verified requirements execution report.',
      'Derivations, semantic check constraints, operations, grammar, prose, libraries and interchange remain separate review queues.',
    ],
    scanRoots: validationScanRoots,
    inputs: inputs.map(({ path, sha256 }) => ({ path, sha256 })),
    structuredInputHashEncoding: 'JSON.stringify(parsed input), UTF-8',
    catalogueSha256: hash(JSON.stringify(catalogue)),
    assessmentsSha256: hash(JSON.stringify(assessments)),
    matrixSha256: hash(JSON.stringify(matrix)),
    counts: {
      rules: rows.length,
      registered: registered.size,
      withSourceMentions: rows.filter((row) => row.mentions.length).length,
      withoutSourceMentions: rows.filter((row) => !row.mentions.length).length,
      withDirectAssessments: rows.filter((row) => row.assessments.length).length,
      withoutDirectAssessments: rows.filter((row) => !row.assessments.length).length,
      unmatchedNames: [...mentions.keys()].filter((name) => !names.has(name)).length,
    },
    rules: rows,
    unmatchedNames: [...mentions]
      .filter(([name]) => !names.has(name))
      .sort(([a], [b]) => a.localeCompare(b, 'en'))
      .map(([name, sites]) => ({ name, mentions: sites })),
  };
}

export function renderValidationAudit(audit, catalogue, sources) {
  const byId = new Map(catalogue.items.map((item) => [item.id, item]));
  const location = (site) => `[${site.path}:${site.line}](../${site.path}#L${site.line})`;
  const counts = audit.counts;
  const lines = [
    '# Named validation-rule audit',
    '',
    '<!-- Generated by npm run requirements:docs. Do not edit by hand. -->',
    '',
    audit.scope,
    'The [machine-readable report](evidence/validation-audit.json) retains every source location, input hash and direct assessment edge.',
    'The [review plan](requirements.md#validation-rule-audit) explains the next work and known findings.',
    '',
    ...audit.limitations.map((text) => `- ${text}`),
    '',
    `${counts.rules} named rules; ${counts.registered} registry entries; ${counts.withSourceMentions} names mentioned in the scanned Rust source; ${counts.withoutSourceMentions} not located by name.`,
    `${counts.withDirectAssessments} rules have direct bounded assessments; ${counts.withoutDirectAssessments} have no direct assessment. ${counts.unmatchedNames} names do not match the pinned validation inventory.`,
    'These are independent search/mapping counts, not a completion percentage.',
    '',
    '| Rule | Registered | First source mention | Direct bounded assessments | Next action |',
    '| --- | --- | --- | --- | --- |',
  ];
  for (const rule of audit.rules) {
    const source = byId.get(rule.sourceId);
    lines.push(
      `| [${rule.name}](${sourceUrl(source, sources)}) | ${rule.registrations.length ? 'yes' : 'no'} | ${rule.mentions.length ? location(rule.mentions[0]) : 'not located by name'} | ${rule.assessments.map((entry) => entry.id).join(', ') || 'none'} | ${rule.nextAction} |`,
    );
  }
  lines.push(
    '',
    '## Names outside the pinned validation inventory',
    '',
    'This queue includes deliberate compatibility aliases and labels for other checks. Similar spelling is not sufficient to map a label to a normative rule.',
    '',
    '| Name | First source mention |',
    '| --- | --- |',
  );
  for (const item of audit.unmatchedNames)
    lines.push(`| ${item.name} | ${location(item.mentions[0])} |`);
  return `${lines.join('\n')}\n`;
}
