import type { CompileResponse, D2 } from '@d2lang/d2';
import type { LayoutEngine, LayoutRequest, LayoutResult } from './layout-engine.ts';
import type { Point } from './map-order.ts';
import type { Rect } from './system-map.ts';
import { arrangeOwnershipRegions } from './region-layout.ts';

export const TALA_VERSION = '@d2lang/d2@0.1.34';

/** Safe generated identifiers only. User names, source and metadata never become
 * D2 program text. Our own renderer supplies all labels and interaction. */
export function encodeTala(request: LayoutRequest) {
  if (request.nodes.length > 10000 || request.edges.length > 30000)
    throw new Error('This map exceeds the automatic layout size limit.');
  const nodes = new Map(request.nodes.map((node) => [node.id, node]));
  if (
    nodes.size !== request.nodes.length ||
    new Set(request.edges.map((e) => e.id)).size !== request.edges.length
  )
    throw new Error('Duplicate layout identifiers');
  const aliases = new Map([...nodes.keys()].sort().map((id, i) => [id, `n${i}`]));
  const paths = new Map<string, string>();
  const visiting = new Set<string>();
  const path = (id: string): string => {
    if (paths.has(id)) return paths.get(id)!;
    const node = nodes.get(id);
    if (!node || visiting.has(id)) throw new Error('Invalid layout ownership');
    visiting.add(id);
    const value = `${node.parentId ? `${path(node.parentId)}.` : ''}${aliases.get(id)}`;
    visiting.delete(id);
    paths.set(id, value);
    return value;
  };
  const containers = new Set(request.nodes.map((n) => n.parentId));
  const lines = ['vars: {d2-config: {layout-engine: tala; data: {tala-seeds: [1]}}}'];
  for (const node of request.nodes) {
    if (![node.width, node.height].every((n) => Number.isFinite(n) && n > 0 && n <= 10000000))
      throw new Error('Invalid layout dimensions');
    // Container labels reserve a heading. The text is never rendered by D2.
    lines.push(
      `${path(node.id)}: {label: "Element";${containers.has(node.id) ? '' : ` width: ${node.width}; height: ${node.height};`}}`,
    );
  }
  // Duplicate relations share one geometric route without losing provenance.
  const edges = new Map<
    string,
    { alias: string; ids: string[]; sourceId: string; targetId: string }
  >();
  for (const edge of request.edges) {
    const source = path(edge.sourceId),
      target = path(edge.targetId);
    const key = JSON.stringify([edge.sourceId, edge.targetId, edge.directed]);
    const previous = edges.get(key);
    if (previous) {
      previous.ids.push(edge.id);
      continue;
    }
    const alias = `e${edges.size}`;
    edges.set(key, { alias, ids: [edge.id], sourceId: edge.sourceId, targetId: edge.targetId });
    lines.push(`${source} ${edge.directed ? '->' : '--'} ${target}: {tooltip: ${alias}}`);
  }
  return { source: lines.join('\n'), paths, edges: [...edges.values()] };
}

type Encoded = ReturnType<typeof encodeTala>;
const finite = (value: unknown): value is number =>
  typeof value === 'number' && Number.isFinite(value);

/** Validate the engine boundary before replacing a usable scene. */
export function decodeTala(
  request: LayoutRequest,
  encoded: Encoded,
  response: CompileResponse,
): LayoutResult {
  const boxes = new Map<string, Rect>(),
    routes = new Map<string, Point[]>();
  const ids = new Map([...encoded.paths].map(([id, path]) => [path, id]));
  for (const shape of response.diagram.shapes ?? []) {
    const id = ids.get(shape.id);
    if (
      !id ||
      boxes.has(id) ||
      !shape.pos ||
      ![shape.pos.x, shape.pos.y, shape.width, shape.height].every(finite) ||
      shape.width <= 0 ||
      shape.height <= 0
    )
      throw new Error('TALA returned invalid element geometry');
    boxes.set(id, { x: shape.pos.x, y: shape.pos.y, width: shape.width, height: shape.height });
  }
  if (boxes.size !== request.nodes.length) throw new Error('TALA omitted model elements');
  for (const node of request.nodes) {
    if (!node.parentId) continue;
    const a = boxes.get(node.id)!,
      b = boxes.get(node.parentId)!;
    if (
      a.x < b.x - 1 ||
      a.y < b.y - 1 ||
      a.x + a.width > b.x + b.width + 1 ||
      a.y + a.height > b.y + b.height + 1
    )
      throw new Error('TALA returned invalid containment');
  }
  const edges = new Map(encoded.edges.map((edge) => [edge.alias, edge]));
  const seen = new Set<string>();
  for (const connection of response.diagram.connections ?? []) {
    const edge = edges.get(connection.tooltip);
    if (
      !edge ||
      connection.src !== encoded.paths.get(edge.sourceId) ||
      connection.dst !== encoded.paths.get(edge.targetId)
    )
      throw new Error('TALA returned an unsupported relationship route');
    const points: Point[] = connection.route.map((p: unknown) => {
      if (!p || typeof p !== 'object' || !('x' in p) || !('y' in p) || !finite(p.x) || !finite(p.y))
        throw new Error('TALA returned invalid route coordinates');
      return { x: p.x, y: p.y };
    });
    if (points.length < 2 || seen.has(edge.ids[0]))
      throw new Error('TALA returned an incomplete route');
    for (const id of edge.ids) seen.add(id);
    // TALA sometimes emits diagonal/curved routes for mixed type and connection
    // graphs. Keep validated placement; the viewer's obstacle-aware router handles
    // these edges. Never feed non-orthogonal paths into the trimming algorithm.
    if (
      connection.isCurve ||
      points.some(
        (point, i) =>
          i > 0 &&
          Math.abs(point.x - points[i - 1].x) > 0.01 &&
          Math.abs(point.y - points[i - 1].y) > 0.01,
      )
    )
      continue;
    for (const id of edge.ids) routes.set(id, points);
  }
  if (seen.size !== request.edges.length) throw new Error('TALA omitted model relationships');
  const points = [...boxes.values()].flatMap((r) => [
    { x: r.x, y: r.y },
    { x: r.x + r.width, y: r.y + r.height },
  ]);
  for (const route of routes.values()) points.push(...route);
  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const p of points) {
    minX = Math.min(minX, p.x);
    minY = Math.min(minY, p.y);
    maxX = Math.max(maxX, p.x);
    maxY = Math.max(maxY, p.y);
  }
  return {
    engine: `TALA (${TALA_VERSION})`,
    boxes,
    routes,
    bounds: points.length
      ? { x: minX, y: minY, width: Math.max(1, maxX - minX), height: Math.max(1, maxY - minY) }
      : { x: 0, y: 0, width: 960, height: 660 },
  };
}

