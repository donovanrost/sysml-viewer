import type { ViewerModel, ModelElement } from './model.ts';
import type { MapNode, SystemMap, MapLayout, Rect } from './system-map.ts';

/** Display occurrences of kernel-provided effective features. No type lookup,
 * inheritance, name matching or redefinition rules belong in this projection. */
export function buildStructureMap(
  model: ViewerModel,
  scopeId: string,
  options: {
    limit?: number;
    contextName?: string;
    inheritedFrom?: ModelElement;
  } = {},
): SystemMap {
  const limit = Math.max(1, options.limit ?? 600);
  const root = model.byId.get(scopeId);
  if (!root) throw new Error('Unknown structure scope');
  const graph: SystemMap = {
    nodes: new Map(),
    roots: [scopeId],
    relations: [],
    issues: [],
    libraryReferences: 0,
    view: { kind: 'structure', elementId: scopeId, title: options.contextName ?? root.name },
  };
  const add = (
    element: ModelElement,
    path: string[],
    parentId: string | null,
    contextName: string,
  ) => {
    const id = parentId === null ? scopeId : `context:${JSON.stringify(path)}`;
    const node: MapNode = { element, parentId, children: [], contextName };
    graph.nodes.set(id, node);
    if (parentId) {
      const parent = graph.nodes.get(parentId)!;
      parent.children.push(id);
      if (element.parentId !== parent.element.id)
        node.inheritedFrom = model.byId.get(element.parentId ?? '');
    }
    const members = model
      .members(element.id)
      .filter(
        (e) => e.semantic.metaclass.endsWith('Usage') && e.semantic.metaclass !== 'MetadataUsage',
      );
    if (!element.semantic.inheritanceComplete) node.note = 'Inherited structure is incomplete';
    if (path.slice(0, -1).includes(element.id)) {
      node.note = 'Recursive structure · explore to continue';
      node.truncated = true;
      return id;
    }
    if (path.length >= 7) {
      if (members.length) {
        node.note = 'More structure · explore to continue';
        node.truncated = true;
      }
      return id;
    }
    for (const member of members) {
      if (graph.nodes.size >= limit) {
        node.note = 'Detail limit reached · explore to continue';
        node.truncated = true;
        break;
      }
      add(member, [...path, member.id], id, `${contextName}.${member.name}`);
    }
    return id;
  };
  add(root, [scopeId], null, options.contextName ?? root.name);
  graph.nodes.get(scopeId)!.inheritedFrom = options.inheritedFrom;

  // Resolve only display positions along the exact endpoint chain supplied by
  // the kernel. Repeated uses of a definition never share a display occurrence.
  for (const [id, node] of graph.nodes) {
    const endpoint = (chain: string[]) => {
      let current = id;
      let matched = false;
      for (const semanticId of chain) {
        const next = graph.nodes
          .get(current)!
          .children.find((child) => graph.nodes.get(child)!.element.id === semanticId);
        if (!next) return matched && graph.nodes.get(current)!.truncated ? current : undefined;
        current = next;
        matched = true;
      }
      return matched ? current : undefined;
    };
    for (const connection of model.connectionsFor(node.element.id)) {
      const owner = model.byId.get(connection.elementId);
      if (!owner || owner.library || !owner.semantic.source) continue;
      const ends = connection.ends;
      const a =
        ends.length === 2 && ends[0].resolution.status === 'resolved'
          ? endpoint(ends[0].chain)
          : undefined;
      const b =
        ends.length === 2 && ends[1].resolution.status === 'resolved'
          ? endpoint(ends[1].chain)
          : undefined;
      if (!a || !b) {
        graph.issues.push({
          ownerId: id,
          kind: 'connection',
          target: owner.name,
          status:
            ends.length !== 2
              ? `${ends.length}-ended connection; map requires two ends`
              : 'Endpoint is unresolved or outside this structure',
        });
        continue;
      }
      graph.relations.push({
        id: `context-connection:${JSON.stringify([id, connection.elementId])}`,
        sourceId: a,
        targetId: b,
        kind: 'connection',
        directed: false,
        connectionId: connection.elementId,
        sourceLabel: `${node.contextName}.${ends[0].spelling}`,
        targetLabel: `${node.contextName}.${ends[1].spelling}`,
      });
    }
  }
  return graph;
}

/** Explicit definition ancestry/descendants, independent of package ownership.
 * Usage typing is intentionally not promoted to definition specialization. */
