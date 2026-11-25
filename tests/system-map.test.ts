import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import init, { analyze } from '../packages/kernel-wasm/sysml_wasm.js';
import { ViewerModel } from '../src/viewer/model.ts';
import {
  buildSystemMap,
  layoutSystemMap,
  projectSystemMap,
  groupMapRelations,
  fitCamera,
  focusCamera,
  zoomCamera,
  screenRect,
} from '../src/viewer/system-map.ts';
import type { Snapshot } from '../src/kernel/generated/Snapshot';

let snapshot: Snapshot;
let model: ViewerModel;
before(async () => {
  await init({
    module_or_path: readFileSync(
      new URL('../packages/kernel-wasm/sysml_wasm_bg.wasm', import.meta.url),
    ),
  });
  snapshot = JSON.parse(
    analyze(
      JSON.stringify({
        revision: 1,
        documents: [
          {
            id: 'map-fixture',
            name: 'Map.sysml',
            text: `
package Tenancy { part def Mission; }
package MissionModel { part def Layer; item def Revision; }
package Catalog {
  part def Database {
    part mission : Tenancy::Mission;
    item revisions : CatalogRevision[*];
    attribute missing : MissingType;
    attribute title : ScalarValues::String;
    @Code { path = "implementation.ex"; }
  }
  item def CatalogRevision {
    part layer : MissionModel::Layer;
    item revision : MissionModel::Revision;
  }
}
`,
          },
        ],
      }),
    ),
  );
  model = new ViewerModel(snapshot);
});
const id = (name: string) => model.elements.find((e) => e.qualifiedName === name)!.id;

test('overview rolls nested cross-package type references up without fabricating connections', () => {
  const graph = buildSystemMap(model),
    layout = layoutSystemMap(graph);
  const projection = projectSystemMap(
    graph,
    layout,
    { x: 0, y: 0, scale: 0.1 },
    { x: 0, y: 0, width: 1200, height: 600 },
  );
  const groups = groupMapRelations(graph, projection, 'types');
  const mission = groups.find(
    (g) => g.sourceId === id('Catalog') && g.targetId === id('MissionModel'),
  )!;
  assert.equal(mission.kind, 'typing');
  assert.equal(mission.records.length, 2);
  assert.equal(mission.directed, true);
  assert.deepEqual(mission.records.map((r) => r.sourceLabel).sort(), [
    'Catalog::CatalogRevision::layer',
    'Catalog::CatalogRevision::revision',
  ]);
  assert.ok(mission.records.every((r) => r.reference?.source.documentId === 'map-fixture'));
  assert.equal(groupMapRelations(graph, projection, 'connections').length, 0);
  assert.equal(
    groups.find((g) => g.sourceId === id('Catalog') && g.targetId === id('Tenancy'))?.records
      .length,
    1,
  );
});

test('detail changes preserve exact provenance and do not move layout positions', () => {
  const graph = buildSystemMap(model),
    layout = layoutSystemMap(graph),
    original = structuredClone(layout.boxes);
  const overview = projectSystemMap(
    graph,
    layout,
    { x: 0, y: 0, scale: 0.1 },
    { x: 0, y: 0, width: 1200, height: 600 },
  );
  const camera = fitCamera(layout.boxes.get(id('Catalog'))!, 1200, 800);
  const detail = projectSystemMap(graph, layout, camera, { x: 0, y: 0, width: 1200, height: 800 });
  const records = (p: typeof overview) =>
    groupMapRelations(graph, p, 'types')
      .flatMap((g) => g.records)
      .filter((r) => r.targetLabel.startsWith('MissionModel::'))
      .map((r) => r.id)
      .sort();
  assert.deepEqual(records(detail), records(overview));
  assert.ok(detail.expanded.has(id('Catalog')));
  assert.notEqual(detail.representatives.get(id('Catalog::CatalogRevision::layer')), id('Catalog'));
  assert.deepEqual(layout.boxes, original);
});

