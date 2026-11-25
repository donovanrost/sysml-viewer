import type { ModelElement, ViewerModel, Reference } from './model.ts';
import { orderMapSiblings, siblingLinks } from './map-order.ts';

export interface MapNode {
  element: ModelElement;
  parentId: string | null;
  children: string[];
  /** A display occurrence retains its source element; it is not a new declaration. */
  contextName?: string;
  inheritedFrom?: ModelElement;
  note?: string;
  truncated?: boolean;
}
export interface MapRelation {
  id: string;
  sourceId: string;
  targetId: string;
  kind: string;
  directed: boolean;
  sourceLabel: string;
  targetLabel: string;
  reference?: Reference;
  connectionId?: string;
}
export interface MapIssue {
  ownerId: string;
  kind: string;
  target: string;
  status: string;
}
export interface SystemMap {
  nodes: Map<string, MapNode>;
  roots: string[];
  relations: MapRelation[];
  issues: MapIssue[];
  libraryReferences: number;
  view?: { kind: 'structure' | 'hierarchy'; elementId: string; title: string };
}
export type MapLens = 'types' | 'connections';
const typeKinds = new Set(['typing', 'specialization', 'subsetting', 'redefinition']);

/** Presentation policy, not a primitive-type inference. A complete attribute
 * definition with no visible owned/effective detail is useful as an inline type
 * label. Structural leaves (parts, ports, etc.) still belong on the system map. */
export function isInlineValueDefinition(model: ViewerModel, element: ModelElement): boolean {
  return (
    element.semantic.metaclass === 'AttributeDefinition' &&
    element.semantic.membersAvailable &&
    element.semantic.inheritanceComplete &&
    model.children(element.id).length === 0 &&
    model.members(element.id).length === 0
  );
}

/** A source-owned presentation graph. It never resolves names or duplicates
 * inherited members into new ownership positions. Annotation bodies are details. */
