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

## KERNEL-004: expression proof isolation (open)

The combined import-guard workspace revealed an existing conservative limit:
`part def Base :> MissingBase` can make unrelated arithmetic and conditional bounds
unknown. The same request was reproduced against the preceding commit `1adeadc`
(engine suffix `.4`), confirming that import-query projection did not introduce it.
The native/WASM suite now retains both the valid workspace's exact results and
this incomplete workspace's unknown results, with parity checked for both requests.
This is a limitation of proof availability, not evidence that the independent
arithmetic is invalid. Dependency-local proof isolation remains work to do.

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

On macOS arm64, the completed run has **69 passing conformance cases**, **no
known-gap cases**, and **no planned cases in the current matrix**. All **40 Rust facade tests**,
**30 Node tests**, and **193 dependency proof tests** pass.
The dependency suites cover type relations (41), model-level evaluability (26),
cardinality (22), invocation bindings (16), constructor bindings (16), dynamic graph
(17), owned results (4), end constancy (10) and checked namespace operations (41). The corpus and the 32
import-rule partitions are included in the Rust suite, not additional independently qualified
specification counts. Native and WebAssembly outputs match exactly.

Rust formatting, Clippy with warnings denied, TypeScript checking, Prettier,
generated-contract consistency, source/patch verification and inventory checks
pass. The saved [matrix execution](evidence/conformance-macos-arm64.json) and
[requirements execution](evidence/requirements-macos-arm64.json) record the current
source, patch and fixture fingerprints. Windows/Linux execution remains pending CI.

The latest [import-query benchmark](evidence/benchmark-imports-macos-arm64.json)
records three fresh processes per native/WASM workload, for 42 samples. All
repeated native and WASM snapshots agree exactly.

| Workload | Native median (ms) | WASM median (ms) | Native/WASM maximum RSS (MiB) |
| --- | ---: | ---: | ---: |
| Imported memberships | 2,187 | 2,570 | 394.3/340.0 |
| Viewer example | 2,165 | 2,671 | 393.5/354.4 |
| Attribute providers | 2,246 | 2,713 | 392.3/346.5 |
| Arithmetic providers | 2,622 | 3,132 | 382.8/353.7 |
| Conditional providers | 2,937 | 3,514 | 380.8/351.2 |
| 100 usages | 2,104 | 2,549 | 397.0/353.2 |
| 1,000 usages | 2,405 | 2,709 | 405.2/370.8 |

Measurements include startup, bundled-library analysis and serialization. They are
observations on this Mac, not cross-platform latency guarantees. Canonical
preparation is a whole-model pass. Compared with the retained
[conditional run](evidence/benchmark-conditionals-macos-arm64.json), these
uncontrolled runs do not isolate incremental query cost or establish a regression
budget. Large import-heavy production models and incremental performance remain
open engineering work.

The production web build and macOS arm64 bundle build pass. The new local bundle
is `src-tauri/target/release/bundle/macos/SysML Viewer.app` (11.97 MiB). This run
checks automated viewer integration and packaging; it does not repeat the earlier
interactive file-picker/navigation checks or establish Windows/Linux packaging.
