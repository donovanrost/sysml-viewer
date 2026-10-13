import type { ViewerModel, ModelElement } from './model.ts';
import { referenceTarget } from './model.ts';

export interface Edge {
  a?: ModelElement;
  b?: ModelElement;
  source: string;
  target: string;
  kind: string;
  status: string;
}

/** Choosing the visible card for a resolved chain is presentation, not resolution. */
export function diagramEdges(
  model: ViewerModel,
  scopeId: string | null,
  mode: 'connections' | 'types',
  nodes: ModelElement[],
): Edge[] {
  const visible = new Set(nodes.map((n) => n.id));
  if (mode === 'types')
    return nodes.flatMap((e) =>
      e.references
        .filter((r) => ['typing', 'specialization', 'subsetting', 'redefinition'].includes(r.kind))
        .map((r) => {
          const target = referenceTarget(model, r);
          return {
            a: e,
            b: target,
            source: e.name,
            target: r.spelling,
            kind: r.kind,
            status:
              r.resolution.status === 'resolved'
                ? target && visible.has(target.id)
                  ? 'resolved'
                  : 'outside view'
                : r.resolution.status,
          };
        }),
    );
  return model.connectionsFor(scopeId).map((c) => {
    if (c.ends.length !== 2)
      return {
        source: model.byId.get(c.elementId)?.name ?? 'Connection',
        target: `${c.ends.length} ends`,
        kind: 'connect',
        status: 'n-ary view unavailable',
      };
    const [from, to] = c.ends;
    const owner = (end: typeof from) =>
      end.resolution.status === 'resolved'
        ? model.byId.get(end.chain.find((id) => visible.has(id)) ?? '')
        : undefined;
    const a = owner(from),
      b = owner(to);
    const status =
      from.resolution.status !== 'resolved'
        ? from.resolution.status
        : to.resolution.status !== 'resolved'
          ? to.resolution.status
          : !a || !b
            ? 'outside view'
            : 'resolved';
    return { a, b, source: from.spelling, target: to.spelling, kind: 'connect', status };
  });
}