export function buildSystemMap(model: ViewerModel): SystemMap {
  const nodes = new Map<string, MapNode>();
  const visible = new Set(model.elements.map((e) => e.id));
  const inlineTypes = new Set(
    model.elements.filter((e) => isInlineValueDefinition(model, e)).map((e) => e.id),
  );
  const excluded = new Map<string, boolean>();
  const isAnnotation = (id: string): boolean => {
    if (excluded.has(id)) return excluded.get(id)!;
    const trail: string[] = [],
      seen = new Set<string>();
    let current = model.byId.get(id),
      hide = false;
    while (current) {
      if (seen.has(current.id)) {
        hide = true;
        break;
      }
      seen.add(current.id);
      if (excluded.has(current.id)) {
        hide = excluded.get(current.id)!;
        break;
      }
      trail.push(current.id);
      if (current.semantic.metaclass === 'MetadataUsage') {
        hide = true;
        break;
      }
      current = model.byId.get(current.parentId ?? '');
    }
    for (const key of trail) excluded.set(key, hide);
    return hide;
  };
  for (const e of model.elements) {
    if (!isAnnotation(e.id) && !inlineTypes.has(e.id))
      nodes.set(e.id, { element: e, parentId: null, children: [] });
  }
  const roots: string[] = [];
  for (const [id, node] of nodes) {
    let parent = model.byId.get(node.element.parentId ?? '');
    const seen = new Set([id]);
    while (parent && !nodes.has(parent.id) && !seen.has(parent.id)) {
      seen.add(parent.id);
      parent = model.byId.get(parent.parentId ?? '');
    }
    if (parent && nodes.has(parent.id) && !seen.has(parent.id)) {
      node.parentId = parent.id;
      nodes.get(parent.id)!.children.push(id);
    } else roots.push(id);
  }
  const compare = (a: string, b: string) => {
    const x = nodes.get(a)!.element,
      y = nodes.get(b)!.element;
    return x.qualifiedName.localeCompare(y.qualifiedName) || x.line - y.line;
  };
  roots.sort(compare);
  for (const node of nodes.values()) node.children.sort(compare);
  const relations: MapRelation[] = [],
    issues: MapIssue[] = [];
  let libraryReferences = 0;
  for (const [i, r] of model.snapshot.references.entries()) {
    if (!r.ownerId || !nodes.has(r.ownerId) || !typeKinds.has(r.kind)) continue;
    if (r.resolution.status !== 'resolved') {
      issues.push({
        ownerId: r.ownerId,
        kind: r.kind,
        target: r.spelling,
        status: r.resolution.status,
      });
      continue;
    }
    const target = model.byId.get(r.resolution.targetId);
    if (target?.library) {
      libraryReferences++;
      continue;
    }
    // Deliberately represented on the source card, not an unmapped relationship.
    // The original reference remains available to the inspector/type hierarchy.
    if (target && inlineTypes.has(target.id)) continue;
    if (!target || !nodes.has(target.id)) {
      issues.push({
        ownerId: r.ownerId,
        kind: r.kind,
        target: r.spelling,
        status: 'target omitted from map',
      });
      continue;
    }
    relations.push({
      id: `reference:${i}`,
      sourceId: r.ownerId,
      targetId: target.id,
      kind: r.kind,
      directed: true,
      sourceLabel: nodes.get(r.ownerId)!.element.qualifiedName,
      targetLabel: target.qualifiedName,
      reference: r,
    });
  }
  for (const [i, c] of model.snapshot.connections.entries()) {
    const owner = model.byId.get(c.elementId);
    if (!owner || owner.library || !owner.semantic.source) continue;
    const ownerId = owner.parentId ?? '';
    if (!nodes.has(ownerId)) continue;
    if (c.ends.length !== 2) {
      issues.push({
        ownerId,
        kind: 'connection',
        target: owner.name,
        status: `${c.ends.length}-ended connection; map requires two ends`,
      });
      continue;
    }
    const endpoint = (end: (typeof c.ends)[number]) => {
      if (end.resolution.status !== 'resolved') return undefined;
      // Preserve the first contextual usage in a feature chain. A later inherited
      // port can belong to its type definition, which is not a new endpoint owner.
      const chain = end.chain.filter((id) => nodes.has(id));
      let anchor = chain[0];
      for (const candidate of chain.slice(1)) {
        let parent = nodes.get(candidate)?.parentId;
        while (parent && parent !== anchor) parent = nodes.get(parent)?.parentId;
        if (parent === anchor) anchor = candidate;
      }
      return anchor ?? (nodes.has(end.resolution.targetId) ? end.resolution.targetId : undefined);
    };
    const a = endpoint(c.ends[0]),
      b = endpoint(c.ends[1]);
    if (!a || !b) {
      issues.push({
        ownerId,
        kind: 'connection',
        target: owner.name,
        status: c.ends
          .map((e) => (e.resolution.status === 'resolved' ? 'outside map' : e.resolution.status))
          .join(' / '),
      });
      continue;
    }
    relations.push({
      id: `connection:${i}`,
      sourceId: a,
      targetId: b,
      kind: 'connection',
      directed: false,
      sourceLabel: c.ends[0].spelling,
      targetLabel: c.ends[1].spelling,
      connectionId: c.elementId,
    });
  }
  // Keep the projection tied to the public viewer's source visibility contract.
  for (const id of nodes.keys()) if (!visible.has(id)) throw new Error('Invalid map visibility');
  return { nodes, roots, relations, issues, libraryReferences };
}

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}
export interface Camera {
  x: number;
  y: number;
  scale: number;
}
export interface MapLayout {
  boxes: Map<string, Rect>;
  bounds: Rect;
  /** Routes keyed by original relationship ID, in the same world coordinates. */
  routes?: Map<string, import('./map-order.ts').Point[]>;
  routeEndpoints?: Map<string, { sourceId: string; targetId: string }>;
}

