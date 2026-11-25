# Architecture and contracts

Status: kernel migration and initial qualification implemented. Normative target: SysML 2.0 and
KerML 1.0; bundled model library: the March 2026 release. This is not a claim of
complete language or graphical-notation conformance.

The [public contract review](kernel-contract.md) defines supported operations and
compatibility policy. The [executable matrix](conformance.md) records bounded
standards coverage; [qualification evidence](foundation-qualification.md) records results.
The [requirements inventory](requirements.md) indexes all specification clauses and
formal source units, with separate implementation, API and test evidence. Indexed
units and partially tested clauses are never promoted to full conformance.

## Dependency direction

```text
vendor/sysml-toolkit/sysmlv2-syntax    lexer, syntax AST, recovery, source spans
                  ↓
vendor/sysml-toolkit/sysmlv2-model     semantic graph, resolution, checks
                  ↓
crates/sysml-kernel                   sources, policy, analysis, query snapshots
          ↙                   ↘
crates/sysml-cli              crates/sysml-wasm
                                  ↓
                             src/kernel/worker + client
                                  ↓
                             src/viewer/model + diagram
                                  ↓
                             src/viewer/layout + src/main.ts

src/platform/files → supplies source documents
src-tauri          → packaging and native file-dialog capabilities
```

The desktop and browser run the same WASM kernel in a disposable worker. The CLI
uses its native Rust build. This avoids introducing a second desktop transport
and proves the core works outside a webview. A future native Tauri transport can
consume the same snapshot contract without moving language rules into the host.

## Ownership

- Upstream owns grammar, syntax construction, semantic relationships, resolution,
  effective-member queries, standard-library interpretation and validation rules.
- The kernel facade owns input limits, document identities, source coordinates,
  query-result lifetimes, capability reporting and the versioned wire contract.
- Viewer projection chooses visible nodes and which visible card represents a
  resolved endpoint chain. It never resolves a name or computes inheritance.
- Layout owns coordinates. UI owns selection, navigation, rendering and commands.
- Hosts own filesystem access, thread/worker creation, cancellation and timeouts.

The engine imports no browser, Tauri or application code and performs no runtime
filesystem or network access. Upstream offers filesystem APIs, but the adapter
uses only explicitly supplied source text. All libraries are bundled locally.

## Source, syntax and semantic model

The upstream lexer preserves trivia in token spans; its AST represents written
syntax. It does **not** supply a rowan-style lossless CST. The facade preserves
original text and exposes exact declaration/reference ranges, so viewing is
lossless. Structured source editing and incremental CST operations are deferred.

The semantic graph is separate from the AST. The public snapshot is a query result,
not an interchange representation or a replacement for the full SysML metamodel.
It includes written declarations plus referenced and inherited elements needed for
inspection. Library records are navigation stubs (`membersAvailable: false`), not
fully expanded library queries. User effective-member queries include implied
library heritage. The default diagram hides library members as a display policy.

The facade selects the upstream CanonicalV3 graph before loading sources and
requests its shared implied-relationship publication before validation and
projection. Checked arithmetic bindings require that publication. The facade does
not manufacture positional arguments or substitute compatibility evaluation when
a semantic family is incomplete. Graph format and snapshot schema are independent.

Graphical notation support is deliberately smaller than language support. Binary
connections are rendered; other arities remain in the snapshot and receive a
visible unsupported-view label. A resolved connection establishes endpoint lookup,
not proof that every semantic rule applicable to it has been validated.

## Snapshot contract v4

Rust types in `crates/sysml-kernel/src/types.rs` are authoritative. `ts-rs` generates
the checked-in TypeScript files. `npm run contract:check` detects drift.

- `revision` identifies a request; callers must use a fresh revision for changed
  sources. IDs are opaque and valid only within that snapshot. They are not
  persistent model identity and must not be persisted as bookmarks.
- Document IDs are separate from display names. Duplicate IDs and the reserved
  `library://` namespace are rejected. Equal basenames are permitted.
- Ranges are half-open. Byte and UTF-16 offsets are zero-based; line and UTF-16
  column numbers are one-based. Source text is not normalized.
- References report `resolved`, `unresolved`, `ambiguous` or `unsupported`.
  Unsupported is never silently converted into successful lookup.
- Relationship kinds distinguish typing, specialization, subsetting, redefinition,
  imports and references. Metaclass names follow the pinned upstream schema.
- Diagnostics retain stage, code, severity, message and source range. Named
  upstream rule codes pass through; unnamed diagnostics use stable stage codes.
