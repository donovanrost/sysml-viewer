import { test, before } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import init, { analyze } from '../packages/kernel-wasm/sysml_wasm.js';
import { ViewerModel, sourceOf } from '../src/viewer/model.ts';
import { diagramEdges } from '../src/viewer/diagram.ts';
import type { Snapshot } from '../src/kernel/generated/Snapshot';

let snapshot: Snapshot;
let importGuards: Snapshot;
const documents = ['Definitions', 'Mission', 'Errors'].map((name) => ({
  id: name,
  name: `${name}.sysml`,
  text: readFileSync(new URL(`./fixtures/kernel/${name}.sysml`, import.meta.url), 'utf8'),
}));
for (const name of [
  'mult-011',
  'mult-012',
  'mult-013',
  'mult-014',
  'mult-017',
  'mult-018',
  'mult-022',
  'mult-023',
  'mult-024',
  'mult-025',
  'mult-026',
  'mult-027',
  'mult-028',
  'mult-029',
  'mult-030',
  'mult-031',
  'mult-032',
  'mult-033',
  'mult-034',
  'imp-020',
]) {
  documents.push({
    id: name,
    name: `${name}.sysml`,
    text: readFileSync(new URL(`./conformance/fixtures/${name}.sysml`, import.meta.url), 'utf8'),
  });
}
documents.push({
  id: 'unicode',
  name: 'unicode.sysml',
  text: "// 🚢\r\npackage '海' { part def 'Éngine'; part x : 'Éngine'; }",
});
const request = JSON.stringify({ revision: 11, documents });
const guardRequest = JSON.stringify({
  revision: 12,
  documents: [
    ...documents,
    {
      id: 'imp-026',
      name: 'imp-026.sysml',
      text: readFileSync(new URL('./conformance/fixtures/imp-026.sysml', import.meta.url), 'utf8'),
    },
  ],
});

before(async () => {
  await init({
    module_or_path: readFileSync(
      new URL('../packages/kernel-wasm/sysml_wasm_bg.wasm', import.meta.url),
    ),
  });
  snapshot = JSON.parse(analyze(request));
  importGuards = JSON.parse(analyze(guardRequest));
});

test('native CLI and WebAssembly emit identical snapshots', () => {
  const executable = fileURLToPath(
    new URL(
      `../target/release/sysml-inspect${process.platform === 'win32' ? '.exe' : ''}`,
      import.meta.url,
    ),
  );
  for (const [input, wasm] of [
    [request, snapshot],
    [guardRequest, importGuards],
  ] as const) {
    const native = JSON.parse(
      execFileSync(executable, ['--request-json'], {
        input,
        encoding: 'utf8',
        maxBuffer: 10_000_000,
      }),
    );
    assert.deepEqual(wasm, native);
  }
});

test('checked import closure preserves aliases, contextual visibility and unknown providers in WebAssembly', () => {
  const source = snapshot.elements.find((e) => e.qualifiedName === 'ImportSource')!;
  const alias = snapshot.memberships.find((m) => m.namespaceId === source.id && m.isAlias)!;
  const consumer = snapshot.elements.find((e) => e.qualifiedName === 'ImportConsumer')!;
  const closure = consumer.importedMemberships;
  assert.equal(closure.status, 'complete');
  if (closure.status !== 'complete') return;
  assert.equal(closure.memberships.length, 2);
  assert.deepEqual(closure.memberships[0], { membershipId: alias.id, visibility: 'protected' });
  assert.equal(alias.visibility, 'public');
  assert.equal(alias.shortName, 'E');
  assert.deepEqual(
    snapshot.elements.find((e) => e.qualifiedName === 'ImportEmpty')!.importedMemberships,
    { status: 'complete', memberships: [] },
  );
  for (const name of ['MissingImport', 'IncompleteImport', 'FilterImport']) {
    assert.deepEqual(
      importGuards.elements.find((e) => e.qualifiedName === name)!.importedMemberships,
      {
        status: 'unknown',
        reason: 'incompleteProvider',
      },
    );
  }
});

