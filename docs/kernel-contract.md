# Kernel public contract review

Decision: retain the headless facade and versioned snapshot, add checked native
queries and a syntax-only operation. Keep the upstream AST/metamodel private to
the facade. Status: implemented, reviewed against the qualification suite.

The stable seam is source text → analysis → explicit query results. The current
facade is a **declaration projection**, not a complete metamodel API. In particular,
complete imported-membership enumeration, arbitrary scoped-name resolution, AST editing,
expression evaluation and simulation are not public operations. Consumers requiring
raw syntax can use the pinned syntax crate directly, accepting its upstream API;
we do not disguise that coupling by re-exporting its types from `sysml-kernel`.

## Operations and guarantees

| Operation | Result | Boundaries |
| --- | --- | --- |
| `check_syntax(document)` | Original source and syntax/structure diagnostics | Parses one document without libraries or linking. Recovered AST stays internal. Language errors are findings; invalid inputs return an error. |
| `Engine::analyze(request, cancelled)` | Owned `Snapshot` | Full workspace rebuild; only supplied documents and bundled libraries participate. No runtime filesystem/network access. |
| `snapshot.queries()` | Indexed immutable borrow or `QueryError` | Checks schema, unique IDs, referenced handles, source coordinates and endpoint target consistency. Does not prove semantic validity or certify an arbitrary imported model graph. |
| `query.element(id)` | One element or `UnknownElement` | IDs are opaque and valid only for this snapshot. |
| `query.find(qualified_name)` | Zero, one or several matches | Exact canonical-name lookup; does not enumerate aliases or resolve a name in a scope. Never selects an arbitrary first match. |
| `query.owned_members(id)` | Declaration records or explicit failure | Includes projected owned declarations. Does not promise every metamodel membership. A known empty scope returns an empty collection. |
| `query.effective_members(id)` | Records plus `inheritance_complete` | Retains library and inherited records. The viewer alone chooses which to display. Partial computation stays explicit. |
| `query.owned_memberships(id)` | Explicit owned binding records | Includes aliases with their own identity, long/short names, target status and visibility. |
| `query.inherited_memberships(id)` | Binding records plus `inheritance_complete` | Preserves the original declaring namespace; does not enumerate the full imported closure. |
| `query.membership(id)` | Binding or explicit failure | Membership identity is separate from member-element identity. |
| `query.cardinality(id)` | Evaluated interval, unknown reason, or not applicable | Integer bounds are decimal strings; a null upper bound means infinity only within an evaluated result. |
| `query.references_from/at(...)` | Written references with `Resolution` | User sources only. Library reference enumeration reports `ReferencesUnavailable`. Multiple source sites can overlap. |
| `query.diagnostics(document_id)` | Findings for a user document | Library diagnostics are not projected and report `DiagnosticsUnavailable`. Empty findings do not certify a model. |
| `query.source(range)` | Exact source slice or range error | Uses UTF-8 byte offsets. Sources can include referenced library files. |

The existing `Snapshot::element`, `find` and `references_at` helpers remain for
compatibility and trusted locally constructed values. The indexed API should be
used for repeated queries and deserialized values. `references_from/at` and
diagnostic filtering currently scan their arrays; indexed element/document/name
lookups avoid repeated whole-snapshot scans.

## Identity, ownership and lifetimes

- Callers provide stable document IDs and display filenames separately. Two files
  may have the same basename. The engine rejects duplicate IDs and reserves
  `library://` for bundled sources. Extension selects SysML or KerML dialect.
- Callers assign a fresh `u32` revision to each request in their session. Element
  IDs include that revision but are **not globally unique** across sessions or
  engine instances. Reusing a revision defeats stale-handle detection; no global
  counter or persistent bookmark identity is implied.
- Analysis takes ownership of request text. Snapshots own their data and survive
  later analyses or cancellation. Rust query objects borrow the snapshot, so Rust
  prevents mutation while those borrows are used.
- Snapshot fields remain public transport data. They are not intrinsically
  immutable: do not mutate and then trust old cached results. Rebuild/validate
  queries after deserialization or deliberate edits to the transport value.
- The engine owns no threads or event loop. Hosts own scheduling, stack allocation,
  shutdown and worker/process isolation. Native callers use the exported 16 MiB
  stack requirement. Native cancellation is cooperative between upstream passes;
  it cannot interrupt a pass. Browser cancellation terminates its disposable worker.

## Source and semantic interpretation

Analysis builds the upstream CanonicalV3 graph and requests shared implied
relationship publication before validation and projection. Checked operator
queries consume this published callee/input/argument evidence. Publication can
decline a family; its checked query still returns unknown. The graph format is
internal and distinct from snapshot schema 2; this is not an interchange API.

Ranges are half-open. Byte/UTF-16 offsets are zero-based; line and UTF-16 column
numbers are one-based. Original encoding after UTF-8 decoding and line endings
are preserved. Indexed query construction verifies coordinate agreement.
`SourceIndex::position` is a low-level helper that clamps to the preceding valid
character boundary (and EOF); checked source queries reject invalid boundaries.

A `Reference` describes a **written name reference**. Its resolved target is what
that name denotes. For conjugated port typing `~P`, the written spelling includes
`~`, while the name target is `P`; this is not a promise that it is the complete
semantic FeatureTyping target after implicit conjugated-definition construction.