- Library member-query stubs and incomplete inheritance are explicit. Validation
  coverage remains partial even when no diagnostics are reported.
- Membership records retain binding identity, aliases, names and declared visibility.
  Checked import closures add contextual visibility and explicit unknown reasons;
  recursive imports and name-collision exclusion stay in the language kernel.
  Cardinality distinguishes exact decimal-string intervals from unknown results;
  its limits and supported provider families are documented in the public contract.
- State subaction queries enumerate owned entry/do/exit bindings in declaration
  order with separate membership and action identities. Duplicate kinds remain
  visible; complete enumeration does not establish state validity or execution
  order. Syntax recovery in the state's document and unavailable library bodies
  produce explicit unknown results.
- Unresolved declared heritage makes dependent member queries incomplete, including
  transitive dependencies. The facade accounts for missing edges that the upstream
  graph's completeness flag does not represent.

`Snapshot::element`, `find` and `references_at` are convenience queries. The viewer
builds maps once per snapshot for interactive lookups. The native CLI separates
argument parsing, bounded file/directory discovery, kernel execution and output.
`sysml parse` uses syntax-only batch checking; `validate` and `inspect` consume the
same semantic analysis as the viewer. Validation reports have a separate CLI JSON
schema (v1); inspection preserves snapshot v4 and the `sysml-inspect` entry point.
`--request-json` exercises exactly the browser request contract. Filesystem access
and recursive discovery stay in the CLI; no language rules move into the host.
Native consumers can call `Snapshot::queries()` for indexed lookup, checked source
ranges and explicit unavailable-member/reference/diagnostic errors. `check_syntax`
parses and checks one source without invoking semantic analysis or loading libraries.
`check_syntax_batch` applies aggregate workspace limits and unique document-ID
validation before checking independent sources. It does not resolve cross-file names
or expose an AST. CLI tests exercise the actual binaries and their exit/output
contracts; a separate release workflow builds archives without the desktop stack.
Snapshot values own public transport fields; the query object borrows them
immutably. Treating the value as immutable is a consumer rule, not a property of
the public-field type itself.

## Reliability and scheduling

Analysis currently rebuilds the complete workspace. It does not claim incremental
performance. In particular, upstream's public ambiguity report builds a second
graph. This cost is recorded rather than classifying diagnostics by parsing prose.

Native callers must use a 16 MiB analysis-thread stack. The CLI and tests do so;
the WASM linker reserves the same stack. The WASM memory ceiling is 1 GiB.
Input limits are 32 documents, 5 MB each, 20 MB total, 500,000 lexical tokens and
20,000 written declarations. These are resource limits, not performance promises.

The native API checks cancellation between passes. It cannot interrupt an upstream
pass. The UI terminates the previous worker when a request is superseded, discards
stale revisions and gives analysis 60 seconds. A trap or failure discards the
worker; there is no fallback to the old parser. Existing displayed results survive
a failed load. Files are never written by analysis.

## Test and release gates

1. Vendor file inventory and content hashes match the recorded snapshot.
2. Rust formatting/lints and generated TypeScript contract checks pass.
3. Native acceptance cases pass for imports, standard libraries, inheritance,
   redefinition, valid/broken endpoints, ambiguity, recovery, Unicode ranges,
   handle integrity and revision lifetimes.
4. Native and WASM snapshots match for a shared multi-file request.
5. Viewer projection tests preserve the same semantic facts and the built-in
   example's four subsystem cards and five connections.
6. Contract and worker tests cover unavailable queries, source integrity,
   cancellation, stale replies, traps and deadlines. The standards matrix passes
   without unexplained failures or unexpected passes; its report is retained.
7. Web build and platform package build pass on each supported OS.

Windows and Linux packaging require those CI runners; a successful Mac build is
not evidence that the other packages work.

## Subsequent milestones

- Expand the initial executable matrix against official grammar and semantic
  rules, including pilot differential runs. Parsing the example corpus does not
  establish semantic conformance.
- Add continuous fuzzing and larger real-model performance benchmarks.
- Evaluate a prepared-library cache and incremental workspace queries based on
  measured use cases. Keep cache invalidation tied to engine and library versions.
- Extend the query facade with demand-loaded library exploration and interchange
  access when a second consumer requires them.
- Extend graphical notation independently of the semantic implementation.

## Application direction: model-centered development (proposed)

