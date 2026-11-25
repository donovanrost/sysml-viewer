import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import init, { analyze } from '../packages/kernel-wasm/sysml_wasm.js';
import { ViewerModel, referenceTarget } from '../src/viewer/model.ts';
import { buildSystemMap } from '../src/viewer/system-map.ts';
import {
  buildStructureMap,
  buildTypeHierarchy,
  layoutTypeHierarchy,
} from '../src/viewer/semantic-views.ts';
import type { Snapshot } from '../src/kernel/generated/Snapshot';

let model: ViewerModel;
before(async () => {
  await init({
    module_or_path: readFileSync(
      new URL('../packages/kernel-wasm/sysml_wasm_bg.wasm', import.meta.url),
    ),
  });
  const snapshot: Snapshot = JSON.parse(
    analyze(
      JSON.stringify({
        revision: 1,
        documents: [
          {
            id: 'semantic-view-fixture',
            name: 'Vessels.sysml',
            text:
              readFileSync(new URL('../examples/CoastalExplorer.sysml', import.meta.url), 'utf8') +
              `
package Fleet {
  attribute def PositivePercentage :> CoastalExplorer::Percentage;
  attribute def Coordinate { attribute x; attribute y; }
  attribute def Position :> Coordinate;
  part def Gauge { attribute reading : CoastalExplorer::Percentage; }
  part fleet {
    part alpha : CoastalExplorer::SurveyVessel;
    part beta : CoastalExplorer::SurveyVessel;
  }
  part def Sensor { attribute range; }
  part def Platform { part sensor : Sensor; }
  part def Improved :> Platform { part sensor : Sensor :>> Platform::sensor; }
  part upgraded : Improved;
  part def Recursive { part next : Recursive; }
  part recursive : Recursive;
  part def Top;
  part def Left :> Top;
  part def Right :> Top;
  part def Bottom :> Left, Right;
}
`,
          },
        ],
      }),
    ),
  );
  model = new ViewerModel(snapshot);
});
const element = (name: string) => model.elements.find((e) => e.qualifiedName === name)!;

test('shared leaf value types stay on property labels without becoming topology hubs', () => {
  const graph = buildSystemMap(model);
  const percentage = element('CoastalExplorer::Percentage');
  for (const name of [
    'CoastalExplorer::Percentage',
    'CoastalExplorer::Metres',
    'Fleet::PositivePercentage',
  ])
    assert.ok(!graph.nodes.has(element(name).id), name);
  for (const name of ['CoastalExplorer::Battery::charge', 'Fleet::Gauge::reading']) {
    const property = element(name);
    assert.ok(graph.nodes.has(property.id));
    const typing = property.references.find((r) => r.kind === 'typing')!;
    assert.equal(referenceTarget(model, typing)?.id, percentage.id);
    assert.ok(property.typeName?.endsWith('Percentage'));
    assert.ok(!graph.relations.some((r) => r.sourceId === property.id && r.kind === 'typing'));
    assert.ok(!graph.issues.some((i) => i.ownerId === property.id));
  }
  // Filtering is local to the canvas; the explorer and inspector keep the facts.
  assert.ok(model.children(element('CoastalExplorer').id).includes(percentage));
  assert.equal(element('CoastalExplorer::Battery::charge').value, '100');
  for (const name of ['CoastalExplorer::PowerPort', 'CoastalExplorer::DataPort', 'Fleet::Top'])
    assert.ok(graph.nodes.has(element(name).id), `Structural leaf ${name} remains visible`);
});

test('value definitions can be explored explicitly, and structured or incomplete types stay visible', () => {
  const percentage = element('CoastalExplorer::Percentage');
  const hierarchy = buildTypeHierarchy(model, percentage.id);
  assert.ok(hierarchy.nodes.has(percentage.id));
  assert.ok(hierarchy.nodes.has(element('Fleet::PositivePercentage').id));
  assert.equal(hierarchy.relations.length, 1);
  assert.equal(hierarchy.relations[0].targetId, percentage.id);
  const graph = buildSystemMap(model);
  assert.ok(graph.nodes.has(element('Fleet::Coordinate').id));
  assert.ok(graph.nodes.has(element('Fleet::Position').id), 'Inherited members are also detail');
  const snapshot = structuredClone(model.snapshot);
  snapshot.elements.find((e) => e.id === percentage.id)!.inheritanceComplete = false;
  assert.ok(buildSystemMap(new ViewerModel(snapshot)).nodes.has(percentage.id));
});

test('repeated attributes retain their type and declared value without external type nodes', () => {
  const graph = buildStructureMap(model, element('Fleet::fleet').id);
  const charges = [...graph.nodes].filter(
    ([, n]) => n.element.qualifiedName === 'CoastalExplorer::Battery::charge',
  );
  assert.equal(charges.length, 2);
  assert.notEqual(charges[0][0], charges[1][0]);
  assert.ok(
    charges.every(([, n]) => n.element.typeName === 'Percentage' && n.element.value === '100'),
  );
  assert.ok(
    ![...graph.nodes.values()].some(
      (n) => n.element.id === element('CoastalExplorer::Percentage').id,
    ),
  );
  assert.ok(graph.relations.every((r) => r.kind === 'connection'));
});