test('unresolved types, library types and annotation bodies remain distinct from domain links', () => {
  const graph = buildSystemMap(model);
  assert.ok(graph.issues.some((i) => i.target === 'MissingType' && i.status === 'unresolved'));
  assert.ok(graph.libraryReferences > 0);
  assert.ok(
    ![...graph.nodes.values()].some((n) => n.element.semantic.metaclass === 'MetadataUsage'),
  );
  assert.ok(![...graph.nodes.values()].some((n) => n.element.name === 'path'));
  assert.ok(graph.relations.every((r) => graph.nodes.has(r.targetId)));
  assert.ok(!graph.relations.some((r) => r.targetLabel === 'MissingType'));
});

test('undirected connection grouping preserves both authored records and reports n-ary omissions', () => {
  const copy = structuredClone(snapshot);
  const owner = model.byId.get(id('Catalog::Database'))!.semantic;
  const connection = {
    ...owner,
    id: 'test-connection',
    metaclass: 'ConnectionUsage',
    name: 'link',
    parentId: id('Catalog'),
    ownedMembers: [],
    effectiveMembers: [],
  };
  copy.elements.push(connection);
  const a = {
    chain: [id('Catalog::Database::mission')],
    spelling: 'mission',
    resolution: { status: 'resolved' as const, targetId: id('Catalog::Database::mission') },
  };
  const b = {
    chain: [id('Tenancy::Mission')],
    spelling: 'Mission',
    resolution: { status: 'resolved' as const, targetId: id('Tenancy::Mission') },
  };
  copy.connections = [
    { elementId: connection.id, ends: [a, b] },
    { elementId: connection.id, ends: [b, a] },
    { elementId: connection.id, ends: [a, b, a] },
  ];
  const graph = buildSystemMap(new ViewerModel(copy)),
    layout = layoutSystemMap(graph);
  const p = projectSystemMap(
    graph,
    layout,
    { x: 0, y: 0, scale: 0.1 },
    { x: 0, y: 0, width: 1200, height: 600 },
  );
  const groups = groupMapRelations(graph, p, 'connections');
  assert.equal(groups.length, 1);
  assert.equal(groups[0].directed, false);
  assert.equal(groups[0].records.length, 2);
  assert.ok(graph.issues.some((i) => i.status.includes('3-ended')));

  // A port inherited from a reusable definition must stay anchored to the
  // contextual usage, not jump to the definition's ownership region.
  copy.connections = [
    {
      elementId: connection.id,
      ends: [
        {
          ...a,
          chain: [id('Catalog::Database::mission'), id('Tenancy::Mission')],
          resolution: { status: 'resolved', targetId: id('Tenancy::Mission') },
        },
        {
          ...b,
          chain: [id('MissionModel::Layer')],
          resolution: { status: 'resolved', targetId: id('MissionModel::Layer') },
        },
      ],
    },
  ];
  const contextual = buildSystemMap(new ViewerModel(copy));
  assert.equal(
    contextual.relations.find((r) => r.kind === 'connection')!.sourceId,
    id('Catalog::Database::mission'),
  );
});

test('camera zoom keeps the pointed-at model coordinate fixed; fit contains the chosen region', () => {
  const camera = { x: 56, y: -82, scale: 0.4 },
    point = { x: 270, y: 150 };
  const next = zoomCamera(camera, 2.5, point);
  assert.equal((point.x - next.x) / next.scale, (point.x - camera.x) / camera.scale);
  assert.equal((point.y - next.y) / next.scale, (point.y - camera.y) / camera.scale);
  const box = { x: 1000, y: 400, width: 600, height: 380 };
  const fitted = screenRect(box, fitCamera(box, 800, 500));
  assert.ok(
    fitted.x >= 0 &&
      fitted.y >= 0 &&
      fitted.x + fitted.width <= 800 &&
      fitted.y + fitted.height <= 500,
  );
});