The application should become a project workspace connecting requirements, SysML
system design, authored documentation and implementation work. The intended
workflow makes implementation downstream of the model. This section records the
product direction under discussion; these capabilities are not implemented yet.

### Authority and change flow

The model captures the intended system structure, behavior, interfaces and design
contracts at the project's chosen level of abstraction. Authored documents supply
explanation, rationale, requirements text and supporting material. Each item needs
a clear authoritative source and revision; a linked document does not become an
independently maintained copy inside the application.

A meaningful design change can proceed from a requirement or finding to a reviewed
model change, associated implementation work, code and tests, and finally
verification evidence. Evidence feeds back into assessment of the design. Routine
implementation refactoring need not change the model when the modeled contracts
remain unchanged. The application should make the connection between an intended
change and its implementation visible.

A link to code establishes traceability. A linked verification case establishes
verification intent. A test result supplies evidence for particular revisions of
the model and implementation. These facts must remain distinguishable; none alone
establishes that a requirement has been satisfied.

### Application layers

Keep the existing reusable language kernel responsible for SysML meaning. Add a
project layer that consumes its typed queries and owns project roots, artifact
references, persistent navigation, baselines and cross-artifact relationships.
Presentation consumes that layer through views and commands. External providers
supply document content and issue data without introducing provider-specific
concepts into the language kernel.

Project metadata conventions can associate model elements with documentation,
implementation locations, decisions, findings and work items. Their interpretation
belongs in an explicit project profile. The kernel should expose generic typed
metadata and annotation relationships; the UI must not parse SysML excerpts or
infer semantics from annotation display names. An unresolved metadata type stays
unresolved even if its spelling resembles a known project convention.

Durable links need a model identity policy before persistent bookmarks, review
annotations or issue associations ship. The current kernel snapshot IDs are valid
only within one snapshot. The project layer must reconcile persistent anchors
across source revisions, renames and moves, and surface ambiguous or stale anchors
instead of silently binding them to another element.

An artifact reference should retain its provider, locator, optional revision and
section/symbol anchor. The relationship should retain its purpose, originating
model element and provenance. Resolved, unavailable, stale and ambiguous states
must be visible. A live issue status and evidence from a historical baseline must
not be presented as if they describe the same revision.

### Project entry points and requirements navigation

On first opening a project, propose an overview with its purpose, loaded model
scope, prominent access to requirements and authored project views, and actionable
loading or dependency problems. Returning users should resume their previous
context. A project can choose a different default view as its workflow develops.

Offer task-oriented navigation: Overview, Requirements, Design, Behavior,
Verification and Documents. Analysis and configuration views can appear when
relevant. Keep the package explorer and source available for structural navigation
without making users traverse packages to discover every requirement.

The Requirements view should support a readable outline and a table. Show authored
short names or identifiers when available, requirement text, subject, design
associations and verification associations. Do not invent persistent identifiers
from snapshot IDs. Keep reusable definitions distinguishable from their usages in
particular contexts, and distinguish nested requirements from package organization.

Selecting a requirement should show its text, assumptions, subject, child
requirements, modeled satisfaction assertions, verification cases and linked
documents. Following a relationship opens the relevant design context while
retaining the originating requirement and a navigable history. Selecting a design
element should provide the reverse path to related requirements. Show the actual
relationship kind and provenance; indirect paths must not look like direct links.

Keep modeled satisfaction assertions, verification plans and revision-specific
verification evidence separate. Positive and negative assertions need distinct
presentation. A missing link can be reported as a traceability gap under a stated
project policy; it is not automatically a failed requirement. An unresolved or
unsupported query must not be displayed as a confirmed absence of relationships.

SysML views, viewpoints and renderings provide a native foundation for authored
project presentations (language specification, section 7.26). Initial application
views remain product features; do not claim full standard view/rendering support
until the corresponding semantics and presentation behavior are implemented.

### Semantic zoom and relationships across domains

An initial System map is implemented in `src/viewer/system-map.ts` (pure graph,
layout, projection and camera operations) and `src/viewer/map-view.ts` (interaction
and rendering). It uses source-owned hierarchy, type/specialization references and
binary connections from the existing facade. The initial map has pan/zoom, bounded
progressive detail, grouped-edge provenance, off-screen target labels, search
navigation and camera history. Annotation bodies and library references are not
drawn; unresolved targets and unsupported connections are listed separately.
`semantic-views.ts` adds explicit structure and definition-hierarchy projections.
Structure consumes the facade's effective members and connections, including
inherited records and the kernel's redefinition result. Display occurrences have
path-scoped identities separate from their semantic element IDs; two uses of a
definition do not merge their ports or connections. Exact kernel endpoint chains
are followed within each display context. Missing contextual endpoints are
reported rather than matched by name or substituted by a frontend inheritance
algorithm. Projection stops at recursive members, depth seven, or 600 occurrences;
continuation and incomplete inheritance remain explicit. Source ownership stays
unchanged, and the inspector preserves contextual path and declaring type.