test('typed usage exposes effective structure and connections with declaration provenance', () => {
  const usage = element('CoastalExplorer::researchVessel');
  const graph = buildStructureMap(model, usage.id);
  const root = graph.nodes.get(usage.id)!;
  assert.deepEqual(root.children.map((id) => graph.nodes.get(id)!.element.name).sort(), [
    'battery',
    'navigation',
    'propulsion',
    'sonar',
  ]);
  assert.equal(graph.relations.length, 5);
  assert.deepEqual(graph.issues, []);
  assert.ok(root.children.every((id) => graph.nodes.get(id)!.inheritedFrom?.definition));
  const battery = graph.nodes.get(
    root.children.find((id) => graph.nodes.get(id)!.element.name === 'battery')!,
  )!;
  assert.equal(battery.inheritedFrom?.name, 'ResearchVessel');
  assert.equal(battery.contextName, 'researchVessel.battery');
  assert.equal(battery.element.parentId, element('CoastalExplorer::ResearchVessel').id);
  assert.ok(
    graph.relations.every(
      (r) =>
        graph.nodes.get(r.sourceId)!.element.kind === 'port' &&
        graph.nodes.get(r.targetId)!.element.kind === 'port',
    ),
  );
  assert.ok(graph.relations.every((r) => model.byId.get(r.connectionId!)?.semantic.source));
  assert.equal(graph.relations.filter((r) => r.kind === 'typing').length, 0);
});

test('two uses of one definition keep inherited ports and connections in their own contexts', () => {
  const graph = buildStructureMap(model, element('Fleet::fleet').id);
  assert.equal(graph.relations.length, 10);
  const supplies = [...graph.nodes].filter(
    ([, n]) => n.element.qualifiedName === 'CoastalExplorer::Battery::supply',
  );
  assert.equal(supplies.length, 2);
  assert.notEqual(supplies[0][0], supplies[1][0]);
  assert.equal(supplies[0][1].element.id, supplies[1][1].element.id);
  for (const r of graph.relations) {
    const from = graph.nodes.get(r.sourceId)!.contextName!.split('.')[1];
    const to = graph.nodes.get(r.targetId)!.contextName!.split('.')[1];
    assert.equal(from, to);
  }
});

test('projection respects the kernel redefinition result without adding the replaced member', () => {
  const graph = buildStructureMap(model, element('Fleet::upgraded').id);
  const sensors = [...graph.nodes.values()].filter((n) => n.element.name === 'sensor');
  assert.equal(sensors.length, 1);
  assert.equal(sensors[0].element.id, element('Fleet::Improved::sensor').id);
  assert.ok(sensors[0].element.references.some((r) => r.kind === 'redefinition'));
});

test('recursive and bounded structure remain finite and expose their continuation boundary', () => {
  const graph = buildStructureMap(model, element('Fleet::recursive').id);
  assert.ok(graph.nodes.size < 10);
  assert.ok([...graph.nodes.values()].some((n) => n.note?.includes('Recursive structure')));
  const bounded = buildStructureMap(model, element('Fleet::fleet').id, { limit: 8 });
  assert.equal(bounded.nodes.size, 8);
  assert.ok([...bounded.nodes.values()].some((n) => n.note?.includes('Detail limit')));
});

test('unknown endpoint identities are reported, never matched to another usage by name', () => {
  const snapshot = structuredClone(model.snapshot);
  const target = snapshot.connections.find(
    (c) => model.byId.get(c.elementId)?.parentId === element('CoastalExplorer::ResearchVessel').id,
  )!;
  target.ends[0].chain[0] = 'missing-context';
  const graph = buildStructureMap(
    new ViewerModel(snapshot),
    element('CoastalExplorer::researchVessel').id,
  );
  assert.equal(graph.relations.length, 4);
  assert.equal(graph.issues.length, 1);
});

test('incomplete inheritance remains explicit in a structure view', () => {
  const snapshot = structuredClone(model.snapshot);
  const id = element('CoastalExplorer::researchVessel').id;
  snapshot.elements.find((e) => e.id === id)!.inheritanceComplete = false;
  const graph = buildStructureMap(new ViewerModel(snapshot), id);
  assert.match(graph.nodes.get(id)!.note!, /incomplete/);
});

test('definition hierarchy separates specialization from usage typing and containment', () => {
  const survey = element('CoastalExplorer::SurveyVessel').id;
  const research = element('CoastalExplorer::ResearchVessel').id;
  const graph = buildTypeHierarchy(model, survey);
  assert.deepEqual(new Set(graph.nodes.keys()), new Set([survey, research]));
  assert.equal(graph.relations.length, 1);
  assert.equal(graph.relations[0].sourceId, survey);
  assert.equal(graph.relations[0].targetId, research);
  assert.ok([...graph.nodes.values()].every((n) => n.parentId === null));
  const layout = layoutTypeHierarchy(graph);
  assert.ok(layout.boxes.get(research)!.y < layout.boxes.get(survey)!.y);
  assert.throws(() => buildTypeHierarchy(model, element('CoastalExplorer::researchVessel').id));
});

test('multiple inheritance is a graph with shared ancestors and all general types above specifics', () => {
  const graph = buildTypeHierarchy(model, element('Fleet::Bottom').id);
  const layout = layoutTypeHierarchy(graph);
  assert.equal(graph.nodes.size, 4);
  assert.equal(graph.relations.length, 4);
  assert.ok(
    graph.relations.every((r) => layout.boxes.get(r.targetId)!.y < layout.boxes.get(r.sourceId)!.y),
  );
});

test('cyclic specialization stays visible and is not presented as a valid ordered hierarchy', () => {
  const graph = buildTypeHierarchy(model, element('Fleet::Bottom').id);
  const edge = graph.relations[0];
  graph.relations.push({
    ...edge,
    id: 'cycle',
    sourceId: element('Fleet::Top').id,
    targetId: element('Fleet::Bottom').id,
  });
  const layout = layoutTypeHierarchy(graph);
  assert.equal(layout.boxes.size, 4);
  assert.ok([...graph.nodes.values()].every((n) => n.note?.includes('Cyclic')));
});