test('unresolved heritage preserves independent arithmetic and conditional certificates', () => {
  for (const e of snapshot.elements.filter(
    (e) =>
      e.qualifiedName?.startsWith('ArithmeticBounds::') ||
      e.qualifiedName?.startsWith('ConditionalProviders::'),
  )) {
    assert.deepEqual(
      importGuards.elements.find((other) => other.qualifiedName === e.qualifiedName)!.cardinality,
      e.cardinality,
      e.qualifiedName!,
    );
  }
});

test('checked attribute bounds preserve exact values and unknown results in WebAssembly', () => {
  for (const [name, lower, upper] of [
    ['viaAlias', '3', '3'],
    ['chain', '3', '3'],
    ['ranged', '3', '6'],
    ['exactLarge', '9007199254740993', '9007199254740993'],
    ['empty', '0', '0'],
  ]) {
    const element = snapshot.elements.find((e) => e.qualifiedName === `ProviderBounds::${name}`)!;
    assert.deepEqual(element.cardinality, { status: 'evaluated', lower, upper });
  }
  for (const [name, reason] of [
    ['unvaluedBound', 'incompleteProvider'],
    ['cycleBound', 'unsupportedBound'],
    ['variationBound', 'unsupportedBound'],
    ['featuredBound', 'incompleteProvider'],
  ]) {
    const element = snapshot.elements.find((e) => e.qualifiedName === `ProviderGuards::${name}`)!;
    assert.deepEqual(element.cardinality, { status: 'unknown', reason });
    assert.equal(element.declaredMultiplicity, null);
  }
});

test('canonical arithmetic bindings agree across native and WebAssembly without rounding or defaults', () => {
  for (const [name, value] of [
    ['add', '10'],
    ['subtract', '5'],
    ['multiply', '16'],
    ['divide', '4'],
    ['modulo', '2'],
    ['power', '8'],
    ['compound', '30'],
    ['unary', '1'],
    ['exactLarge', '9007199254740993'],
  ]) {
    const e = snapshot.elements.find((e) => e.qualifiedName === `ArithmeticBounds::${name}`)!;
    assert.deepEqual(e.cardinality, { status: 'evaluated', lower: value, upper: value });
    assert.equal(e.declaredMultiplicity, `[${value}]`);
  }
  for (const name of ['division', 'fraction', 'belowZero', 'overflow', 'cycle', 'call']) {
    const e = snapshot.elements.find((e) => e.qualifiedName === `ArithmeticGuards::${name}`)!;
    assert.deepEqual(e.cardinality, { status: 'unknown', reason: 'unsupportedBound' });
    assert.equal(e.declaredMultiplicity, null);
  }
});

test('conditional bounds agree across native and WebAssembly with lazy branches and strict Boolean tests', () => {
  for (const [name, value] of [
    ['trueBranch', '3'],
    ['falseBranch', '4'],
    ['nestedBranch', '5'],
    ['arithmetic', '10'],
    ['skippedDivision', '6'],
    ['skippedUnbound', '7'],
    ['exactLarge', '9007199254740993'],
    ['nestedCondition', '8'],
  ]) {
    const e = snapshot.elements.find((e) => e.qualifiedName === `ConditionalProviders::${name}`)!;
    assert.deepEqual(e.cardinality, { status: 'evaluated', lower: value, upper: value });
    assert.equal(e.declaredMultiplicity, `[${value}]`);
  }
  for (const name of [
    'unavailable',
    'wrongType',
    'unbound',
    'division',
    'belowZero',
    'fraction',
    'overflow',
    'recursive',
    'ineligible',
  ]) {
    const e = snapshot.elements.find((e) => e.qualifiedName === `ConditionalGuards::${name}`)!;
    assert.deepEqual(e.cardinality, {
      status: 'unknown',
      reason: ['unavailable', 'unbound'].includes(name) ? 'incompleteProvider' : 'unsupportedBound',
    });
    assert.equal(e.declaredMultiplicity, null);
  }
});

test('compound ranges retain exact endpoints and negative guards across native and WebAssembly', () => {
  for (const state of [snapshot, importGuards]) {
    for (const [name, lower, upper] of [
      ['exact', '5', '5'],
      ['chosen', '10', '10'],
      ['range', '5', '10'],
      ['viaAlias', '5', '10'],
      ['repeated', '10', '10'],
      ['repeatedLower', '5', '5'],
    ]) {
      const e = state.elements.find((e) => e.qualifiedName === `CompoundRanges::${name}`)!;
      assert.deepEqual(e.cardinality, { status: 'evaluated', lower, upper }, name);
    }
    for (const name of ['incomplete', 'fraction', 'nonBoolean', 'reversed']) {
      const e = state.elements.find((e) => e.qualifiedName === `CompoundRangeGuards::${name}`)!;
      assert.deepEqual(
        e.cardinality,
        {
          status: 'unknown',
          reason: name === 'incomplete' ? 'incompleteProvider' : 'unsupportedBound',
        },
        name,
      );
    }
  }
});