/** Fixed nested regions are computed once per snapshot, never during zoom. */
export function layoutSystemMap(graph: SystemMap): MapLayout {
  const boxes = new Map<string, Rect>();
  const links = siblingLinks(graph);
  const columns = Math.max(
    1,
    Math.min(graph.roots.length, Math.round(Math.sqrt(graph.roots.length * 1.4))),
  );
  const place = (id: string, box: Rect, depth: number) => {
    if (boxes.has(id) || depth > 80) return;
    boxes.set(id, box);
    const children = graph.nodes.get(id)!.children;
    if (!children.length) return;
    const inset =
        Math.min(box.width, box.height) *
        Math.min(0.045, 0.35 / Math.ceil(Math.sqrt(children.length))),
      header = box.height * 0.12;
    let cols = 1,
      best = Infinity;
    // Evaluate complete rows: rounding a square root alone can leave three
    // crowded columns plus a single orphan card in the next row.
    for (
      let candidate = 1;
      candidate <= Math.min(children.length, Math.ceil(Math.sqrt(children.length) * 2));
      candidate++
    ) {
      const rows = Math.ceil(children.length / candidate);
      const w = (box.width - inset * (candidate + 1)) / candidate;
      const h = (box.height - header - inset * (rows + 1)) / rows;
      if (w <= 0 || h <= 0) continue;
      const cost =
        Math.abs(Math.log(w / h / 1.5)) +
        ((candidate * rows - children.length) / children.length) * 0.7;
      if (cost < best) {
        best = cost;
        cols = candidate;
      }
    }
    const rows = Math.ceil(children.length / cols);
    const width = (box.width - inset * (cols + 1)) / cols;
    const height = (box.height - header - inset * (rows + 1)) / rows;
    orderMapSiblings(children, cols, links.get(id) ?? []).forEach((child, i) =>
      place(
        child,
        {
          x: box.x + inset + (i % cols) * (width + inset),
          y: box.y + header + inset + Math.floor(i / cols) * (height + inset),
          width,
          height,
        },
        depth + 1,
      ),
    );
  };
  orderMapSiblings(graph.roots, columns, links.get(null) ?? []).forEach((id, i) =>
    place(
      id,
      { x: (i % columns) * 1080, y: Math.floor(i / columns) * 780, width: 960, height: 660 },
      0,
    ),
  );
  return {
    boxes,
    bounds: {
      x: 0,
      y: 0,
      width: columns * 1080 - 120,
      height: Math.max(1, Math.ceil(graph.roots.length / columns)) * 780 - 120,
    },
  };
}
export const screenRect = (box: Rect, camera: Camera): Rect => ({
  x: box.x * camera.scale + camera.x,
  y: box.y * camera.scale + camera.y,
  width: box.width * camera.scale,
  height: box.height * camera.scale,
});
export const intersects = (a: Rect, b: Rect) =>
  a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y;
export function fitCamera(box: Rect, width: number, height: number): Camera {
  const scale = Math.max(
    0.00001,
    Math.min((Math.max(120, width) - 60) / box.width, (Math.max(120, height) - 60) / box.height),
  );
  return {
    scale,
    x: (width - box.width * scale) / 2 - box.x * scale,
    y: (height - box.height * scale) / 2 - box.y * scale,
  };
}
/** Focus must also keep every ownership region on the route to the destination
 * expanded. Fitting a tall leaf alone can otherwise collapse its own parent. */
