import type { Camera, MapGroup, MapLayout, MapProjection, Rect } from './system-map.ts';
import type { Point } from './map-order.ts';

const inside = (p: Point, b: Rect) =>
  p.x >= b.x - 0.01 &&
  p.x <= b.x + b.width + 0.01 &&
  p.y >= b.y - 0.01 &&
  p.y <= b.y + b.height + 0.01;

/** Remove the portion concealed by a collapsed ownership region. */
export function trimRouteStart(points: Point[], box: Rect): Point[] {
  if (!points.length || !inside(points[0], box)) return points;
  for (let i = 1; i < points.length; i++) {
    const a = points[i - 1],
      b = points[i];
    if (inside(b, box)) continue;
    const exit =
      Math.abs(a.x - b.x) < 0.01
        ? { x: a.x, y: b.y < box.y ? box.y : box.y + box.height }
        : { x: b.x < box.x ? box.x : box.x + box.width, y: a.y };
    return [exit, ...points.slice(i)];
  }
  return points;
}

/** Visible segments are only used to place labels. SVG clips the complete path,
 * so paths that exit and re-enter the viewport do not gain invented joins. */
export function visibleRouteSegments(points: Point[], viewport: Rect): Point[][] {
  const result: Point[][] = [];
  const clamp = (n: number, start: number, size: number) =>
    Math.max(start, Math.min(start + size, n));
  for (let i = 1; i < points.length; i++) {
    const a = points[i - 1],
      b = points[i];
    if (
      Math.max(a.x, b.x) < viewport.x ||
      Math.min(a.x, b.x) > viewport.x + viewport.width ||
      Math.max(a.y, b.y) < viewport.y ||
      Math.min(a.y, b.y) > viewport.y + viewport.height
    )
      continue;
    const start = {
      x: clamp(a.x, viewport.x, viewport.width),
      y: clamp(a.y, viewport.y, viewport.height),
    };
    const end = {
      x: clamp(b.x, viewport.x, viewport.width),
      y: clamp(b.y, viewport.y, viewport.height),
    };
    if (Math.hypot(end.x - start.x, end.y - start.y) > 0.01) result.push([start, end]);
  }
  return result.sort(
    (a, b) =>
      Math.hypot(b[0].x - b[1].x, b[0].y - b[1].y) - Math.hypot(a[0].x - a[1].x, a[0].y - a[1].y),
  );
}

export function projectLayoutRoute(
  layout: MapLayout,
  group: MapGroup,
  projection: MapProjection,
  camera: Camera,
): Point[] | null {
  const record = group.records.find((r) => {
    if (!layout.routes?.has(r.id)) return false;
    const endpoints = layout.routeEndpoints?.get(r.id) ?? r;
    if (group.sourceId === group.targetId && endpoints.sourceId !== group.sourceId) return false;
    const a = projection.representatives.get(endpoints.sourceId);
    const b = projection.representatives.get(endpoints.targetId);
    return (
      (a === group.sourceId && b === group.targetId) ||
      (!group.directed && a === group.targetId && b === group.sourceId)
    );
  });
  if (!record) return null;
  let points = layout.routes!.get(record.id)!;
  const endpoints = layout.routeEndpoints?.get(record.id) ?? record;
  if (projection.representatives.get(endpoints.sourceId) !== group.sourceId)
    points = [...points].reverse();
  if (group.sourceId !== group.targetId) {
    points = trimRouteStart(points, layout.boxes.get(group.sourceId)!);
    points = trimRouteStart([...points].reverse(), layout.boxes.get(group.targetId)!).reverse();
  }
  return points.map((p) => ({
    x: p.x * camera.scale + camera.x,
    y: p.y * camera.scale + camera.y,
  }));
}