test('viewer uses kernel member results and draws resolved endpoint owners', () => {
  const view = new ViewerModel(snapshot);
  const vessel = view.elements.find((e) => e.qualifiedName === 'Mission::vehicle')!;
  const nodes = view.members(vessel.id);
  assert.deepEqual(
    nodes.map((n) => n.name),
    ['a', 'b'],
  );
  const edges = diagramEdges(view, vessel.id, 'connections', nodes);
  assert.equal(edges.length, 1);
  assert.equal(edges[0].a?.name, 'a');
  assert.equal(edges[0].b?.name, 'b');
  assert.equal(edges[0].status, 'resolved');
  const names = view.members(nodes[0].id).map((e) => e.qualifiedName);
  assert(names.includes('Components::SpecialDevice::signal'));
  assert(names.includes('Components::Device::mass'));
  assert(!names.includes('Components::Device::signal'));
});

test('UI source slicing matches kernel byte ranges for non-ASCII input', () => {
  const view = new ViewerModel(snapshot);
  const e = view.elements.find((e) => e.qualifiedName === "'海'::x")!;
  assert.equal(sourceOf(view, e), "part x : 'Éngine';");
});

test('ambiguous and missing references remain distinct in diagrams', () => {
  const view = new ViewerModel(snapshot);
  const problems = view.elements.find((e) => e.name === 'Problems')!;
  const edges = diagramEdges(view, problems.id, 'types', view.members(problems.id));
  assert(edges.some((e) => e.source === 'ambiguous' && e.status === 'ambiguous' && !e.b));
  assert(edges.some((e) => e.source === 'missing' && e.status === 'unresolved' && !e.b));
});

test('bundled example retains four subsystem cards and five semantic connections', () => {
  const result = JSON.parse(
    analyze(
      JSON.stringify({
        revision: 12,
        documents: [
          {
            id: 'example',
            name: 'CoastalExplorer.sysml',
            text: readFileSync(
              new URL('../examples/CoastalExplorer.sysml', import.meta.url),
              'utf8',
            ),
          },
        ],
      }),
    ),
  ) as Snapshot;
  const view = new ViewerModel(result);
  const vessel = view.elements.find((e) => e.name === 'SurveyVessel')!;
  const nodes = view.members(vessel.id);
  assert.equal(nodes.length, 4);
  assert.equal(
    view.elements.find((e) => e.qualifiedName === 'CoastalExplorer::Propulsion::power')?.typeName,
    '~PowerPort',
  );
  const edges = diagramEdges(view, vessel.id, 'connections', nodes);
  assert.equal(edges.length, 5);
  assert(edges.every((e) => e.status === 'resolved' && e.a && e.b));
});

test('wire format rejects duplicate document identities and unknown fields', () => {
  for (const value of [
    { revision: 1, documents: [documents[0], documents[0]] },
    { revision: 1, documents: [], accidentalField: true },
  ])
    assert.throws(() => analyze(JSON.stringify(value)));
});

test('schema 2 memberships and exact cardinalities agree across native and WebAssembly', () => {
  const input = JSON.stringify({
    revision: 24,
    documents: ['imp-008', 'imp-014', 'mult-004', 'mult-005'].map((name) => ({
      id: name,
      name: `${name}.sysml`,
      text: readFileSync(new URL(`./conformance/fixtures/${name}.sysml`, import.meta.url), 'utf8'),
    })),
  });
  const wasm: Snapshot = JSON.parse(analyze(input));
  const executable = fileURLToPath(
    new URL(
      `../target/release/sysml-inspect${process.platform === 'win32' ? '.exe' : ''}`,
      import.meta.url,
    ),
  );
  const native = JSON.parse(
    execFileSync(executable, ['--request-json'], {
      input,
      encoding: 'utf8',
      maxBuffer: 10_000_000,
    }),
  );
  assert.deepEqual(wasm, native);
  assert.equal(wasm.schemaVersion, 3);
  assert(wasm.memberships.some((m) => m.isAlias && m.shortName === 'OnlyShort' && m.name === null));
  assert.deepEqual(
    wasm.elements.find((e) => e.qualifiedName === 'ExactBounds::Assembly::large')!.cardinality,
    { status: 'evaluated', lower: '9007199254740993', upper: '9007199254740993' },
  );
  assert.deepEqual(
    wasm.elements.find((e) => e.qualifiedName === 'UnknownBounds::Assembly::unknown')!.cardinality,
    { status: 'unknown', reason: 'incompleteProvider' },
  );
});

