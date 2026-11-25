import type { Rect } from './system-map.ts';
import { segmentsCross } from './map-order.ts';
import type { Point } from './map-order.ts';

const clamp = (n: number, min: number, max: number) => Math.max(min, Math.min(max, n));
const length = (a: Point, b: Point) => Math.abs(a.x - b.x) + Math.abs(a.y - b.y);
const overlaps = (a: Rect, b: Rect) =>
  a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y;

/** Interior intersections only; a route may touch an attachment's boundary. */
export function segmentHitsRect(a: Point, b: Point, box: Rect): boolean {
  if (Math.abs(a.x - b.x) < 0.001)
    return (
      a.x > box.x + 0.01 &&
      a.x < box.x + box.width - 0.01 &&
      Math.max(a.y, b.y) > box.y + 0.01 &&
      Math.min(a.y, b.y) < box.y + box.height - 0.01
    );
  if (Math.abs(a.y - b.y) < 0.001)
    return (
      a.y > box.y + 0.01 &&
      a.y < box.y + box.height - 0.01 &&
      Math.max(a.x, b.x) > box.x + 0.01 &&
      Math.min(a.x, b.x) < box.x + box.width - 0.01
    );
  throw new Error('Map routes must be orthogonal');
}

function simplify(points: Point[]): Point[] {
  const result: Point[] = [];
  for (const point of points) {
    if (result.length && length(result[result.length - 1], point) < 0.001) continue;
    const a = result[result.length - 2],
      b = result[result.length - 1];
    if (
      a &&
      ((a.x === b.x && b.x === point.x) || (a.y === b.y && b.y === point.y)) &&
      length(a, b) + length(b, point) <= length(a, point) + 0.001
    )
      result.pop();
    result.push(point);
  }
  return result;
}

function ports(box: Rect, fraction: number) {
  const x = box.x + box.width * fraction,
    y = box.y + box.height * fraction;
  return [
    [
      { x: box.x + box.width, y },
      { x: box.x + box.width + 8, y },
    ],
    [
      { x: box.x, y },
      { x: box.x - 8, y },
    ],
    [
      { x, y: box.y },
      { x, y: box.y - 8 },
    ],
    [
      { x, y: box.y + box.height },
      { x, y: box.y + box.height + 8 },
    ],
  ];
}

/** Route through available gutters. Candidate cost prioritizes avoiding cards,
 * then avoiding existing lines, then length and bends. Work is bounded even for
 * dense views; this is deliberately independent of graph semantics. */
