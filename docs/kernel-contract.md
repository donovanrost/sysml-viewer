# Kernel public contract review

Decision: retain the headless facade and versioned snapshot, add checked native
queries and a syntax-only operation. Keep the upstream AST/metamodel private to
the facade. Status: implemented, reviewed against the qualification suite.

The stable seam is source text → analysis → explicit query results. The current
facade is a **declaration projection**, not a complete metamodel API. In particular,
arbitrary scoped-name resolution, AST editing,
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
| `query.inherited_memberships(id)` | Binding records plus `inheritance_complete` | Preserves the original declaring namespace; imports have their own checked query. |
| `query.imported_memberships(id)` | Complete ordered bindings with contextual visibility, or an unknown reason | Uses checked Namespace/Package operations; incomplete evidence never becomes an empty set. Navigation stubs return unknown/unavailable. |
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
internal and distinct from snapshot schema 3; this is not an interchange API.

Ranges are half-open. Byte/UTF-16 offsets are zero-based; line and UTF-16 column
numbers are one-based. Original encoding after UTF-8 decoding and line endings
are preserved. Indexed query construction verifies coordinate agreement.
`SourceIndex::position` is a low-level helper that clamps to the preceding valid
character boundary (and EOF); checked source queries reject invalid boundaries.

The shared lexer accepts form feed as whitespace in both dialects. Quoted names
and strings use the seven escapes in KerML 8.2.2.3 Table 4. `\n` decodes to LF on
every host. Nonstandard escapes (including `\r`) and unescaped C0/C1 controls
produce syntax errors; diagnostics cover the complete offending character or
escape. Scanning continues to the closing quote so later declarations remain
recoverable. Unterminated quoted tokens remain errors through EOF. Raw control
characters inside comments retain their separate comment rules. Exhaustive Unicode
printability and every possible line-terminator encoding remain unqualified.
See KERNEL-012 in [foundation qualification](foundation-qualification.md).

Numeric tokens follow the decimal/exponential boundaries of KerML 8.2.2.4.
Real values are composed from numeric token text, so whitespace and notes around
the decimal point do not alter the internal value. Original source spans and
`valueExpression` text retain that trivia. Numeric spelling is never normalized
through floating-point parsing at this boundary. The exact-decimal reader's
existing scale allowance remains bounded, including extreme negative exponents;
refusal there does not make the source syntactically invalid. Real-valued bounds
remain unqualified and unknown. See KERNEL-013 in the qualification report.

Reserved words follow the selected dialect. Quoting permits reserved names;
words reserved only in the other dialect and differently cased basic names remain
usable. The keyword audit covers all entries in the two published lists across
five syntax contexts. The existing reservation of `new` remains an explicit
compatibility policy: it appears in the constructor grammar but is missing from
the published lists (open OMG SYSML21-641 and KERML11-200). This ambiguity remains
unqualified. See KERNEL-014 in the qualification report.

A `Reference` describes a **written name reference**. Its resolved target is what
that name denotes. For conjugated port typing `~P`, the written spelling includes
`~`, while the name target is `P`; this is not a promise that it is the complete
semantic FeatureTyping target after implicit conjugated-definition construction.

Connection endpoints are resolved independently. Unresolved or ambiguous references
in one end's ownership subtree make that end unsupported, including when the graph
retains a resolved chain prefix. A broken peer or unrelated member in the
connection body does not hide an otherwise resolved end. References inside an end's
multiplicity still conservatively block that end. Implicit ends remain unsupported;
a resolved endpoint is not a connection-validity certificate. Endpoint display
spelling can still represent a retained prefix; written references carry their own
source ranges. See KERNEL-015 in the qualification report.

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
aliases and nested conditionals, plus ordered comparisons (`<`, `<=`, `>`, `>=`)
between finite exact integers and rationals. Equality and sameness (`==`, `!=`,
`===`, `!==`) support Boolean pairs, exact numeric pairs, string pairs, explicit empty pairs
and empty versus an admitted scalar. Empty values compare equal to each other and
unequal to zero, false and every other admitted present scalar. Both operands must
execute successfully; an unavailable or failed operand never becomes empty. Boolean operators
include eager `not`, `xor`, `&`, `|` and conditional `and`, `or`, `implies`.
Null-coalescing (`??`) selects its deferred fallback only for a certified empty
`NullExpression` (`null` or `()`). Present zero, Boolean false and the empty string retain their
values. String literals use decoded graph values, preserving Unicode and escapes.
Binary string concatenation (`+`) and exact string equality/sameness compose with
aliases and conditional/coalescing providers. Copies, concatenation and comparison
charge UTF-8 bytes before work against the same cumulative query limit. Unavailable references and evaluation failures remain unknown, and an
empty result cannot serve as a numeric endpoint. The same argument eligibility,
identity and reciprocal wrapper requirements apply even to a skipped fallback.
Arithmetic results must remain exact; approximate overflow/underflow and
irrational-root results cannot establish bounds.
Conditional branches and deferred logical operands execute only when selected,
after every argument retains complete eligibility and reciprocal ownership proof.
A skipped operand may be unbound or undefined, but cannot have malformed evidence
or recursive/ineligible dependencies. Required conditions must be Boolean. Comparisons preserve exact arithmetic
intermediates, including rationals whose numerator exceeds i128; exported endpoints
still require a nonnegative signed-128-bit integer. Boolean and string results cannot become
numeric endpoints. Unselected unbound values or arithmetic errors are not executed.
Unsupported featuring, variation, cross-domain Boolean/numeric equality, nonempty sequences, string ordering and mixed-domain string operations,
occurrence identity, approximate numbers, quantities and other control operators
remain unknown. Evaluated cardinality does not certify whole-model type validity. Canonical unresolved classifier bases retain independent arithmetic
and conditional proofs when source/target evidence is consistent. Direct and transitive
dependents remain incomplete; malformed relationships and other unsupported dynamic
families can still prevent publication. Within a single cardinality query, related
expressions reuse stamped topology and type evidence while eligibility and execution
remain independently checked. Checked callee signatures also share evidence, while
every invocation and constructor argument list is validated separately. First range,
alias and repeated-endpoint queries over the full pinned library are covered after
implied relationships are published. Other entry paths and larger compositions can
still reach `workLimit`. See KERNEL-003/004/005/006/007/008/009/010/011 in
[foundation qualification](foundation-qualification.md).