Type names are navigable card labels. Definition specialization opens a separate
hierarchy of explicit ancestors and descendants, ranked general-to-specific;
multiple inheritance shares ancestor nodes. Cyclic or cycle-dependent nodes are
marked instead of silently imposing a valid ancestry. Hollow triangular arrows
are used only for exact definition-to-definition specialization, never for usage
typing or aggregated package references. Navigation history retains the view,
graph, geometry and camera. The map defaults to connections; the source-owned
catalogue retains the optional type-reference lens.

The catalogue omits complete leaf `AttributeDefinition` records with no visible
owned or effective members. Their types remain clickable labels on property
cards, and declared attribute values appear at detail zoom. References to these
intentionally omitted types do not become edges or unmapped warnings. This is a
presentation policy, independent of reference count or spelling; it does not
classify primitives or hide structural leaves such as parts and ports. Structured
or incomplete attribute definitions stay on the map. The explorer retains all
declarations, and explicitly opening an omitted value definition displays its
type hierarchy. The semantic snapshot and explicit hierarchy are never filtered
by this catalogue policy.

The map occupies
all workspace height below the compact app header. View tabs, navigation and zoom
float over the canvas; relationship settings and the relationship browser are
on-demand overlays. Successful layout has no visible engine label or status bar.
Only pending or failed layout shows a temporary message; diagnostics appear as a
header badge when findings exist. View tabs support arrow-key navigation, and
Escape dismisses relationship overlays without moving the camera.
The map draws at
most 300 nodes and 80 relationship groups; paginated relationship inspection
retains groups beyond that drawing limit.

`layout-engine.ts` defines a geometry-only asynchronous boundary: IDs, ownership,
node dimensions and directed edges go in; boxes, orthogonal routes and route
endpoint identities come out. SysML parsing and resolution remain in the kernel.
`tala-layout.ts` adapts the exact `@d2lang/d2@0.1.34` npm distribution (MPL-2.0).
Generated aliases keep user labels and source text out of the D2 program. The
adapter validates identities, containment and finite dimensions before publishing
geometry. Diagonal or curved routes are left to the existing detail router while
valid node placement is retained; only orthogonal engine routes enter projection
trimming. Missing relationships or invalid coordinates still fail validation.
D2 loads lazily and runs TALA locally in its
own WebAssembly worker; model data is not sent to a server. Its compressed WASM
payload adds approximately 11.5 MB to the frontend distribution as a separate
JavaScript chunk.

`region-layout.ts` arranges immediate children bottom-up. Relationships are
lifted to their nearest common ownership region, retaining every original record;
duplicate endpoint pairs share geometry. Linked regions use TALA with a fixed
seed. Regions without links use deterministic packing because there are no
crossings to optimize. Containers have a consistent overview footprint; inner
geometry is fitted into their content area and becomes readable with zoom.
Each solver invocation normalizes dimensions to stay
within TALA's coordinate limits, then restores world coordinates. Layout is
computed once per snapshot, independent of zoom, lens selection and labels.
This bounds individual optimization problems by region size; it does not guarantee
that a single very dense region will complete. A 45-second deadline, cancellation
on replacement/unload, and stale-result guards protect the usable scene. Input
limits are 10,000 nodes and 30,000 original relationships. Failure leaves the
provisional map visible with a retry action; it is not represented as TALA success.

`layout-projection.ts` clips cached routes at collapsed region boundaries and
preserves the original route endpoints. When zoom reveals members inside a routed
region, that aggregate route is not misrepresented as a member-level connection:
`map-routing.ts` supplies the detailed route. Ancestor/member links and collapsed
self-links also use this router. Full TALA routing through expanded region
boundaries remains future work. The existing `map-order.ts` and grid layout remain
the provisional/failure placement. Their bounded heuristics cannot guarantee
crossing-free or obstacle-free geometry in every dense view.