test('detail uses hysteresis and culls rendering while retaining off-screen relationship targets', () => {
  const graph = buildSystemMap(model),
    layout = layoutSystemMap(graph);
  const root = id('Catalog'),
    box = layout.boxes.get(root)!;
  const camera = { x: -box.x * 0.35, y: -box.y * 0.35, scale: 0.35 };
  const viewport = { x: 0, y: 0, width: 1000, height: 600 };
  const collapsed = projectSystemMap(graph, layout, camera, viewport);
  const expanded = projectSystemMap(graph, layout, camera, viewport, new Set([root]));
  assert.ok(!collapsed.expanded.has(root));
  assert.ok(expanded.expanded.has(root));
  const far = projectSystemMap(graph, layout, { x: -100000, y: -100000, scale: 1 }, viewport);
  assert.equal(far.visible.length, 0);
  assert.equal(far.representatives.size, graph.nodes.size);
  const bounded = projectSystemMap(
    graph,
    layout,
    fitCamera(box, 1000, 700),
    viewport,
    new Set(),
    3,
  );
  assert.ok(bounded.visible.length <= 3);
});

test('focusing a leaf keeps its ancestors open even when the relationship tray reduces the viewport', () => {
  const graph = buildSystemMap(model),
    layout = layoutSystemMap(graph);
  const target = id('MissionModel::Layer');
  const viewport = { x: 0, y: 0, width: 620, height: 200 };
  const camera = focusCamera(graph, layout, target, viewport.width, viewport.height);
  const projection = projectSystemMap(graph, layout, camera, viewport);
  assert.ok(projection.expanded.has(id('MissionModel')));
  assert.ok(projection.visible.includes(target));
  assert.equal(projection.representatives.get(target), target);
});

test('a 6,048-element project retains cross-domain provenance with bounded rendering', () => {
  const base = model.byId.get(id('Catalog::Database'))!.semantic;
  const large: Snapshot = {
    ...structuredClone(snapshot),
    elements: [],
    references: [],
    connections: [],
    memberships: [],
    diagnostics: [],
  };
  for (let domain = 0; domain < 27; domain++) {
    const parent = `domain-${domain}`;
    large.elements.push({
      ...base,
      id: parent,
      parentId: null,
      name: `Domain${domain}`,
      qualifiedName: `Domain${domain}`,
      metaclass: 'Package',
      ownedMembers: [],
      effectiveMembers: [],
    });
    for (let member = 0; member < 223; member++) {
      const key = `${parent}-${member}`;
      large.elements.push({
        ...base,
        id: key,
        parentId: parent,
        name: `Member${member}`,
        qualifiedName: `Domain${domain}::Member${member}`,
        ownedMembers: [],
        effectiveMembers: [],
      });
      large.references.push({
        ownerId: key,
        kind: 'typing',
        spelling: `Domain${(domain + 1) % 27}::Member${member}`,
        source: base.source!,
        resolution: { status: 'resolved', targetId: `domain-${(domain + 1) % 27}-${member}` },
      });
    }
  }
  const graph = buildSystemMap(new ViewerModel(large)),
    layout = layoutSystemMap(graph);
  const overview = projectSystemMap(graph, layout, fitCamera(layout.bounds, 1000, 700), {
    x: 0,
    y: 0,
    width: 1000,
    height: 700,
  });
  assert.equal(graph.nodes.size, 6048);
  assert.ok(overview.visible.length <= 300);
  const groups = groupMapRelations(graph, overview, 'types');
  assert.equal(
    groups.reduce((sum, g) => sum + g.records.length, 0),
    6021,
  );
  assert.equal(groups.length, 27);
  const target = layout.boxes.get('domain-0-100')!;
  const detail = projectSystemMap(graph, layout, fitCamera(target, 1000, 700), {
    x: 0,
    y: 0,
    width: 1000,
    height: 700,
  });
  assert.ok(detail.visible.includes('domain-0-100'));
  assert.ok(detail.visible.length <= 300);
});