test('ordered predicates retain exact comparisons and operand guards in both workspace contexts', () => {
  for (const state of [snapshot, importGuards]) {
    for (const [name, value] of [
      ['ltLess', '7'],
      ['ltEqual', '9'],
      ['ltGreater', '9'],
      ['leLess', '7'],
      ['leEqual', '7'],
      ['leGreater', '9'],
      ['gtLess', '9'],
      ['gtEqual', '9'],
      ['gtGreater', '7'],
      ['geLess', '9'],
      ['geEqual', '7'],
      ['geGreater', '7'],
      ['large', '11'],
      ['largeReverse', '12'],
      ['limit', '13'],
      ['negative', '14'],
      ['rational', '15'],
      ['rationalEqual', '16'],
      ['rationalMixed', '17'],
      ['rationalClose', '18'],
      ['arithmetic', '19'],
      ['viaAlias', '20'],
      ['nested', '21'],
      ['lazy', '22'],
      ['wideArithmetic', '23'],
    ]) {
      const e = state.elements.find((e) => e.qualifiedName === `OrderedComparisons::${name}`)!;
      assert.deepEqual(e.cardinality, { status: 'evaluated', lower: value, upper: value }, name);
    }
    for (const name of [
      'booleanLeft',
      'booleanRight',
      'real',
      'infinity',
      'division',
      'excessive',
      'booleanBound',
      'missingOperand',
      'selectedFraction',
    ]) {
      const e = state.elements.find((e) => e.qualifiedName === `OrderedComparisonGuards::${name}`)!;
      assert.deepEqual(
        e.cardinality,
        {
          status: 'unknown',
          reason: name === 'missingOperand' ? 'incompleteProvider' : 'unsupportedBound',
        },
        name,
      );
    }
  }
});