`cardinality` uses the upstream checked constraint reader. Evaluated integer
bounds remain exact through JSON and JavaScript, up to the engine's signed 128-bit
integer limit. Unknown providers, unsupported bounds, cycles and work limits are
explicit reasons; they are never represented as `[1]`, `[0..*]`, or an empty result.
An evaluated interval does not establish whole-model validity or instantiate its
members. Library navigation stubs return `unknown/unavailable`.

Any syntax error in the workspace makes cardinality unknown for this analysis:
recovery may have dropped a constraint needed by another declaration or file.
This conservative policy is intentional until dependency-sensitive validity is
proved. `declaredMultiplicity` is a display string produced only when a header
multiplicity exists and checked bounds are available; no floating-point bound is
exported. Null still does not establish absence or an effective default.
`valueExpression` remains source text, not a public general-purpose evaluation API.
Static package-owned attributes can supply checked bounds through aliases,
reference chains and numeric operators (`+`, `-`, `*`, `/`, `%`, `^`), including
nested and unary expressions. Conditional providers support static Boolean tests,
aliases and nested conditionals, evaluating only the selected branch after proving
argument eligibility. Unselected unbound values or arithmetic errors are not
executed. Unsupported featuring, variation, predicates and other control operators
remain unknown. See KERNEL-003 in [foundation qualification](foundation-qualification.md).

`membersAvailable: false` describes a navigation record. It must not be read as an
empty namespace. `inheritanceComplete` is meaningful when member results are
available. It combines the upstream effective-member result with conservative
propagation of unresolved written typing, specialization, subsetting and redefinition
dependencies. This includes definitions/usages that depend transitively on a
missing parent. Ownership alone does not propagate inheritance uncertainty.
The snapshot is not a serialization of every relationship, annotation, import membership or
implied element in the semantic graph. The remaining imported-membership closure case in
the coverage matrix makes this gap visible.

## Failure policy

- Malformed language input produces partial results and diagnostics where upstream
  recovery succeeds. Every returned range and handle must remain usable.
- Input-policy violations return `AnalysisError::InvalidInput`; requested
  cancellation returns `Cancelled`. `Engine` reports projection invariant failures;
  upstream panics are not currently converted into that variant.
- Native callers needing fault containment should use a process boundary. A stack
  overflow/abort is not recoverable by the facade. CLI and benchmark processes are
  bounded by their test hosts; interactive WASM workers have a 60-second deadline.
- A worker reply must match both the message revision and the snapshot revision,
  and use a supported schema. Superseded replies, traps and timeouts never install
  a new displayed model. The previous display survives a failed request.
- Connection ends containing unresolved references remain conservatively marked
  unsupported. The facade does not label a resolved prefix as a complete endpoint.

## Compatibility and release policy

The current Rust packages are experimental `0.2.x`, unpublished and consumed as
workspace/path dependencies. This milestone does not freeze a 1.0 API.

The JSON wire contract remains schema version 1. Generated TypeScript describes
that contract; adding native query methods does not change it. Incompatible field
changes, meaning changes, removals or enum variants require a schema version bump
and coordinated consumer changes. Additive optional fields require compatibility
tests and documented absence semantics. Avoid new fields until a consumer needs
them. Input objects reject unknown fields deliberately.

Rust API incompatibilities require a documented pre-1.0 minor-version change.
Correctness fixes can change erroneous results without changing the contract's
intended meaning; retain their regression and document the change.
Engine/library version pins are separate from the protocol version. Every upstream
upgrade must rerun the executable coverage matrix and native/WASM agreement checks,
with all newly passing gaps and regressions reviewed. See
[dependency upgrades](dependency-upgrades.md).

## Review findings and disposition

| Finding | Disposition |
| --- | --- |
| Raw member ID arrays conflated empty scopes with unavailable queries for callers | Checked member methods now return explicit unavailable/unknown errors. |
| Library references/diagnostics were absent without a query-level capability result | Checked queries now report unavailability. Original wire data remains compatible. |
| “Immutable snapshot” overstated the guarantees of public Rust fields | Documentation now distinguishes an owned transport value from its immutable query borrow. |
| Revision IDs could be mistaken for globally persistent identity | Caller/session scope and reuse hazards are explicit and tested at the existing revision boundary. |
| Public syntax checking required invoking semantic analysis | Added a library-free syntax/structure operation. An editable AST API remains deferred. |
| Worker accepted a mismatched snapshot inside a matching reply envelope | Added a snapshot revision check and lifecycle/deadline regression tests. |
| Missing general types could leave inherited members marked complete | The facade propagates unresolved heritage through dependent definitions/usages; SPEC-002/003 retain the regression. Upstream source is unchanged. |
| Membership/alias enumeration and multiplicity completeness exceed this projection | Recorded planned cases; no synthetic completeness claim or UI-side semantic fallback. |

Implementation tests are in `crates/sysml-kernel/tests/contracts.rs`, supplemented
by the original acceptance tests, the matrix and `tests/kernel-client.test.ts`.

## Schema 2 migration

Schema 2 adds `Snapshot.memberships`, each element's `ownedMemberships`,
`inheritedMemberships`, and `cardinality`. Consumers must regenerate/recompile their
types and reject schema 1; there is no lossy migration that invents these results
from an old snapshot. The bundled worker and viewer now require schema 2. Element
IDs and membership IDs use the same snapshot identity domain; navigation records
may describe a relationship that also has a membership record. Neither changes
the owning namespace of an aliased element.
