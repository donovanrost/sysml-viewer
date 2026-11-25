import './style.css';
import './system-map.css';
import { icon } from './ui/icons.ts';
import example from '../examples/CoastalExplorer.sysml?raw';
import { ViewerModel, emptyModel, children, sourceOf, referenceTarget } from './viewer/model.ts';
import { diagramEdges } from './viewer/diagram.ts';
import { gridLayout } from './viewer/layout.ts';
import { SystemMapView } from './viewer/map-view.ts';
import type { MapGroup } from './viewer/system-map.ts';
import { KernelClient } from './kernel/client.ts';
import { desktop, readBrowserFiles, openNativeFiles } from './platform/files.ts';
import type { ModelElement, SourceDocument, Reference } from './viewer/model.ts';

const $ = <T extends HTMLElement = HTMLElement>(selector: string) =>
  document.querySelector<T>(selector)!;
const esc = (value: unknown) =>
  String(value ?? '').replace(
    /[&<>"']/g,
    (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!,
  );
function kindIcon(e: ModelElement) {
  return e.kind === 'package'
    ? 'folder'
    : e.kind === 'port'
      ? 'port'
      : e.kind === 'attribute'
        ? 'attribute'
        : e.kind === 'requirement'
          ? 'requirement'
          : 'cube';
}
function tone(e: ModelElement) {
  return ['package', 'port', 'attribute', 'requirement'].includes(e.kind) ? e.kind : 'part';
}
function label(e: ModelElement) {
  return e.kind + (e.definition ? ' def' : '');
}

const kernel = new KernelClient();
let model = emptyModel();
let scopeId: string | null = null;
let selectedId: string | null = scopeId;
let mode: 'map' | 'diagram' | 'source' = 'map';
let mapGroup: MapGroup | null = null;
let mapGroupPage = 0;
let edgeMode: 'connections' | 'types' = 'connections';
let sourceId = 'example';
let isExample = true,
  zoom = 0.9,
  query = '',
  revision = 0,
  loading = false;
let nativePaths: string[] = [];
const collapsed = new Set<string>();
const selected = () => model.byId.get(selectedId ?? '');
const scope = () => model.byId.get(scopeId ?? '');
const members = (e: ModelElement) => model.members(e.id);

$('#app').innerHTML = `
  <header class="app-header">
    <div class="brand"><div class="brand-mark">${icon('cube')}</div><strong>SysML Viewer</strong></div>
    <div class="header-actions"><span id="status-text" class="loading-status" role="status" hidden></span><button data-action="diagnostics" id="diagnostics-button" class="diagnostics-badge" hidden></button><button class="icon-button" data-action="theme" aria-label="Toggle light and dark theme" title="Toggle theme">${icon('sun')}</button><button class="icon-button" data-action="inspector" aria-label="Toggle inspector" title="Toggle inspector">${icon('panel')}</button><button class="primary" data-action="open">${icon('open')} Open models <kbd>${navigator.platform.includes('Mac') ? '⌘' : 'Ctrl+'}O</kbd></button></div>
  </header>
  <div class="app-body">
    <aside class="sidebar"><div class="sidebar-title"><span>MODEL EXPLORER</span><span id="element-count"></span></div>
      <label class="search">${icon('search')}<input id="search" type="search" placeholder="Find an element…" aria-label="Find an element" autocomplete="off"><kbd>/</kbd></label>
      <div id="tree" class="tree" aria-label="Model elements"></div>
      <div class="sidebar-bottom"><div class="privacy">${icon('lock')} Files stay on your device</div><button data-action="example">Explore the example ${icon('arrow')}</button></div>
    </aside>
    <main class="workspace" data-mode="map">
      <nav id="breadcrumbs" class="breadcrumbs" aria-label="Model location"></nav>
      <section class="view-heading"><h1 id="scope-title"></h1>
        <div class="segmented" role="tablist" aria-label="View mode"><button data-action="map" id="map-tab" role="tab" aria-controls="system-map">${icon('grid')} System map</button><button data-action="diagram" id="diagram-tab" role="tab" aria-controls="canvas">${icon('grid')} Diagram</button><button data-action="source" id="source-tab" role="tab" aria-controls="source-pane">${icon('source')} Source</button></div>
      </section>
      <div class="view-tools"><div class="relationship-tabs"><button data-action="connections" id="connections-tab">Connections</button><button data-action="types" id="types-tab">Types & specialization</button></div><button class="text-button" data-action="up" title="Go up one level">${icon('up')} Up a level</button></div>
      <div id="canvas" role="tabpanel" aria-labelledby="diagram-tab" class="canvas"></div>
      <div id="system-map" role="tabpanel" aria-labelledby="map-tab" class="system-map" hidden></div>
      <div id="source-pane" role="tabpanel" aria-labelledby="source-tab" class="source-pane" hidden></div>
    </main>
    <aside id="inspector" class="inspector"></aside>
  </div>
  <input id="file-input" type="file" accept=".sysml,.kerml" multiple hidden>
  <dialog id="diagnostics"><div class="dialog-heading"><h2>Model diagnostics</h2><button class="icon-button" data-action="close-dialog" aria-label="Close diagnostics">${icon('close')}</button></div><p class="limitations">Analysis uses the pinned SysML 2.0 / KerML 1.0 library. Imports, inheritance, redefinition and endpoint resolution come from the language engine. Validation coverage is partial. Diagrams hide standard-library members and show resolved binary connections between visible owner cards.</p><div id="diagnostic-list"></div><div class="dialog-footer"><button class="primary" data-action="close-dialog">Done</button></div></dialog>
  <div id="toast" class="toast" role="status" hidden></div>
  <div id="drop-overlay" class="drop-overlay" hidden><div>${icon('open')}<h2>Open your SysML models</h2><p>Drop .sysml and .kerml files together</p></div></div>`;

const systemMap = new SystemMapView(
  $('#system-map'),
  (id) => {
    selectedId = id;
    mapGroup = null;
    const e = selected();
    scopeId = e?.parentId ?? null;
    sourceId = e?.documentId || sourceId;
    render();
  },
  (group) => {
    mapGroup = group;
    mapGroupPage = 0;
    renderInspector();
  },
);

function row(e: ModelElement, depth: number): string {
  const kids = children(model, e.id),
    open = !collapsed.has(e.id);
  return `<div class="tree-row ${selectedId === e.id ? 'selected' : ''}" style="--depth:${Math.min(depth, 8)}">
    <button class="disclosure ${open ? 'expanded' : ''}" data-action="collapse" data-id="${esc(e.id)}" ${kids.length ? '' : 'disabled'} aria-label="${open ? 'Collapse' : 'Expand'} ${esc(e.name)}" aria-expanded="${open}">${kids.length ? icon('chevron') : ''}</button>
    <button class="tree-element ${tone(e)}" data-action="select-tree" data-id="${esc(e.id)}" title="${esc(e.qualifiedName)}">${icon(kindIcon(e))}<span>${esc(e.name)}</span>${e.definition ? '<small>D</small>' : ''}</button>
  </div>${open && depth < 30 ? kids.map((c) => row(c, depth + 1)).join('') : ''}`;
}
function renderTree() {
  $('#element-count').textContent = String(model.elements.length);
  if (query) {
    const matches = model.elements.filter((e) =>
      e.qualifiedName.toLowerCase().includes(query.toLowerCase()),
    );
    $('#tree').innerHTML = matches.length
      ? matches
          .slice(0, 300)
          .map(
            (e) =>
              `<button class="search-result ${tone(e)}" data-action="select-tree" data-id="${esc(e.id)}">${icon(kindIcon(e))}<span>${esc(e.name)}<small>${esc(e.qualifiedName)}</small></span></button>`,
          )
          .join('') +
        (matches.length > 300
          ? '<p class="empty-hint">First 300 results. Refine your search.</p>'
          : '')
      : '<p class="empty-hint">No matching elements</p>';
  } else
    $('#tree').innerHTML = children(model, null)
      .map((e) => row(e, 0))
      .join('');
}
function render() {
  const scrollPositions = ['#tree', '#canvas', '#source-pane'].map((s) => ({
    s,
    top: $(s).scrollTop,
    left: $(s).scrollLeft,
  }));
  renderTree();
  const path: ModelElement[] = [];
  let current = scope();
  while (current) {
    path.unshift(current);
    current = model.byId.get(current!.parentId ?? '');
  }
  $('#breadcrumbs').innerHTML =
    `<button data-action="root" title="All top-level elements" aria-label="Model root">${icon('grid')}</button>${path.map((e) => `${icon('chevron')}<button data-action="scope" data-id="${esc(e.id)}">${esc(e.name)}</button>`).join('')}<span class="model-badge">${isExample ? 'EXAMPLE MODEL' : `${model.documents.filter((d) => !d.id.startsWith('library://')).length} LOCAL FILE${model.documents.filter((d) => !d.id.startsWith('library://')).length === 1 ? '' : 'S'}`}</span>`;
  $('#scope-title').textContent =
    mode === 'map' ? 'System map' : (scope()?.name ?? 'Model overview');
  const nodes = model.members(scopeId);
  $('.workspace').dataset.mode = mode;
  $('#breadcrumbs').hidden = mode === 'map';
  $('#scope-title').hidden = mode === 'map';
  for (const view of ['map', 'diagram', 'source']) {
    $(`#${view}-tab`).setAttribute('aria-selected', String(mode === view));
    $(`#${view}-tab`).tabIndex = mode === view ? 0 : -1;
  }
  $('#map-tab').classList.toggle('active', mode === 'map');
  $('#diagram-tab').classList.toggle('active', mode === 'diagram');
  $('#source-tab').classList.toggle('active', mode === 'source');
  $('#connections-tab').classList.toggle('active', edgeMode === 'connections');
  $('#types-tab').classList.toggle('active', edgeMode === 'types');
  $('#canvas').hidden = mode !== 'diagram';
  $('#system-map').hidden = mode !== 'map';
  $('#source-pane').hidden = mode !== 'source';
  $('.view-tools').hidden = mode !== 'diagram';
  $('#status-text').hidden = !loading;
  $('#status-text').textContent = loading ? 'Reading model…' : '';
  const count = model.diagnostics.length;
  $('#diagnostics-button').hidden = loading || count === 0;
  $('#diagnostics-button').innerHTML = `${icon('warning')} ${count}`;
  $('#diagnostics-button').setAttribute('aria-label', `${count} model diagnostics`);
  $('#diagnostics-button').title = `View ${count} model diagnostics`;
  if (mode === 'diagram') renderDiagram(nodes);
  else if (mode === 'source') renderSource();
  else systemMap.refresh();
  renderInspector();
  for (const p of scrollPositions) {
    $(p.s).scrollTop = p.top;
    $(p.s).scrollLeft = p.left;
  }
}

function renderDiagram(allNodes: ModelElement[]) {
  const nodes = allNodes.slice(0, 150);
  const edges = diagramEdges(model, scopeId, edgeMode, nodes);
  const { width, height, positions } = gridLayout(nodes.map((e) => e.id));
  const position = (e: ModelElement) => positions.get(e.id)!;
  const lines = edges
    .map((edge, i) => {
      if (!edge.a || !edge.b || !nodes.includes(edge.a) || !nodes.includes(edge.b)) return '';
      const a = position(edge.a),
        b = position(edge.b),
        offset = (i % 5) * 9 - 18;
      let d: string, lx: number, ly: number;
      if (edge.a.id === edge.b.id) {
        d = `M ${a.x + 240} ${a.y + 60} C ${a.x + 320} ${a.y + 30}, ${a.x + 320} ${a.y + 135}, ${a.x + 240} ${a.y + 115}`;
        lx = a.x + 300;
        ly = a.y + 88;
      } else if (a.x === b.x) {
        const down = b.y > a.y;
        const y1 = a.y + (down ? 175 : 0),
          y2 = b.y + (down ? 0 : 175);
        d = `M ${a.x + 120 + offset} ${y1} L ${b.x + 120 + offset} ${y2}`;
        lx = a.x + 120 + offset;
        ly = (y1 + y2) / 2;
      } else {
        const right = b.x > a.x,
          x1 = a.x + (right ? 240 : 0),
          x2 = b.x + (right ? 0 : 240);
        const y1 = a.y + 86 + offset,
          y2 = b.y + 86 + offset,
          mid = (x1 + x2) / 2 + offset;
        d = `M ${x1} ${y1} C ${mid} ${y1}, ${mid} ${y2}, ${x2} ${y2}`;
        lx = (x1 + x2) / 2;
        ly = (y1 + y2) / 2 + offset;
      }
      return `<g class="edge ${edgeMode === 'types' ? 'type-edge' : ''}"><path d="${d}"/><circle cx="${lx}" cy="${ly}" r="10"/><text x="${lx}" y="${ly + 3}">${i + 1}</text></g>`;
    })
    .join('');
  const cards = nodes
    .map((e) => {
      const p = position(e),
        childElements = members(e),
        own = children(model, e.id);
      return `<article class="node ${tone(e)} ${selectedId === e.id ? 'selected' : ''}" style="left:${p.x}px;top:${p.y}px" data-action="select-node" data-id="${esc(e.id)}" tabindex="0" role="button" aria-label="Inspect ${esc(label(e))} ${esc(e.name)}">
      <div class="node-tag">${icon(kindIcon(e))}<span>«${esc(label(e))}»</span>${e.definition ? '<small>DEF</small>' : ''}</div>
      <div class="node-title"><strong>${esc(e.name)}</strong><span>${esc(e.typeName ? ': ' + e.typeName : e.baseName ? ':> ' + e.baseName : e.kind === 'package' ? 'Namespace' : `${childElements.length} visible model members`)}</span></div>
      <div class="node-members">${
        childElements.length
          ? childElements
              .slice(0, 2)
              .map(
                (c) =>
                  `<div>${icon(kindIcon(c))}<span>${esc(c.name)}${c.typeName ? ' : ' + esc(c.typeName) : ''}</span></div>`,
              )
              .join('')
          : `<p>${esc(e.value ?? (e.documentation || 'No explicit members'))}</p>`
      }</div>
      <div class="node-footer"><span>${childElements.length > 2 ? `+${childElements.length - 2} more` : !own.length && childElements.length ? 'Inherited members' : ''}</span>${childElements.length ? `<button data-action="explore" data-id="${esc(e.id)}" aria-label="Explore ${esc(e.name)}">Explore ${icon('arrow')}</button>` : ''}</div>
    </article>`;
    })
    .join('');
  const legend = edges.length
    ? `<div class="edge-legend"><h3>${edgeMode === 'connections' ? 'EXPLICIT CONNECTIONS' : 'TYPE RELATIONSHIPS'}</h3>${edges.map((e, i) => `<div class="legend-row"><span class="edge-number">${i + 1}</span><code>${esc(e.source)}</code><span class="edge-kind">${esc(e.kind)}</span><code>${esc(e.target)}</code>${e.status !== 'resolved' ? `<span class="outside">${esc(e.status)}</span>` : ''}</div>`).join('')}<p>${edgeMode === 'connections' ? 'Lines follow resolved endpoint chains to visible owner cards. Standard-library members are hidden.' : 'Dashed lines show explicit references between visible declarations.'}</p></div>`
    : '';
  $('#canvas').innerHTML = nodes.length
    ? `<div class="diagram-scroll"><div class="scaled-space" style="width:${width * zoom}px;min-height:${(height + (edges.length ? edges.length * 28 + 110 : 0) + 50) * zoom}px"><div class="diagram-content" style="width:${width}px;transform:scale(${zoom})"><div class="graph" style="width:${width}px;height:${height}px"><svg class="edges" width="${width}" height="${height}" aria-hidden="true">${lines}</svg>${cards}</div>${legend}${allNodes.length > 150 ? '<p class="graph-hint">Showing 150 elements. Use the tree to explore the rest.</p>' : ''}<p class="graph-hint">Select to inspect <span>·</span> Double-click to explore <span>·</span> Scroll to pan</p></div></div></div><div class="zoom-tools"><button data-action="zoom-out" aria-label="Zoom out">−</button><span>${Math.round(zoom * 100)}%</span><button data-action="zoom-in" aria-label="Zoom in">+</button><i></i><button data-action="fit" aria-label="Fit diagram width" title="Fit width">${icon('fit')}</button></div>`
    : `<div class="empty-state">${icon('grid')}<h2>No members at this level</h2><p>Go up a level or choose an element from the model tree.</p><button class="secondary" data-action="up">Up a level</button></div>`;
}

function renderInspector() {
  if (mode === 'map' && mapGroup) {
    const group = mapGroup;
    const records = group.records.slice(mapGroupPage * 50, (mapGroupPage + 1) * 50);
    $('#inspector').innerHTML =
      `<h2 class="inspector-eyebrow">RELATIONSHIPS</h2><h2 class="inspector-name">${esc(systemMap.elementFor(group.sourceId)?.name)} ${group.directed ? '→' : '—'} ${esc(systemMap.elementFor(group.targetId)?.name)}</h2><p>${esc(group.kind)} · ${group.records.length} source records</p>${records
        .map((r) => {
          const location =
            r.reference?.source ?? model.byId.get(r.connectionId ?? '')?.semantic.source;
          return `<div class="map-relation-record"><button data-action="inspect-reference" data-id="${esc(systemMap.elementFor(r.sourceId)?.id)}">${esc(r.sourceLabel)}</button><p>${esc(r.kind)}</p><button data-action="inspect-reference" data-id="${esc(systemMap.elementFor(r.targetId)?.id)}">${esc(r.targetLabel)}</button>${location ? `<p>${esc(model.documentsById.get(location.documentId)?.name)} · line ${location.start.line}</p><button data-action="diagnostic-source" data-doc="${esc(location.documentId)}" data-line="${location.start.line}">Reveal relationship source</button>` : ''}</div>`;
        })
        .join(
          '',
        )}<div class="map-navigation"><button data-action="map-group-previous" ${mapGroupPage === 0 ? 'disabled' : ''}>Previous</button><span>${mapGroupPage * 50 + 1}–${mapGroupPage * 50 + records.length}</span><button data-action="map-group-next" ${(mapGroupPage + 1) * 50 >= group.records.length ? 'disabled' : ''}>Next</button></div>`;
    return;
  }
  const e = selected();
  const field = (name: string, value: string) =>
    `<div class="field"><label>${name}</label><div>${esc(value)}</div></div>`;
  const reference = (r: Reference) => {
    const target = referenceTarget(model, r);
    const names: Record<string, string> = {
      typing: 'Type',
      specialization: 'Specializes',
      subsetting: 'Subsets',
      redefinition: 'Redefines',
    };
    const name = names[r.kind] ?? r.kind;
    return `${field(name, r.spelling)}${target ? `<button class="text-link" data-action="inspect-reference" data-id="${esc(target.id)}">Go to ${esc(target.name)} ${icon('arrow')}</button>` : `<span class="unresolved">${esc(r.resolution.status)}</span>`}`;
  };
  const context = mode === 'map' ? systemMap.selectionContext() : undefined;
  const provenance =
    context && context.element.id === e?.id
      ? `${field('In this structure', context!.contextName!)}${context!.inheritedFrom ? field('Inherited from', context!.inheritedFrom.qualifiedName) : ''}${context!.note ? `<p class="unresolved">${esc(context!.note)}</p>` : ''}`
      : '';
  $('#inspector').innerHTML = `<h2 class="inspector-eyebrow">INSPECTOR</h2>${
    e
      ? `<div class="inspector-icon ${tone(e)}">${icon(kindIcon(e))}</div><h2 class="inspector-name">${esc(e.name)}</h2><div class="element-kind ${tone(e)}">«${esc(label(e))}»</div>${e.documentation ? `<p class="documentation">${esc(e.documentation)}</p>` : ''}<hr>${provenance}${field('Qualified name', e.qualifiedName)}${e.definition ? `<button class="text-link" data-action="type-hierarchy" data-id="${esc(e.id)}">Show type hierarchy ${icon('arrow')}</button>` : ''}${e.references
          .filter((r) =>
            ['typing', 'specialization', 'subsetting', 'redefinition'].includes(r.kind),
          )
          .map(reference)
          .join(
            '',
          )}${e.multiplicity ? field('Multiplicity', e.multiplicity) : ''}${e.value ? field('Value expression', e.value) : ''}${members(e).length ? `<button class="explore-button" data-action="explore" data-id="${esc(e.id)}">Explore ${members(e).length} members ${icon('arrow')}</button>` : ''}<hr>${field('Source', `${model.documents.find((d) => d.id === e.documentId)?.name} · line ${e.line}`)}<button class="text-link" data-action="source">${icon('source')} Reveal in source</button><pre class="source-excerpt">${esc(sourceOf(model, e).split('\n').slice(0, 18).join('\n'))}</pre>`
      : `<div class="inspector-empty">${icon('cube')}<h3>Select an element</h3><p>Choose a diagram card or a model tree item to inspect its declaration.</p></div>`
  }`;
}
function highlight(text: string) {
  const expression =
    /\/\*[\s\S]*?\*\/|\/\/[^\n]*|"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|\b(?:package|part|def|port|attribute|item|requirement|action|state|constraint|enum|interface|connection|connect|to|import|private|public|doc|ref|in|out|inout)\b/g;
  let output = '',
    last = 0;
  for (const match of text.matchAll(expression)) {
    output += esc(text.slice(last, match.index));
    const cls = match[0].startsWith('/')
      ? 'comment'
      : /^["']/.test(match[0])
        ? 'string'
        : 'keyword';
    output += `<span class="syntax-${cls}">${esc(match[0])}</span>`;
    last = match.index! + match[0].length;
  }
  return output + esc(text.slice(last));
}
function renderSource() {
  const doc = model.documents.find((d) => d.id === sourceId) ?? model.documents[0];
  if (!doc) {
    $('#source-pane').innerHTML = '<p class="empty-hint">No source loaded.</p>';
    return;
  }
  const e = selected(),
    target = e?.documentId === doc.id ? e : undefined;
  // Token coloring is display-only. Source is escaped before HTML insertion.
  const lines = doc.text.split('\n');
  const large = lines.length > 20000;
  $('#source-pane').innerHTML =
    `<div class="source-toolbar"><select id="source-file" aria-label="Source file">${model.documents.map((d) => `<option value="${esc(d.id)}" ${d.id === doc.id ? 'selected' : ''}>${esc(d.name)}</option>`).join('')}</select><span>READ ONLY · UTF-8</span><button class="text-button" data-action="reveal-line">Jump to selection</button></div>${large ? `<p class="large-source-note">Large source: syntax coloring and line highlighting are disabled. Use your text editor for line navigation.</p><pre class="raw-source">${esc(doc.text)}</pre>` : `<div class="source-code">${lines.map((line, i) => `<div class="source-line ${target && i + 1 >= target.line && i + 1 <= target.endLine ? 'highlighted' : ''}" id="line-${i + 1}"><span class="line-number">${i + 1}</span><code>${highlight(line) || ' '}</code></div>`).join('')}</div>`}`;
}
function diagnostics() {
  $('#diagnostic-list').innerHTML = model.diagnostics.length
    ? model.diagnostics
        .map(
          (d) =>
            `<div class="diagnostic ${d.severity}">${icon('warning')}<div><p>${esc(d.message)}</p><small>${esc(model.documents.find((s) => s.id === d.documentId)?.name)} · line ${d.line}</small></div><button class="text-button" data-action="diagnostic-source" data-doc="${esc(d.documentId)}" data-line="${d.line}">Source ${icon('arrow')}</button></div>`,
        )
        .join('')
    : `<div class="no-diagnostics">${icon('info')}<h3>No analysis diagnostics</h3><p>The implemented checks reported no findings. This does not establish full language conformance.</p></div>`;
  $<HTMLDialogElement>('#diagnostics').showModal();
}
function notify(message: string) {
  $('#toast').textContent = message;
  $('#toast').hidden = false;
  window.setTimeout(() => {
    $('#toast').hidden = true;
  }, 6500);
}
function explore(e: ModelElement) {
  if (mode === 'map') {
    if (!systemMap.explore(e.id))
      notify('This annotation is available in the inspector, Diagram and Source.');
    return;
  }
  selectedId = e.id;
  if (members(e).length) scopeId = e.id;
  sourceId = e.documentId || sourceId;
  mode = 'diagram';
  render();
  resetCanvas();
}
function resetCanvas() {
  $('#canvas').scrollTo(0, 0);
}
function jumpToLine(line?: number) {
  document
    .getElementById(`line-${line ?? selected()?.line ?? 1}`)
    ?.scrollIntoView({ block: 'center' });
}
function install(parsed: ViewerModel, exampleModel: boolean) {
  model = parsed;
  mapGroup = null;
  systemMap.setModel(model);
  isExample = exampleModel;
  query = '';
  $<HTMLInputElement>('#search').value = '';
  collapsed.clear();
  scopeId = null;
  selectedId = null;
  sourceId = selected()?.documentId ?? model.documents[0]?.id ?? '';
  mode = 'map';
  systemMap.select(selectedId);
  loading = false;
  render();
  resetCanvas();
  if (!model.elements.length)
    notify('No supported declarations found. Your original file is available in Source.');
}
function beginLoad() {
  const current = ++revision;
  kernel.cancel();
  loading = true;
  render();
  return current;
}
async function loadDocuments(
  documents: SourceDocument[],
  paths: string[] = [],
  exampleModel = false,
  current = beginLoad(),
) {
  if (current !== revision) return;
  try {
    const snapshot = await kernel.analyze(documents, current);
    if (current !== revision) return;
    nativePaths = paths;
    install(new ViewerModel(snapshot), exampleModel);
  } catch (error) {
    if (current !== revision) return;
    loading = false;
    notify(`Could not analyze model: ${String(error)}`);
    render();
  }
}
async function loadBrowserFiles(files: File[]) {
  if (!files.length) return;
  const current = beginLoad();
  try {
    const selection = await readBrowserFiles(files);
    void loadDocuments(selection.documents, [], false, current);
  } catch (error) {
    if (current !== revision) return;
    loading = false;
    notify(String(error));
    render();
  }
}
async function openModels(reload = false) {
  if (!desktop) {
    $('#file-input').click();
    return;
  }
  const current = beginLoad();
  try {
    const selection = await openNativeFiles(nativePaths, reload);
    if (current !== revision) return;
    if (selection) void loadDocuments(selection.documents, selection.paths, false, current);
    else {
      loading = false;
      render();
    }
  } catch (error) {
    if (current !== revision) return;
    loading = false;
    notify(`Could not open models: ${String(error)}`);
    render();
  }
}

document.addEventListener('click', (event) => {
  const button = (event.target as HTMLElement).closest<HTMLElement>('[data-action]');
  if (!button || button.hasAttribute('disabled')) return;
  const action = button.dataset.action,
    e = model.byId.get(button.dataset.id ?? '');
  if (action === 'map-group-next' || action === 'map-group-previous') {
    mapGroupPage = Math.max(0, mapGroupPage + (action === 'map-group-next' ? 1 : -1));
    renderInspector();
    return;
  }
  if (action === 'open') {
    void openModels();
    return;
  }
  if (action === 'example') {
    void loadDocuments([{ id: 'example', name: 'CoastalExplorer.sysml', text: example }], [], true);
    return;
  }
  if (action === 'inspect-reference' && e) {
    mapGroup = null;
    selectedId = e.id;
    sourceId = e.documentId || sourceId;
    mode = e.library ? 'source' : mode === 'map' ? 'map' : 'diagram';
    if (!e.library) scopeId = e.parentId;
    render();
    if (mode === 'map') systemMap.select(e.id, true);
    if (mode === 'source') jumpToLine();
    return;
  }
  if (action === 'collapse' && e) {
    collapsed.has(e.id) ? collapsed.delete(e.id) : collapsed.add(e.id);
    renderTree();
    return;
  }
  if (action === 'select-tree' && e) {
    mapGroup = null;
    selectedId = e.id;
    sourceId = e.documentId;
    scopeId = e.parentId;
    render();
    if (mode === 'map') systemMap.select(e.id, true);
    if (mode === 'source') jumpToLine();
    return;
  }
  if (action === 'select-node' && e) {
    selectedId = e.id;
    sourceId = e.documentId;
    renderTree();
    renderInspector();
    document
      .querySelectorAll<HTMLElement>('.node')
      .forEach((node) => node.classList.toggle('selected', node.dataset.id === e.id));
    return;
  }
  if (action === 'explore' && e) {
    explore(e);
    return;
  }
  if (action === 'type-hierarchy' && e) {
    mode = 'map';
    systemMap.showHierarchy(e.id);
    render();
    return;
  }
  if (mode === 'map' && ['root', 'up', 'scope'].includes(action ?? '')) {
    if (action === 'root' || (action === 'up' && !scope())) systemMap.overview();
    else {
      const target = action === 'scope' ? e : scope();
      if (target) systemMap.focus(target.id);
    }
    return;
  }
  if (action === 'scope' && e) {
    scopeId = e.id;
    selectedId = e.id;
  }
  if (action === 'root') {
    scopeId = null;
    selectedId = null;
  }
  if (action === 'up') {
    const s = scope();
    if (s) {
      selectedId = s.id;
      scopeId = s.parentId;
    }
  }
  if (action === 'diagram') mode = 'diagram';
  if (action === 'map') {
    mode = 'map';
    systemMap.select(selectedId);
  }
  if (action === 'source') {
    mode = 'source';
    sourceId = selected()?.documentId ?? sourceId;
  }
  if (action === 'connections' || action === 'types') edgeMode = action;
  if (action === 'zoom-in') zoom = Math.min(1.8, zoom + 0.1);
  if (action === 'zoom-out') zoom = Math.max(0.35, zoom - 0.1);
  if (action === 'fit')
    zoom = Math.max(
      0.35,
      Math.min(
        1,
        ($('#canvas').clientWidth - 24) / (model.members(scopeId).length > 1 ? 730 : 340),
      ),
    );
  if (action === 'inspector') {
    $('.app-body').classList.toggle('hide-inspector');
    return;
  }
  if (action === 'theme') {
    document.documentElement.dataset.theme =
      document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
    return;
  }
  if (action === 'diagnostics') {
    diagnostics();
    return;
  }
  if (action === 'close-dialog') {
    $<HTMLDialogElement>('#diagnostics').close();
    return;
  }
  if (action === 'diagnostic-source') {
    sourceId = button.dataset.doc!;
    mode = 'source';
    $<HTMLDialogElement>('#diagnostics').close();
    render();
    jumpToLine(Number(button.dataset.line));
    return;
  }
  if (action === 'reveal-line') {
    jumpToLine();
    return;
  }
  render();
  if (action === 'source') jumpToLine();
  if (['root', 'up', 'scope'].includes(action!)) resetCanvas();
});
document.addEventListener('dblclick', (event) => {
  const node = (event.target as HTMLElement).closest<HTMLElement>('.node');
  const e = model.byId.get(node?.dataset.id ?? '');
  if (e) explore(e);
});
document.addEventListener('keydown', (event) => {
  const input = (event.target as HTMLElement).matches('input,textarea,select');
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'o') {
    event.preventDefault();
    void openModels();
  }
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'r' && desktop) {
    event.preventDefault();
    void openModels(true);
  }
  if (event.key === '/' && !input && !$<HTMLDialogElement>('#diagnostics').open) {
    event.preventDefault();
    $('#search').focus();
  }
  if (
    (event.key === 'Enter' || event.key === ' ') &&
    (event.target as HTMLElement).matches('.node')
  ) {
    event.preventDefault();
    (event.target as HTMLElement).click();
  }
});
$('.segmented').addEventListener('keydown', (event) => {
  const tabs = [...document.querySelectorAll<HTMLButtonElement>('.segmented [role="tab"]')];
  const index = tabs.indexOf(event.target as HTMLButtonElement);
  if (index < 0) return;
  const next =
    event.key === 'ArrowRight'
      ? (index + 1) % tabs.length
      : event.key === 'ArrowLeft'
        ? (index + tabs.length - 1) % tabs.length
        : event.key === 'Home'
          ? 0
          : event.key === 'End'
            ? tabs.length - 1
            : null;
  if (next === null) return;
  event.preventDefault();
  tabs[next].click();
  tabs[next].focus();
});
$('#search').addEventListener('input', () => {
  query = $<HTMLInputElement>('#search').value;
  renderTree();
});
$('#file-input').addEventListener('change', () => {
  const input = $<HTMLInputElement>('#file-input');
  void loadBrowserFiles(Array.from(input.files ?? []));
  input.value = '';
});
$('#source-pane').addEventListener('change', (event) => {
  if ((event.target as HTMLElement).id === 'source-file') {
    sourceId = (event.target as HTMLSelectElement).value;
    renderSource();
  }
});
let dragDepth = 0;
document.addEventListener('dragenter', (event) => {
  if (event.dataTransfer?.types.includes('Files')) {
    event.preventDefault();
    dragDepth++;
    $('#drop-overlay').hidden = false;
  }
});
document.addEventListener('dragover', (event) => event.preventDefault());
document.addEventListener('dragleave', () => {
  if (--dragDepth <= 0) {
    dragDepth = 0;
    $('#drop-overlay').hidden = true;
  }
});
document.addEventListener('drop', (event) => {
  event.preventDefault();
  dragDepth = 0;
  $('#drop-overlay').hidden = true;
  void loadBrowserFiles(Array.from(event.dataTransfer?.files ?? []));
});
render();
void loadDocuments([{ id: 'example', name: 'CoastalExplorer.sysml', text: example }], [], true);
