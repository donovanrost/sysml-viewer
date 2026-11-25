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
| `query.state_subactions(id)` | Complete ordered owned bindings, unknown reason, or not applicable | Retains entry/do/exit kind, membership identity and action identity; duplicates remain visible and inherited actions are separate. |
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
internal and distinct from snapshot schema 4; this is not an interchange API.

Transition construction supplies an owned source input before any trigger payload.
The public owned-membership query exposes those distinct ParameterMemberships in
declaration order. With the bundled library, the first input's effective name is
`transitionLinkSource` even though it has no declared name. Directed body features
can supply additional inputs without being ParameterMemberships. The internal AST
retains grammar-created versus semantic-only source-slot provenance, including
conditional target and guarded entry-transition context, across textual formatting.
The public snapshot does not expose a complete implied-relationship graph or certify normative interchange.
See KERNEL-021 in [foundation qualification](foundation-qualification.md).

Accept construction supplies an inout payload and a separate input receiver, even
without a written `via`. Named ordinary accepts expose their ordered parameter
memberships; state subaction identities remain available through the state query.
A transition's nested accepter may be a navigation stub, so callers must check
`membersAvailable` before inspecting its parameters. Internal trigger expressions
are payload values with argument wrappers, not additional accept inputs. Generated
model lifting preserves payload/receiver expressions and directed body members;
this is not a public interchange guarantee. Default receiver bindings, complete
trigger semantics and execution are not qualified by these queries.
See KERNEL-022 in [foundation qualification](foundation-qualification.md).

The current worktree adds checked trigger bindings through the engine's
`invocation_binding_report` and checked `argument` property. The CanonicalV3
provider requires a valid trigger kind, a unique loaded public library Function,
complete callee inputs and current argument-redefinition evidence. It omits a
receiver with no FeatureValue without inventing a default value. Trigger-only
models activate the same atomic planner as other invocation models. Aliases,
broader signature/redefinition configurations and incomplete dependencies remain
explicitly qualified. This engine-level report does not add snapshot fields or
establish the three trigger argument validation rules; see the unreleased
[binding evidence](evidence/trigger-binding-progress-macos-arm64.json) and
[remaining result dependencies](evidence/trigger-result-dependency-review.json).

The worktree also admits ordinary Predicate signatures and their authored input
and return Features through the shared checked membership provider. Both
`function_result_report` and the checked Function `result` property require
canonical BooleanEvaluation heritage and complete positional dependencies.
Missing library, ownership or metadata evidence remains explicit; the queries
do not certify whole-Predicate validity or return uniqueness. At this checkpoint,
the full-library Boolean literal test verifies a Predicate-owned return located
through a compatibility expression query. The subsequent checked expression
selection is described below; other expression construction remains open.
See the unreleased [Predicate evidence](evidence/predicate-result-progress-macos-arm64.json).

The worktree constructs contextual Step specializations determined by the
owning Type's metaclass and the source's compositeness. It follows the recorded
[plural-library interpretation](evidence/step-context-construction-review.json).
The checked generated reference-result domain additionally admits noncomposite,
undirected sources owned through plain FeatureMembership, only with the actual
enclosure path when required. Missing ownership links and parameter/result
membership contexts remain incomplete. Effective-type-dependent composite owner
construction now admits complete owner predicates independent of all prospective
specialization writers. Direct/inherited Structure typing constructs the required
path; a Structure pruned from the effective type set does not. Coupled predicates
remain incomplete; the
[result-selection review](evidence/expression-result-dependency-review.json)
keeps these obligations distinct from a selected result's feature-domain proof.
The unreleased [Step-context evidence](evidence/step-context-progress-macos-arm64.json)
records the native and dependency checks and the remaining effective-type cases.
The subsequent [typed Step evidence](evidence/typed-step-progress-macos-arm64.json)
records the complete-type predicate, atomic final-write audit, five private
regressions and integration against all 94 library files. These internal engine
capabilities do not expand the facade into a complete metamodel API.

