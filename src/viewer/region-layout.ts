import type { LayoutRequest, LayoutResult } from './layout-engine.ts';
import type { Rect } from './system-map.ts';

/** Arrange each ownership region independently, bottom-up. Cross-region links
 * influence the common owner's placement without flattening thousands of members
 * into one optimization problem. Routes retain their actual layout endpoints. */
export async function arrangeOwnershipRegions(
  request: LayoutRequest,
  arrange: (region: LayoutRequest) => Promise<LayoutResult>,
  signal?: AbortSignal,
): Promise<LayoutResult> {
  const nodes = new Map(request.nodes.map((node) => [node.id, node]));
  const children = new Map<string | null, string[]>();
  for (const node of request.nodes) {
    if (!children.has(node.parentId)) children.set(node.parentId, []);
    children.get(node.parentId)!.push(node.id);
  }
  const paths = new Map<string, string[]>();
  const path = (id: string): string[] => {
    if (paths.has(id)) return paths.get(id)!;
    const node = nodes.get(id)!;
    const result = [...(node.parentId ? path(node.parentId) : []), id];
    paths.set(id, result);
    return result;
  };
  const edges = new Map<string | null, LayoutRequest['edges']>();
  for (const edge of request.edges) {
    const a = path(edge.sourceId),
      b = path(edge.targetId);
    let i = 0;
    while (i < Math.min(a.length, b.length) && a[i] === b[i]) i++;
    if (edge.sourceId === edge.targetId) i--;
    // Ancestor/member relationships use the projection router. An ancestor is
    // not its own sibling, so it must not be invented as a peer in this region.
    if (i === a.length || i === b.length) continue;
    const owner = i ? a[i - 1] : null;
    if (!edges.has(owner)) edges.set(owner, []);
    edges.get(owner)!.push({ ...edge, sourceId: a[i], targetId: b[i] });
  }
  const regions = new Map<string | null, LayoutResult>();
  const sizes = new Map<string, Rect>();
  const visit = async (owner: string | null): Promise<void> => {
    signal?.throwIfAborted();
    const members = children.get(owner) ?? [];
    for (const id of members) {
      if (children.has(id)) await visit(id);
      else
        sizes.set(id, { x: 0, y: 0, width: nodes.get(id)!.width, height: nodes.get(id)!.height });
    }
    const region: LayoutRequest = {
      nodes: members.map((id) => ({
        id,
        parentId: null,
        width: sizes.get(id)!.width,
        height: sizes.get(id)!.height,
      })),
      edges: edges.get(owner) ?? [],
    };
    // No crossing objective exists in an edgeless region. Packing it directly
    // avoids spending the solver budget on hundreds of unrelated declarations.
    const result = region.edges.length ? await arrange(region) : packUnconnected(region);
    regions.set(owner, result);
    if (owner)
      sizes.set(owner, {
        x: 0,
        y: 0,
        // A domain has a readable overview footprint regardless of how many
        // descendants it owns. Zoom reveals the fitted inner coordinate space.
        width: 960,
        height: 660,
      });
  };
  await visit(null);
  const result: LayoutResult = {
    engine: regions.get(null)!.engine,
    boxes: new Map(),
    routes: new Map(),
    routeEndpoints: new Map(),
    bounds: regions.get(null)!.bounds,
  };
  const place = (owner: string | null, x: number, y: number, scale = 1) => {
    const region = regions.get(owner)!;
    for (const [id, box] of region.boxes) {
      const absolute = {
        x: box.x * scale + x,
        y: box.y * scale + y,
        width: box.width * scale,
        height: box.height * scale,
      };
      result.boxes.set(id, absolute);
      if (regions.has(id)) {
        const inner = regions.get(id)!;
        const contentScale =
          Math.min(
            (box.width - 120) / inner.bounds.width,
            (box.height - 150) / inner.bounds.height,
          ) * scale;
        const insetX = (absolute.width - inner.bounds.width * contentScale) / 2;
        const insetY =
          90 * scale + (absolute.height - 150 * scale - inner.bounds.height * contentScale) / 2;
        place(
          id,
          absolute.x + insetX - inner.bounds.x * contentScale,
          absolute.y + insetY - inner.bounds.y * contentScale,
          contentScale,
        );
      }
    }
    for (const edge of edges.get(owner) ?? []) {
      const route = region.routes.get(edge.id);
      if (!route) continue;
      result.routes.set(
        edge.id,
        route.map((p) => ({ x: p.x * scale + x, y: p.y * scale + y })),
      );
      result.routeEndpoints!.set(edge.id, { sourceId: edge.sourceId, targetId: edge.targetId });
    }
  };
  place(null, 0, 0);
  return result;
}

function packUnconnected(request: LayoutRequest): LayoutResult {
  const count = request.nodes.length;
  const columns = Math.max(1, Math.ceil(Math.sqrt(count)));
  const rows = Math.ceil(count / columns);
  const widths = Array<number>(columns).fill(0),
    heights = Array<number>(rows).fill(0);
  request.nodes.forEach((node, i) => {
    widths[i % columns] = Math.max(widths[i % columns], node.width);
    heights[Math.floor(i / columns)] = Math.max(heights[Math.floor(i / columns)], node.height);
  });
  const offsets = (sizes: number[]) => {
    let next = 0;
    return sizes.map((size) => {
      const position = next;
      next += size + 120;
      return position;
    });
  };
  const xs = offsets(widths),
    ys = offsets(heights);
  return {
    engine: 'Unconnected region packing',
    boxes: new Map(
      request.nodes.map((node, i) => [
        node.id,
        {
          x: xs[i % columns],
          y: ys[Math.floor(i / columns)],
          width: node.width,
          height: node.height,
        },
      ]),
    ),
    routes: new Map(),
    bounds: {
      x: 0,
      y: 0,
      width: Math.max(1, (xs.at(-1) ?? 0) + (widths.at(-1) ?? 0)),
      height: Math.max(1, (ys.at(-1) ?? 0) + (heights.at(-1) ?? 0)),
    },
  };
}