type Compiler = Pick<D2, 'compile' | 'dispose'>;
type CompilerFactory = () => Promise<Compiler>;
const defaultCompiler: CompilerFactory = async () => {
  const { D2 } = await import('@d2lang/d2');
  return new D2();
};

export class TalaLayoutEngine implements LayoutEngine {
  private createCompiler: CompilerFactory;
  private timeoutMs: number;
  constructor(createCompiler = defaultCompiler, timeoutMs = 45000) {
    this.createCompiler = createCompiler;
    this.timeoutMs = timeoutMs;
  }
  async layout(
    request: LayoutRequest,
    { signal }: { signal?: AbortSignal } = {},
  ): Promise<LayoutResult> {
    signal?.throwIfAborted();
    encodeTala(request); // Validate the complete ownership graph before partitioning.
    if (!request.nodes.length)
      return {
        engine: `TALA (${TALA_VERSION})`,
        boxes: new Map(),
        routes: new Map(),
        bounds: { x: 0, y: 0, width: 960, height: 660 },
      };
    let compiler: Compiler | undefined;
    let timer: ReturnType<typeof setTimeout> | undefined;
    let abort: () => void = () => {};
    let finished = false;
    const stop = new Promise<never>((_, reject) => {
      abort = () => reject(new DOMException('Layout superseded', 'AbortError'));
      signal?.addEventListener('abort', abort, { once: true });
      timer = setTimeout(
        () => reject(new Error('Automatic layout exceeded 45 seconds.')),
        this.timeoutMs,
      );
    });
    const run = async () => {
      compiler = await this.createCompiler();
      if (finished || signal?.aborted) {
        await compiler.dispose();
        throw new DOMException('Layout superseded', 'AbortError');
      }
      const arranged = await arrangeOwnershipRegions(
        request,
        async (region) => {
          if (finished) throw new DOMException('Layout superseded', 'AbortError');
          // TALA bounds its internal coordinate space. Large nested regions
          // are solved at a normalized scale, then restored to world units.
          const maxDimension = region.nodes.reduce(
            (n, node) => Math.max(n, node.width, node.height),
            1,
          );
          const factor = Math.max(1, maxDimension / Math.min(600, 20000 / region.nodes.length));
          const normalized = {
            ...region,
            nodes: region.nodes.map((node) => ({
              ...node,
              width: Math.ceil(node.width / factor),
              height: Math.ceil(node.height / factor),
            })),
          };
          const encoded = encodeTala(normalized);
          const result = decodeTala(
            normalized,
            encoded,
            await compiler!.compile(encoded.source, { layout: 'tala' }),
          );
          const rect = (box: Rect): Rect => ({
            x: box.x * factor,
            y: box.y * factor,
            width: box.width * factor,
            height: box.height * factor,
          });
          return {
            ...result,
            boxes: new Map([...result.boxes].map(([id, box]) => [id, rect(box)])),
            routes: new Map(
              [...result.routes].map(([id, route]) => [
                id,
                route.map((p) => ({ x: p.x * factor, y: p.y * factor })),
              ]),
            ),
            bounds: rect(result.bounds),
          };
        },
        signal,
      );
      return { ...arranged, engine: `TALA (${TALA_VERSION})` };
    };
    try {
      return await Promise.race([run(), stop]);
    } finally {
      finished = true;
      clearTimeout(timer);
      signal?.removeEventListener('abort', abort);
      if (compiler) void compiler.dispose().catch(() => {});
    }
  }
}
