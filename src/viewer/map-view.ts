import {
  buildSystemMap,
  isInlineValueDefinition,
  layoutSystemMap,
  projectSystemMap,
  groupMapRelations,
  screenRect,
  fitCamera,
  focusCamera,
  zoomCamera,
} from './system-map.ts';
import type { Camera, MapGroup, MapLens, MapLayout, SystemMap, Rect } from './system-map.ts';
import type { ViewerModel } from './model.ts';
import { referenceTarget } from './model.ts';
import { buildStructureMap, buildTypeHierarchy, layoutTypeHierarchy } from './semantic-views.ts';
import { routeMapEdge, routePath, placeMapLabel, fallbackMapLabel } from './map-routing.ts';
import type { Point } from './map-order.ts';
import { TalaLayoutEngine } from './tala-layout.ts';
import { icon } from '../ui/icons.ts';
import { mapLayoutRequest } from './layout-engine.ts';
import type { LayoutEngine } from './layout-engine.ts';
import { projectLayoutRoute, visibleRouteSegments } from './layout-projection.ts';

const esc = (value: string) =>
  value.replace(
    /[&<>"']/g,
    (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!,
  );
const clamp = (n: number, min: number, max: number) => Math.max(min, Math.min(max, n));
interface MapLocation {
  graph: SystemMap;
  layout: MapLayout;
  camera: Camera;
  selected: string | null;
  expanded: Set<string>;
  lens: MapLens;
  layoutState: 'loading' | 'ready' | 'fallback';
  layoutFailure: string;
}

/** Camera and rendering own no SysML language rules. One instance survives view
 * switches, so inspector updates never reset navigation or rebuild the graph. */
export class SystemMapView {
  private model?: ViewerModel;
  private catalogue = buildSystemMapPlaceholder();
  private graph = buildSystemMapPlaceholder();
  private layout = layoutSystemMap(this.graph);
  private camera: Camera = { x: 0, y: 0, scale: 1 };
  private expanded = new Set<string>();
  private selected: string | null = null;
  private lens: MapLens = 'connections';
  private showLabels = false;
  private activeGroup: string | null = null;
  private routingKey = '';
  private routed = new Map<string, Point[]>();
  private history: MapLocation[] = [];
  private groups: MapGroup[] = [];
  private page = 0;
  private showingRelationships = false;
  private showingIssues = false;
  private needsFit = true;
  private frame = 0;
  private layoutEngine: LayoutEngine;
  private layoutAbort?: AbortController;
  private layoutState: 'loading' | 'ready' | 'fallback' = 'loading';
  private layoutFailure = '';
  private stage: HTMLElement;
  private host: HTMLElement;
  private onSelect: (id: string | null) => void;
  private onGroup: (group: MapGroup) => void;

  constructor(
    host: HTMLElement,
    onSelect: (id: string | null) => void,
    onGroup: (group: MapGroup) => void,
    layoutEngine: LayoutEngine = new TalaLayoutEngine(),
  ) {
    this.layoutEngine = layoutEngine;
    this.host = host;
    this.onSelect = onSelect;
    this.onGroup = onGroup;
    host.innerHTML = `
      <div class="map-stage" tabindex="0" role="region" aria-label="System map. Arrow keys pan; plus and minus zoom; Home shows overview."><div class="map-nodes"></div><svg class="map-edges" role="group" aria-label="Model relationships"></svg></div>
      <div class="map-context" hidden></div>
      <details class="map-options"><summary aria-label="Relationship options" title="Relationship options">${icon('link')}<span class="map-lens-name">Types</span>${icon('down')}</summary>
        <div class="map-options-panel"><strong>Relationships</strong><label class="map-lens-select">Show<select aria-label="Map relationships"><option value="types">Types & specialization</option><option value="connections">Connections</option></select></label>
          <label class="map-label-toggle"><input type="checkbox" aria-label="Show relationship labels"> Show labels</label>
          <div class="map-options-actions"><button data-map-action="relations" aria-controls="map-relationship-tray">Browse relationships ${icon('arrow')}</button><button data-map-action="issues" aria-controls="map-relationship-tray" hidden>Unmapped</button></div>
          <p class="map-detail-notice" hidden></p>
        </div>
      </details>
      <div class="map-navigation map-control" role="group" aria-label="Map navigation"><button type="button" data-map-action="back" aria-label="Previous map location" title="Previous map location">${icon('back')}</button><button type="button" data-map-action="overview" aria-label="Overview" title="Fit the whole model (Home)">${icon('fit')}</button><button type="button" data-map-action="focus" aria-label="Focus selection" title="Focus selection">${icon('target')}</button></div>
      <div class="map-zoom map-control" role="group" aria-label="Map zoom controls"><button data-map-action="out" aria-label="Zoom map out" title="Zoom out (−)">−</button><output aria-label="Map zoom"></output><button data-map-action="in" aria-label="Zoom map in" title="Zoom in (+)">+</button></div>
      <div class="map-layout-status" role="status" hidden></div><section id="map-relationship-tray" class="map-tray" aria-label="Relationship browser" tabindex="-1" hidden></section>`;
    this.stage = host.querySelector('.map-stage')!;
    host.addEventListener('click', (event) => {
      const button = (event.target as Element).closest<HTMLElement>(
        '[data-map-action],[data-map-id],[data-map-group],[data-map-definition],[data-map-hierarchy],[data-map-explore]',
      );
      if (!button) return;
      event.stopPropagation();
      if (button.dataset.mapDefinition) {
        this.goToDefinition(button.dataset.mapDefinition);
        return;
      }
      if (button.dataset.mapHierarchy) {
        this.showHierarchy(button.dataset.mapHierarchy);
        return;
      }
      if (button.dataset.mapExplore) {
        this.explore(button.dataset.mapExplore);
        return;
      }
      if (button.dataset.mapId) {
        this.activeGroup = null;
        this.selected = button.dataset.mapId;
        this.publishSelection();
        this.schedule();
        return;
      }
      if (button.dataset.mapGroup) {
        const group = this.groups.find((g) => g.id === button.dataset.mapGroup);
        if (group) {
          this.activeGroup = group.id;
          this.onGroup(group);
          this.schedule();
        }
        return;
      }
      const action = button.dataset.mapAction;
      if (action === 'retry-layout') this.arrange();
      if (action === 'in' || action === 'out') this.zoom(action === 'in' ? 1.4 : 1 / 1.4);
      if (action === 'overview') this.overview();
      if (action === 'focus' && this.selected) this.focus(this.selected);
      if (action === 'back') {
        const previous = this.history.pop();
        if (previous) {
          this.layoutAbort?.abort();
          this.graph = previous.graph;
          this.layout = previous.layout;
          this.camera = previous.camera;
          this.selected = previous.selected;
          this.expanded = previous.expanded;
          this.lens = previous.lens;
          this.layoutState = previous.layoutState;
          this.layoutFailure = previous.layoutFailure;
          this.needsFit = false;
          this.resetRelationships();
          if (this.layoutState === 'loading') this.arrange();
          this.publishSelection();
          this.schedule();
        }
      }
      if (action === 'close-tray') {
        this.showingRelationships = this.showingIssues = false;
        host.querySelector<HTMLElement>('.map-options summary')!.focus();
        this.schedule();
      }
      if (action === 'relations' || action === 'issues') {
        host.querySelector<HTMLDetailsElement>('.map-options')!.open = false;
        this.showingRelationships = action === 'relations' ? !this.showingRelationships : false;
        this.showingIssues = action === 'issues' ? !this.showingIssues : false;
        this.page = 0;
        this.schedule();
        requestAnimationFrame(() => host.querySelector<HTMLElement>('.map-tray')!.focus());
      }
      if (action === 'next' || action === 'previous') {
        this.page = Math.max(0, this.page + (action === 'next' ? 1 : -1));
        this.schedule();
      }
    });
    const options = host.querySelector<HTMLDetailsElement>('.map-options')!;
    document.addEventListener('pointerdown', (event) => {
      if (!options.contains(event.target as Node)) options.open = false;
    });
    host.addEventListener('keydown', (event) => {
      if (event.key !== 'Escape') return;
      if (!options.open && !this.showingRelationships && !this.showingIssues) return;
      options.open = false;
      this.showingRelationships = this.showingIssues = false;
      host.querySelector<HTMLElement>('.map-options summary')!.focus();
      this.schedule();
      event.preventDefault();
      event.stopPropagation();
    });
    host.querySelector('input')!.addEventListener('change', (event) => {
      this.showLabels = (event.target as HTMLInputElement).checked;
      this.schedule();
    });
    host.querySelector('select')!.addEventListener('change', (event) => {
      this.lens = (event.target as HTMLSelectElement).value as MapLens;
      this.activeGroup = null;
      this.page = 0;
      this.schedule();
    });
    this.stage.addEventListener('dblclick', (event) => {
      const id = (event.target as Element).closest<HTMLElement>('[data-map-id]')?.dataset.mapId;
      if (id) this.explore(id);
    });
    this.stage.addEventListener(
      'wheel',
      (event) => {
        event.preventDefault();
        if (event.ctrlKey || event.metaKey) {
          const rect = this.stage.getBoundingClientRect();
          this.zoom(Math.exp(-clamp(event.deltaY, -100, 100) * 0.012), {
            x: event.clientX - rect.left,
            y: event.clientY - rect.top,
          });
        } else {
          this.camera = {
            ...this.camera,
            x: this.camera.x - event.deltaX,
            y: this.camera.y - event.deltaY,
          };
          this.schedule();
        }
      },
      { passive: false },
    );
    const pointers = new Map<number, { x: number; y: number }>();
    let moved = false,
      origin = { x: 0, y: 0 };
    this.stage.addEventListener('pointerdown', (event) => {
      if (event.button !== 0) return;
      pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
      origin = { x: event.clientX, y: event.clientY };
      moved = false;
    });
    this.stage.addEventListener('pointermove', (event) => {
      const previous = pointers.get(event.pointerId);
      if (!previous) return;
      const next = { x: event.clientX, y: event.clientY };
      if (Math.hypot(next.x - origin.x, next.y - origin.y) > 4) moved = true;
      if (moved) this.stage.setPointerCapture(event.pointerId);
      const other = [...pointers.entries()].find(([id]) => id !== event.pointerId)?.[1];
      if (other) {
        moved = true;
        const before = Math.hypot(previous.x - other.x, previous.y - other.y);
        const after = Math.hypot(next.x - other.x, next.y - other.y);
        const rect = this.stage.getBoundingClientRect();
        if (before > 0)
          this.camera = zoomCamera(this.camera, after / before, {
            x: (previous.x + other.x) / 2 - rect.left,
            y: (previous.y + other.y) / 2 - rect.top,
          });
        this.camera = {
          ...this.camera,
          x: this.camera.x + (next.x - previous.x) / 2,
          y: this.camera.y + (next.y - previous.y) / 2,
        };
      } else if (moved)
        this.camera = {
          ...this.camera,
          x: this.camera.x + next.x - previous.x,
          y: this.camera.y + next.y - previous.y,
        };
      pointers.set(event.pointerId, next);
      if (moved) this.schedule();
    });
    const release = (event: PointerEvent) => {
      pointers.delete(event.pointerId);
    };
    this.stage.addEventListener('pointerup', release);
    this.stage.addEventListener('pointercancel', release);
    this.stage.addEventListener(
      'click',
      (event) => {
        if (moved) {
          event.preventDefault();
          event.stopPropagation();
          moved = false;
        }
      },
      true,
    );
    this.stage.addEventListener('keydown', (event) => {
      const groupId = (event.target as Element)
        .closest('[data-map-group]')
        ?.getAttribute('data-map-group');
      if (groupId && (event.key === 'Enter' || event.key === ' ')) {
        const group = this.groups.find((g) => g.id === groupId);
        if (group) {
          this.activeGroup = group.id;
          this.onGroup(group);
          this.schedule();
        }
        event.preventDefault();
        return;
      }
      if (event.key === '+' || event.key === '=') this.zoom(1.4);
      else if (event.key === '-') this.zoom(1 / 1.4);
      else if (event.key === 'Home') this.overview();
      else if (event.key.startsWith('Arrow')) {
        const dx = event.key === 'ArrowLeft' ? 70 : event.key === 'ArrowRight' ? -70 : 0;
        const dy = event.key === 'ArrowUp' ? 70 : event.key === 'ArrowDown' ? -70 : 0;
        this.camera = { ...this.camera, x: this.camera.x + dx, y: this.camera.y + dy };
        this.schedule();
      } else return;
      event.preventDefault();
      event.stopPropagation();
    });
    new ResizeObserver(() => this.schedule()).observe(this.stage);
    window.addEventListener('pagehide', () => this.layoutAbort?.abort());
  }
  setModel(model: ViewerModel) {
    this.model = model;
    this.catalogue = buildSystemMap(model);
    this.graph = this.catalogue;
    this.lens = 'connections';
    this.layout = layoutSystemMap(this.graph);
    this.activeGroup = null;
    this.routingKey = '';
    this.routed.clear();
    this.history = [];
    this.expanded.clear();
    this.selected = null;
    this.needsFit = true;
    this.page = 0;
    this.resetRelationships();
    this.arrange();
    this.schedule();
  }
  private arrange() {
    this.layoutAbort?.abort();
    const controller = new AbortController();
    this.layoutAbort = controller;
    this.layoutState = 'loading';
    this.layoutFailure = '';
    const graph = this.graph;
    this.schedule();
    void this.layoutEngine
      .layout(mapLayoutRequest(graph), { signal: controller.signal })
      .then((result) => {
        if (controller.signal.aborted || this.graph !== graph) return;
        // Preserve an inspection performed while the worker was arranging the map.
        // History retains its own graph and geometry, including provisional scenes.
        this.layout = result;
        this.expanded.clear();
        this.routingKey = '';
        this.layoutState = 'ready';
        if (this.selected && result.boxes.has(this.selected)) {
          this.camera = this.selectionCamera(this.selected);
          this.needsFit = false;
        } else this.needsFit = true;
        this.schedule();
      })
      .catch((error: unknown) => {
        if (controller.signal.aborted || this.graph !== graph) return;
        this.layoutState = 'fallback';
        this.layoutFailure = error instanceof Error ? error.message : 'Automatic layout failed';
        this.schedule();
      });
  }
  select(id: string | null, focus = false) {
    const key = id ? this.keyFor(id) : null;
    if (key !== this.selected) this.activeGroup = null;
    if (focus && id) this.explore(id);
    else {
      this.selected = key;
      this.schedule();
    }
  }
  refresh() {
    this.schedule();
  }
  private remember() {
    this.history.push({
      graph: this.graph,
      layout: this.layout,
      camera: { ...this.camera },
      selected: this.selected,
      expanded: new Set(this.expanded),
      lens: this.lens,
      layoutState: this.layoutState,
      layoutFailure: this.layoutFailure,
    });
    if (this.history.length > 40) this.history.shift();
  }
  focus(id: string) {
    const key = this.keyFor(id);
    const element = this.model?.byId.get(id);
    if (!key && element && isInlineValueDefinition(this.model!, element)) {
      this.showHierarchy(id);
      return true;
    }
    if (!key && this.catalogue.nodes.has(id)) {
      this.remember();
      this.activate(this.catalogue, id, 'connections');
      return true;
    }
    if (!key) return false;
    // Hidden annotations navigate to their nearest mapped owner through the caller.
    const box = this.layout.boxes.get(key);
    if (!box) return false;
    this.remember();
    this.needsFit = false;
    this.selected = key;
    this.activeGroup = null;
    this.camera = this.selectionCamera(key);
    this.publishSelection();
    this.schedule();
    return true;
  }
  overview() {
    this.remember();
    if (this.graph !== this.catalogue) {
      this.activate(this.catalogue, null, 'connections');
      return;
    }
    this.needsFit = true;
    this.selected = null;
    this.activeGroup = null;
    this.onSelect(null);
    this.schedule();
  }
  private keyFor(id: string): string | null {
    if (this.selected && this.graph.nodes.get(this.selected)?.element.id === id)
      return this.selected;
    if (this.graph.nodes.has(id)) return id;
    return [...this.graph.nodes].find(([, n]) => n.element.id === id)?.[0] ?? null;
  }
  private selectionCamera(id: string) {
    if (this.graph.view?.kind === 'structure' && this.graph.roots.includes(id)) {
      this.expanded.add(id);
      const children = this.graph.nodes
        .get(id)!
        .children.map((key) => this.layout.boxes.get(key)!)
        .filter(Boolean);
      const minX = Math.min(...children.map((b) => b.x)),
        minY = Math.min(...children.map((b) => b.y));
      const content = children.length
        ? {
            x: minX,
            y: minY,
            width: Math.max(...children.map((b) => b.x + b.width)) - minX,
            height: Math.max(...children.map((b) => b.y + b.height)) - minY,
          }
        : this.layout.boxes.get(id)!;
      const camera = fitCamera(
        content,
        this.stage.clientWidth,
        Math.max(120, this.stage.clientHeight - 160),
      );
      return { ...camera, y: camera.y + 100 };
    }
    return focusCamera(
      this.graph,
      this.layout,
      id,
      this.stage.clientWidth,
      this.stage.clientHeight,
    );
  }
  elementFor(id: string) {
    return this.graph.nodes.get(id)?.element ?? this.model?.byId.get(id);
  }
  selectionContext() {
    const node = this.graph.nodes.get(this.selected ?? '');
    return node?.contextName ? node : undefined;
  }
  private publishSelection() {
    this.onSelect(this.selected ? this.graph.nodes.get(this.selected)!.element.id : null);
  }
  private resetRelationships() {
    this.activeGroup = null;
    this.routingKey = '';
    this.routed.clear();
    this.page = 0;
    this.showingRelationships = this.showingIssues = false;
    this.host.querySelector<HTMLDetailsElement>('.map-options')!.open = false;
  }
  private activate(graph: SystemMap, selected: string | null, lens: MapLens) {
    this.layoutAbort?.abort();
    this.graph = graph;
    this.layout =
      graph.view?.kind === 'hierarchy' ? layoutTypeHierarchy(graph) : layoutSystemMap(graph);
    this.selected = selected;
    this.lens = lens;
    this.expanded.clear();
    this.needsFit = true;
    this.resetRelationships();
    this.layoutState = 'ready';
    this.layoutFailure = '';
    if (graph.view?.kind !== 'hierarchy') this.arrange();
    this.publishSelection();
    this.schedule();
  }
  explore(id: string) {
    const key = this.keyFor(id);
    const e = key ? this.graph.nodes.get(key)!.element : this.model?.byId.get(id);
    if (!e || !this.model) return false;
    if (e.kind === 'package' || !this.model.members(e.id).length) return this.focus(id);
    if (this.graph.view?.kind === 'structure' && key === this.graph.roots[0]) return this.focus(id);
    const occurrence = key ? this.graph.nodes.get(key) : undefined;
    const graph = buildStructureMap(this.model, e.id, {
      contextName: occurrence?.contextName,
      inheritedFrom: occurrence?.inheritedFrom,
    });
    this.remember();
    this.activate(graph, e.id, 'connections');
    return true;
  }
  showHierarchy(id: string) {
    if (!this.model?.byId.get(id)?.definition) return;
    this.remember();
    this.activate(buildTypeHierarchy(this.model, id), id, 'types');
  }
  private goToDefinition(id: string) {
    if (!this.model?.byId.has(id)) return;
    if (isInlineValueDefinition(this.model, this.model.byId.get(id)!)) {
      this.showHierarchy(id);
      return;
    }
    if (!this.catalogue.nodes.has(id)) {
      this.onSelect(id);
      return;
    }
    this.remember();
    this.activate(
      this.model.members(id).length ? buildStructureMap(this.model, id) : this.catalogue,
      id,
      'connections',
    );
  }
  private zoom(
    factor: number,
    point = { x: this.stage.clientWidth / 2, y: this.stage.clientHeight / 2 },
  ) {
    this.camera = zoomCamera(this.camera, factor, point);
    this.needsFit = false;
    this.schedule();
  }
  private schedule() {
    if (!this.frame)
      this.frame = requestAnimationFrame(() => {
        this.frame = 0;
        this.draw();
      });
  }
  private draw() {
    const width = this.stage.clientWidth,
      height = this.stage.clientHeight;
    if (!width || !height) return;
    const context = this.host.querySelector<HTMLElement>('.map-context')!;
    context.hidden = !this.graph.view;
    context.textContent = this.graph.view
      ? `${this.graph.view.kind === 'structure' ? 'Structure' : 'Type hierarchy'} · ${this.graph.view.title}`
      : '';
    this.host.querySelector<HTMLSelectElement>('select')!.value = this.lens;
    this.host.querySelector<HTMLSelectElement>('select')!.disabled = !!this.graph.view;
    this.host.querySelector<HTMLElement>('.map-lens-select')!.hidden = !!this.graph.view;
    const layoutStatus = this.host.querySelector<HTMLElement>('.map-layout-status')!;
    const status =
      this.layoutState === 'loading'
        ? 'Arranging system map…'
        : this.layoutState === 'ready'
          ? ''
          : 'Layout unavailable. <button data-map-action="retry-layout">Retry</button>';
    layoutStatus.hidden = this.layoutState === 'ready';
    if (layoutStatus.innerHTML !== status) layoutStatus.innerHTML = status;
    layoutStatus.title = this.layoutFailure;
    if (this.needsFit) {
      this.camera = fitCamera(this.layout.bounds, width, height);
      this.needsFit = false;
    }
    const viewport = { x: 0, y: 0, width, height };
    const projection = projectSystemMap(
      this.graph,
      this.layout,
      this.camera,
      viewport,
      this.expanded,
    );
    this.expanded = projection.expanded;
    this.groups = groupMapRelations(this.graph, projection, this.lens);
    const focusId = (document.activeElement as HTMLElement)?.dataset?.mapId;
    const boxes = new Map<string, Rect>();
    const nodeLayer = this.host.querySelector('.map-nodes')!;
    const existing = new Map(
      [...nodeLayer.children].map((el) => [(el as HTMLElement).dataset.mapKey!, el as HTMLElement]),
    );
    // When inside a structure, the canvas itself is its enclosing region. Keep
    // the root in projection/history, but avoid a clipped, redundant outer box.
    const visible = projection.visible.filter(
      (id) =>
        !(
          this.graph.view?.kind === 'structure' &&
          this.graph.roots.includes(id) &&
          projection.expanded.has(id)
        ),
    );
    for (const [id, el] of existing) if (!visible.includes(id)) el.remove();
    visible.map((id) => {
      const node = this.graph.nodes.get(id)!,
        box = screenRect(this.layout.boxes.get(id)!, this.camera);
      boxes.set(id, box);
      const expanded = projection.expanded.has(id),
        e = node.element;
      const referenceLinks = (kind: 'typing' | 'redefinition') =>
        e.references
          .filter((r) => r.kind === kind)
          .map((r) => {
            const target = this.model ? referenceTarget(this.model, r) : undefined;
            return target
              ? `<button class="map-type-link" data-map-definition="${esc(target.id)}" title="Go to ${esc(target.qualifiedName)}" aria-label="Go to ${esc(target.name)} ${kind === 'typing' ? 'definition' : 'declaration'}">${esc(r.spelling)}</button>`
              : `<span class="map-unresolved" title="${esc(r.resolution.status)}">${esc(r.spelling)}</span>`;
          })
          .join(', ');
      const types = referenceLinks('typing');
      const typeLabel = types ? `<span class="map-node-type">: ${types}</span>` : '';
      const specialization =
        e.baseName && e.definition && this.graph.view?.kind !== 'hierarchy'
          ? `<button class="map-specializes" data-map-hierarchy="${esc(e.id)}" title="Show type hierarchy">Specializes ${esc(e.baseName)} ${icon('arrow')}</button>`
          : '';
      const redefines = referenceLinks('redefinition');
      const provenance = node.inheritedFrom
        ? `From ${node.inheritedFrom.name}`
        : redefines
          ? 'Redefined here'
          : '';
      const detail = box.height > 115 && box.width > 150;
      const valueLabel =
        box.height > 75 &&
        box.width > 150 &&
        e.semantic.metaclass === 'AttributeUsage' &&
        e.value !== undefined
          ? `<span class="map-node-value" title="Declared value: ${esc(e.value)}">= ${esc(e.value)}</span>`
          : '';
      const showProvenance = !!provenance && box.height > 75 && box.width > 100;
      const title = expanded
        ? `<div class="map-node-header"><button class="map-node-heading" data-map-id="${esc(id)}" title="${esc(node.contextName ?? e.qualifiedName)}">${esc(e.name)}</button>${typeLabel}${specialization}</div>`
        : `<div class="map-node-content"><div class="map-node-name"><button class="map-node-body" data-map-id="${esc(id)}" title="${esc(node.contextName ?? e.qualifiedName)}"><strong>${esc(e.name)}</strong></button>${typeLabel}${valueLabel}</div>${!showProvenance && box.height > 65 && box.width > 120 ? `<span class="map-node-kind">${esc(e.kind)}${e.definition ? ' definition' : ''}</span>` : ''}${specialization}${showProvenance ? `<small class="map-provenance" title="${esc(node.inheritedFrom?.qualifiedName ?? 'Redefines an inherited feature')}">${esc(provenance)}</small>` : ''}${detail && redefines ? `<span class="map-node-redefines">Redefines ${redefines}</span>` : ''}${detail && node.note ? `<small class="map-node-note">${esc(node.note)}</small>` : ''}${detail && this.model!.members(e.id).length && e.kind !== 'package' ? `<button class="map-explore" data-map-explore="${esc(id)}" aria-label="Explore ${esc(e.name)} structure">Explore structure ${icon('arrow')}</button>` : ''}</div>`;
      let el = existing.get(id);
      if (!el) {
        el = document.createElement('div');
        el.dataset.mapKey = id;
        nodeLayer.appendChild(el);
      }
      el.className = `map-node ${expanded ? 'expanded' : ''} ${this.selected === id ? 'selected' : ''}`;
      el.style.cssText = `left:${box.x}px;top:${box.y}px;width:${box.width}px;height:${box.height}px`;
      if (el.dataset.content !== title) {
        el.innerHTML = title;
        el.dataset.content = title;
      }
    });
    if (focusId)
      [...this.host.querySelectorAll<HTMLElement>('[data-map-id]')]
        .find((e) => e.dataset.mapId === focusId)
        ?.focus({ preventScroll: true });
    const visibleGroups = this.groups.filter((g) => boxes.has(g.sourceId) || boxes.has(g.targetId));
    const selectedRepresentative = this.selected
      ? projection.representatives.get(this.selected)
      : null;
    const related = (group: MapGroup) => {
      if (!selectedRepresentative) return false;
      // Selecting an expanded domain emphasizes relationships owned anywhere in it.
      const within = (id: string) => {
        let current: string | null = id;
        while (current) {
          if (current === selectedRepresentative) return true;
          current = this.graph.nodes.get(current)!.parentId;
        }
        return false;
      };
      return within(group.sourceId) || within(group.targetId);
    };
    const drawn = [...visibleGroups]
      .sort(
        (a, b) =>
          Number(b.id === this.activeGroup) - Number(a.id === this.activeGroup) ||
          Number(related(b)) - Number(related(a)) ||
          a.id.localeCompare(b.id),
      )
      .slice(0, 80);
    // Routing order is independent of selection, so emphasis does not move lines.
    drawn.sort((a, b) => a.id.localeCompare(b.id));
    const routingKey = JSON.stringify([
      this.camera,
      width,
      height,
      projection.visible,
      [...projection.expanded],
      drawn.map((g) => g.id),
    ]);
    if (this.routingKey !== routingKey) {
      this.routingKey = routingKey;
      this.routed.clear();
    }
    const clipped = (box: Rect): Rect => {
      const x = clamp(box.x, 16, width - 32),
        y = clamp(box.y, 16, height - 32);
      return {
        x,
        y,
        width: Math.max(1, Math.min(box.x + box.width, width - 16) - x),
        height: Math.max(1, Math.min(box.y + box.height, height - 16) - y),
      };
    };
    const attachment = (id: string) => {
      const box = screenRect(this.layout.boxes.get(id)!, this.camera);
      return clipped(projection.expanded.has(id) ? { ...box, height: 28 } : box);
    };
    const obstacles = [...boxes].map(([id]) => attachment(id));
    const routes: Point[][] = [],
      labels: Rect[] = [];
    const svg = this.host.querySelector<SVGSVGElement>('.map-edges')!;
    if (!svg.querySelector('defs'))
      svg.innerHTML = `<defs><marker id="map-arrow" markerWidth="7" markerHeight="7" refX="6" refY="3.5" orient="auto"><path class="arrow" d="M0,0 L7,3.5 L0,7Z"/></marker><marker id="map-specialization" markerUnits="userSpaceOnUse" markerWidth="15" markerHeight="15" refX="13" refY="7.5" orient="auto"><path class="specialization-arrow" d="M1,1 L13,7.5 L1,14Z"/></marker></defs>`;
    const oldEdges = new Map(
      [...svg.querySelectorAll<SVGGElement>('.map-edge')].map((el) => [el.dataset.mapGroup!, el]),
    );
    for (const [id, el] of oldEdges) if (!drawn.some((g) => g.id === id)) el.remove();
    const memberships = new Map<string, string[]>();
    for (const group of drawn)
      for (const id of new Set([group.sourceId, group.targetId])) {
        if (!memberships.has(id)) memberships.set(id, []);
        memberships.get(id)!.push(group.id);
      }
    const fraction = (id: string, group: string) => {
      const siblings = memberships.get(id)!;
      return siblings.length < 2
        ? 0.5
        : 0.3 + (0.4 * siblings.indexOf(group)) / (siblings.length - 1);
    };
    for (const group of drawn) {
      const specialization =
        group.kind === 'specialization' &&
        this.graph.nodes.get(group.sourceId)!.element.definition &&
        this.graph.nodes.get(group.targetId)!.element.definition &&
        group.records.every((r) => r.sourceId === group.sourceId && r.targetId === group.targetId);
      const a = attachment(group.sourceId),
        b = attachment(group.targetId);
      const outsideA = !boxes.has(group.sourceId),
        outsideB = !boxes.has(group.targetId);
      const outside = outsideA ? group.sourceId : outsideB ? group.targetId : null;
      const points =
        projectLayoutRoute(this.layout, group, projection, this.camera) ??
        this.routed.get(group.id) ??
        routeMapEdge(
          a,
          b,
          obstacles,
          viewport,
          routes,
          fraction(group.sourceId, group.id),
          group.sourceId === group.targetId ? 0.8 : fraction(group.targetId, group.id),
        );
      this.routed.set(group.id, points);
      routes.push(points);
      const text = outside
        ? `${this.graph.nodes.get(outside)!.element.name} · ${group.records.length}`
        : `${group.kind} · ${group.records.length}`;
      const labelText = text.length > 26 ? `${text.slice(0, 25)}…` : text;
      const labelWidth = Math.max(48, labelText.length * 6 + 16);
      const labelSegments = visibleRouteSegments(points, viewport);
      const placed =
        labelSegments
          .map((segment) =>
            placeMapLabel(segment, labelWidth, 24, [...obstacles, ...labels], viewport),
          )
          .find((label) => label !== null) ?? null;
      const persistent =
        (this.showLabels || !!outside || group.id === this.activeGroup) && !!placed;
      if (persistent) labels.push(placed!);
      const label = placed ?? fallbackMapLabel(labelSegments[0] ?? points, labelWidth, viewport);
      const path = routePath(points);
      const name = `${group.kind}: ${this.graph.nodes.get(group.sourceId)!.element.name} to ${this.graph.nodes.get(group.targetId)!.element.name}, ${group.records.length} relationships`;
      let el = oldEdges.get(group.id);
      if (!el) {
        el = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        el.dataset.mapGroup = group.id;
        el.setAttribute('tabindex', '0');
        el.setAttribute('role', 'button');
        svg.appendChild(el);
      }
      el.setAttribute('aria-label', name);
      el.setAttribute(
        'class',
        `map-edge ${related(group) ? 'related' : ''} ${selectedRepresentative && !related(group) ? 'muted' : ''} ${group.id === this.activeGroup ? 'selected' : ''} ${persistent ? 'labeled' : ''}`,
      );
      // Keep the focusable group itself stable across redraws.
      el.innerHTML = `<path class="map-edge-line ${specialization ? 'map-specialization-edge' : group.directed ? 'map-type-edge' : ''}" d="${path}" ${group.directed ? `marker-end="url(#${specialization ? 'map-specialization' : 'map-arrow'})"` : ''}/><path class="map-edge-hit" d="${path}"/><g class="map-edge-label ${outside ? 'portal' : ''}" aria-hidden="true"><rect x="${label.x}" y="${label.y}" width="${label.width}" height="24" rx="12"/><text x="${label.x + label.width / 2}" y="${label.y + 16}">${esc(labelText)}</text></g>`;
    }
    this.host.querySelector('output')!.textContent = `${Math.round(this.camera.scale * 100)}%`;
    const lensIssues = this.graph.issues.filter(
      (i) => (i.kind === 'connection') === (this.lens === 'connections'),
    );
    this.host.querySelector('.map-lens-name')!.textContent =
      this.graph.view?.kind === 'hierarchy'
        ? 'Specialization'
        : this.lens === 'types'
          ? 'Types'
          : 'Connections';
    const detailNotice = this.host.querySelector<HTMLElement>('.map-detail-notice')!;
    detailNotice.hidden = !projection.limited && visibleGroups.length <= 80;
    detailNotice.textContent =
      'Zoom in for more detail. Browse relationships to see links beyond the drawing limit.';
    (this.host.querySelector('[data-map-action="back"]') as HTMLButtonElement).disabled =
      !this.history.length;
    (this.host.querySelector('[data-map-action="focus"]') as HTMLButtonElement).disabled =
      !this.selected;
    const issuesButton = this.host.querySelector<HTMLButtonElement>('[data-map-action="issues"]')!;
    issuesButton.textContent = `Unmapped relationships (${lensIssues.length})`;
    issuesButton.hidden = lensIssues.length === 0;
    issuesButton.setAttribute('aria-expanded', String(this.showingIssues));
    this.host
      .querySelector('[data-map-action="relations"]')!
      .setAttribute('aria-expanded', String(this.showingRelationships));
    const tray = this.host.querySelector<HTMLElement>('.map-tray')!;
    tray.hidden = !this.showingRelationships && !this.showingIssues;
    if (!tray.hidden) {
      const records = this.showingIssues ? lensIssues : this.groups;
      this.page = Math.min(this.page, Math.max(0, Math.ceil(records.length / 40) - 1));
      let content: string;
      if (this.showingIssues)
        content = lensIssues
          .slice(this.page * 40, (this.page + 1) * 40)
          .map(
            (i) =>
              `<div><button data-map-id="${esc(i.ownerId)}">${esc(this.graph.nodes.get(i.ownerId)!.element.qualifiedName)}</button><span>${esc(i.kind)} → ${esc(i.target)} · ${esc(i.status)}</span></div>`,
          )
          .join('');
      else
        content = this.groups
          .slice(this.page * 40, (this.page + 1) * 40)
          .map(
            (g) =>
              `<button data-map-group="${esc(g.id)}">${esc(this.graph.nodes.get(g.sourceId)!.element.name)} ${g.directed ? '→' : '—'} ${esc(this.graph.nodes.get(g.targetId)!.element.name)} <small>${esc(g.kind)} · ${g.records.length}</small></button>`,
          )
          .join('');
      const trayContent = `<div class="map-tray-heading"><strong>${this.showingIssues ? 'Unmapped relationships' : 'Relationships at this detail level'}</strong><button class="icon-button" data-map-action="close-tray" aria-label="Close relationship browser" title="Close (Esc)">${icon('close')}</button></div><div class="map-tray-pagination"><span>${records.length} total</span><button data-map-action="previous" ${this.page === 0 ? 'disabled' : ''}>Previous</button><button data-map-action="next" ${(this.page + 1) * 40 >= records.length ? 'disabled' : ''}>Next</button></div>${content || '<p>No relationships in this category.</p>'}${this.lens === 'types' ? `<p>${this.graph.libraryReferences} standard-library type references omitted. Annotation bodies are available in the inspector and source.</p>` : ''}`;
      if (tray.innerHTML !== trayContent) tray.innerHTML = trayContent;
    }
  }
}
function buildSystemMapPlaceholder() {
  return {
    nodes: new Map<string, import('./system-map.ts').MapNode>(),
    roots: [] as string[],
    relations: [],
    issues: [],
    libraryReferences: 0,
  } as import('./system-map.ts').SystemMap;
}
