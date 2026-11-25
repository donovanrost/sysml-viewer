import { test } from 'node:test';
import assert from 'node:assert/strict';
import { orderMapSiblings, segmentsCross } from '../src/viewer/map-order.ts';
import { routeMapEdge, segmentHitsRect, placeMapLabel } from '../src/viewer/map-routing.ts';

test('relationship-aware ordering removes avoidable crossings without changing membership', () => {
  const ids = ['A', 'B', 'C', 'D'];
  const links = [
    { source: 'A', target: 'D', weight: 1 },
    { source: 'B', target: 'C', weight: 1 },
  ];
  const crossing = (order: string[]) => {
    const p = (id: string) => ({ x: order.indexOf(id) % 2, y: Math.floor(order.indexOf(id) / 2) });
    return segmentsCross(p('A'), p('D'), p('B'), p('C'));
  };
  assert.ok(crossing(ids));
  const ordered = orderMapSiblings(ids, 2, links);
  assert.ok(!crossing(ordered));
  assert.deepEqual([...ordered].sort(), ids);
  assert.deepEqual(orderMapSiblings(ids, 2, links), ordered);
  assert.deepEqual(orderMapSiblings(ids, 2, [...links].reverse()), ordered);
  assert.deepEqual(orderMapSiblings(ids, 2, []), ids);
});

test('routes avoid cards between endpoints and retain exact boundary attachments', () => {
  const source = { x: 30, y: 100, width: 100, height: 100 };
  const target = { x: 530, y: 100, width: 100, height: 100 };
  const obstacles = [
    source,
    target,
    { x: 230, y: 50, width: 110, height: 200 },
    { x: 370, y: 70, width: 90, height: 170 },
  ];
  const viewport = { x: 0, y: 0, width: 700, height: 400 };
  const route = routeMapEdge(source, target, obstacles, viewport);
  assert.ok(route.length >= 4);
  for (let i = 1; i < route.length; i++)
    for (const box of obstacles) assert.equal(segmentHitsRect(route[i - 1], route[i], box), false);
  const onBoundary = (point: (typeof route)[number], box: typeof source) =>
    ((point.x === box.x || point.x === box.x + box.width) &&
      point.y >= box.y &&
      point.y <= box.y + box.height) ||
    ((point.y === box.y || point.y === box.y + box.height) &&
      point.x >= box.x &&
      point.x <= box.x + box.width);
  assert.ok(onBoundary(route[0], source));
  assert.ok(onBoundary(route.at(-1)!, target));
  assert.deepEqual(routeMapEdge(source, target, obstacles, viewport), route);
});

test('self relationships have a visible external route', () => {
  const box = { x: 100, y: 100, width: 120, height: 90 };
  const route = routeMapEdge(box, box, [box], { x: 0, y: 0, width: 400, height: 300 });
  assert.ok(route.length >= 4);
  for (let i = 1; i < route.length; i++) assert.ok(!segmentHitsRect(route[i - 1], route[i], box));
});

test('persistent labels avoid cards and each other, and decline to cover a crowded view', () => {
  const viewport = { x: 0, y: 0, width: 700, height: 400 };
  const route = [
    { x: 40, y: 80 },
    { x: 660, y: 80 },
  ];
  const obstacle = { x: 290, y: 60, width: 120, height: 40 };
  const first = placeMapLabel(route, 100, 24, [obstacle], viewport)!;
  const second = placeMapLabel(route, 100, 24, [obstacle, first], viewport)!;
  assert.ok(first && second);
  assert.ok(first.x + first.width <= second.x || second.x + second.width <= first.x);
  assert.equal(placeMapLabel(route, 100, 24, [viewport], viewport), null);
});