The worktree's `expression_result_report` now selects the first effective
ReturnParameterMembership-owned parameter for ordinary noncomposite, undirected
Expressions and the admitted Boolean/literal kinds. It shares the complete
Type feature-membership reducer and exact return selector with Function results.
Inherited ordering and redefinition determine the result; a complete empty
sequence yields `Ok(None)`, while the required `result` property reports
`MissingRequiredValue`. Exactly-one and output-direction validation are separate
obligations. Missing/cyclic evidence remains an error, and checked properties
require closure including implied relationships. Queries do not publish rows.
Both graph formats pass actual-library literal selection, and the native trigger
test now obtains its Boolean literal result through this report and property.
General invocation/operator result selection, references with existing authored
results, owned result-expression bindings and broader composite/directed/contextual
configurations remain open.
See the unreleased [expression-result evidence](evidence/expression-results-progress-macos-arm64.json).

Checked Type queries now pass their admitted semantic membership nodes and exact
authored redefinition closure into the same positional ordering and reduction
used by construction. This preserves materialized memberships that the stored
source selectors omit. The internal adapter requires a current proof stamp,
matching base domains and import selectors, and rechecks recorded authored edges.
It recomputes positional edges without publishing rows or retaining a query plan.
Construction keeps its existing stored-input preparation and cache checks.
See the unreleased [semantic positional evidence](evidence/semantic-positional-progress-macos-arm64.json).

Registered generated-reference sources now use the same result report and checked
property. Their admission requires current construction evidence, reciprocal
source/result ownership, the complete required source bases and the exact generated
binary binding endpoints. The binding's ordinary OwningMembership does not create
an owning Type or a positional parameter/end/result role. Its current redefinition
edges are still included in the shared membership proof. This does not certify all
BindingConnector ancestry, featuring or validity obligations.

Nonowning Feature members bring their declaring Types into the proof as dependencies,
not additional supertypes. The common positional planner schedules supported foreign
parameter/end/result dependencies before reducing memberships. Missing or cyclic
foreign ancestry stays incomplete; legacy foreign positional prerequisites remain
unsupported. No query adds graph rows. The full-library trigger examples now select
all six argument results through the checked report and property in CanonicalV3.
See the unreleased
[reference-result evidence](evidence/reference-results-progress-macos-arm64.json).

The shared foreign-member provider now also admits namespace-owned members from
the existing Step/Expression family. The same reciprocal OwningMembership,
namespace, flags, metadata and redefinition checks apply. Such a referent remains
a member of the referring expression and does not become one of its features.
The generated return and inherited library features retain their own identities
and ordering. Raw, prepared and serialized prepared-library regression tests
exercise this distinction. Checked report/property reads now select the results
of all 51 defaults in the full-library call/trigger integration fixture. This
does not establish complete Feature.type or default BindingConnector semantics;
see the [repair evidence](evidence/foreign-step-membership-progress-macos-arm64.json)
for qualification status and the [FeatureValue review](evidence/feature-value-obligation-review.json)
for remaining typing, initial-value, overriding and binding obligations.

The worktree's `trigger_argument_report` now exposes the active canonical rule,
`Result<bool, TriggerArgumentIssue>` and cumulative query work. Canonical trigger
construction must already be prepared. The report uses checked invocation arguments
and expression results, then the shared global resolver and specialization proof.
For `after`, an effective result feature must transitively redefine the standard
measurement reference and specialize DurationUnit; an unrelated duration-typed
feature cannot satisfy that condition. The shared membership certificate supplies
its complete redefinition graph, with identity alone excluded from the transitive
predicate. For `when`, the reference wrapper and its expression referent are checked
separately before selecting the Boolean result.

Semantic validation prepares trigger construction before creating explicit-only
Facts, consumes this report without passing generated indices to those arrays,
and emits a canonical diagnostic only for a proven false predicate. A clean
list is not evidence that every trigger was checked: unavailable providers remain
explicit errors in the report. Legacy trigger bindings, general result providers,
defaults and complete family conformance remain open. See the unreleased
[trigger-predicate evidence](evidence/trigger-predicates-progress-macos-arm64.json).

