import type { SystemMap } from './system-map.ts';

export interface SiblingLink {
  source: string;
  target: string;
  weight: number;
}

/** Lift each relationship to the two children of its lowest common owner.
 * Presentation only: source ownership and relationship records stay untouched. */
export function siblingLinks(graph: SystemMap): Map<string | null, SiblingLink[]> {
  const paths = new Map<string, string[]>();
  const path = (id: string): string[] => {
    let result = paths.get(id);
    if (!result) {
      const parent = graph.nodes.get(id)!.parentId;
      result = [...(parent ? path(parent) : []), id];
      paths.set(id, result);
    }
    return result;
  };
  const owners = new Map<string | null, Map<string, SiblingLink>>();
  for (const relation of graph.relations) {
    const a = path(relation.sourceId),
      b = path(relation.targetId);
    let i = 0;
    while (i < a.length && a[i] === b[i]) i++;
    if (i === a.length || i === b.length) continue;
    const owner = i ? a[i - 1] : null;
    let links = owners.get(owner);
    if (!links) owners.set(owner, (links = new Map()));
    const [source, target] = [a[i], b[i]].sort();
    const key = JSON.stringify([source, target]);
    const previous = links.get(key);
    if (previous) previous.weight++;
    else links.set(key, { source, target, weight: 1 });
  }
  return new Map([...owners].map(([owner, links]) => [owner, [...links.values()]]));
}

export interface Point {
  x: number;
  y: number;
}
export function segmentsCross(a: Point, b: Point, c: Point, d: Point): boolean {
  const side = (p: Point, q: Point, r: Point) =>
    (q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x);
  return side(a, b, c) * side(a, b, d) < -0.001 && side(c, d, a) * side(c, d, b) < -0.001;
}

/** Bounded, deterministic local improvement of a fixed grid. Small sibling
 * graphs score crossings and length; large graphs score the strongest 256 links
 * by length only. This is a heuristic, not a globally optimal graph layout. */
export function orderMapSiblings(ids: string[], columns: number, links: SiblingLink[]): string[] {
  if (ids.length < 3 || !links.length) return [...ids];
  const indices = new Map(ids.map((id, i) => [id, i]));
  const edges = [...links]
    .sort(
      (a, b) =>
        b.weight - a.weight || a.source.localeCompare(b.source) || a.target.localeCompare(b.target),
    )
    .slice(0, 256)
    .filter((e) => indices.has(e.source) && indices.has(e.target))
    .map((e) => ({
      a: indices.get(e.source)!,
      b: indices.get(e.target)!,
      weight: Math.log2(e.weight + 1),
    }));
  const positions = ids.map((_, i) => ({ x: i % columns, y: Math.floor(i / columns) }));
  const slots = ids.map((_, i) => i);
  const score = () => {
    let cost = 0;
    for (let i = 0; i < edges.length; i++) {
      const e = edges[i],
        a = positions[slots[e.a]],
        b = positions[slots[e.b]];
      cost += Math.hypot(a.x - b.x, a.y - b.y) * e.weight * 0.15;
      if (ids.length <= 32 && edges.length <= 80) {
        for (let j = i + 1; j < edges.length; j++) {
          const f = edges[j];
          if (e.a === f.a || e.a === f.b || e.b === f.a || e.b === f.b) continue;
          if (segmentsCross(a, b, positions[slots[f.a]], positions[slots[f.b]])) cost += 1;
        }
      }
    }
    return cost;
  };
  let best = score(),
    attempts = 0;
  for (let pass = 0; pass < 3; pass++) {
    let improved = false;
    for (let a = 0; a < ids.length && attempts < 4096; a++) {
      const end = ids.length <= 32 ? ids.length : Math.min(ids.length, a + 9);
      for (let b = a + 1; b < end && attempts < 4096; b++) {
        attempts++;
        [slots[a], slots[b]] = [slots[b], slots[a]];
        const next = score();
        if (next < best - 0.0001) {
          best = next;
          improved = true;
        } else [slots[a], slots[b]] = [slots[b], slots[a]];
      }
    }
    if (!improved) break;
  }
  const ordered = [...ids];
  ids.forEach((id, i) => {
    ordered[slots[i]] = id;
  });
  return ordered;
}
