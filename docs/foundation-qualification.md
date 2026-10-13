# Kernel foundations: membership, names and cardinality

This update closes public-query and correctness gaps on the path to full language
coverage. It does not establish full SysML/KerML conformance. The target remains a
reusable language kernel; editing, interchange and complete graphical notation have
separate conformance obligations.

## Implemented behavior

Snapshot schema 2 adds membership records and checked cardinality results. Native
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

## Remaining foundation work

Complete imported membership enumeration remains planned (IMP-020). Filtered imports,
all contextual name-resolution rules, implicit metamodel construction and every
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

On macOS arm64, the completed run has **64 passing conformance cases**, **no
known-gap cases**, and **one planned case**. All **35 Rust facade tests**, **27 Node
tests**, and **152 dependency proof tests** pass.
The dependency suites cover type relations (41), model-level evaluability (26),
cardinality (22), invocation bindings (16), constructor bindings (16), dynamic graph
(17), owned results (4) and end constancy (10). The corpus and the 32 import-rule
partitions are included in the Rust suite, not additional independently qualified
specification counts. Native and WebAssembly outputs match exactly.

Rust formatting, Clippy with warnings denied, TypeScript checking, Prettier,
generated-contract consistency, source/patch verification and inventory checks
pass. The saved [matrix execution](evidence/conformance-macos-arm64.json) and
[requirements execution](evidence/requirements-macos-arm64.json) record the current
source, patch and fixture fingerprints. Windows/Linux execution remains pending CI.

The latest [conditional benchmark](evidence/benchmark-conditionals-macos-arm64.json)
records three fresh processes per native/WASM workload, for 36 samples. Median
whole-process times were 2,166/2,606 ms for the viewer example, 2,537/2,751 ms for
attribute providers, 2,651/3,106 ms for arithmetic providers, 3,519/3,374 ms for
conditional providers, 2,618/2,653 ms for 100 usages and 2,590/2,917 ms for 1,000
usages. Maximum peak RSS for the largest workload was 407.7/362.7 MiB. These include startup, bundled-library analysis and
serialization; they are observations on this Mac, not cross-platform latency
guarantees. Canonical preparation is a whole-model pass. Results vary in both
directions compared with the retained [arithmetic run](evidence/benchmark-arithmetic-macos-arm64.json);
these uncontrolled runs do not isolate the incremental cost or establish a regression budget.
Performance remains an open engineering concern alongside semantic coverage.

The production web build and macOS arm64 bundle build pass. The new local bundle
is `src-tauri/target/release/bundle/macos/SysML Viewer.app` (11.95 MiB). This run
checks automated viewer integration and packaging; it does not repeat the earlier
interactive file-picker/navigation checks or establish Windows/Linux packaging.
