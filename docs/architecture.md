# Architecture and contracts

Status: kernel migration and initial qualification implemented. Normative target: SysML 2.0 and
KerML 1.0; bundled model library: the March 2026 release. This is not a claim of
complete language or graphical-notation conformance.

The [public contract review](kernel-contract.md) defines supported operations and
compatibility policy. The [executable matrix](conformance.md) records bounded
standards coverage; [qualification evidence](qualification.md) records results.

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

Graphical notation support is deliberately smaller than language support. Binary
connections are rendered; other arities remain in the snapshot and receive a
visible unsupported-view label. A resolved connection establishes endpoint lookup,
not proof that every semantic rule applicable to it has been validated.

## Snapshot contract v1

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
- Unresolved declared heritage makes dependent member queries incomplete, including
  transitive dependencies. The facade accounts for missing edges that the upstream
  graph's completeness flag does not represent.

`Snapshot::element`, `find` and `references_at` are convenience queries. The viewer
builds maps once per snapshot for interactive lookups. The native CLI reads files
and writes JSON; `--request-json` exercises exactly the browser request contract.
Native consumers can call `Snapshot::queries()` for indexed lookup, checked source
ranges and explicit unavailable-member/reference/diagnostic errors. `check_syntax`
parses and checks one source without invoking semantic analysis or loading libraries.
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