export function focusCamera(
  graph: SystemMap,
  layout: MapLayout,
  id: string,
  width: number,
  height: number,
): Camera {
  const box = layout.boxes.get(id)!;
  let scale = fitCamera(box, width, height).scale;
  let current: string | null = id;
  while (current) {
    const node: MapNode = graph.nodes.get(current)!;
    const parentBox = layout.boxes.get(current)!;
    const child = layout.boxes.get(node.children[0]);
    if (child)
      scale = Math.max(
        scale,
        390 / parentBox.width,
        255 / parentBox.height,
        135 / child.width,
        65 / child.height,
      );
    current = node.parentId;
  }
  return {
    scale,
    x: width / 2 - (box.x + box.width / 2) * scale,
    y: height / 2 - (box.y + box.height / 2) * scale,
  };
}
export function zoomCamera(
  camera: Camera,
  factor: number,
  point: { x: number; y: number },
): Camera {
  const scale = Math.max(0.00001, Math.min(100000, camera.scale * factor));
  const ratio = scale / camera.scale;
  return {
    scale,
    x: point.x - (point.x - camera.x) * ratio,
    y: point.y - (point.y - camera.y) * ratio,
  };
}
export interface MapProjection {
  visible: string[];
  representatives: Map<string, string>;
  expanded: Set<string>;
  limited: boolean;
}
export function projectSystemMap(
  graph: SystemMap,
  layout: MapLayout,
  camera: Camera,
  viewport: Rect,
  previous = new Set<string>(),
  limit = 300,
): MapProjection {
  const visible: string[] = [],
    representatives = new Map<string, string>(),
    expanded = new Set<string>();
  let visited = 0,
    limited = false;
  const assign = (id: string, representative: string) => {
    representatives.set(id, representative);
    for (const child of graph.nodes.get(id)!.children) assign(child, representative);
  };
  const visit = (id: string) => {
    const box = layout.boxes.get(id);
    if (!box) {
      assign(id, id);
      limited = true;
      return;
    }
    const rect = screenRect(box, camera),
      inView = intersects(rect, viewport);
    const kids = graph.nodes.get(id)!.children;
    const threshold = previous.has(id) ? 310 : 370;
    const childBox = layout.boxes.get(kids[0]);
    const readableChildren =
      childBox &&
      childBox.width * camera.scale >= (previous.has(id) ? 100 : 120) &&
      childBox.height * camera.scale >= (previous.has(id) ? 45 : 55);
    const visibleChildren = kids.filter((child) => {
      const childBox = layout.boxes.get(child);
      return childBox && intersects(screenRect(childBox, camera), viewport);
    }).length;
    const expand =
      inView &&
      kids.length > 0 &&
      readableChildren &&
      rect.width >= threshold &&
      rect.height >= threshold * 0.65 &&
      visited + visibleChildren + 1 <= limit;
    if (inView && visible.length < limit) visible.push(id);
    else if (inView) limited = true;
    if (inView) visited++;
    if (expand) {
      expanded.add(id);
      representatives.set(id, id);
      for (const child of kids) visit(child);
    } else {
      if (inView && kids.length && rect.width >= threshold && visited + kids.length > limit)
        limited = true;
      assign(id, id);
    }
  };
  for (const id of graph.roots) visit(id);
  return { visible, representatives, expanded, limited };
}
export interface MapGroup {
  id: string;
  sourceId: string;
  targetId: string;
  kind: string;
  directed: boolean;
  records: MapRelation[];
}
export function groupMapRelations(
  graph: SystemMap,
  projection: MapProjection,
  lens: MapLens,
): MapGroup[] {
  const groups = new Map<string, MapGroup>();
  for (const relation of graph.relations) {
    if ((relation.kind === 'connection') !== (lens === 'connections')) continue;
    let a = projection.representatives.get(relation.sourceId),
      b = projection.representatives.get(relation.targetId);
    if (!a || !b || (a === b && relation.sourceId !== relation.targetId)) continue;
    if (!relation.directed && a > b) [a, b] = [b, a];
    const id = JSON.stringify([a, b, relation.kind, relation.directed]);
    let group = groups.get(id);
    if (!group) {
      group = {
        id,
        sourceId: a,
        targetId: b,
        kind: relation.kind,
        directed: relation.directed,
        records: [],
      };
      groups.set(id, group);
    }
    group.records.push(relation);
  }
  return [...groups.values()];
}
