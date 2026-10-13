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
const documents = ['Definitions', 'Mission', 'Errors'].map((name) => ({
  id: name,
  name: `${name}.sysml`,
  text: readFileSync(new URL(`./fixtures/kernel/${name}.sysml`, import.meta.url), 'utf8'),
}));
documents.push({
  id: 'unicode',
  name: 'unicode.sysml',
  text: "// 🚢\r\npackage '海' { part def 'Éngine'; part x : 'Éngine'; }",
});
const request = JSON.stringify({ revision: 11, documents });

before(async () => {
  await init({
    module_or_path: readFileSync(
      new URL('../packages/kernel-wasm/sysml_wasm_bg.wasm', import.meta.url),
    ),
  });
  snapshot = JSON.parse(analyze(request));
});

test('native CLI and WebAssembly emit identical snapshots', () => {
  const executable = fileURLToPath(
    new URL(
      `../target/release/sysml-inspect${process.platform === 'win32' ? '.exe' : ''}`,
      import.meta.url,
    ),
  );
  const native = JSON.parse(
    execFileSync(executable, ['--request-json'], {
      input: request,
      encoding: 'utf8',
      maxBuffer: 10_000_000,
    }),
  );
  assert.deepEqual(snapshot, native);
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
