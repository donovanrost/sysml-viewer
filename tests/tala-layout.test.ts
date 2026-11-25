import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { CompileResponse } from '@d2lang/d2';
import { TalaLayoutEngine, encodeTala, decodeTala } from '../src/viewer/tala-layout.ts';
import type { LayoutRequest } from '../src/viewer/layout-engine.ts';
import {
  projectLayoutRoute,
  trimRouteStart,
  visibleRouteSegments,
} from '../src/viewer/layout-projection.ts';
import type { MapGroup, MapProjection } from '../src/viewer/system-map.ts';

const request: LayoutRequest = {
  nodes: [
    { id: 'domain', parentId: null, width: 240, height: 120 },
    { id: 'a', parentId: 'domain', width: 240, height: 120 },
    { id: 'b', parentId: 'domain', width: 240, height: 120 },
    { id: 'external', parentId: null, width: 240, height: 120 },
  ],
  edges: [
    { id: 'type', sourceId: 'a', targetId: 'external', directed: true },
    { id: 'same-type', sourceId: 'a', targetId: 'external', directed: true },
    { id: 'connection', sourceId: 'a', targetId: 'b', directed: false },
    { id: 'self', sourceId: 'b', targetId: 'b', directed: true },
  ],
};

test('real TALA preserves nested identities and all routes, with deterministic geometry', async () => {
  const engine = new TalaLayoutEngine();
  const original = structuredClone(request);
  const first = await engine.layout(request),
    second = await engine.layout(request);
  assert.equal(first.engine, 'TALA (@d2lang/d2@0.1.34)');
  assert.deepEqual(first, second);
  assert.deepEqual(request, original);
  assert.deepEqual([...first.boxes.keys()].sort(), request.nodes.map((n) => n.id).sort());
  assert.deepEqual([...first.routes.keys()].sort(), request.edges.map((e) => e.id).sort());
  assert.deepEqual(first.routes.get('type'), first.routes.get('same-type'));
  assert.ok(first.routes.get('self')!.length >= 4);
  const root = first.boxes.get('domain')!,
    child = first.boxes.get('a')!;
  assert.ok(child.x > root.x && child.y > root.y);
  assert.ok(child.x + child.width < root.x + root.width);
  const group: MapGroup = {
    id: 'summary',
    sourceId: 'domain',
    targetId: 'external',
    kind: 'typing',
    directed: true,
    records: [
      {
        id: 'type',
        sourceId: 'a',
        targetId: 'external',
        kind: 'typing',
        directed: true,
        sourceLabel: 'A',
        targetLabel: 'External',
      },
    ],
  };
  const projection: MapProjection = {
    visible: ['domain', 'external'],
    expanded: new Set(),
    representatives: new Map([
      ['domain', 'domain'],
      ['a', 'domain'],
      ['external', 'external'],
    ]),
    limited: false,
  };
  const route = projectLayoutRoute(first, group, projection, { x: 0, y: 0, scale: 1 })!;
  assert.ok(
    route[0].x === root.x ||
      route[0].x === root.x + root.width ||
      route[0].y === root.y ||
      route[0].y === root.y + root.height,
  );
  assert.deepEqual(first, second, 'Projection must not modify cached geometry');
  const detailed = {
    ...projection,
    representatives: new Map([
      ['domain', 'domain'],
      ['a', 'a'],
      ['external', 'external'],
    ]),
  };
  assert.equal(
    projectLayoutRoute(first, { ...group, sourceId: 'a' }, detailed, { x: 0, y: 0, scale: 1 }),
    null,
    'A route to a region must not pretend to connect directly to its expanded member',
  );
});

test('encoding uses safe synthetic aliases and rejects malformed ownership and dimensions', () => {
  const hostile = 'a\n malicious -> node: @import "remote"';
  const encoded = encodeTala({
    nodes: [{ id: hostile, parentId: null, width: 240, height: 120 }],
    edges: [],
  });
  assert.ok(!encoded.source.includes('malicious'));
  assert.ok(encoded.paths.has(hostile));
  assert.throws(
    () => encodeTala({ nodes: [{ id: 'a', parentId: 'a', width: 240, height: 120 }], edges: [] }),
    /ownership/,
  );
  assert.throws(
    () =>
      encodeTala({ nodes: [{ id: 'a', parentId: 'missing', width: 240, height: 120 }], edges: [] }),
    /ownership/,
  );
  assert.throws(
    () => encodeTala({ nodes: [{ id: 'a', parentId: null, width: NaN, height: 120 }], edges: [] }),
    /dimensions/,
  );
});

test('invalid output cannot replace a usable scene', () => {
  const encoded = encodeTala(request);
  assert.throws(
    () =>
      decodeTala(request, encoded, {
        diagram: { shapes: [], connections: [] },
      } as unknown as CompileResponse),
    /omitted model elements/,
  );
});