test('equality and logical predicates preserve values and guards in both workspace contexts', () => {
  for (const state of [snapshot, importGuards]) {
    for (const [qualifiedName, value] of [
      ['PredicateGuards::stringEquality', '7'],
      ['EmptyComparisonGuards::string', '7'],
      ['PredicateGuards::nullEquality', '7'],
      ['CoalescingGuards::emptyEquality', '7'],
      ['EqualityPredicates::eqEqual', '7'],
      ['EqualityPredicates::eqUnequal', '9'],
      ['EqualityPredicates::eqTrue', '7'],
      ['EqualityPredicates::eqFalse', '9'],
      ['EqualityPredicates::eqRational', '7'],
      ['EqualityPredicates::eqLarge', '9'],
      ['EqualityPredicates::neEqual', '9'],
      ['EqualityPredicates::neUnequal', '7'],
      ['EqualityPredicates::neTrue', '9'],
      ['EqualityPredicates::neFalse', '7'],
      ['EqualityPredicates::neRational', '9'],
      ['EqualityPredicates::neLarge', '7'],
      ['EqualityPredicates::sameEqual', '7'],
      ['EqualityPredicates::sameUnequal', '9'],
      ['EqualityPredicates::sameTrue', '7'],
      ['EqualityPredicates::sameFalse', '9'],
      ['EqualityPredicates::sameRational', '7'],
      ['EqualityPredicates::sameLarge', '9'],
      ['EqualityPredicates::differentEqual', '9'],
      ['EqualityPredicates::differentUnequal', '7'],
      ['EqualityPredicates::differentTrue', '9'],
      ['EqualityPredicates::differentFalse', '7'],
      ['EqualityPredicates::differentRational', '9'],
      ['EqualityPredicates::differentLarge', '7'],
      ['EqualityPredicates::integerRational', '7'],
      ['EqualityPredicates::wideExact', '7'],
      ['EqualityPredicates::closeRational', '9'],
      ['EqualityPredicates::signed', '7'],
      ['EqualityPredicates::viaAlias', '7'],
      ['LogicalPredicates::negateTrue', '9'],
      ['LogicalPredicates::negateFalse', '7'],
      ['LogicalPredicates::conjunction00', '9'],
      ['LogicalPredicates::conjunction01', '9'],
      ['LogicalPredicates::conjunction10', '9'],
      ['LogicalPredicates::conjunction11', '7'],
      ['LogicalPredicates::disjunction00', '9'],
      ['LogicalPredicates::disjunction01', '7'],
      ['LogicalPredicates::disjunction10', '7'],
      ['LogicalPredicates::disjunction11', '7'],
      ['LogicalPredicates::exclusive00', '9'],
      ['LogicalPredicates::exclusive01', '7'],
      ['LogicalPredicates::exclusive10', '7'],
      ['LogicalPredicates::exclusive11', '9'],
      ['LogicalPredicates::and00', '9'],
      ['LogicalPredicates::and01', '9'],
      ['LogicalPredicates::and10', '9'],
      ['LogicalPredicates::and11', '7'],
      ['LogicalPredicates::or00', '9'],
      ['LogicalPredicates::or01', '7'],
      ['LogicalPredicates::or10', '7'],
      ['LogicalPredicates::or11', '7'],
      ['LogicalPredicates::implies00', '7'],
      ['LogicalPredicates::implies01', '7'],
      ['LogicalPredicates::implies10', '9'],
      ['LogicalPredicates::implies11', '7'],
      ['LogicalPredicates::skipAnd', '9'],
      ['LogicalPredicates::skipOr', '7'],
      ['LogicalPredicates::skipImplies', '7'],
      ['LogicalPredicates::skipAndDivision', '9'],
      ['LogicalPredicates::skipOrDivision', '7'],
      ['LogicalPredicates::skipImpliesDivision', '7'],
      ['LogicalPredicates::nested', '7'],
      ['LogicalPredicates::combined', '7'],
      ['LogicalPredicates::viaAlias', '7'],
    ]) {
      const e = state.elements.find((e) => e.qualifiedName === qualifiedName)!;
      assert.deepEqual(
        e.cardinality,
        { status: 'evaluated', lower: value, upper: value },
        qualifiedName,
      );
    }
    for (const [name, reason] of [
      ['mixedEquality', 'unsupportedBound'],
      ['mixedInequality', 'unsupportedBound'],
      ['mixedSame', 'unsupportedBound'],
      ['mixedNotSame', 'unsupportedBound'],
      ['realEquality', 'unsupportedBound'],
      ['infinityEquality', 'unsupportedBound'],
      ['missingEquality', 'incompleteProvider'],
      ['undefinedEquality', 'unsupportedBound'],
      ['excessiveEquality', 'unsupportedBound'],
      ['notNumber', 'unsupportedBound'],
      ['andNumber', 'unsupportedBound'],
      ['orNumber', 'unsupportedBound'],
      ['xorNumber', 'unsupportedBound'],
      ['eagerAnd', 'incompleteProvider'],
      ['eagerOr', 'incompleteProvider'],
      ['eagerXor', 'incompleteProvider'],
      ['neededAnd', 'incompleteProvider'],
      ['neededOr', 'incompleteProvider'],
      ['neededImplies', 'incompleteProvider'],
      ['badFirst', 'unsupportedBound'],
      ['badSecond', 'unsupportedBound'],
      ['eagerDivision', 'unsupportedBound'],
      ['selectedDivision', 'unsupportedBound'],
      ['recursive', 'unsupportedBound'],
      ['booleanBound', 'unsupportedBound'],
      ['logicalBound', 'unsupportedBound'],
    ]) {
      const e = state.elements.find((e) => e.qualifiedName === `PredicateGuards::${name}`)!;
      assert.deepEqual(e.cardinality, { status: 'unknown', reason }, name);
    }
  }
});