Abstract-model regressions now distinguish complete empty arguments from malformed
valuations, and verify first non-parameter membership selection across multiple
when-wrapper members. A Boolean result alone does not replace the required
reference wrapper. The shared Type feature and specialization providers admit
authored parameters of supported Step/Expression families only after checking the
complete owning signature and its positional obligations. Expression-owned returns
can therefore supply definite negative type comparisons as well as positive paths;
invalid ownership, metadata or required library evidence revokes the
certificate. A separate invalid return direction does not erase an independently
proved false trigger predicate. Required library roles also recheck the current
declaration name against the loaded role identity. These internal abstract fixtures do not establish a public editing
API or complete arbitrary-graph validation. See the
[abstract trigger evidence](evidence/trigger-abstract-progress-macos-arm64.json).

Canonical textual invocations now construct their own output Feature through an
implied ReturnParameterMembership before specialization planning. This includes
ordinary and arrow calls, operators and their chain/index/select/collect subclasses,
and SysML triggers. Cast/extent grammar returns and constructor result-owned
arguments retain their existing construction; legacy graphs retain their shape.
The shared positional planner supplies result redefinitions without copying the
callee's return or introducing a lexical scope. Owned result construction alone
does not certify the checked Expression.result or result feature-domain queries.
See the [construction review](evidence/invocation-result-construction-review.json).
This increment is not qualified: the first full run exposed incomplete library
result dependencies, compact-graph migration/pairing assumptions and a cold-query
work-budget regression. The required receiver ordering is repaired through the
shared semantic ownership view, which inserts the generated input before the
stored return without rewriting source ownership. The remaining failures must be
resolved and the complete gates rerun before release.

The follow-up migration repair reconstructs canonical invocation returns in
subtree order and retains grammar-provided returns. Compact validation admits
only the recognized implied return pairs and omitted accept receiver slots;
it does not accept arbitrary generated semantic graphs. Authored identity
pairing follows the same omitted-subtree policy as textual lifting. Contextual
lookup now derives unnamed slot names from their actual redefinition targets
and applies the shared membership reducer to inherited hits. These repairs are
still in qualification; the invocation-result review records the exact tested
revisions. Full-library invocation and trigger checks now pass in the focused
run. The cold-query repair streams redefinition reachability into the same
membership suppression rules, avoiding intermediate closure collections and
separate feature-projection scans. The unchanged full-library cold query now
passes at 9,789,848 canonical steps under the 10,000,000 limit; all 63 focused
reduction/input/positional tests, all 927 vendor model tests and 97 native
workspace tests pass. The full-library native check verifies both stored
Redefinition endpoints through the public property API. All 55 Node tests pass,
including public native/WebAssembly parity, provenance and evidence checks.
Reviewed implementation fingerprints retain all existing assessment limits;
all 133 conformance cases and the matching requirements execution report pass
on macOS arm64.

The subsequent lookup index caches only recorded source-row identities, guarded
by the source table's content revision. Resolution outcomes remain live reads;
clones, append, same-length edits, replacement and truncation retain dependency
tracking. Effective-default selection now follows KerML 8.4.4.11: a fully known
set of distinct inherited default identities establishes absence, while one
shared identity remains a default and incomplete paths remain unknown. Owned
values override inherited defaults. The full-library regression preserves the
eight inherited Occurrence defaults while checking the local difference. These
changes pass all 930 vendor model tests and 98 native workspace tests, with clean
Clippy and all 55 Node tests, including public native/WebAssembly runtime,
provenance and evidence checks. All 133 conformance cases and the matching
current-source requirements report pass on macOS arm64. See the
[lookup/default review](evidence/lookup-default-progress-macos-arm64.json).