Relationship labels appear on hover, keyboard focus or selection; optional
persistent labels must fit without covering cards or other labels. Label candidates
come from separately clipped visible segments so off-screen excursions do not
invent connecting lines. Element names and the complete relationship list remain
accessible. The D2 license and bundled notices ship in `public/licenses`; the
package lock pins the distribution integrity. Dependency upgrades must rerun the
TALA contract tests, representative large-region checks and browser/CSP checks.
Persistent layouts, project-defined
domains, requirement overlays and a minimap remain subsequent work.

Propose a continuous, pannable system map whose representations change with the
available screen space. Domain summaries reveal important definitions and usages,
then members, ports and specific relationships as users approach. The detail level
should depend on projected size, focus and the chosen task, with explicit expand
and collapse controls as alternatives to gestures. Requirements navigation and
search should locate elements in this same map and retain the navigation path.

The loaded project illustrates the current limitation: the root view shows 27
packages and zero explicit connections, while CatalogDatabase::mission has a
resolved type reference to Tenancy::Mission. CatalogRevision::layer and
CatalogRevision::missionModelRevision refer to MissionModel::Layer and
MissionModel::MissionModelRevision. The current scope-based diagram marks these
targets as outside the view. This is a presentation limitation; the zero count
does not establish that the domains are unrelated.

Use top-level packages as initial grouping candidates, with explicit project
configuration for domain boundaries. Package ownership, part composition and
architectural abstraction are different relationships. An element may appear in
several views; placement must not create ownership or semantic relationships.

Construct a presentation graph from kernel-resolved relationships across the
loaded project. At a coarse level, group edges by visible source ancestor, visible
target ancestor, relationship kind and direction. Every grouped edge retains its
underlying records and source locations; selecting it reveals those records.
Unresolved and ambiguous targets remain explicit and do not create invented domain
links. Distinguish imports, type references, connections, flows and requirement
relationships; a type reference must not look like runtime data flow. Do not hide
additional endpoints of n-ary connections in a binary projection.

Zooming should preserve the spatial arrangement of existing elements. Reveal
details inside stable regions, retain related neighboring domains or labeled
boundary links, and use separate expansion/collapse thresholds to avoid flicker.
Provide back/forward navigation, breadcrumbs, fit-to-selection and an overview
locator. Allow focused detail in one region while other regions stay summarized;
zooming need not expand every domain simultaneously.

Keep camera state, layout, detail policies and relationship grouping in the
presentation layer. Keep semantic resolution and requirement relationship queries
in the kernel facade, and durable identity and saved views in the project layer.
Cull off-screen elements and prepare bounded detail projections; drawing every
element at every frame is not the design. Choose a renderer after measuring real
project interaction, without making its API part of the language contract.

The first map milestone should show cross-package relationships, inspectable edge
groups and stable drill-down on the loaded project. Add continuous zoom and detail
transitions to that verified graph. Acceptance checks should cover preservation of
edge meaning across zoom levels, relationship provenance, unresolved targets,
selection and camera stability, and bounded rendering cost. The interactive sketch
in the conversation illustrates three observed type references; it is not a full
project analysis or an implemented canvas in the application.

### First complete application workflow

Start with a requirement in the real project and its design and document context:

1. Open a project with declared model and document roots and a repeatable dependency
   configuration. Present the overview and a requirements index.
2. Select a requirement and read its details; follow its subject or a modeled
   satisfaction assertion into the relevant design, retaining the navigation path.
3. Inspect related verification cases and follow a document reference beside the
   model, retaining its path, revision and section context. Follow links back to
   the model without losing the current selection.
4. Show missing files, unresolved relationships or annotation types and stale anchors explicitly.
   Organize diagnostics so a missing dependency can be investigated without
   treating every downstream finding as an independent design defect.
5. Reload a changed source or document and preserve valid links and navigation;
   make invalidated references apparent.

Choose the first document adapter from the user's actual project. Repository
Markdown, local PDFs and remotely hosted documents have different rendering,
identity and access needs; support should be explicit per adapter. The first
workflow can use external authoring while the application provides reading and
navigation. Model and document editing are separate later capabilities.

Issue integration remains a product decision. A first useful boundary is a linked
work item showing its title, provider, status and relationship to the model, with
navigation to the provider. Creating or editing GitHub/Jira issues inside the app
can follow if the project workflow benefits from it. Status synchronization and
conflict policy require explicit design before bidirectional editing.

After requirements-to-design navigation and document reading work on the real
project, extend them to linked implementation work, revision-aware verification
evidence and change-impact review. The initial diagram remains one view of the
project alongside these other ways of navigating the same model and its related
artifacts.