test('null-coalescing preserves empty, present and unknown values in both workspace contexts', () => {
  for (const state of [snapshot, importGuards]) {
    for (const [name, value] of [
      ['literalNull', '7'],
      ['emptyParentheses', '7'],
      ['presentInteger', '3'],
      ['presentZero', '0'],
      ['falsePresent', '9'],
      ['truePresent', '7'],
      ['nullBoolean', '7'],
      ['rationalPresent', '7'],
      ['viaAlias', '7'],
      ['viaValueAlias', '3'],
      ['nestedEmpty', '7'],
      ['nestedFallback', '7'],
      ['conditionalEmpty', '7'],
      ['conditionalPresent', '3'],
      ['skippedMissing', '3'],
      ['skippedDivision', '3'],
      ['skippedOverflow', '3'],
      ['skippedReal', '3'],
      ['skippedNull', '3'],
      ['arithmeticFallback', '7'],
      ['composedArithmetic', '7'],
      ['composedLogical', '7'],
      ['largeExact', '9007199254740993'],
      ['conditionalSkip', '7'],
    ]) {
      const element = state.elements.find((e) => e.qualifiedName === `CoalescingValues::${name}`)!;
      assert.deepEqual(
        element.cardinality,
        { status: 'evaluated', lower: value, upper: value },
        name,
      );
    }
    for (const [name, reason] of [
      ['overflowEndpoint', 'unsupportedBound'],
      ['underflowFirst', 'unsupportedBound'],
      ['irrationalFirst', 'unsupportedBound'],
      ['missingFirst', 'incompleteProvider'],
      ['selectedMissing', 'incompleteProvider'],
      ['undefinedFirst', 'unsupportedBound'],
      ['selectedDivision', 'unsupportedBound'],
      ['overflowFirst', 'unsupportedBound'],
      ['bothEmpty', 'unsupportedBound'],
      ['bareEmpty', 'unsupportedBound'],
      ['booleanEndpoint', 'unsupportedBound'],
      ['fractionalEndpoint', 'unsupportedBound'],
      ['negativeEndpoint', 'unsupportedBound'],
      ['realFirst', 'unsupportedBound'],
      ['selectedReal', 'unsupportedBound'],
      ['stringFirst', 'unsupportedBound'],
      ['sequenceFirst', 'unsupportedBound'],
      ['selectedSequence', 'unsupportedBound'],
      ['recursiveFirst', 'unsupportedBound'],
      ['recursiveSkipped', 'unsupportedBound'],
      ['emptyUnaryPlus', 'unsupportedBound'],
      ['emptyArithmetic', 'unsupportedBound'],
      ['emptyCondition', 'unsupportedBound'],
    ]) {
      const element = state.elements.find((e) => e.qualifiedName === `CoalescingGuards::${name}`)!;
      assert.deepEqual(element.cardinality, { status: 'unknown', reason }, name);
    }
  }
});

test('empty comparisons preserve presence and refuse unavailable operands in both workspace contexts', () => {
  for (const state of [snapshot, importGuards]) {
    for (const [name, value] of [
      ['eqNull', '7'],
      ['eqParentheses', '7'],
      ['eqZeroRight', '9'],
      ['eqZeroLeft', '9'],
      ['eqFalseRight', '9'],
      ['eqFalseLeft', '9'],
      ['eqRational', '9'],
      ['neNull', '9'],
      ['neParentheses', '9'],
      ['neZeroRight', '7'],
      ['neZeroLeft', '7'],
      ['neFalseRight', '7'],
      ['neFalseLeft', '7'],
      ['neRational', '7'],
      ['sameNull', '7'],
      ['sameParentheses', '7'],
      ['sameZeroRight', '9'],
      ['sameZeroLeft', '9'],
      ['sameFalseRight', '9'],
      ['sameFalseLeft', '9'],
      ['sameRational', '9'],
      ['differentNull', '9'],
      ['differentParentheses', '9'],
      ['differentZeroRight', '7'],
      ['differentZeroLeft', '7'],
      ['differentFalseRight', '7'],
      ['differentFalseLeft', '7'],
      ['differentRational', '7'],
      ['rationalLeft', '7'],
      ['aliasEmpty', '7'],
      ['aliasPresent', '9'],
      ['conditionalEmpty', '7'],
      ['coalescedEmpty', '7'],
      ['coalescedPresent', '7'],
      ['logicalSkip', '7'],
      ['fallbackFalse', '9'],
    ]) {
      const element = state.elements.find((e) => e.qualifiedName === `EmptyComparisons::${name}`)!;
      assert.deepEqual(
        element.cardinality,
        { status: 'evaluated', lower: value, upper: value },
        name,
      );
    }
    for (const [name, reason] of [
      ['eqMissingRight', 'incompleteProvider'],
      ['eqMissingLeft', 'incompleteProvider'],
      ['neMissingRight', 'incompleteProvider'],
      ['neMissingLeft', 'incompleteProvider'],
      ['sameMissingRight', 'incompleteProvider'],
      ['sameMissingLeft', 'incompleteProvider'],
      ['differentMissingRight', 'incompleteProvider'],
      ['differentMissingLeft', 'incompleteProvider'],
      ['divisionRight', 'unsupportedBound'],
      ['divisionLeft', 'unsupportedBound'],
      ['overflow', 'unsupportedBound'],
      ['real', 'unsupportedBound'],
      ['infinity', 'unsupportedBound'],
      ['sequence', 'unsupportedBound'],
      ['recursive', 'unsupportedBound'],
      ['booleanEndpoint', 'unsupportedBound'],
      ['emptySelected', 'unsupportedBound'],
      ['numericCondition', 'unsupportedBound'],
      ['eagerMissing', 'incompleteProvider'],
    ]) {
      const element = state.elements.find(
        (e) => e.qualifiedName === `EmptyComparisonGuards::${name}`,
      )!;
      assert.deepEqual(element.cardinality, { status: 'unknown', reason }, name);
    }
  }
});