export function routeMapEdge(
  source: Rect,
  target: Rect,
  obstacles: Rect[],
  viewport: Rect,
  previous: Point[][] = [],
  sourceFraction = 0.5,
  targetFraction = 0.5,
): Point[] {
  if (
    source.x === target.x &&
    source.y === target.y &&
    source.width === target.width &&
    source.height === target.height &&
    sourceFraction === targetFraction
  )
    targetFraction = sourceFraction < 0.6 ? 0.8 : 0.2;
  const allObstacles = [...obstacles, source, target];
  const mid = {
    x: (source.x + source.width / 2 + target.x + target.width / 2) / 2,
    y: (source.y + source.height / 2 + target.y + target.height / 2) / 2,
  };
  const lanes = (axis: 'x' | 'y') => {
    const size = axis === 'x' ? 'width' : 'height';
    const min = viewport[axis] + 8,
      max = viewport[axis] + viewport[size] - 8;
    const candidates = [
      ...new Set(allObstacles.flatMap((box) => [box[axis] - 8, box[axis] + box[size] + 8])),
    ]
      .filter((n) => n >= min && n <= max)
      .sort((a, b) => Math.abs(a - mid[axis]) - Math.abs(b - mid[axis]) || a - b)
      .slice(0, 16);
    return [...new Set([mid[axis], min, max, ...candidates])];
  };
  const xs = lanes('x'),
    ys = lanes('y');
  const pairs = ports(source, sourceFraction)
    .flatMap((a) => ports(target, targetFraction).map((b) => ({ a, b })))
    .sort((a, b) => length(a.a[1], a.b[1]) - length(b.a[1], b.b[1]))
    .slice(0, 8);
  let best: Point[] = [],
    bestCost = Infinity;
  const consider = (raw: Point[]) => {
    if (
      raw.some(
        (p) =>
          p.x < viewport.x ||
          p.x > viewport.x + viewport.width ||
          p.y < viewport.y ||
          p.y > viewport.y + viewport.height,
      )
    )
      return;
    const points = simplify(raw);
    let cost = (points.length - 2) * 12;
    for (let i = 1; i < points.length; i++) {
      const a = points[i - 1],
        b = points[i];
      cost += length(a, b);
      for (const box of allObstacles) {
        if (segmentHitsRect(a, b, box)) cost += 100000;
        if (cost >= bestCost) return;
      }
      for (const route of previous)
        for (let j = 1; j < route.length; j++) {
          const c = route[j - 1],
            d = route[j];
          if (segmentsCross(a, b, c, d)) cost += 70;
          // Shared line runs obscure multiplicity, even when they do not cross.
          if (
            a.x === b.x &&
            c.x === d.x &&
            Math.abs(a.x - c.x) < 3 &&
            Math.min(Math.max(a.y, b.y), Math.max(c.y, d.y)) -
              Math.max(Math.min(a.y, b.y), Math.min(c.y, d.y)) >
              6
          )
            cost += 100;
          if (
            a.y === b.y &&
            c.y === d.y &&
            Math.abs(a.y - c.y) < 3 &&
            Math.min(Math.max(a.x, b.x), Math.max(c.x, d.x)) -
              Math.max(Math.min(a.x, b.x), Math.min(c.x, d.x)) >
              6
          )
            cost += 100;
        }
      if (cost >= bestCost) return;
    }
    best = points;
    bestCost = cost;
  };
  for (const {
    a: [start, a],
    b: [end, b],
  } of pairs) {
    consider([start, a, { x: b.x, y: a.y }, b, end]);
    consider([start, a, { x: a.x, y: b.y }, b, end]);
    for (const x of xs) consider([start, a, { x, y: a.y }, { x, y: b.y }, b, end]);
    for (const y of ys) consider([start, a, { x: a.x, y }, { x: b.x, y }, b, end]);
  }
  // Tiny/clipped regions may leave no room for an outward stub.
  return best.length
    ? best
    : [
        { x: source.x, y: source.y },
        { x: target.x, y: source.y },
        { x: target.x, y: target.y },
      ];
}

export const routePath = (points: Point[]) =>
  points.map((p, i) => `${i ? 'L' : 'M'}${p.x},${p.y}`).join(' ');

/** Persistent labels never cover a card, another label, or the viewport edge.
 * If none fits, the relationship remains available on hover/focus and in the list. */
export function placeMapLabel(
  points: Point[],
  width: number,
  height: number,
  obstacles: Rect[],
  viewport: Rect,
): Rect | null {
  const segments = points
    .slice(1)
    .map((b, i) => ({ a: points[i], b }))
    .sort((a, b) => length(b.a, b.b) - length(a.a, a.b));
  for (const { a, b } of segments)
    for (const t of [0.5, 0.25, 0.75]) {
      const box = {
        x: a.x + (b.x - a.x) * t - width / 2,
        y: a.y + (b.y - a.y) * t - height / 2,
        width,
        height,
      };
      if (
        box.x < viewport.x + 4 ||
        box.y < viewport.y + 4 ||
        box.x + width > viewport.x + viewport.width - 4 ||
        box.y + height > viewport.y + viewport.height - 4
      )
        continue;
      if (
        !obstacles.some((o) =>
          overlaps({ x: o.x - 3, y: o.y - 3, width: o.width + 6, height: o.height + 6 }, box),
        )
      )
        return box;
    }
  return null;
}

export function fallbackMapLabel(points: Point[], width: number, viewport: Rect): Rect {
  const point = points[Math.floor(points.length / 2)];
  return {
    x: clamp(point.x - width / 2, 4, viewport.width - width - 4),
    y: clamp(point.y - 12, 4, viewport.height - 28),
    width,
    height: 24,
  };
}
