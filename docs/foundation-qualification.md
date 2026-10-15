# Kernel foundations: membership, names and cardinality

This update closes public-query and correctness gaps on the path to full language
coverage. It does not establish full SysML/KerML conformance. The target remains a
reusable language kernel; editing, interchange and complete graphical notation have
separate conformance obligations.

## Implemented behavior

Snapshot schema 3 retains membership records and checked cardinality results,
and adds checked imported-membership closure. Native
queries preserve alias identity, long and short names, visibility, original owning
namespace and unresolved/ambiguous target status. Inherited membership queries
return their completeness flag alongside the records. Canonical element lookup
continues to identify elements, independently of their aliases.

A facade validation pass fills a specific upstream omission: plain alias memberships
are checked for distinguishability against explicit owned and inherited memberships.
It compares both names and short names, and only reports a collision when the target
metaclasses conform in at least one direction. Unrelated metaclasses remain legal.
Imported closure is not enumerated by this pass. Findings use the normative
`validateNamespaceDistinguishibility` identifier and locate the containing namespace.

Checked cardinality uses the upstream constraint report, not its approximate numeric
compatibility result. Integer bounds cross JSON as decimal strings. Evaluated zero,
an unbounded upper limit, and an unknown result are distinct. Feature-valued providers and inherited subset bounds have executable cases;
conditional providers select checked Boolean branches. Unknown providers and
invalid intervals do not become default or empty intervals. Recovery can discard a
malformed constraint, so syntax errors conservatively invalidate cardinality results
across the workspace, including clean dependent files.

The legacy `declaredMultiplicity` display string now uses checked exact bounds. Its
upstream compatibility query is used only to detect a written header declaration.
No floating-point bound is exported. Every emitted snapshot passes the same handle,
source-coordinate, membership-owner and interval-integrity validation available to
native clients through `queries()`.

## Evidence and limits

The [matrix](conformance.md) links cases for lexical/global scope, aliases, short
names, protected/private visibility, recursive imports, root import validation,
quoted names, dialect-specific reserved words, exact integers and unknown bounds.
The [assessment ledger](../standards/assessments.json) maps those assertions to
specific clauses, grammar and named-rule anchors. Mapped named constraints remain
partial assessments; their full closure is not inferred from these cases.

The Rust foundation suite additionally checks:

- Membership identity and alias target ownership through native queries.
- Unknown, unavailable and malformed transport results.
- Integer bounds through JSON round-trip, including the maximum signed 128-bit value.
- Recovery effects across files.
- Exact arithmetic bounds alongside cyclic, overflowing, non-natural and unbound
  providers in the same workspace.
- Conditional bounds with strict Boolean tests, nested expressions and lazy
  branches, plus unknown results for unavailable conditions and invalid values.
- All 32 combinations of SysML/KerML, membership/namespace import, four written
  visibility forms, and root/nested ownership for `validateImportTopLevelVisibility`.
- Syntax and structural validation of all 94 pinned library source files.

Library syntax acceptance is not library semantic qualification or exhaustive
textual grammar coverage. Native/WebAssembly snapshot parity includes aliases,
exact large integers, arithmetic, conditionals and unknown providers. Existing viewer behavior remains tested.

The initial inherited-alias expectation was reviewed against the normative XMI:
`Type::removeRedefinedFeatures` includes each Feature itself in the reduction set.
Distinct inherited memberships of that same Feature can therefore remove each
other. IMP-021 preserves that boundary; IMP-014 tests aliases of a non-Feature
nested definition, which retains its inherited memberships. This is an explicit
standards-based expectation, not an adjustment to make an unexplained result pass.

