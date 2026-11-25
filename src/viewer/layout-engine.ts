import type { Rect, SystemMap } from './system-map.ts';
import type { Point } from './map-order.ts';

/** Geometry-only boundary. Engines never receive source text or resolve SysML. */
export interface LayoutRequest {
  nodes: { id: string; parentId: string | null; width: number; height: number }[];
  edges: { id: string; sourceId: string; targetId: string; directed: boolean }[];
}
export interface LayoutResult {
  engine: string;
  boxes: Map<string, Rect>;
  routes: Map<string, Point[]>;
  bounds: Rect;
  routeEndpoints?: Map<string, { sourceId: string; targetId: string }>;
}
export interface LayoutEngine {
  layout(request: LayoutRequest, options?: { signal?: AbortSignal }): Promise<LayoutResult>;
}

export function mapLayoutRequest(graph: SystemMap): LayoutRequest {
  return {
    // Stable input ordering, fixed card dimensions and a fixed seed prevent
    // selection, label display and camera changes from influencing the layout.
    nodes: [...graph.nodes]
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([id, node]) => ({
        id,
        parentId: node.parentId,
        width: 240,
        height: 120,
      })),
    edges: graph.relations.map(({ id, sourceId, targetId, directed }) => ({
      id,
      sourceId,
      targetId,
      directed,
    })),
  };
}
