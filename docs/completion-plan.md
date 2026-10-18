# Completion plan for SysML 2.0 and KerML 1.0

Planning baseline: 2026-10-17. This plan preserves the full language-coverage goal.
It defines the work to close that goal; it is not a conformance claim or an atomic
requirements catalogue. Existing assessments remain bounded until their complete
obligation sets and dependencies have been reviewed.

## Scope and completion claims

The source baseline is the pinned SysML 2.0 Part 1 Language specification and
KerML 1.0, including their normative metamodels, libraries and interchange schemas.
See [source authority and scope](requirements.md#source-authority-and-scope).
The separate Systems Modeling API and Services and SysML v1-to-v2 transformation
specifications are outside this baseline. Referenced standards must still be
reviewed wherever they impose dependencies on these language obligations.

We will track three deliverables separately, without using an early deliverable
to declare the whole goal complete:

1. **Language libraries:** complete textual parsing and construction, abstract
   model operations, resolution, derivations, validation, semantic analysis,
   library integration and interchange, available through reusable APIs.
2. **Tool conformance:** evidence for each claimed category. Clause 2 of both
   standards requires abstract-syntax CRUD/validation and project interchange for
   a conformant modeling tool. Textual notation adds concrete-syntax CRUD;
   SysML graphical notation adds its notation requirements. A read-only snapshot
   and a structural diagram alone do not satisfy these categories.
3. **Usable cross-platform application:** the viewer consumes those libraries,
   supports the intended model inspection and notation, and has verified macOS,
   Windows and Linux builds and behavior. Editing APIs can live in the libraries;
   the viewer does not need to become the owner of language semantics.

Semantic conformance requires a demonstrated interpretation of model meaning;
Clause 2 permits semantic analysis or execution. A simulator is not automatically
required. The semantic analysis profile must nevertheless identify its obligations
and limitations explicitly. Domain libraries are optional for a minimum tool claim;
all inventoried domain libraries remain in our full-coverage review queue. Their
applicability cannot be decided merely by whether current code supports them.

## Evidence baseline and uncertainty

The last committed qualification before this plan is KERNEL-021: 130 passing
conformance cases and 15 directly assessed named validation rules. KERNEL-022
extends accept construction and is separately qualified in
[foundation qualification](foundation-qualification.md). Neither checkpoint closes
a whole language family. Consult the latest records instead of treating these
historical counts as live status.

The reproducible catalogue contains 627 grammar productions, 175 metaclasses,
715 properties, 103 operations, 94 library model files and 270 schema definitions.
Its 724 constraint records comprise 180 names beginning `validate`, 235 beginning
`derive`, 208 beginning `check`, and 101 other records, including operation conditions.
These categories are source records, not independent engineering tasks. There are
also 1,571 numbered clauses and 1,008 prose candidates requiring contextual review.
All counts overlap; none is a denominator for completion.

The greatest uncertainty is how much existing engine code satisfies complete
obligations once its dependencies and counterexamples are examined. Code presence,
registry membership, parsed official examples and a clean diagnostic list are
insufficient evidence. Current implementations may need verification, repair or
replacement. The core architecture is retained unless evidence requires a change.

## Work packages and closure conditions

Every package below is open. These are review boundaries, not mutually exclusive
sets of specification requirements. A shared obligation has one reviewed record
with links to each dependent package.

| ID | Work package and source boundary | Required closure evidence | Dependencies |
| --- | --- | --- | --- |
| C01 | Conformance scope, applicability and obligation decomposition; both Clause 2 provisions, all clauses and prose candidates | Every source unit has a reviewed disposition; normative obligations have atomic acceptance criteria, dependencies and a claim category; conflicts have recorded resolutions | None |
| C02 | Lexical and textual syntax; SysML 8.2.2 and KerML 8.2 | Every production alternative and synthesis assignment reviewed; positive, negative and boundary cases; exact constructed model facts and source spans; contextual and recovery interactions | C01, iterates with C03–C10 |
| C03 | Root model, identity, relationships, dependencies, annotations, namespaces and imports; KerML Root plus corresponding SysML families | Ownership/inverse relationships, identity, visibility, distinguishability, filtered imports, aliases, cycles and contextual resolution tested through reusable APIs | C01–C02 |
| C04 | Types and features; KerML Core and SysML definition/usage | Explicit and implicit typing, specialization, subsetting, redefinition, conjugation, multiplicity, direction, variability, individuals and portions reviewed with derived properties and constraints | C03 |
| C05 | KerML Kernel expressions, functions, predicates, associations, behaviors, occurrences, interactions and metadata | Complete metaclass, operation and relationship obligations; expression typing and evaluation profiles; shared invocation, parameter/result and implicit construction rules | C04, C11 |
| C06 | SysML attributes, enumerations, occurrences, items, parts and ports | Body grammar, inherited/derived features, implicit bases, port conjugation and all applicable constraints, beyond nominal declaration/type examples | C04–C05, C11 |
| C07 | Connections, interfaces, allocations, flows and messages | End construction, direction/type/multiplicity compatibility, payloads, binding, succession and complete endpoint semantics | C04–C06, C11 |
| C08 | Actions, states, calculations and constraints | Every body/member form and action subtype; source/target/trigger/guard/effect relations; control nodes, parameters/results, time/change-event obligations and state derivations | C04–C05, C07, C11 |
| C09 | Requirements, concerns, cases, analysis, verification and use cases | Subject, actor, stakeholder and objective membership; satisfaction, verification, inclusion and associated derived/implicit semantics | C04–C08, C11 |
| C10 | Views, viewpoints, renderings, expose/filter behavior and semantic metadata | Membership construction and derivation, annotations, predicate/filter dependencies, rendering selection and semantic metadata effects | C03–C09, C11 |
| C11 | KerML and SysML libraries; both Clause 9 provisions and ten pinned archives | Review definitions and function bodies, identities, defaults and library dependency closure; qualify mandatory and selected domain behavior, not merely successful loading | Iterates with C04–C10 |
| C12 | Model interchange; KerML Clause 10, SysML Clause 2 and three schemas | Normative JSON and KPAR import/export; identities, references, metadata, implied relationships and cross-project dependencies; malformed inputs; round trips and independent-tool exchange | C03–C11 |
| C13 | Abstract and textual authoring APIs | Create/read/update/delete with validated ownership, reference integrity, source fidelity and defined identity lifetimes; invalidation/cancellation and editing sequences | C02–C12 |
| C14 | SysML graphical notation and viewer projection; 8.2.3 and related Clause 7 tables | Every claimed notation form and text rendering; semantic identity preserved through projection; graphical CRUD where claiming that category; usable inspection of all supported families | C03–C13 |
| C15 | Qualification and release engineering | Differential corpus with adjudicated differences, fuzzing, adversarial graphs, production-scale measurements, API compatibility and reproducible provenance; native/WASM and macOS/Windows/Linux evidence | All packages; infrastructure runs throughout |

## What closes an obligation

Each atomic record must contain its exact source anchors and authority, a plain
statement of required behavior, applicability, dependencies, implementation/API
locations, acceptance partitions, executed evidence and remaining exceptions.
Keep implementation status, API availability and verified behavior separate.

A closure review must show all of the following:

- The full obligation was decomposed, including relevant prose, grammar actions,
  metamodel conditions, derivations, library definitions and external dependencies.
- Positive and negative partitions are justified. Where legal text cannot construct
  an invalid abstract model, a public model API or deliberate internal graph
  mutation tests the constraint; its narrower evidence boundary is explicit.
- Exact identities, ownership, ordering, directions, types and derived facts are
  checked as applicable. Parsing success or absence of diagnostics is not enough.
- Interactions with inheritance, unresolved references, malformed input and cycles
  are assessed. An unknown result never becomes a complete empty result.
- Required behavior is available to library consumers with a stable contract;
  private engine functionality alone does not establish public API completion.
- Dependency evidence and runtime results match the reviewed source/binary/library
  versions. Native/WASM and platform assertions have corresponding executions.
- A second review challenges the acceptance partitions and unresolved discrepancies.
  Record the review explicitly; do not infer it from a green automated gate.

Parent packages close only when their complete applicable obligation set and all
required dependencies close. Independent implementations provide corroboration;
they do not override the normative artifacts when expectations differ.

## Near-term sequence

1. Finish KERNEL-022 qualification and preserve the accept reconstruction regression.
2. Complete the [first C08 accept/trigger decomposition](evidence/accept-obligation-review.json)
   as the first complete family slice:
   all three accept properties and their derivations, `isTriggerAction`, the owned
   input rule, ordinary/subaction/transition specializations, receiver binding,
   all three trigger argument validations and `instantiatedType`. Include inherited
   ActionUsage/InvocationExpression obligations and library defaults. The current
   payload-wrapper fix closes none of those dependencies by implication.
3. Review the remaining transition bindings and the recorded state derivation
   conflicts together with this slice. Establish public query acceptance criteria
   before adding more projections that could conceal missing model facts. The
   [receiver-binding probe](evidence/accept-receiver-binding-before-macos-arm64.json)
   already reproduces a missing second invocation parameter and missing binding
   connector for all three ordinary trigger kinds after implicit publication.
4. Allocate all remaining source units across C01–C15 and break each package into
   estimable obligations. Preserve a disposition for context/informative material
   and extraction duplicates so omissions remain visible.
5. Re-estimate using reviewed obligations and observed repair effort. Track time
   spent on decomposition, verification, fixes and qualification separately.

## Initial effort range

The working estimate is 38–76 engineer-weeks for the read/analysis kernel and its
qualification: 2–4 for decomposition, 4–8 for syntax, 8–16 for core semantics,
12–24 for language families/validation, 6–12 for libraries/interchange and 6–12
for API qualification and hardening. Shared work must not be counted twice.

This range is an engineering judgment, not a measured schedule or a guarantee.
It assumes continued reuse of the current engine and architecture. Broader
modeling-tool coverage including authoring and graphical notation has a provisional
18–36+ engineer-month range. These numbers exclude the separate API/services and
v1 transformation standards. AI session duration, test count and named-rule count
cannot be linearly extrapolated into remaining calendar time. C01 must refine both
the deliverables and the estimates before they are treated as commitments.