Source authority: [OMG KerML 1.0](https://www.omg.org/spec/KerML/1.0) and
[OMG SysML 2.0](https://www.omg.org/spec/SysML/2.0), using the independently pinned
artifacts in `standards/sources.json`.

## KERNEL-001: feature-valued multiplicity providers (fixed for the recorded cases)

MULT-010 evaluates a literal-valued provider to `[3]`. MULT-003 evaluates addition
and multiplication providers to `[5]` and `[3..6]`. Neither case retains a known
failure. Two complementary changes support these results:

- The recorded featuring patch admits exact SysML `AttributeUsage` instances with
  ordinary package ownership. It preserves reciprocal ownership, explicit/inverse
  featuring, metadata, variable/variation, stale-evidence and work-budget checks.
- The facade builds CanonicalV3 and requests shared implied-relationship publication
  before validation and projection. Checked operator queries then consume the
  current designated callee, formal input and supplied-argument redefinition evidence.
  Arithmetic requires no further change to vendored source.

The upstream checked binding query deliberately does not publish rows. LegacyV2
rejects numeric operator bindings; CanonicalV3 before publication still reports
unmapped arguments. A full-library probe demonstrated that canonical publication
supplies the missing evidence and the checked bound succeeds. This lifecycle is
now explicit in `Engine::analyze`, with cancellation checkpoints around the phase.
Publication can decline individual families; preparation is not itself a claim of
complete semantic evidence. The checked reader remains authoritative.

KerML 1.0 §8.3.4.8.5 permits an unfeatured reference with an evaluable valuation.
§8.3.4.8.8 ties model-level invocation eligibility to both the function and its
arguments and orders arguments by their redefined formal inputs. §8.3.4.8.17 defines
operator-function selection. No compatibility evaluator, source-text arithmetic,
private positional shortcut or floating-point fallback is introduced.

MULT-011 covers aliases, reference chains, separate range endpoints, zero and
`9007199254740993`. MULT-013 covers addition, subtraction, multiplication, exact
division, remainder, exponentiation, nested and unary expressions, and exact integer
arithmetic beyond JavaScript's safe range. MULT-012/MULT-014 retain unknown results
for incomplete, variation, contextual, cyclic, fractional, negative, overflowing,
division-by-zero and unbound providers. MULT-015 checks that a shadowing user package
cannot certify the designated arithmetic operator. Native/WebAssembly parity tests
combine valid arithmetic and unadmitted calls in the same workspace.

CanonicalV3 also changes conditional operand wrappers, constructor argument
ownership and checked end constancy. The dependency test runner now covers the
related invocation, constructor, dynamic graph, owned-result and end-constancy
suites as well as the existing type-relation, evaluability and cardinality suites.
Snapshot schema 2 is unchanged; the internal graph version is not an interchange
format exposed by this facade. Full constructor and general expression semantics remain
unqualified by this milestone.

## KERNEL-003: conditional-valued bounds (fixed for the recorded cases)

MULT-016 retains the original `if true ? 3 else 4` provider and expected bound `[3]`.
The checked provider now admits the exact loaded `ControlFunctions::'if'` identity,
certifying absence in both earlier packages in the normative operator search order.
All three arguments must map to current formal inputs. Both branch arguments must
retain their canonical expression-reference wrappers and reciprocal ownership.

KerML 1.0 §8.4.4.9.6 passes the second and third operands as Expressions rather than
immediately evaluating them; §9.4.17 defines the control-function signature. The
existing model-level eligibility proof still covers the complete argument list.
The bounded execution pass requires an actual Boolean condition and certifies only
the selected value graph. The exact evaluator then executes that branch. An integer
is never coerced to Boolean and an unavailable condition never selects a default.
Stored source syntax and compatibility evaluation are not fallback evidence.

MULT-017 covers both outcomes, aliases, nested branches and conditions, arithmetic,
range endpoints and exact large integers. Unselected division-by-zero and unbound
values do not prevent a valid selected result. MULT-018 keeps unavailable/non-Boolean
conditions, invalid selected values and recursive/ineligible branches unknown.
MULT-019 rejects ambiguous designated library identity. Private dependency tests
cover missing canonical publication, both earlier lookup packages, malformed skipped
branch wrappers, metadata, stale publication, work exhaustion and edited graph
conditions before publication. The original failure was reproduced before the fix.

The change is recorded as five per-file patches under KERNEL-003, with original and
result hashes; upstream source inventory remains intact. It reuses the shared
binding, ownership, eligibility and exact arithmetic services. Snapshot schema 2
is unchanged. This qualifies static Boolean conditions within the existing bounded
expression domain; general predicates, contextual execution and other control
operators remain broader work.

## Checked import enumeration (schema 3)

The facade exposes the pinned kernel's checked `Namespace::importedMemberships`
and `Namespace::visibilityOf` operations. No additional vendor patch was required.
KerML §8.3.2.4.5 defines collision exclusion and visibility relative to a namespace;
§8.3.2.4.4/6 define membership/namespace imports, and §8.3.4.13.4 defines Package's
filtered override. The operation dispatcher selects the applicable override.

IMP-020 covers aliases, short names, duplicate import paths, contextual visibility,
public re-export, library targets and complete empty results. IMP-024 covers
recursive namespace/member imports, private branches, import-all and bounded cycles.
IMP-025 retains unrelated bindings when colliding aliases or owned names exclude
imports. IMP-026 keeps missing endpoints, incomplete inherited providers and inline
filters unknown, while a proved false package filter returns a complete empty set.
IMP-027 prevents syntax recovery from certifying an empty result.

The new Rust suite checks round-trip identity, binding order, declared versus
contextual visibility, stubs and malformed transport. A separate unit regression
exhausts the visibility allowance and verifies that no partial closure escapes.
The worker rejects schema 2, and native/WASM parity includes valid and unknown
import results in the same workspace. Namespace dependency tests check stale and
malformed graph evidence, recursion/work limits and serialization replay. These
are bounded qualifications, not proof of all import/filter contexts.

## KERNEL-004: expression proof isolation (fixed for the recorded cases)

An unresolved classifier base previously aborted shared dynamic-relationship
preparation: `part def Base :> MissingBase` could make unrelated arithmetic and
conditional bounds unknown. The original combined import-guard request also failed
against preceding commit `1adeadc` (engine suffix `.4`), so import projection did not
introduce the defect.

Canonical planning now localizes exact `Specialization` and `Subclassification`
rows with unresolved targets. Every supplied source alias and generic endpoint must
agree with the local source identity; the semantic carrier and all unresolved target
aliases are checked independently. The existing dependency audit still refuses a
family that reaches an incomplete source. No missing base is replaced with an empty
ancestry or a default value. LegacyV2 keeps its prior refusal.

MULT-020 retains exact arithmetic, aliases and conditional bounds in the same file
as a missing base, while direct and transitive dependents remain unknown. MULT-021
combines arithmetic, conditional and incomplete-import fixtures across files. The
Rust regression checks both file orders, and native/WASM parity compares the valid
and incomplete workspaces. Dependency tests cover both relationship kinds,
contradictory source aliases, malformed targets, dependent signatures, stale evidence
and work exhaustion. The patch and original/result hashes are recorded under
KERNEL-004. Snapshot schema 3 is unchanged; the engine patch suffix is `.6`.

KerML §8.3.3.1.8 and §8.3.3.2.3 define the specialization endpoints and classifier
restriction; the existing invocation and operator proof obligations still apply.
This qualifies the recorded independent components. Dependent dynamic families,
malformed evidence, metadata and other unsupported configurations can still prevent
shared publication; general proof isolation and incremental analysis remain open.

## KERNEL-005: compound range proof reuse (qualified for the recorded facade cases)

With engine suffix `.6`, a range whose lower provider is `2 + 3` and upper
provider is `if true ? count * 2 else 0` returned `unknown/workLimit`, even though
each bound succeeded separately. MULT-022 retains this `[5..10]` expectation,
alongside aliases and repeated endpoints. The original facade failure was reproduced
before the change; removing unrelated unresolved definitions did not resolve it.

The checked cardinality walk now shares a private eligibility batch across its
expressions. It retains the complete topology certificate, checked TypeRelations
and designated self identity under one semantic stamp. Each expression still gets
a fresh eligibility proof with its own path-local cycle guards. No eligibility
classification or evaluated value is cached across reports, snapshots or requests.
The stamp checks source rows, publication, ownership, metadata, library boundary,
provider readiness and closure policy. A stale batch refuses further reuse.

Work accounting remains cumulative across cache hits and interleaved readers;
callers cannot reset the allowance by supplying a smaller count. Partial graph
construction is never retained. Tests cover row edits, semantic generation changes,
different models, malformed identities, cold/warm work exhaustion and fresh retries.
MULT-023 keeps missing, fractional, non-Boolean and reversed endpoints unknown.
Native round-trip and WASM tests include both clean and missing-base workspaces.
The original vendor inventory is unchanged: combined KERNEL-003/005 patches retain
the earlier conditional changes, and a new module export patch is recorded.
Snapshot schema 3 remains unchanged; the engine patch suffix is `.7`.

With `.7`, a direct full-library probe used about 7.5 million steps for a prepared
range report instead of exceeding the unchanged 10-million-step allowance. A first
direct range query with cold library evidence still exceeded that allowance. The
recorded first-query failure is addressed by KERNEL-006 below; larger compositions
still need separate qualification.

## KERNEL-006: first-query range budget (fixed for the recorded entry path)

The `.7` first range query spent about 5.2 million steps on shared preparation and
constraint validation before checking expressions. Model-level eligibility then
repeated complete callee signature projections for each call, including calls
visited again from another bound. That repeated work exhausted the 10-million-step
allowance even though a later query on the prepared model succeeded.

The eligibility adapter now retains the existing `ConstructorBatchEvidence` within
its private query context. Constructor and invocation readers share checked callee
feature sequences and designated operator targets. Each supplied argument still
passes the existing binding reader: successful signature reuse never substitutes
for argument mapping, valuation checks or recursive eligibility. No new binding
algorithm or global cache is introduced. Semantic stamps and cumulative work
accounting remain in force; the allowance is unchanged.

A direct probe now completes the first `[5..10]` query in about 9.3 million steps
and a subsequent query in about 4.3 million. The full-library regression independently
rebuilds the model for a range, an alias endpoint and repeated endpoints, both with
and without an unrelated missing base. Each first query must succeed without an
earlier proof query or a retry. These queries require the canonical graph with
implied relationships already published; unpublished graphs remain unqualified.
This is a bounded-work regression, not a latency promise for every cold entry path.

Private regressions check constructor and operator reuse, a different call to the
same constructor with an unmapped argument, changed callee directions and argument kinds, and
exhausted cold/warm budgets followed by fresh retries. Existing stale-generation,
cycle, malformed-graph and invalid-endpoint tests remain required. KERNEL-006 is
recorded in the combined adapter patch against the original upstream bytes.
The engine patch suffix is `.8`; snapshot schema 3 and package version 0.3.0 remain
unchanged.

## KERNEL-007: exact ordered predicates (qualified for the recorded cases)

Before this change, a static attribute valued `if 2 < 3 ? 7 else 9` produced an
unknown/unsupported bound when used as a multiplicity provider. KERNEL-007 admits
`<`, `<=`, `>` and `>=` through the existing checked canonical invocation path.
Operator resolution proves the designated DataFunctions identity and absence in
BaseFunctions, following KerML 1.0 §8.3.4.8.17. The ordering signatures and numeric
specializations are specified by §§9.4.3.2, 9.4.10.2 and 9.4.11.2 of the
[pinned normative specification](https://www.omg.org/spec/KerML/1.0).

The execution certificate restricts both operands to finite exact integers or
rationals and reuses the existing exact comparator. It returns a Boolean for the
conditional test; it does not coerce that Boolean into a numeric bound. This also
preserves exact arithmetic intermediates beyond i128, while exported cardinality
endpoints retain their existing i128 limit. No new cache, parser, public evaluator,
wire field or resource allowance is introduced. Engine version advances to `.9`;
snapshot schema 3 and package version 0.3.0 are unchanged.

MULT-024 covers every less/equal/greater outcome for all four operators, adjacent
integers above 2^53, signed and i128-edge values, close/equal/mixed rationals,
arithmetic, aliases, nested conditionals and lazy branch selection. MULT-025 keeps
Boolean operands and endpoints, real/infinite values, division by zero, excessive
arithmetic, missing operands and selected fractional endpoints unknown. Dependency
tests extend canonical-versus-legacy publication and namespace-shadowing checks to
each operator, and exercise stale operand/owner/operator evidence, work exhaustion
and graph operand edits before publication. The facade round-trip test and native/
WASM parity suite retain the same values and guards.

Three existing per-file patches carry KERNEL-007 and still reconstruct their
original upstream bytes. Other predicates, equality, logical control, units,
sequences, arbitrary user functions and general contextual execution remain open.
This qualifies the bounded provider path, not complete expression semantics.

## KERNEL-008: scalar equality and Boolean predicates

The `.9` baseline returned unknown bounds for `if 2 == 2 ? 7 else 9`,
`if true and false ? 7 else 9` and `if not false ? 7 else 9`. The checked provider
now resolves equality/sameness through BaseFunctions, eager Boolean operators
through DataFunctions, and conditional logical operators through ControlFunctions.
Every selected function must retain its exact loaded-library identity and current
input bindings. Earlier-package absence is proved only where required; equality
continues to select BaseFunctions when DataFunctions contains a specialization.

KerML 1.0 §8.2.5.8.1 Table 5 and §8.3.4.8.17 specify operator mapping and lookup.
Sections 9.4.2.2 and 9.4.5.2 supply equality and Boolean signatures. Section
8.4.4.9.6 and §9.4.17.2 specify that `and`, `or` and `implies` pass their second
operand as an Expression. The implementation uses a shared three-variant control
rule during certification and execution. It checks eligibility and reciprocal
wrapper ownership even when that operand is skipped, while avoiding execution of
an unneeded value. It does not cache computed results or increase resource limits.
The source authority remains the [pinned KerML specification](https://www.omg.org/spec/KerML/1.0).

MULT-026 covers `==`, `!=`, `===` and `!==` for Boolean and finite exact numeric
pairs, including rational equivalence, adjacent integers above 2^53, arithmetic
beyond i128 and aliases. Sameness and equality coincide only for these admitted
data values; occurrence identity is outside this certificate. MULT-027 covers all
truth-table rows for eager `&`, `|`, `xor` and conditional `and`, `or`, `implies`,
as well as `not`, nested predicates, aliases and skipped unavailable/undefined
operands. MULT-028 refuses mixed domains, approximate/string/null/infinite values,
required missing operands, undefined/excessive arithmetic, recursive eligibility,
invalid Boolean operands and Boolean cardinality endpoints. Required operands
never turn into default Boolean values.

Private dependency regressions cover all eleven operators in canonical and legacy
graphs, changed formal inputs, work exhaustion, earlier-package shadow/import/
alias ambiguity, user-defined lookalikes and later-package specializations.
Deferred-operand tests retain unknown results after wrapper ownership, metadata,
kind or condition edits, and reselect from current graph values before publication.
Public Rust round-trip tests and native/WASM snapshots exercise the same results.

The combined native/WASM integration workspace initially exceeded the unchanged
10-million-step publication allowance: standalone cases passed while existing
arithmetic, conditional and comparison bounds became unknown. Operator selection
now shares complete direct-name and loaded-library identity projections within the
existing source/publication-stamped batch. It still validates every namespace
relationship, distinguishes ambiguous names from certified absence, and discards
projections after source, visibility, ownership, metadata or publication changes.
No value results are cached. The integration test retains all earlier and new
fixtures together, including the additional unresolved-import workspace.

Four existing reversible patches carry KERNEL-008. The engine advances to `.10`;
snapshot schema 3 and package version 0.3.0 remain unchanged. Strings, sequences,
null/coalescing semantics, occurrence identity, units, arbitrary function dispatch
and general contextual execution remain open. An evaluated bound does not prove
that every expression or the enclosing model is well typed.

## Remaining foundation work

Checked imported membership enumeration now has executable coverage (IMP-020 and
IMP-024–027). General filtered imports, all contextual name-resolution rules, implicit metamodel construction and every
membership override still need qualified queries and additional cases. Broader
language families and the remaining standards inventory remain open. Full coverage
cannot be inferred from the passing count in this matrix.

## KERNEL-002: alias syntax false positive (fixed)

The syntax checker treated unknown alias target metaclasses as a definite
distinguishability conflict. IMP-022 demonstrates a legal same-name pair of aliases
whose resolved target metaclasses are unrelated. The recorded patch defers that
comparison until the facade semantic pass can inspect target identity. IMP-015 and
IMP-023 preserve rejection of actual alias collisions. Anonymous non-owning
memberships also retain null names instead of borrowing their target's name.

`patches/manifest.json` pins the upstream revision, original file hash, patch hash
and resulting file hash. Verification reverses each patch in a temporary tree and
checks that it reconstructs the original upstream bytes. The original vendor hash
manifest is retained, import reapplies reviewed patches, and qualification
fingerprints include the patch inputs. The engine version carries a local patch
suffix. Unrecorded edits, changed bases, incorrect patch targets and changed patches
are rejected.

## Recorded verification

On macOS arm64, the completed run has **78 passing conformance cases**, **no
known-gap cases**, and **no planned cases in the current matrix**. All **46 Rust kernel tests**,
**33 Node tests**, and **232 dependency proof tests** pass.
The dependency suites cover type relations (41), model-level evaluability (33),
cardinality (26), invocation bindings (19), constructor bindings (16), dynamic graph
(17), dynamic invocations (25), owned results (4), end constancy (10) and checked
namespace operations (41). The corpus and the 32 import-rule partitions are included
in the Rust suite, not additional independently qualified specification counts.
Native and WebAssembly outputs match exactly.

Rust formatting, Clippy with warnings denied, TypeScript checking, Prettier,
generated-contract consistency, source/patch verification and inventory checks
pass. The saved [matrix execution](evidence/conformance-macos-arm64.json) and
[requirements execution](evidence/requirements-macos-arm64.json) record the current
source, patch and fixture fingerprints. Windows/Linux execution remains pending CI.

The complete [scalar-predicate benchmark rerun](evidence/benchmark-predicates-macos-arm64.json)
records three fresh processes per native/WASM workload, for 72 successful samples.
All repeated native and WASM snapshots agree exactly. Every equality sample passes
all 30 MULT-026 assertions; every Boolean sample passes all 36 MULT-027 assertions.
The ordered-predicate, compound-range and isolation assertions and seven other
workloads remain included.

| Workload | Native median (ms) | WASM median (ms) | Native/WASM maximum RSS (MiB) |
| --- | ---: | ---: | ---: |
| Scalar equality | 2,522 | 3,049 | 432.5/358.3 |
| Boolean predicates | 2,671 | 3,171 | 417.1/364.8 |
| Ordered predicates | 2,471 | 3,020 | 408.1/357.5 |
| Compound ranges | 2,132 | 2,542 | 408.7/352.9 |
| Independent bounds with missing base | 2,174 | 2,527 | 408.0/347.2 |
| Imported memberships | 1,982 | 2,396 | 413.8/343.8 |
| Viewer example | 1,967 | 2,435 | 413.9/359.7 |
| Attribute providers | 2,025 | 2,512 | 418.8/351.0 |
| Arithmetic providers | 2,100 | 2,595 | 424.7/358.2 |
| Conditional providers | 2,130 | 2,577 | 431.7/362.3 |
| 100 usages | 2,005 | 2,430 | 425.3/359.3 |
| 1,000 usages | 2,300 | 2,618 | 427.8/375.9 |

The earlier `.6` isolation run had an initial native sample time out at 60
seconds; a direct retry completed in 3.2 seconds, and its full 48-sample rerun
passed. The cause of that earlier timeout remains unresolved. No timeout occurred
in the recorded `.7`, `.8` or `.9` runs.

The [initial `.10` attempt](evidence/benchmark-predicates-initial-macos-arm64.json) stopped
after 45 successful samples when the first attribute-provider WASM sample exceeded 60 seconds.
It also retained an 18,985 ms Boolean WASM sample (the other two took 4,100 and 3,329 ms).
Three direct attribute-provider retries completed in 2,628, 2,537 and 2,480 ms with identical
snapshots. The separate complete 72-sample rerun is tabulated above. The initial timeout
remains unexplained; these results do not establish a stable latency bound.

Measurements include startup, bundled-library analysis and serialization. They are
observations on this Mac, not cross-platform latency guarantees. Canonical
preparation is a whole-model pass. The facade runs its model scan before projecting
cardinality; the separate direct-query regression exercises the first-query budget.
Compared with the retained [ordered-predicate run](evidence/benchmark-comparisons-macos-arm64.json),
these uncontrolled runs do not isolate incremental query cost or establish a
regression budget. Large import-heavy production models and incremental performance
remain open engineering work.

The production web build and macOS arm64 app and DMG builds pass. The new local app
is `src-tauri/target/release/bundle/macos/SysML Viewer.app` (11.97 MiB), with a
4.27 MiB installer under `src-tauri/target/release/bundle/dmg/`. This run
checks automated viewer integration and packaging; it does not repeat the earlier
interactive file-picker/navigation checks or establish Windows/Linux packaging.