The subsequent invocation-default increment exposes
`invocation_default_report`: complete actual argument bindings plus effective
defaults over all callee features. Selection neither executes default expressions
nor certifies whole-invocation validity. The construction transaction now stages
invocation-owned default bindings through the same identity and binary-endpoint
recipe used by constructors, with one connector per effective valuation identity.
It consumes accepted positional evidence, preserves source rows, and invalidates
held query evidence on publication. Targeted cold cardinality and post-edit
trigger regressions pass within the unchanged work allowance. Static positional
reuse requires the exact prior, current authority; differential tests compare
fresh planning, incomplete providers, source edits and rebuilt external plans.
Eligibility proofs also share successful designated Function identities within
the same checked snapshot. Each call still checks its supplied arguments, and
each expression retains a fresh path-sensitive proof. Library-name changes
invalidate the snapshot; cumulative work cannot be reset by a cache hit. Both
full mixed-corpus cold cardinality variants return the expected exact value at
about 6.14 million steps under the unchanged 10 million limit. The frozen-source
qualification passes all 945 vendor model tests, 99 native workspace tests,
55 public/provenance/evidence Node tests, Clippy, native/WebAssembly builds,
contract checks, 133 conformance cases and the matching requirements execution
report on macOS arm64. The initial no-build conformance report was rejected by
the requirements evidence check; the official build-and-conformance rerun passes.
See the [qualification record](evidence/eligibility-identity-progress-macos-arm64.json)
and [construction review](evidence/invocation-default-construction-review.json).
The [direct-obligation review](evidence/invocation-obligation-decomposition-review.json)
also records unresolved endpoint and membership formulations in the metamodel,
Table 11 and Feature Values Semantics. Current raw connector identities follow
the metamodel prose; they do not yet establish every formulation's full semantics.

The [valuation typing repair](evidence/valuation-typing-progress-macos-arm64.json)
keeps a valued Feature's declared static type when its valid direction or explicit
owned specialization makes the value-result specialization condition false.
Reciprocal ownership, valuation flags and specialization endpoints remain checked.
Mismatched or unresolved values do not replace independent declaration evidence;
this does not certify value compatibility or binding validity. Undirected values
with only implied specializations still require their own inference proof. The
macOS arm64 qualification passes 958 vendor model tests, 100 native workspace
tests, the 39 public and 16 provenance/evidence tests, Clippy, native/WASM builds,
contract checks, 133 conformance cases and matching requirements execution evidence.

The [semantic positional type repair](evidence/semantic-type-positional-progress-macos-arm64.json)
uses the same complete membership certificate for generated-reference positional
targets and type dependencies, including results without lexical scopes. Exact
library types, unresolved dependencies, closed circular typing, edits, cumulative
budgets and raw/prepared/serialized library identities are tested. Its macOS arm64
qualification passes 965 vendor model tests, 101 native workspace tests, 39 public
and 16 provenance/evidence tests, Clippy, native/WASM builds, contract checks and
133 source-matched conformance cases. Owner, chaining, portion and complete
default-binding obligations remain separate.

The unreleased [ordinary Feature context repair](evidence/ordinary-feature-context-progress-macos-arm64.json)
extends declared-type queries to plain noncomposite, undirected, non-end Features
under Type owners. Exact FeatureMembership ownership and the existing conditional
specialization checks remain required; FlowEnd payload obligations remain open.
Owned FeatureInverting relationships receive reciprocal-carrier and endpoint
checks in the shared type and specialization proof. Their targets do not become
typing dependencies. Native acceptance checks all ten selected library-default
feature types and six of their result types. Qualification passes 972 vendor
model tests, 102 native workspace tests, 39 public and 16 provenance/evidence tests,
Clippy, native/WASM builds, contract checks and 133 source-matched conformance
cases with matching requirements evidence. Chaining, portions, inverse value
semantics and complete default bindings remain open.