export function buildTypeHierarchy(model: ViewerModel, selectedId: string): SystemMap {
  const root = model.byId.get(selectedId);
  if (!root?.definition) throw new Error('A type hierarchy requires a definition');
  const references = model.snapshot.references.filter(
    (r) => r.kind === 'specialization' && r.ownerId && model.byId.get(r.ownerId)?.definition,
  );
  const edges = references.flatMap((r, index) => {
    const target =
      r.resolution.status === 'resolved' ? model.byId.get(r.resolution.targetId) : undefined;
    return target?.definition && !target.library
      ? [{ r, index, source: r.ownerId!, target: target.id }]
      : [];
  });
  const included = new Set([selectedId]);
  for (const direction of ['ancestors', 'descendants'] as const) {
    const visited = new Set<string>();
    const queue = [selectedId];
    for (let i = 0; i < queue.length; i++) {
      const id = queue[i];
      if (visited.has(id)) continue;
      visited.add(id);
      for (const edge of edges) {
        const from = direction === 'ancestors' ? edge.source : edge.target;
        const to = direction === 'ancestors' ? edge.target : edge.source;
        if (from === id && !visited.has(to)) {
          included.add(to);
          queue.push(to);
        }
      }
    }
  }
  const graph: SystemMap = {
    nodes: new Map(
      [...included].map((id) => [
        id,
        { element: model.byId.get(id)!, parentId: null, children: [] },
      ]),
    ),
    roots: [...included].sort(),
    relations: [],
    issues: [],
    libraryReferences: 0,
    view: { kind: 'hierarchy', elementId: selectedId, title: root.name },
  };
  for (const { r, index, source, target } of edges) {
    if (!included.has(source) || !included.has(target)) continue;
    graph.relations.push({
      id: `specialization:${index}`,
      sourceId: source,
      targetId: target,
      kind: 'specialization',
      directed: true,
      sourceLabel: model.byId.get(source)!.qualifiedName,
      targetLabel: model.byId.get(target)!.qualifiedName,
      reference: r,
    });
  }
  for (const r of references) {
    if (!included.has(r.ownerId!)) continue;
    if (r.resolution.status !== 'resolved')
      graph.issues.push({
        ownerId: r.ownerId!,
        kind: r.kind,
        target: r.spelling,
        status: r.resolution.status,
      });
    else if (model.byId.get(r.resolution.targetId)?.library) graph.libraryReferences++;
  }
  return graph;
}

/** Rank the explicit specialization graph, general types above specific types.
 * This is presentation geometry, not an inheritance computation. Cycles remain
 * visible and are marked instead of being silently broken into false ancestry. */
export function layoutTypeHierarchy(graph: SystemMap): MapLayout {
  const rank = new Map<string, number>();
  const pending = new Map([...graph.nodes.keys()].map((id) => [id, 0]));
  for (const r of graph.relations) pending.set(r.sourceId, pending.get(r.sourceId)! + 1);
  const queue = [...pending]
    .filter(([, n]) => !n)
    .map(([id]) => id)
    .sort();
  for (const id of queue) rank.set(id, 0);
  for (let i = 0; i < queue.length; i++) {
    const id = queue[i];
    for (const r of graph.relations.filter((r) => r.targetId === id)) {
      rank.set(r.sourceId, Math.max(rank.get(r.sourceId) ?? 0, rank.get(id)! + 1));
      pending.set(r.sourceId, pending.get(r.sourceId)! - 1);
      if (!pending.get(r.sourceId)) queue.push(r.sourceId);
    }
  }
  const cycleRank = Math.max(0, ...rank.values()) + 1;
  const levels = new Map<number, string[]>();
  for (const [id, node] of graph.nodes) {
    if (pending.get(id)) {
      node.note = 'Cyclic or cycle-dependent specialization';
      rank.set(id, cycleRank);
    }
    const level = rank.get(id)!;
    levels.set(level, [...(levels.get(level) ?? []), id]);
  }
  const width = Math.max(1, ...[...levels.values()].map((ids) => ids.length)) * 300;
  const boxes = new Map<string, Rect>();
  for (const [level, ids] of levels) {
    ids.sort((a, b) =>
      graph.nodes
        .get(a)!
        .element.qualifiedName.localeCompare(graph.nodes.get(b)!.element.qualifiedName),
    );
    ids.forEach((id, index) =>
      boxes.set(id, {
        x: (width - ids.length * 300) / 2 + index * 300,
        y: level * 220,
        width: 240,
        height: 130,
      }),
    );
  }
  return {
    boxes,
    bounds: { x: -30, y: -30, width, height: Math.max(0, ...levels.keys()) * 220 + 190 },
  };
}
