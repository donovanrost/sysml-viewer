# Kernel foundations: membership, names and cardinality

This update closes public-query and correctness gaps on the path to full language
coverage. It does not establish full SysML/KerML conformance. The target remains a
reusable language kernel; editing, interchange and complete graphical notation have
separate conformance obligations.

## Implemented behavior

Snapshot schema 4 retains membership records, checked cardinality results and
checked imported-membership closure, and adds owned state-subaction queries. Native
queries preserve alias identity, long and short names, visibility, original owning
namespace and unresolved/ambiguous target status. Inherited membership queries
return their completeness flag alongside the records. Canonical element lookup
continues to identify elements, independently of their aliases.

State queries preserve each owned entry/do/exit membership, its kind and its action
identity in declaration order. They retain duplicate roles, distinguish unavailable
or recovered bodies from empty ones, and do not infer inherited execution slots.

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

## KERNEL-009: explicit empty values and null-coalescing

The checked evaluator now admits `NullExpression` as an explicitly empty value
and selects the deferred right operand of `??` only when the left value is empty.
`null` and `()` select the fallback; zero and Boolean false remain present values.
Unavailable references, failed evaluation and an exhausted work allowance remain
unknown. An empty result cannot establish a numeric endpoint or prove absence.
This follows KerML 7.4.9.2, 8.2.5.8.1, 8.3.4.8.16 and the ControlFunctions library
in 9.4.17.2. The existing exact scalar execution domain and designated loaded
function identity checks apply.

MULT-029 exercises 24 positive providers: both empty spellings, exact scalar
presence, alias identity, nested fallback selection, arithmetic/Boolean/conditional
composition and skipped unavailable or undefined values. MULT-030 exercises 24
negative providers: unavailable first or selected values, recursive skipped
operands, unsupported domains, empty/nonnatural endpoints and arithmetic failures.
The complete argument list still requires model-level eligibility, and even a
skipped deferred operand must retain reciprocal wrapper ownership. Private tests
mutate wrapper ownership, metaclasses, annotations and current values before and
after publication: unpublished values reselect the branch, while stale published
evidence is refused. They also check the unchanged cumulative work limit. Native round-trip
and the combined native/WASM corpus exercise the same values and unknown reasons.

The negative cases exposed approximate exponent overflow being accepted as an
unbounded endpoint. Checked binary arithmetic now admits only exact integer or
rational results; floating-point overflow, underflow and irrational approximations
remain unknown. Explicit infinity retains its separate existing endpoint path.
Unary plus also rejects empty and Boolean values instead of passing them through.

The enlarged integration workspace initially exhausted the 10-million-step
publication allowance during staging. Dynamic preparation rebuilt the entire
library-name map even when the same transaction had supplied the static prefix
and the map was unused. Name materialization now occurs only when a prefix must
be constructed. Complete physical authority, recipe identity and metadata checks
still run. An instrumented probe of the combined corpus reduced total publication
work to 9,779,125 steps; no limit changed. Private tests compare both paths' plans,
retain exhausted-budget and stale-source refusal, and verify preparation does not
publish rows. The full earlier and new fixtures remain together, including the
unresolved-import variant; larger compositions can still exhaust the allowance.

Four reversible patches carry KERNEL-009. The engine advances to `.11`; package
version 0.3.0 and snapshot schema 3 remain unchanged. Null equality, nonempty
sequences, strings, quantities, occurrence identity and general function dispatch
remain open. These cases do not establish full specification or type-checking coverage.

## KERNEL-010: empty equality and sameness