The unreleased [shared chain typing change](evidence/feature-chaining-type-progress-macos-arm64.json)
uses the common one-hop chain reader to validate every link and supply the final
Feature to both type dependencies and required paths. Exact repeated, nested,
inherited and redefined chains, malformed carriers/endpoints, edits, finite
circular typing closures and library replay have acceptance tests. Canonical
construction supplies the tested implicit bases; legacy graphs still require
explicit paths where their construction omits those bases. Qualification passes
978 vendor model tests, 103 native workspace tests, 39 public and 16 provenance
tests, Clippy, native/WASM builds, contract checks and 133 source-matched conformance
cases with matching requirements evidence. Portion families and
the three named chain-validation obligations remain open; a static type is not a
whole-model validity verdict.

An edit that makes a callee membership unresolved can invalidate the static
positional return proof. Retracting the dynamic tail does not authorize reuse of
that immutable static prefix. Such queries remain incomplete until the required
construction is valid again; a general abstract-model edit/rebuild API is still
open.

The [portion context candidate](evidence/portion-context-progress-macos-arm64.json)
is admitted to the working source after 13 targeted tests, 143 broader regressions,
one public integration test and 16 provenance checks. It shares complete effective
owner types across portion construction and negative specialization, distinguishes
owned from inverse typing, and preserves actual result types for the ten selected
library defaults. Malformed standalone typing is rejected before semantic-plan
refresh; recursion, edits, future writers and budgets retain explicit uncertainty.
Final-source qualification passes 992 vendor model tests, 104 native workspace
tests, 39 public and 16 provenance tests, Clippy, native/WASM builds, contract
checks and 133 conformance cases with matching requirements execution evidence
on macOS arm64. Mutually dependent construction predicates, SysML portionKind
and complete portion validation remain open.

The [namespace Feature member repair](evidence/namespace-feature-members-progress-macos-arm64.json)
preserves the distinction between a Type's namespace members and its owned
features. Ordinary `member` declarations retain checked declared/chained types
and expression results, with no invented owningType or parameter role. Local
owned-feature redefinitions and inherited membership suppression keep their
separate rules. Possible owned cross features remain incomplete until their
crossing obligations are supported. A valid membership-role edit now preserves
its exact declared type on the first query within the unchanged work limit.
Current-row specialization indexing and SCC-aware graph searches preserve
ordered relationships; positional reuse requires fresh physical-recipe and
authority checks. All 1,008 isolated model tests and the public integration pass.
Final-source qualification passes 105 native tests, Clippy, native/WebAssembly
builds, 39 public regressions, all 133 conformance cases and matching requirements
evidence.
These engine queries do not expand the declaration facade into a full metamodel
API or close contextual default-binding semantics.

The [binding namespace-member increment](evidence/binding-namespace-members-progress-macos-arm64.json)
adds authored BindingConnector members to the same enclosing membership and
redefinition proof without inventing outer feature or parameter roles. Necessary
local ownership and neutral-role checks run before rebuilding positional evidence
after an edit, so malformed membership is rejected within the existing work limit.
Generated bindings still require their exact construction certificate. All 1,013
isolated model tests and public integration pass. Final-source qualification passes
106 native tests, Clippy, native/WebAssembly builds and 39 public regressions.
The initial conformance run has 132 passes and one 30-second process timeout.
An isolated retry also timed out; a subsequent identical-source run passes all
19 assertions in 8.533 seconds. The cause of the variability remains unproven.
The full retry passes 128 cases; five later cases time out across recorded Mac
sleep periods. ACCEPT-003 passes in 4.954 seconds. Matching requirements evidence
records those failures after two stale audit hashes were refreshed. The complete
unchanged-source rerun with scoped idle-sleep prevention passes all 133 cases and
matching requirements evidence; ACCEPT-003 completes in 4.272 seconds. No timeout
was relaxed, and the earlier failed attempts remain preserved.
Canonical typing uses ordinary source. Legacy positive typing requires
complete authored signature paths; its incomplete literal-result cases remain
explicit. Connector ancestry, effective ends, featuring and contextual default
binding semantics remain separate open obligations.

