import type { Snapshot } from '../kernel/generated/Snapshot';
import type { Element } from '../kernel/generated/Element';
import type { Reference } from '../kernel/generated/Reference';
import type { SourceDocument } from '../kernel/generated/SourceDocument';
export type { SourceDocument, Reference };

/** Presentation records contain labels and source coordinates, never language rules. */
export interface ModelElement {
  id: string;
  parentId: string | null;
  documentId: string;
  library: boolean;
  kind: string;
  definition: boolean;
  name: string;
  qualifiedName: string;
  typeName?: string;
  baseName?: string;
  multiplicity?: string;
  value?: string;
  documentation: string;
  start: number;
  end: number;
  line: number;
  endLine: number;
  references: Reference[];
  semantic: Element;
}

export class ViewerModel {
  readonly snapshot: Snapshot;
  readonly byId = new Map<string, ModelElement>();
  readonly elements: ModelElement[] = [];
  readonly documents: SourceDocument[];
  readonly documentsById: Map<string, SourceDocument>;
  readonly owned = new Map<string | null, ModelElement[]>();
  readonly diagnostics;
  readonly connectionIds: Set<string>;

  constructor(snapshot: Snapshot) {
    this.snapshot = snapshot;
    if (snapshot.schemaVersion !== 3) throw new Error('Unsupported kernel snapshot');
    this.connectionIds = new Set(snapshot.connections.map((c) => c.elementId));
    this.documents = [...snapshot.documents].sort(
      (a, b) => Number(a.id.startsWith('library://')) - Number(b.id.startsWith('library://')),
    );
    this.documentsById = new Map(this.documents.map((d) => [d.id, d]));
    const references = new Map<string, Reference[]>();
    for (const r of snapshot.references)
      if (r.ownerId) references.set(r.ownerId, [...(references.get(r.ownerId) ?? []), r]);
    for (const e of snapshot.elements) {
      const refs = references.get(e.id) ?? [];
      const label = e.metaclass
        .replace(/Definition$|Usage$/, '')
        .replace(/([a-z])([A-Z])/g, '$1 $2')
        .toLowerCase();
      const joinReferences = (kind: Reference['kind']) =>
        refs
          .filter((r) => r.kind === kind)
          .map((r) => r.spelling)
          .join(', ') || undefined;
      const source = e.source;
      const view: ModelElement = {
        id: e.id,
        parentId: e.parentId,
        library: e.library,
        kind: label,
        definition: e.metaclass.endsWith('Definition'),
        name: e.name,
        qualifiedName: e.qualifiedName ?? e.name,
        typeName: joinReferences('typing'),
        baseName: joinReferences('specialization'),
        multiplicity: e.declaredMultiplicity ?? undefined,
        value: e.valueExpression ?? undefined,
        documentation: e.documentation,
        documentId: source?.documentId ?? '',
        start: source?.start.utf16Offset ?? 0,
        end: source?.end.utf16Offset ?? 0,
        line: source?.start.line ?? 1,
        endLine: source?.end.line ?? 1,
        references: refs,
        semantic: e,
      };
      this.byId.set(e.id, view);
      if (this.visible(view)) {
        this.elements.push(view);
        this.owned.set(e.parentId, [...(this.owned.get(e.parentId) ?? []), view]);
      }
    }
    this.diagnostics = snapshot.diagnostics.map((d) => ({
      ...d,
      documentId: d.source.documentId,
      line: d.source.start.line,
    }));
  }

  private visible(e: ModelElement) {
    return (
      !e.library &&
      !!e.semantic.source &&
      e.semantic.membersAvailable &&
      !this.connectionIds.has(e.id)
    );
  }

  children(id: string | null) {
    return this.owned.get(id) ?? [];
  }

  /** The kernel supplies inheritance/redefinition. Library members are hidden by
   * this view's display policy; their facts stay available in the snapshot. */
  members(id: string | null): ModelElement[] {
    if (id === null) return this.children(null);
    return (this.byId.get(id)?.semantic.effectiveMembers ?? [])
      .map((key) => this.byId.get(key))
      .filter((e): e is ModelElement => !!e && this.visible(e));
  }

  connectionsFor(id: string | null) {
    const effective = new Set(id ? this.byId.get(id)?.semantic.effectiveMembers : []);
    return this.snapshot.connections.filter(
      (c) => this.byId.get(c.elementId)?.parentId === id || effective.has(c.elementId),
    );
  }
}

export const emptyModel = () =>
  new ViewerModel({
    schemaVersion: 3,
    revision: 0,
    engineVersion: '',
    libraryVersion: '',
    documents: [],
    elements: [],
    memberships: [],
    references: [],
    connections: [],
    diagnostics: [],
  });
export const children = (model: ViewerModel, id: string | null) => model.children(id);
export const sourceOf = (model: ViewerModel, e: ModelElement) =>
  model.documentsById.get(e.documentId)?.text.slice(e.start, e.end) ?? '';
export const referenceTarget = (model: ViewerModel, reference: Reference) =>
  reference.resolution.status === 'resolved'
    ? model.byId.get(reference.resolution.targetId)
    : undefined;