The checked evaluator now supports `==`, `!=`, `===` and `!==` for explicit empty
pairs and empty versus an admitted Boolean or exact numeric scalar. Empty is equal
to empty and distinct from every present scalar, including zero and false. Both
operands must execute successfully; unavailable references and evaluation failure
remain unknown. This follows the empty-value and data-value comparison semantics
in [KerML 7.4.9.2](https://www.omg.org/spec/KerML/1.0/PDF), together with the
optional single-valued BaseFunctions operands in 9.4.2.2. Occurrence identity and
other value domains remain outside this certificate.

MULT-031 adds 36 positive providers covering all four operators, both empty
spellings, zero/false in both operand orders, exact rationals, aliases and
conditional/coalescing/logical composition. MULT-032 adds 20 unknown providers:
unavailable operands on either side of every operator, undefined or approximate
arithmetic, unsupported domains, recursion and invalid endpoints. Existing
MULT-028 `nullEquality` and MULT-030 `emptyEquality` assertions now expect the
correct evaluated bound of 7; all other earlier guards remain exercised.

Private tests mutate null into a present scalar before and after publication,
check reciprocal ownership of each operand, metadata annotations and operator
identity, and retain refusal at the unchanged cumulative work limit. The public
Rust tests serialize and deserialize the snapshot before querying it. The Node
integration corpus includes the earlier fixtures together with both new files,
with and without an unrelated unresolved import.

The expanded combined workspace initially exceeded the unchanged publication
allowance during staging. The transaction cloned the complete ownership table
even when an independently staged result family already held a private copy.
Dynamic rows are now fully constructed before an atomic, prepaid ownership append,
so that copy can be reused. Invalid identities, late invalid owners or a work-limit
refusal leave its prior ownership intact. Tests compare the batch with individual
registration and exercise failures both during row construction and after all
rows have been prepared. No budget limit or integration fixture was removed.

Five reversible patches carry the evaluator, atomic staging and private tests. The engine
advances to `.12`; package version 0.3.0 and snapshot schema 3 remain unchanged.
At this milestone, strings, nonempty sequences, quantities, approximate values,
occurrence identity, general function dispatch and whole-model type validity remain
unqualified. KERNEL-011 below extends the checked string domain.

## KERNEL-011: decoded string values

The checked evaluator admits decoded graph `LiteralString` values, binary string
concatenation (`+`), string-pair equality and sameness (`==`, `!=`, `===`, `!==`),
and comparisons between empty values and strings. An empty string is present;
it differs from `null` and prevents a coalescing fallback. String results cannot
become numeric bounds. The implementation follows KerML 7.4.9.2, 7.4.9.5,
8.2.2.3, 8.2.2.5, 8.3.4.8.14 and 9.4.6.2 in the pinned standard and library.

MULT-033 adds 29 positive providers covering all four comparisons, case and
character differences, empty strings, Unicode and normative escapes, concatenation,
aliases and conditional/coalescing/logical composition. MULT-034 adds 20 unknown
providers covering missing and recursive operands, mixed domains, unsupported
operators and invalid endpoints. Earlier MULT-028 `stringEquality` and MULT-032
`string` assertions now expect the correct bound of 7. All earlier guards remain
in the combined workspace, with and without an unrelated unresolved import.

Private tests inspect decoded graph values, mutate values and property types,
break reciprocal ownership and add metadata before and after publication.
They also exhaust the shared work allowance during string copying, concatenation
and comparison. UTF-8 bytes are charged before allocation or comparison; neither
large values nor repeated reads obtain a fresh budget. Native round-trip queries
and exact native/WebAssembly parity exercise the public transport.

The enlarged combined workspace exposed a redundant second static-prefix audit
inside the same publication transaction. A combined capture-and-prepare path now
consumes the freshly audited prefix under one exclusive Builder borrow. Capture
still verifies each physical recipe and adopts its current identity; separate
entry paths still perform their own identity audit. Existing physical rows use the
complete raw identity index for collision checks, avoiding a duplicate set of the
static IDs. Regression tests compare cold/warm paths and exercise remapped IDs,
malformed rows, collisions and budget exhaustion. The work allowance and complete
integration corpus remain unchanged. Larger workspaces can still reach the limit.

The engine advances to `.13`; package version 0.3.0 and snapshot schema 3 remain
unchanged. String ordering, mixed-domain operations, general StringFunctions calls,
complete lexical conformance, nonempty sequences, quantities, approximate values,
occurrence identity and whole-model type validity remain unqualified.

## KERNEL-012: lexical whitespace and quoted-token boundaries

The shared lexer now accepts form feed as token-separating whitespace and uses
one seven-entry escape table for validation and decoding. KerML 8.2.2.1 explicitly
includes form feed; 8.2.2.3 Table 4 lists `\'`, `\"`, `\b`, `\f`, `\t`, `\n`
and `\\`. The previously accepted `\r` extension now produces a syntax error.
LF is the implementation's decoded newline on every host. Raw C0/C1 controls
inside names and strings are rejected, consistent with the printable-character
requirement in 8.2.2.3 and 8.2.2.5. Source text is preserved unchanged.

LEX-001/002 exercise both dialects, Unicode names, every standard string escape,
form-feed separators, reference resolution, exact declaration spans and bounds.
LEX-003/004 reject nonstandard escapes and raw tab, newline and C1 controls, recover
later declarations, and retain unknown bounds in a clean sibling document.
The valid lexical fixtures join the entire earlier integration corpus, including
its unresolved-import variant. Deliberately malformed fixtures exercise the
workspace-wide syntax-error boundary in a separate recovery analysis.

Five lexer proof tests cover all seven escapes in both quote styles, every one of
the 65 C0/C1 controls, invalid ASCII and multibyte escapes, comment boundaries,
unterminated tokens and trailing escapes. Every token sequence covers the entire
input on UTF-8 boundaries. Three public Rust tests check both dialects, exact byte
and UTF-16 diagnostic ranges after non-BMP text and CRLF, and recovery after JSON
round-trip. Node tests compare native and WebAssembly diagnostics and source spans.
The original 94-library syntax sweep remains part of qualification.

A new reversible lexer patch preserves the immutable vendor inventory. The engine
advances to `.14`; package version 0.3.0 and snapshot schema 3 remain unchanged.
This increment does not establish exhaustive Unicode printability, every line
terminator encoding, numeric syntax, formatter round-trip behavior or complete
lexical conformance. String ordering and general function dispatch remain open.
The [OMG function-definition issue KERML11-209](https://issues.omg.org/issues/spec/KerML/1.0)
records missing function-library definitions; implementation choices in those areas
must be distinguished from normative qualification.

## KERNEL-013: numeric token composition and decimal-scale boundaries

KerML 8.2.2.4 defines decimal and exponential tokens; 8.2.5.8.4 composes a real
literal from those tokens and a decimal point. Whitespace and notes are ignored
between tokens. The parser previously retained that trivia inside the literal's
value: `1 . 25` became the numeric text `1 . 25`, which downstream decimal readers
could not interpret. The fix composes the value `1.25` from the numeric tokens
while preserving the complete original expression span. The same rule applies to
leading-dot forms and fractional tokens containing an exponent. Integer digits, exponent signs and
arbitrary-precision spelling remain intact without a floating-point conversion.
Regular comments retain their separate grammatical role and are not stripped as notes.

The exact-decimal reader also previously took the signed absolute value of the
adjusted scale. For `1e-9223372036854775808`, that overflowed in debug builds and
wrapped in release builds, allowing an incorrect finite result. An inclusive range
check now rejects it under the unchanged scale allowance of ±100000. Fractional
digit adjustment, both signed extremes and the exact allowance boundaries have
regressions. These are implementation resource boundaries, not new syntax limits.

Three new regression assertions failed before the fixes: internal literal text,
exact numeric lowering, and the extreme scale guard. The six-test Rust suite adds
terminal/token-span checks, signs/ranges, malformed forms, both dialects, UTF-16
coordinates, exact lowering and public snapshot/source recovery. LEX-006/007 keep
source spans intact, diagnose fractional multiplicities, and retain unknown results
for checked real-valued bounds. LEX-008/009 reject incomplete exponents and recover
later declarations. The syntactically valid fixtures join the entire existing
workspace and its unresolved-import variant.
Malformed sources exercise the syntax-error boundary separately.

Two reversible patches preserve the original vendor inventory. The rational
library's 13 tests join the existing dependency qualification suites. Engine `.15`
retains package version 0.3.0 and schema 3. Public `valueExpression` remains original
source text; no value evaluation or AST API is added. Numeric/context combinations,
general real-valued execution, normative interchange, full lexical coverage and
formatter round-trip behavior remain unqualified.

## KERNEL-014: reserved-word vocabulary and name contexts

The six-test reserved-word suite checks every entry in the published lists:
128 SysML words (8.2.2.1.2) and 101 KerML words (8.2.2.6). The independent
[vocabulary fixture](../crates/sysml-kernel/tests/data/reserved-words.json) records
the pinned PDF hashes and separates a provisional implementation policy from
those normative lists. Each specified word is rejected bare and accepted quoted
in five contexts: long identification, short identification, alias names, alias
targets and qualified expression references. Words exclusive to the other dialect
remain legal names. Upper-case spellings and identifier prefixes/suffixes do not
accidentally become keywords. Every specified reserved short name also has an
exact diagnostic byte/UTF-16 range check after Unicode trivia.

The audit found a specification ambiguity rather than a confirmed parser defect:
`new` is absent from both published reserved-word lists, but KerML 8.2.5.8.3 uses
it as the constructor keyword. OMG tracks the omission in **SYSML21-641** and
**KERML11-200**, both open at the 2026-10-17 review. See the
[SysML issue list](https://issues.omg.org/issues/spec/SysML) and
[KerML issue list](https://issues.omg.org/issues/spec/KerML?view=ALL).
The existing reservation is retained pending resolution. Separate tests distinguish
constructor expressions from references/invocations using the quoted name `'new'`,
and preserve rejection of malformed constructors. This compatibility decision is
not counted as resolution of a normative obligation.

LEX-010/011 exercise quoted keywords, quoted short names and aliases, dialect and
case boundaries, source preservation and an exact feature-valued bound through the
full library-backed facade. A native snapshot/JSON round-trip test validates its
query handles and resolved references. Both fixtures join the entire existing
native/WASM workspace and its unresolved-import variant. LEX-012/013 check bare
reserved short-name rejection, following-declaration recovery and conservative
cardinality invalidation in a clean dependent file.

This is a qualification increment: production Rust, vendor patches, engine `.15`,
package version 0.3.0 and schema 3 are unchanged. The four new public cases cover
representative semantic contexts; the exhaustive vocabulary tests are syntax
checks, not all contextual name-resolution rules. Complete grammar, Unicode
printability, constructor execution and formatter/editing behavior remain open.

## KERNEL-015: isolate connection-end uncertainty

A missing or ambiguous reference anywhere inside a connector previously made
**every** end unsupported. For `connect a.missing to b`, the safe refusal of the
partial `a` chain also hid the independently resolved `b` end. A missing type on an
unrelated member in the connection body similarly hid two valid endpoints.

The facade now indexes unresolved reference carriers and their owning ancestors
once per analysis, then checks each end feature's own subtree. Synthesized chain
features and flow-end redefinitions are reached through ownership; referenced
semantic targets are not followed. Shared ancestry is visited once, and a visited
set also bounds unexpected ownership cycles. The private index has tests for a
10,000-node spine shared by 10,000 unresolved carriers and for empty/cyclic input.
This avoids the former per-connector scan of every unresolved source span.

The first three public regressions failed before the change and pass afterward.
Four full-library Rust tests cover absent first/middle/last links, deep retained
prefixes, ambiguous names, n-ary ends, body errors, unresolved end multiplicities,
flow prefix/redefinition ownership, implicit ends and JSON/query round-trip.
CONN-002 now expects its healthy second end to resolve; the broken first end
remains unsupported. CONN-004 and CONN-005 add 21 assertions across SysML and KerML.
No other earlier matrix expectation changes.

CONN-004 joins the complete existing native/WASM integration corpus. Its
unresolved-import variant now uses all 32 supported document slots. Both dialects
also run together in a separate parity request, including CONN-005. Viewer tests
check that a healthy peer still has a visible card, an unsupported partial end
never supplies a card, and unrelated body errors do not erase resolved edges.
N-ary rendering retains its explicit unavailable status.

This is a projection improvement, not a claim that the connector is semantically
valid. An unresolved reference anywhere within an end, including its multiplicity,
still conservatively blocks that end. Partial chains remain navigation evidence;
their display spelling may be a prefix rather than the complete written source.
Implicit ends remain unsupported. Flow compatibility, end typing, the full
connector constraints, all contextual resolution and every syntax-recovery shape
remain unqualified. KerML 8.3.4.5.3 and SysML 8.2.2.13.1 establish the separate
end/reference structure; the uncertainty boundary is an explicit product policy.

The first combined integration run exposed eight arithmetic/predicate failures:
canonical publication exceeded its 10-million-step allowance after adding the
connector fixture. The [initial integration record](evidence/connections-initial-integration-macos-arm64.json)
retains that failure. Complete checked callee projections were copied for every
invocation, including vectors the consumer never needed. The existing stamped
constructor/invocation batch now shares immutable projections. Consumers still
charge every traversal and copy the vectors retained in their public reports.
Mutation invalidates the batch; a new dependency test covers budget exhaustion,
retry, same-length edits and unchanged previously retained projections. The full
corpus and the original work allowance remain intact.

Engine `.16` retains schema 3 and package version 0.3.0. Endpoint policy stays in
the reusable Rust facade. Two existing reversible dependency patches record the
projection-sharing change; no UI language rules are added.

## KERNEL-016: declaration families and explicit typing boundaries

The breadth audit now exercises nominal definition/usage construction for 24 SysML
kinds: attributes, enumerations, occurrences, items, parts, ports, connections,
interfaces, allocations, flows, actions, states, calculations, constraints,
requirements, concerns, cases, analysis cases, verification cases, use cases,
views, viewpoints, renderings and metadata. Ten `FAMILY-*` cases check metaclasses,
resolved explicit type identity and exact source extents, plus rejection of an
incompatible type for each kind. Each kind has its own bounded assessment; the
shared case does not qualify every production or constraint in its family.

`TYPE-005` crosses the SysML/KerML boundary for DataType, Class, Structure,
AssociationStructure, Interaction, Behavior, Function, Predicate and Metaclass.
Reading the normative XMI property redefinitions found two bugs in the dependency's
explicit-typing restriction table:

- `StateUsage::stateDefinition` has range Behavior, including when inherited by
  ExhibitStateUsage. The prior StateDefinition-only restriction rejected valid
  kernel behaviors and action definitions. Direct, multiple, related and exhibited
  behavior types now pass; data types still fail.
- `ConcernUsage::concernDefinition` narrows RequirementUsage's type to
  ConcernDefinition. General RequirementDefinition targets were incorrectly
  accepted. These now produce semantic errors while the reference still resolves.
  Requirements typed by concerns remain valid, and redundant ancestor/descendant
  concern types do not become independent types.

The [pre-fix probes](evidence/families-before-macos-arm64.json) preserve the `.16`
failures. `TYPE-006/007` and four Rust tests check diagnostic source locations,
valid peers, mixed types, the existing independent-type limit, unresolved targets,
syntax-only separation and JSON/query round-trip. Missing types still produce
resolution and incomplete-query capability findings rather than an incompatible
resolved-type error. The new mixed-workspace Node test runs all 13 cases together
in both file orders and compares complete native/WASM snapshots. Diagnostic checks
retain each case's document scope; no existing integration documents were removed.

Two private dependency tests exercise the validator directly. Patch 0016 is
independently reversible to the pinned upstream file. Engine `.17` retains schema
3 and package 0.3.0; there is no new UI language rule or wire shape. The benchmark
adds behavior declarations and concern typing to all earlier workloads.

This is a representative declaration/typing baseline, not completion of the
whole-language audit. Body grammar, derived/implicit metamodel construction,
directed-parameter exceptions, inferred typing and complete family semantics
remain open. The [review queue](requirements.md#breadth-audit-declaration-and-explicit-typing-baseline)
records the next obligations. The existing AST/graph functionality remains a
candidate outside the reviewed public behavior.

## KERNEL-017: validation audit and state subaction rule identity

The [named-rule audit](validation-audit.md) joins the complete 180-rule pinned
validation inventory to source search locations, registry declarations and direct
assessment edges. Its JSON report fingerprints every scanned Rust file. Search
hits include comments and private tests; the report never promotes them to tested
implementation. Rules without a name match remain a locate-or-implement queue.
The inventory gate checks the generated report, and dedicated tests preserve the
distinction between source mentions, clause mappings, planned tests and execution.

Five new cases review state subaction uniqueness and view rendering multiplicity.
`STATE-001/002` cover owned entry/do/exit actions on definitions and usages,
anonymous/named actions, local repetition of inherited kinds and valid peers.
The prior engine already rejected duplicates, but its public diagnostic codes
omitted `State` before `SubactionKind`. Patch 0017 maps both old labels to the
pinned XMI identifiers while retaining existing message text, following the
existing view-rendering alias policy. Engine `.18` keeps schema 3 and package
0.3.0; the validation predicates and wire shape are unchanged.

`VIEW-001/002` retain absent/single/diamond-inherited renderings and rejection of
owned duplicates. `VIEW-003` exposes KERNEL-018: the validator misses multiple
inherited renderings and local-plus-inherited renderings on both a definition and
a usage. The `.18` record retained four expected failures. The `.19` correction
is described below. The XMI uses `featureMembership`, which includes inherited
FeatureMemberships. Redefinition and incomplete ancestry must be reviewed before
repairing that predicate. The [pre-fix probes](evidence/validation-before-macos-arm64.json)
retain actual `.17` outputs for both issues, with the existing-binary limitation.

The acceptance harness now scopes diagnostic presence/absence to a named owner's
source in the same document. Harness regressions reject findings on peers, other
documents and overlapping-but-not-contained spans. The family integration corpus
adds all five cases in both document orders, comparing complete native/WASM
snapshots and preserving each case's expectations, including named known gaps.
The benchmark adds the duplicate-subaction case and its canonical-code assertions.

These are four additional bounded rule assessments, not closure of whole state
or view semantics. Membership kinds and derived state action slots are still
absent from the facade. The [review plan](requirements.md#validation-rule-audit)
records these and the remaining validation work.

## KERNEL-018: inherited view rendering memberships

The pinned ViewDefinition and ViewUsage rules select ViewRenderingMembership from
`Type::featureMembership`, which unions owned and inherited feature memberships.
Engine `.19` applies the existing membership projection to these checks, retaining
membership identity, visibility, imports and actual/transitive redefinition. It
also removes view-rendering participation in compatibility same-name shadowing:
`checkRenderingUsageRedefinition` requires redefinition of the library feature
`Views::View::viewRendering`, not automatic replacement of another user rendering.
The canonical rule codes and snapshot schema 3 remain unchanged.

`VIEW-003` now requires all four previously missing findings. `VIEW-004` adds
explicit and transitive replacements, private/protected inherited renderings,
public/private re-export, root imports and ordinary RenderingUsage memberships.
`VIEW-005` rejects same-named local renderings and distinct renderings inherited
through protected memberships or public imports, with exact member-set assertions.
`VIEW-006` preserves unknown outcomes for unresolved bases, redefinitions and
imports and for cyclic heritage. Two owned renderings still produce a definite
error, and complete invalid peers remain diagnosable in that same source.
The [pre-fix record](evidence/view-renderings-before-macos-arm64.json) uses the
existing `.18` binary and records its hash; it is not a fresh baseline build.

The completeness preflight follows the membership projection's base/import
namespaces, owned features and explicit redefinition chains. It audits private
import dependencies conservatively because import-all paths can admit them.
Syntax recovery in any contributing user document also prevents an inherited-count
verdict. Nested expression-body failures do not automatically poison an enclosing
view's membership set. Five private regressions cover reduction, incomplete input,
repeat queries, this dependency boundary and refusal of elements added after the
static index was built. The harness can now assert the
absence of a particular diagnostic code without requiring unrelated cycle errors
to disappear. The three new public cases join the full previous family/body corpus
in both file orders for native/WASM comparison.

These are bounded validation assessments. General semantic metadata, all filtered
and import-all interactions, malformed raw graph injection and complete view
semantics remain unqualified. An absent diagnostic or a complete membership query
is not certification of whole-model validity. The correction is recorded in
patches 0018/0019 and the combined driver patch 0017; all original vendor hashes
remain immutable. The package remains 0.3.0.

## KERNEL-019: parallel-state transition incidence

Engine `.20` checks the inverse `TransitionUsage::source` and `target` associations
for actions directly owned by a parallel StateDefinition or StateUsage. The pinned
rules select `ownedAction` and `nestedAction` respectively; both derive from owned
feature memberships. A transition may be declared in another state or source file.
Plain Successions, nonowning aliases, inherited actions and actions owned by a
nested region do not belong to this direct incidence set.

The old ownership-only check both rejected legal cases and missed externally
declared transitions. `STATE-003` retains those positive boundaries; `STATE-004`
requires incoming and outgoing findings, including performed actions and entry/
do/exit memberships. `STATE-005` retains missing first/middle/final chain links,
ambiguous endpoints, complete peers and independently known opposite endpoints.
`STATE-007` checks imported cross-file transitions and diagnostic source ownership.
The full family/body integration corpus includes all these documents in both file
orders for native/WASM comparison.

A private endpoint preflight guards the shared derived queries: all written chain
links must resolve, and a known ActionUsage keeps its own `featureTarget` identity
rather than following its ReferenceSubsetting. Each endpoint has an independent
work allowance, so missing opposite endpoints cannot hide a known violation.
Repeated queries, explicit-index bounds and exhausted proof work have private
regressions. Each violating state receives one canonical rule diagnostic.

The same preflight corrects target-kind validation. `STATE-006` distinguishes
resolved non-action targets, ReferenceUsage targets referencing actions, valid
PerformActionUsage targets and incomplete chains. An unresolved chain no longer
becomes a known wrong-kind target. Resolution diagnostics remain visible.

The [pre-fix probes](evidence/state-transitions-before-macos-arm64.json) retain the
original source and diagnostics from the existing `.19` binary, whose hash is
recorded; they are not a fresh baseline build. The incomplete-source probe predates
the additional ambiguous-reference cases in the final fixture. Patches 0018/0020
record the structural-check removal and action-validator correction; original
vendor hashes remain unchanged. Package 0.3.0 and snapshot schema 3 are unchanged.

These assessments do not close complete state or transition semantics. The target
projection follows the current lowering's second end of the first succession,
including its unspelled source end. Missing-succession validation, arbitrary
nonbinary/raw connector targets, implicit end construction, every source/target
derivation, triggers, guards and effects remain open. State-slot and membership-
metaclass facade queries still need contracts and independent qualification.

## KERNEL-020: owned state queries and triggered source identity

Schema 4 adds `Element.stateSubactions` and the native
`SnapshotQuery::state_subactions` operation. Package and app versions advance to
0.4.0; consumers must reanalyze older snapshots. The new discriminated result
separates complete owned binding enumeration, unknown input and non-applicability.
It uses the shared semantic library's state-subaction query, then verifies every
owned StateSubactionMembership is represented with consistent action ownership.
Each record retains entry/do/exit kind, membership identity and action identity
in declaration order. Named and anonymous actions are preserved, and performed
actions keep their own identities rather than their referents.

`STATE-009/010` check state definitions/usages, declared order, inherited-versus-
owned actions, aliases/ordinary actions, anonymous actions, duplicates, missing
action types and library navigation stubs. Duplicate kinds remain in the complete
binding enumeration alongside their semantic diagnostic; complete is not model
validity. `STATE-011` checks recovered source and an independent document: any
syntax error in a state's own document prevents a complete body enumeration,
while known declarations in other files retain their owned-query results.
The Rust tests also cover JSON round trip, old-schema rejection, missing handles,
omitted/reordered/duplicated bindings and inconsistent target identities.

The existing full family/body corpus grows with the syntax-valid query and trigger
fixtures. Recovery is exercised in a separate five-document native/WASM request,
in both file orders, because it intentionally changes the existing workspace-wide
import/cardinality availability policy. No earlier corpus fixture is removed.

Engine `.21` also corrects `validateTransitionUsageTriggerActions`. The previous
check could reject a state source because an unrelated alias referred to an action,
and could miss a PerformActionUsage source that referred to a StateUsage. The rule
now consumes shared `triggerAction` derivation and the checked source proof from
KERNEL-019. `STATE-008` retains these cases, chained state/perform sources, ordinary
actions, untriggered transitions and missing/broken sources. An unknown source
cannot establish the rule's non-StateUsage antecedent. The private ambiguity case
now contains directly owned candidate actions rather than only nonowning aliases.

The [pre-fix record](evidence/state-queries-before-macos-arm64.json) preserves the
actual exploratory source and `.20` findings from the unchanged previously
qualified executable. It is not a fresh baseline build. The final trigger fixture
adds a broken-chain case and uses its own package name.

The [pinned XMI review](evidence/state-property-review.json) records six state-slot
expressions returning memberships despite their ActionUsage property types, and
three transition guard/effect expression inconsistencies. No normative artifact
is amended. The new API exposes unambiguous binding/kind/action facts separately;
it does not qualify the contradictory derived slots. General membership role
properties, transition feature collections, implied body construction, all
trigger/guard/effect constraints and execution remain open.

## KERNEL-021: transition input construction and membership validation

Engine `.22` constructs the first owned transition input that the previous
lowering omitted. A transition without a trigger now has its source input; a
triggered transition has that input followed by its separate payload. The
[pre-fix regression](evidence/transition-construction-before-macos-arm64.json)
records the actual raw canonical-model failure: zero and one owned inputs where
the specification requires at least one and two. It was executed before the
production edits, rather than inferred from the corrected implementation.

`STATE-012` checks ordered parameter and transition-feature memberships through
the full-library public query. The first input has no declared name, but its
effective name is `transitionLinkSource`; payload identity remains distinct.
Native regressions exercise canonical and legacy graphs, explicit and target
transition forms, guards, effects, guarded successions and directed body features.
Owned `in` and `inout` features count as inputs; inherited and output-only features
do not satisfy the owned-input requirement.

The syntax tree retains whether the first parameter comes from the grammar or
semantic construction. Explicit transition productions supply EmptyParameterMember.
Guarded/default action-successions omit it, so the shared TransitionUsage input
requirement supplies an implied parameter. Conditional target syntax uses the
owning state/action context, including state subtypes. A regression checks the
relationship's `isImplied` value before and after formatting and reparsing.
Guarded entry transitions use GuardedTargetSuccession even inside a state. The
parser preserves that distinction across repeated guards and intervening target
successions, and resets it at an ordinary state member. The additional
[pre-correction regression](evidence/transition-construction-entry-before-macos-arm64.json)
records the failure found during review of the in-progress implementation; the
final native test checks entry and ordinary target provenance before and after
formatting.

Five registered named constraints validate transition input counts, state
subaction membership owners, transition-feature membership owners, trigger
AcceptActionUsage members and effect ActionUsage members. `STATE-013` checks
invalid part/action owners beside valid state definitions, usages and exhibited
states. Private graph mutations supply negative cases that the textual grammar
cannot construct, including an action that is not an accept action and a reference
that is not an action. Structural and semantic diagnostics remain separate.

The conformance harness can select an exact membership metaclass and assert
declaration order; its positive and negative regressions retain the default
order-insensitive behavior for earlier cases. The family parity corpus retains
every prior fixture and uses all 32 available document slots. Package/app version
0.4.0 and snapshot schema 4 are unchanged: this corrects constructed model facts
without changing the public wire shape.

This increment does not close complete transition semantics. Source/succession
bindings, payload subsetting chains, all guard constraints, implicit specialization
closure, arbitrary raw graph consistency and normative interchange still need
qualification. Normalized JSON lifting chooses explicit transition grammar;
external-payload provenance round trips are not established by the textual
formatting regression.

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

On macOS arm64, the current matrix has **130 passing conformance cases**, **no
known-gap cases**, and **no planned cases**. All **87 Rust kernel tests**, **50 Node
tests**, and **286 dependency proof tests** pass. The dependency suites cover
lexical boundaries (5), rational arithmetic (13), explicit usage typing (2), view
rendering validation (5), transition incidence/target/trigger and membership/input validation (11), type relations (41), model-level evaluability (33),
cardinality (34), invocation bindings (19), constructor bindings (17), dynamic graph
(19), dynamic invocations (27), owned results (4), semantic ownership (5), end
constancy (10) and checked namespace operations (41). The corpus and the 32 import-rule
partitions are included in the Rust suite, not additional independently qualified
specification counts. Native and WebAssembly outputs match exactly, including the
expanded family/body corpus in both file orders.

The [KERNEL-021 validation record](evidence/transition-construction-macos-arm64.json)
records the transition input construction and five named input/membership rules,
two added cases and 22 added assertions. Five native tests cover raw input order,
canonical/legacy graphs, public identity, ordinary and entry-transition provenance
and formatting. The requirements extractor reproduces all 17 pinned artifacts
with seven previously omitted colon-terminated production headers retained; the
inventory now contains 627 grammar productions and 6,174 overlapping source units.
The prior [KERNEL-020 validation record](evidence/state-queries-macos-arm64.json) records
the schema-4 owned state-subaction query and triggered-source correction, four
added cases and 40 added assertions. It retains source-local recovery behavior,
binding integrity checks and explicit limits on derived state/transition slots.
The [KERNEL-019 validation record](evidence/state-transitions-macos-arm64.json) records
the parallel-state and target-kind corrections, five added cases and 94 added
assertions. The prior [KERNEL-018 record](evidence/view-renderings-macos-arm64.json)
retains the inherited-rendering correction at its original source fingerprint.
The prior [KERNEL-017 record](evidence/validation-macos-arm64.json) retains the
named-rule audit and state diagnostic-code correction at its earlier engine/input
fingerprints. The earlier [family validation](evidence/families-macos-arm64.json)
retains the state/concern typing findings and its original scope. The prior
[connection validation](evidence/connections-macos-arm64.json) and
[vocabulary audit](evidence/keywords-macos-arm64.json) remain available. This finite passing matrix does not establish complete standards conformance.

Rust formatting, Clippy with warnings denied, TypeScript checking, Prettier,
generated-contract consistency, source/patch verification and inventory checks
pass. The saved [matrix execution](evidence/conformance-macos-arm64.json) and
[requirements execution](evidence/requirements-macos-arm64.json) record the current
source, patch and fixture fingerprints. Windows/Linux execution remains pending CI.

The [initial `.15` conformance attempt](evidence/conformance-numeric-initial-macos-arm64.json)
retains 89 passes and three process timeouts (SUB-002, NAME-003 and MULT-026).
The [power-event record](evidence/conformance-numeric-power-events-macos-arm64.json)
shows 255-, 901- and 901-second system sleep intervals during those attempts,
consistent with their prolonged wall-clock durations. All [nine focused retries](evidence/conformance-numeric-retries-macos-arm64.json)
passed in 2.0–4.1 seconds, with identical snapshots within each case. Subsequent
validation used a temporary process-scoped sleep assertion, released at completion.
The 30-second conformance and 60-second benchmark deadlines are unchanged. Earlier unexplained
benchmark timeouts below remain separate evidence.

The [KERNEL-021 benchmark](evidence/benchmark-transition-construction-macos-arm64.json)
records 162 successful samples across 27 workloads. All 25 earlier workloads remain;
the two additions check ordered transition input/feature memberships and state
subaction owner diagnostics on every sample. Repeated and native/WASM snapshots
agree exactly. Measurements include fresh process startup and do not establish
production-model latency guarantees.

| Workload | Native median (ms) | WASM median (ms) | Native/WASM maximum RSS (MiB) |
| --- | ---: | ---: | ---: |
| Transition input construction | 2,035 | 2,474 | 405.1/360.6 |
| State subaction membership owners | 2,012 | 2,456 | 397.6/342.9 |

The [KERNEL-020 benchmark](evidence/benchmark-state-queries-macos-arm64.json)
records 150 successful samples across 25 workloads. All 23 earlier workloads remain;
the two additions check ordered state-subaction bindings and triggered transition
source diagnostics on every sample. Repeated and native/WASM snapshots agree
exactly. Measurements include fresh process startup and do not establish
production-model latency guarantees.

| Workload | Native median (ms) | WASM median (ms) | Native/WASM maximum RSS (MiB) |
| --- | ---: | ---: | ---: |
| Owned state subaction queries | 2,025 | 2,469 | 406.0/355.4 |
| Triggered transition sources | 2,026 | 2,468 | 412.9/356.9 |

The [KERNEL-019 benchmark](evidence/benchmark-state-transitions-macos-arm64.json)
records 138 successful samples across 23 workloads, retaining every previous
scenario and adding parallel-state transition validation. Each sample checks its
expected endpoint-incidence diagnostics. Repeated and native/WASM snapshots agree
exactly. The new workload's median is 2,009 ms native and
2,446 ms WASM; maximum peak RSS is
419.2 MiB and 355.2 MiB respectively. These observations include fresh
process startup and do not establish production-model latency guarantees.

The [KERNEL-018 benchmark](evidence/benchmark-view-renderings-macos-arm64.json)
records 132 successful samples across 22 workloads, retaining every previous
scenario and adding inherited view-rendering validation. Each sample checks the
expected memberships and diagnostic codes. Repeated and native/WASM snapshots
agree exactly. The new workload's median is 2,309 ms native and 2,500 ms WASM;
maximum peak RSS is 404.0 MiB and 359.9 MiB respectively. These observations include
fresh process startup and are not production-model latency guarantees.

The [KERNEL-017 benchmark](evidence/benchmark-validation-macos-arm64.json) records
126 successful samples across 21 workloads, retaining all earlier scenarios and
adding state subaction validation with canonical-code assertions. Native/WASM and
repeat snapshots agree exactly. The new workload's median is 2,026 ms native and
2,432 ms WASM, with maximum peak RSS of 391.8 MiB and 353.8 MiB respectively.
These measurements include fresh process startup and are not latency guarantees.

The earlier [KERNEL-016 benchmark](evidence/benchmark-families-macos-arm64.json) records
120 successful samples: all 18 earlier workloads plus concern typing and behavior
declarations, each in three fresh native and three fresh WASM processes. All
repeats and native/WASM snapshots agree exactly; no timeout occurred. The new
workloads run their matrix assertions on every sample. Their observed values are:

| Workload | Native median (ms) | WASM median (ms) | Native/WASM maximum RSS (MiB) |
| --- | ---: | ---: | ---: |
| Concern typing | 2,442 | 4,153 | 384.5/315.9 |
| Behavior declarations | 2,079 | 2,618 | 384.3/332.3 |

The preceding [connection benchmark](evidence/benchmark-connections-macos-arm64.json)
records three fresh processes per native/WASM workload, for 108 successful samples.
All repeated native and WASM snapshots agree exactly. Each connection sample
passes all 14 CONN-004 assertions; all 17 prior workloads remain included.
The earlier [numeric benchmark](evidence/benchmark-numeric-macos-arm64.json)
and KERNEL-013 debug overflow-check evidence remain available.

The following table retains the KERNEL-015 observations for historical comparison:

| Workload | Native median (ms) | WASM median (ms) | Native/WASM maximum RSS (MiB) |
| --- | ---: | ---: | ---: |
| Connection ends | 2,237 | 2,637 | 387.5/354.4 |
| Numeric literals | 2,175 | 2,605 | 393.3/350.6 |
| Lexical boundaries | 2,165 | 2,521 | 394.7/347.6 |
| Strings | 3,031 | 3,203 | 391.8/361.3 |
| Empty comparisons | 3,291 | 3,392 | 384.2/368.4 |
| Null-coalescing | 2,436 | 2,973 | 393.5/361.5 |
| Scalar equality | 2,961 | 3,181 | 406.9/361.1 |
| Boolean predicates | 2,754 | 3,256 | 383.2/359.5 |
| Ordered predicates | 2,578 | 3,299 | 408.3/361.0 |
| Compound ranges | 2,803 | 4,781 | 387.1/323.5 |
| Independent bounds with missing base | 2,931 | 3,431 | 387.9/346.8 |
| Imported memberships | 2,081 | 2,567 | 385.0/342.7 |
| Viewer example | 2,092 | 2,555 | 390.4/354.8 |
| Attribute providers | 2,301 | 2,636 | 380.0/340.2 |
| Arithmetic providers | 2,229 | 3,065 | 385.2/362.3 |
| Conditional providers | 3,228 | 3,306 | 367.8/352.6 |
| 100 usages | 2,242 | 2,643 | 397.5/354.3 |
| 1,000 usages | 2,541 | 2,997 | 413.8/358.3 |

The earlier `.6` isolation run had an initial native sample time out at 60
seconds; a direct retry completed in 3.2 seconds, and its full 48-sample rerun
passed. The cause of that earlier timeout remains unresolved. No timeout occurred
in the recorded `.7`, `.8` or `.9` runs.

The [initial `.10` attempt](evidence/benchmark-predicates-initial-macos-arm64.json) stopped
after 45 successful samples when the first attribute-provider WASM sample exceeded 60 seconds.
It also retained an 18,985 ms Boolean WASM sample (the other two took 4,100 and 3,329 ms).
Three direct attribute-provider retries completed in 2,628, 2,537 and 2,480 ms with identical
snapshots. The separate complete [72-sample rerun](evidence/benchmark-predicates-macos-arm64.json) is retained. The initial timeout
remains unexplained. No benchmark timeout occurred in the `.11` through `.16` runs; these results do not
establish a stable latency bound.

Measurements include startup, bundled-library analysis and serialization. They are
observations on this Mac, not cross-platform latency guarantees. Canonical
preparation is a whole-model pass. The facade runs its model scan before projecting
cardinality; the separate direct-query regression exercises the first-query budget.
Compared with the retained [numeric run](evidence/benchmark-numeric-macos-arm64.json),
these uncontrolled runs do not isolate incremental query cost or establish a
regression budget. Large import-heavy production models and incremental performance
remain open engineering work.

The production web build and macOS arm64 app and DMG builds pass. The local app
is `src-tauri/target/release/bundle/macos/SysML Viewer.app` (version 0.4.0, 12.06 MiB), with a
4.36 MiB installer under `src-tauri/target/release/bundle/dmg/`. This run
checks automated viewer integration and packaging; it does not repeat the earlier
interactive file-picker/navigation checks or establish Windows/Linux packaging.