test('valid placement survives a diagonal route while that edge uses the detail router', () => {
  const input: LayoutRequest = {
    nodes: [
      { id: 'a', parentId: null, width: 100, height: 80 },
      { id: 'b', parentId: null, width: 100, height: 80 },
    ],
    edges: [{ id: 'relation', sourceId: 'a', targetId: 'b', directed: true }],
  };
  const encoded = encodeTala(input);
  const response = {
    diagram: {
      shapes: [
        { id: 'n0', pos: { x: 0, y: 0 }, width: 100, height: 80 },
        { id: 'n1', pos: { x: 200, y: 200 }, width: 100, height: 80 },
      ],
      connections: [
        {
          tooltip: 'e0',
          src: 'n0',
          dst: 'n1',
          route: [
            { x: 100, y: 80 },
            { x: 200, y: 200 },
          ],
        },
      ],
    },
  } as unknown as CompileResponse;
  const result = decodeTala(input, encoded, response);
  assert.equal(result.boxes.size, 2);
  assert.equal(result.routes.has('relation'), false);
  const missing = structuredClone(response);
  missing.diagram.connections = [];
  assert.throws(() => decodeTala(input, encoded, missing), /omitted model relationships/);
});

test('cancelling a running request disposes its worker and rejects promptly', async () => {
  let disposed = 0;
  const engine = new TalaLayoutEngine(async () => ({
    compile: () => new Promise(() => {}),
    dispose: async () => {
      disposed++;
    },
  }));
  const controller = new AbortController();
  const pending = engine.layout(request, { signal: controller.signal });
  await Promise.resolve();
  controller.abort();
  await assert.rejects(pending, { name: 'AbortError' });
  assert.equal(disposed, 1);
});

test('cancellation during module loading disposes the late compiler without compiling', async () => {
  let release!: (compiler: {
    compile: () => Promise<CompileResponse>;
    dispose: () => Promise<void>;
  }) => void;
  let compiled = 0,
    disposed = 0;
  const engine = new TalaLayoutEngine(
    () =>
      new Promise((resolve) => {
        release = resolve;
      }),
  );
  const controller = new AbortController();
  const pending = engine.layout(request, { signal: controller.signal });
  controller.abort();
  await assert.rejects(pending, { name: 'AbortError' });
  release({
    compile: async () => {
      compiled++;
      return {} as CompileResponse;
    },
    dispose: async () => {
      disposed++;
    },
  });
  await Promise.resolve();
  assert.equal(compiled, 0);
  assert.equal(disposed, 1);
});

test('layout timeout releases a stalled worker', async () => {
  let disposed = 0;
  const engine = new TalaLayoutEngine(
    async () => ({
      compile: () => new Promise(() => {}),
      dispose: async () => {
        disposed++;
      },
    }),
    10,
  );
  await assert.rejects(engine.layout(request), /exceeded/);
  assert.equal(disposed, 1);
});

test('zoom projection trims hidden paths and clips label candidates without inventing joins', () => {
  const route = [
    { x: 100, y: 100 },
    { x: 300, y: 100 },
    { x: 300, y: 400 },
    { x: 600, y: 400 },
  ];
  assert.deepEqual(trimRouteStart(route, { x: 0, y: 0, width: 400, height: 300 }), [
    { x: 300, y: 300 },
    { x: 300, y: 400 },
    { x: 600, y: 400 },
  ]);
  const runs = visibleRouteSegments(
    [
      { x: -20, y: 20 },
      { x: 200, y: 20 },
      { x: 200, y: 80 },
      { x: -20, y: 80 },
    ],
    { x: 0, y: 0, width: 100, height: 100 },
  );
  assert.deepEqual(runs, [
    [
      { x: 0, y: 20 },
      { x: 100, y: 20 },
    ],
    [
      { x: 100, y: 80 },
      { x: 0, y: 80 },
    ],
  ]);
});

test('a 6,048-element cross-domain model fits TALA coordinate limits and retains every record', async () => {
  const large: LayoutRequest = { nodes: [], edges: [] };
  for (let d = 0; d < 27; d++) {
    large.nodes.push({ id: `d${d}`, parentId: null, width: 240, height: 120 });
    for (let m = 0; m < 223; m++) {
      large.nodes.push({ id: `d${d}m${m}`, parentId: `d${d}`, width: 240, height: 120 });
      large.edges.push({
        id: `e${d}m${m}`,
        sourceId: `d${d}m${m}`,
        targetId: `d${(d + 1) % 27}m${m}`,
        directed: true,
      });
    }
  }
  const layout = await new TalaLayoutEngine().layout(large);
  assert.equal(layout.boxes.size, 6048);
  assert.equal(layout.routes.size, 6021);
  assert.deepEqual(layout.routeEndpoints!.get('e0m0'), { sourceId: 'd0', targetId: 'd1' });
  for (const node of large.nodes) {
    const box = layout.boxes.get(node.id)!;
    assert.ok(Object.values(box).every(Number.isFinite));
    const parent = node.parentId ? layout.boxes.get(node.parentId)! : layout.bounds;
    assert.ok(box.x >= parent.x && box.y >= parent.y);
    assert.ok(box.x + box.width <= parent.x + parent.width + 0.01);
    assert.ok(box.y + box.height <= parent.y + parent.height + 0.01);
  }
});

test('ancestor-to-member relationships remain available to the detail router', async () => {
  const layout = await new TalaLayoutEngine().layout({
    ...request,
    edges: [
      ...request.edges,
      { id: 'ancestor', sourceId: 'domain', targetId: 'a', directed: true },
    ],
  });
  assert.ok(layout.boxes.has('domain') && layout.boxes.has('a'));
  assert.equal(layout.routes.has('ancestor'), false);
  assert.ok(layout.routes.has('type'));
});