`membersAvailable: false` describes a navigation record. It must not be read as an
empty namespace. `inheritanceComplete` is meaningful when member results are
available. It combines the upstream effective-member result with conservative
propagation of unresolved written typing, specialization, subsetting and redefinition
dependencies. This includes definitions/usages that depend transitively on a
missing parent. Ownership alone does not propagate inheritance uncertainty.
`importedMemberships` is a tagged result: `complete` carries ordered membership
IDs and each binding's visibility in the importing namespace; `unknown` carries
an explicit reason (`invalidSyntax`, `incompleteProvider`, `unsupported`,
`workLimit`, or `unavailable`). Resolve IDs against `Snapshot.memberships` to read
the original namespace, alias names, target and declared visibility. Re-exporting
a binding does not change its owner or identity. A complete empty set is meaningful.

The facade invokes checked `Namespace::importedMemberships` with an empty exclusion
set and `Namespace::visibilityOf` for each resulting membership. The kernel dispatches
Package overrides, bounds recursive imports, excludes indistinguishable imported
bindings, and determines visibility from the first applicable import. Syntax errors
anywhere make import queries unknown because recovery can remove necessary evidence.
Navigation records remain unavailable. There is a snapshot-wide allowance of 4,096
contextual visibility calls in declaration order, in addition to upstream operation
budgets. A closure exceeding its remaining allowance returns `unknown/workLimit`;
no partial closure is exported. This is a resource bound, not a latency guarantee.
General metadata/filter predicates and some Type providers remain incomplete.

Snapshot validation checks import handles, uniqueness and resolved member targets,
and rejects query results attached to unavailable navigation records. It does not
re-prove the semantic evidence of deserialized snapshots. The snapshot is not a
serialization of every relationship, annotation or implied element in the graph.

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

The current Rust packages are experimental `0.3.x`, unpublished and consumed as
workspace/path dependencies. This milestone does not freeze a 1.0 API.

The JSON wire contract is schema version 3. Generated TypeScript describes
that contract; native query methods borrow the same results. Incompatible field
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
| Membership/alias enumeration and multiplicity completeness exceed this projection | Schema 3 exposes checked imports and cardinality with explicit unknown results; complete metamodel traversal remains open. |

Implementation tests are in `crates/sysml-kernel/tests/contracts.rs`, supplemented
by the original acceptance tests, the matrix and `tests/kernel-client.test.ts`.

## Schema 2 migration

Schema 2 adds `Snapshot.memberships`, each element's `ownedMemberships`,
`inheritedMemberships`, and `cardinality`. Consumers must regenerate/recompile their
types and reject schema 1; there is no lossy migration that invents these results
from an old snapshot. This was the schema 2 migration. Element
IDs and membership IDs use the same snapshot identity domain; navigation records
may describe a relationship that also has a membership record. Neither changes
the owning namespace of an aliased element.

## Schema 3 migration

Schema 3 requires `Element.importedMemberships`. Schema 2 cannot establish this
result, so consumers must reanalyze and regenerate/recompile their types. The
worker and viewer reject older schemas. Rust package and application versions
advance to 0.3.0 for this pre-1.0 API change. Existing owned/inherited arrays and
`Membership.visibility` retain their meanings; import visibility is contextual
and lives on the imported binding. No UI-side resolution is introduced.