The [Association end-domain increment](evidence/association-end-domains-progress-macos-arm64.json)
is admitted after 1,027 isolated model tests and a public API integration test.
Exact Association and AssociationStructure end queries use the shared membership
and positional redefinition proof. Required Link/BinaryLink and object-association
paths are checked for every ancestor against its effective ends. Missing required
paths remain incomplete. Foreign aliases retain membership identity without adding
features or inheritance; their positional owners are shared construction/query
prerequisites in both graph formats. The raw library-name index reuses only an
exact immutable name-table revision, with external overrides rebuilt and stale-map
retirement charged conservatively. Semantic roles and current-row proof remain
independent. Final-source qualification passes 107 native workspace tests,
39 public regressions, 13 provenance checks, Clippy, native/WASM builds, contract
verification and all 133 conformance cases with matching requirements evidence.
End typing, related/source/target derivation, complete validation, crossing semantics
and conditional binary-base construction remain open.

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

The current Rust packages are experimental `0.4.x`, unpublished and consumed as
workspace/path dependencies. This milestone does not freeze a 1.0 API.

The JSON wire contract is schema version 4. Generated TypeScript describes
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

## Schema 4: owned state subactions

Schema 4 requires `Element.stateSubactions`; package and app versions are 0.4.0.
Consumers must reanalyze schema-3 snapshots and regenerate/recompile their types.
The worker, viewer and native query constructor reject old schemas. No default
empty action list is synthesized for a snapshot that lacks the new field.

`StateSubactions::Complete` contains an ordered `actions` array of
`{ kind, membershipId, actionId }` records. `kind` is `entry`, `do` or `exit`.
The membership is a StateSubactionMembership owned by the queried state, and its
member is the identified ActionUsage, including a PerformActionUsage. A performed
action retains its own identity rather than being replaced by its referent.
The shared library query supplies the selection and order; the facade requires it
to account for every owned subaction membership with consistent ownership.

The result describes owned, written bindings. It excludes inherited actions,
ordinary action members and aliases. A complete empty set is meaningful even if
the state's definition contributes inherited entry/do/exit actions. Duplicate
roles remain separate records alongside the semantic diagnostic; enumeration
completeness does not imply a valid state or specify execution order.

Any syntax error in the state's source document gives `unknown/invalidSyntax`,
since recovery can omit a binding. Errors in another document do not change this
owned-body enumeration. Missing action types or perform referents do not remove a
known owned action. Library/navigation records return `unknown/unavailable`;
non-state elements return `notApplicable`. Incomplete membership construction
returns `unknown/incompleteMembership` without a partial array.

Native query construction validates that complete records exactly enumerate the
owned subaction memberships in order, and that membership/action IDs, ownership,
and ActionUsage kinds agree. It rejects omitted, duplicated, reordered or rebound
handles. As with other snapshot checks, this is structural integrity validation,
not proof that arbitrary externally modified source facts are true.

The pinned XMI has internal discrepancies in its six entry/do/exit derivations:
the property type and prose specify an ActionUsage, but the OCL expression returns
a StateSubactionMembership. Transition guard/effect derivations also contain
inconsistent selectors or assignments. The [source review](evidence/state-property-review.json)
retains their exact expressions, comments and artifact hash. This API exposes the
unambiguous binding, kind and action facts separately; it does not certify those
higher-level derived slots. General transition-feature collections, implicit body
construction and execution semantics remain open.

The issue review also records the historical [trigger-source rule resolution](https://issues.omg.org/issues/SYSML2_-89)
and [feature-chain correction](https://issues.omg.org/issues/SYSML2_-222), which
corroborate the checked endpoint behavior. The closed [effect-action wording issue](https://issues.omg.org/issues/SYSML2-398)
does not establish that the copied effect OCL in our pinned artifact was corrected.
The separate [library state-action self-redefinition issue](https://issues.omg.org/issues/SYSML21-297)
remains open at review. No new conformance claim follows from issue status alone.
