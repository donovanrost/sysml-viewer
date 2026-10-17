import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { buildValidationAudit, validationScanRoots } from '../tools/lib/validation-audit.mjs';

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url));
const json = (path) => JSON.parse(read(path));
const walk = (path) =>
  readdirSync(new URL(`../${path}`, import.meta.url), { withFileTypes: true }).flatMap((entry) => {
    const child = `${path}/${entry.name}`;
    return entry.isDirectory() ? walk(child) : entry.name.endsWith('.rs') ? [child] : [];
  });
const inputs = () => ({
  catalogue: json('standards/catalogue.json'),
  assessments: json('standards/assessments.json'),
  matrix: json('tests/conformance/matrix.json'),
  paths: validationScanRoots.flatMap(walk),
  read,
});

test('validation audit inventories every named rule without promoting search hits to coverage', () => {
  const data = inputs();
  const audit = buildValidationAudit(data);
  assert.equal(audit.rules.length, 180);
  assert.deepEqual(
    new Set(audit.rules.map((r) => r.sourceId)),
    new Set(
      data.catalogue.items
        .filter((r) => r.kind === 'constraint' && r.ruleKind === 'validate')
        .map((r) => r.id),
    ),
  );
  const absent = audit.rules.find(
    (r) => r.name === 'validateConjugatedPortDefinitionOriginalPortDefinition',
  );
  assert.equal(absent.nextAction, 'locate-or-implement');
  assert.deepEqual(absent.assessments, []);
  const mentioned = audit.rules.find(
    (r) => r.name === 'validateSpecificationSpecificNotConjugated',
  );
  assert.equal(mentioned.nextAction, 'review-code-candidate');
  assert.equal(mentioned.registrations.length, 0);
  assert(mentioned.mentions.some((site) => site.path.endsWith('/json/implied.rs')));
  const assessed = audit.rules.find((r) => r.name === 'validateNamespaceDistinguishibility');
  assert.equal(assessed.nextAction, 'complete-obligation-review');
  assert(
    assessed.assessments.every((a) =>
      a.tests.every((t) => t.status === 'mapped-not-execution-evidence'),
    ),
  );
  assert.deepEqual(buildValidationAudit({ ...data, paths: [...data.paths].reverse() }), audit);
});

test('parent clauses and planned tests cannot silently qualify a rule', () => {
  const data = inputs();
  const rule = data.catalogue.items.find(
    (r) => r.title === 'validateConjugatedPortDefinitionOriginalPortDefinition',
  );
  const assessment = structuredClone(data.assessments.assessments[0]);
  assessment.id = 'AUDIT-TEST';
  assessment.sources = [rule.parent];
  assessment.tests = [{ case: 'AUDIT-PLANNED', checks: [] }];
  data.assessments.assessments.push(assessment);
  data.matrix.cases.push({ id: 'AUDIT-PLANNED', planned: 'Test only', polarity: 'negative' });
  let row = buildValidationAudit(data).rules.find((r) => r.sourceId === rule.id);
  assert.deepEqual(row.assessments, []);
  assessment.sources = [rule.id];
  row = buildValidationAudit(data).rules.find((r) => r.sourceId === rule.id);
  assert.equal(row.assessments[0].tests[0].status, 'planned');
  assert.equal(row.nextAction, 'complete-obligation-review');
  assessment.tests[0].checks = ['nonexistent'];
  assert.throws(() => buildValidationAudit(data), /Missing audit assertion/);
});

test('unmatched labels remain visible and unknown registry claims fail', () => {
  const data = inputs();
  const path = 'vendor/sysml-toolkit/crates/sysmlv2-model/src/check.rs';
  const original = read(path).toString();
  const label = 'validateSyntheticAuditRule';
  const withText = (text) => ({ ...data, read: (p) => (p === path ? Buffer.from(text) : read(p)) });
  const audit = buildValidationAudit(withText(`${original}\n// ${label}\n`));
  assert(audit.unmatchedNames.some((r) => r.name === label));
  assert(!audit.rules.some((r) => r.name === label));
  assert.throws(
    () =>
      buildValidationAudit(
        withText(
          original.replace(
            'pub const IMPLEMENTED_NORMATIVE_SEMANTIC_RULES: &[&str] = &[',
            `pub const IMPLEMENTED_NORMATIVE_SEMANTIC_RULES: &[&str] = &["${label}",`,
          ),
        ),
      ),
    /Non-normative registry entry/,
  );
  assert.throws(
    () => buildValidationAudit({ ...data, paths: data.paths.filter((p) => p !== path) }),
    /Missing validation registry/,
  );
});
