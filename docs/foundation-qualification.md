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
arithmetic provider cases retain the known gap described below. Unknown providers and
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
- All 32 combinations of SysML/KerML, membership/namespace import, four written
  visibility forms, and root/nested ownership for `validateImportTopLevelVisibility`.
- Syntax and structural validation of all 94 pinned library source files.

Library syntax acceptance is not library semantic qualification or exhaustive
textual grammar coverage. Native/WebAssembly snapshot parity includes aliases,
exact large integers and unknown providers. Existing viewer behavior remains tested.

The initial inherited-alias expectation was reviewed against the normative XMI:
`Type::removeRedefinedFeatures` includes each Feature itself in the reduction set.
Distinct inherited memberships of that same Feature can therefore remove each
other. IMP-021 preserves that boundary; IMP-014 tests aliases of a non-Feature
nested definition, which retains its inherited memberships. This is an explicit
standards-based expectation, not an adjustment to make an unexplained result pass.

Source authority: [OMG KerML 1.0](https://www.omg.org/spec/KerML/1.0) and
[OMG SysML 2.0](https://www.omg.org/spec/SysML/2.0), using the independently pinned
artifacts in `standards/sources.json`.

## KERNEL-001: feature-valued multiplicity providers

`tests/conformance/fixtures/mult-003.sysml` declares integer-valued features using
addition and multiplication, then references them from multiplicity bounds. The
expected intervals are `[5]` and `[3..6]`. The current checked reader returns
`unknown/unsupportedBound` instead. The matrix retains the expected values and
marks only these two assertions as known failures; the case is not counted as passing.
The literal-valued provider regression MULT-010 now passes with `[3]`. Its checked
model-level proof previously reported `IncompleteFeaturing` because the audited
domain admitted only exact KerML `Feature` instances. The second recorded patch
also admits exact SysML `AttributeUsage` instances with ordinary package ownership.
It retains checked reciprocal ownership, explicit/inverse featuring, feature-chain,
metadata, variable/variation, stale-evidence and work-budget checks. A failed proof
still cannot supply a numeric interval.

KerML 1.0 §8.3.4.8.5 allows the unfeatured-referent branch when the value expression
is model-level evaluable. SysML's usage-variation featuring rule requires additional
context, so variant ownership and variation attributes remain outside this audited
domain. This is a bounded extension, not a general proof that every attribute is
model-level evaluable.

MULT-011 covers aliases, transitive reference values, separate range endpoints,
zero and the exact integer `9007199254740993`. MULT-012 retains unknown results for
unvalued, cyclic, variation and type-owned contextual providers. Native query/JSON
and WebAssembly tests exercise these same sources. The dependency's private tests
also inject malformed flags, reciprocal ownership and carrier evidence, check
inverse TypeFeaturing, and exhaust/retry the work budget.

Reduced probes with the complete pinned library traced the remaining arithmetic
failures to incomplete checked callee/argument evidence. In the current LegacyV2 graph, checked operator argument
mapping reports `UnsupportedConfiguration`. Switching only to CanonicalV3 still
reports `UnmappedArgument`; model-level eligibility remains `IncompleteCallee`.
Synthetic minimal operator libraries can pass upstream tests without establishing
this full-library integration. The compatibility evaluator is not used as a fallback
because doing so would discard the checked reader's completeness guarantees.

Resolving the remaining arithmetic gap requires qualifying canonical operator construction, formal-input
selection and supplied-argument redefinition together, including negative cases for
unmapped, ambiguous and recursive providers. An upstream source change must use the
[documented patch/provenance procedure](dependency-upgrades.md). The featuring fix does not change operator construction or invocation binding.

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

On macOS arm64, the completed run has **56 passing conformance cases**, **one
known-gap case** (two arithmetic-bound assertions), and **one planned case**. All
**31 Rust facade tests**, **25 Node tests**, and **86 dependency proof tests** pass.
The dependency suites cover type relations (41), model-level evaluability (26),
and cardinality (19), including three new adversarial/regression tests in the patch.
The corpus and the 32 import-rule
partitions are included in the Rust suite, not additional independently qualified
specification counts. Native and WebAssembly outputs match exactly.

Rust formatting, Clippy with warnings denied, TypeScript checking, Prettier,
generated-contract consistency, source/patch verification and inventory checks
pass. The saved [matrix execution](evidence/conformance-macos-arm64.json) and
[requirements execution](evidence/requirements-macos-arm64.json) record the current
source, patch and fixture fingerprints. Windows/Linux execution remains pending CI.

The latest [provider benchmark](evidence/benchmark-providers-macos-arm64.json)
records three fresh processes per native/WASM workload, including the new provider
fixture. Median whole-process times were 1,537/1,917 ms for the viewer example,
1,574/1,884 ms for attribute providers, 1,564/2,222 ms for 100 usages and
1,889/2,350 ms for 1,000 usages. Maximum peak RSS for the largest workload was
276.7/292.0 MiB. These include startup, bundled-library analysis and serialization;
they are observations on this Mac, not cross-platform latency guarantees. Some
times are higher than the retained [foundation run](evidence/benchmark-foundations-macos-arm64.json);
these uncontrolled runs do not establish performance equivalence or a regression budget.

The production web build and macOS arm64 bundle build pass. The new local bundle
is `src-tauri/target/release/bundle/macos/SysML Viewer.app` (11.91 MiB). This run
checks automated viewer integration and packaging; it does not repeat the earlier
interactive file-picker/navigation checks or establish Windows/Linux packaging.