test('strings preserve decoded values, presence and guards in both workspace contexts', () => {
  for (const state of [snapshot, importGuards]) {
    for (const [name, value] of [
      ['eqEqual', '7'],
      ['eqCase', '9'],
      ['eqEmpty', '7'],
      ['neEqual', '9'],
      ['neCase', '7'],
      ['neEmpty', '9'],
      ['sameEqual', '7'],
      ['sameCase', '9'],
      ['sameEmpty', '7'],
      ['differentEqual', '9'],
      ['differentCase', '7'],
      ['differentEmpty', '9'],
      ['unicode', '7'],
      ['distinctUnicode', '7'],
      ['escapedBackslash', '7'],
      ['escapedQuote', '7'],
      ['allEscapes', '7'],
      ['concatenate', '7'],
      ['unicodeConcatenate', '7'],
      ['nestedConcatenate', '7'],
      ['emptyConcatenate', '7'],
      ['viaAlias', '7'],
      ['nullRight', '9'],
      ['nullLeft', '7'],
      ['emptyStringPresent', '7'],
      ['nullFallback', '7'],
      ['skipMissing', '7'],
      ['selectedString', '7'],
      ['logicalSkip', '7'],
    ]) {
      const e = state.elements.find((e) => e.qualifiedName === `StringValues::${name}`)!;
      assert.deepEqual(e.cardinality, { status: 'evaluated', lower: value, upper: value }, name);
    }
    for (const [name, reason] of [
      ['stringEndpoint', 'unsupportedBound'],
      ['concatEndpoint', 'unsupportedBound'],
      ['stringCondition', 'unsupportedBound'],
      ['unaryPlus', 'unsupportedBound'],
      ['unaryMinus', 'unsupportedBound'],
      ['subtract', 'unsupportedBound'],
      ['numberConcat', 'unsupportedBound'],
      ['boolConcat', 'unsupportedBound'],
      ['nullConcat', 'unsupportedBound'],
      ['sequenceConcat', 'unsupportedBound'],
      ['numberEquality', 'unsupportedBound'],
      ['boolEquality', 'unsupportedBound'],
      ['missingRight', 'incompleteProvider'],
      ['missingLeft', 'incompleteProvider'],
      ['selectedMissing', 'incompleteProvider'],
      ['recursive', 'unsupportedBound'],
      ['less', 'unsupportedBound'],
      ['lessEqual', 'unsupportedBound'],
      ['greater', 'unsupportedBound'],
      ['greaterEqual', 'unsupportedBound'],
    ]) {
      const e = state.elements.find((e) => e.qualifiedName === `StringGuards::${name}`)!;
      assert.deepEqual(e.cardinality, { status: 'unknown', reason }, name);
    }
  }
});
