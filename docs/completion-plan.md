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

1. KERNEL-022 and KERNEL-023 qualification are complete within their recorded
   boundaries. Receiver bindings are constructed in the raw graph; public endpoint
   queries and the remaining accept/trigger dependencies still need qualification.
2. Complete the [first C08 accept/trigger decomposition](evidence/accept-obligation-review.json)
   as the first complete family slice:
   all three accept properties and their derivations, `isTriggerAction`, the owned
   input rule, ordinary/subaction/transition specializations, receiver binding,
   all three trigger argument validations and `instantiatedType`. Include inherited
   ActionUsage/InvocationExpression obligations and library defaults. The current
   payload-wrapper fix closes none of those dependencies by implication. The
   [trigger argument probe](evidence/trigger-argument-validation-before-macos-arm64.json)
   now reproduces noncanonical diagnostic IDs for all three invalid argument forms;
   rejection alone does not establish their complete named validation rules.
   The [custom-duration probe](evidence/trigger-duration-before-macos-arm64.json)
   additionally reproduces false rejection of direct and inherited duration types.
   [Worktree result-inheritance progress](evidence/result-inheritance-progress-macos-arm64.json)
   restores their generated result features through checked identities and the shared
   inheritance walk, with explicit incomplete results when library evidence is missing.
   [Checked feature-domain progress](evidence/feature-domain-progress-macos-arm64.json)
   extends the shared proof to ordinary assertion members, duration attributes and
   authored behavior/function parameter and return Features. Verification passes
   94 Rust workspace tests, 356 selected vendor tests and Clippy. It preserves the shared ownership, inheritance and positional proof,
   and keeps legacy graphs incomplete when required typing heritage is absent.
   [Generated result-domain progress](evidence/generated-result-domain-progress-macos-arm64.json)
   now certifies registered reference-expression result roots through current
   construction evidence, complete required bases and the shared membership reducer.
   That checkpoint passes 94 Rust workspace tests, 380 selected vendor tests and
   Clippy. [Trigger binding progress](evidence/trigger-binding-progress-macos-arm64.json)
   subsequently extends the shared invocation certificate and atomic planner to
   canonical trigger calls, including correct activation in trigger-only models.
   It checks all three designated functions, supplied values in input order and
   absent valuations on generated receivers. All 94 Rust workspace tests, 385
   selected vendor tests and Clippy pass. The [result dependency review](evidence/trigger-result-dependency-review.json)
   identifies Predicate-return and nested-expression requirements,
   including an open normative/library naming discrepancy reported in OMG
   KERML11-207. [Predicate-result progress](evidence/predicate-result-progress-macos-arm64.json)
   now admits ordinary Predicate signatures, inherited/owned return selection and
   authored return-feature domains through the shared Function and membership
   providers. The full-library Boolean literal case checks its Predicate-owned
   result; general checked Expression.result selection remains open. All 94 Rust
   workspace tests, 409 selected vendor tests and Clippy pass. Legacy return
   domains remain incomplete when required data-value heritage is missing.
   The [Step construction review](evidence/step-context-construction-review.json)
   records the plural-library interpretation, keeps the exact source obligations
   open, and decomposes contextual ownership, composite, typing, identity,
   mutation and budget acceptance partitions before implementation. Its eight-case
   probe finds no contextual specialization paths in the six examples that require
   them under the recorded interpretation. Compatibility reachability alone does
   not establish a complete negative semantic proof.
   [Step-context progress](evidence/step-context-progress-macos-arm64.json)
   constructs the owner-metaclass Step paths through the
   existing static planner and minimizer. Generated nested reference-result
   domains now require reciprocal source ownership and the applicable enclosure
   path. Six private regressions cover ownership damage, required library roles,
   budget retries and prepared-library replay; the expanded selected vendor run
   passes 512 tests. All 94 Rust workspace tests and Clippy pass. A full-library
   probe confirms the implemented paths and separately reproduces the missing
   direct/inherited effective-type owner cases. Construction based on a composite
   owner's effective Feature.type remains open; a pruned Structure candidate
   explicitly demonstrates why written typing cannot supply that condition.
   The [Expression result review](evidence/expression-result-dependency-review.json)
   separates complete result selection from checking the selected result's
   feature domain. It records seven pinned source anchors and the remaining
   return-membership, inheritance and generated-source acceptance partitions.
   [Typed Step progress](evidence/typed-step-progress-macos-arm64.json) now constructs
   the structural-owner path for direct and inherited effective typing. The shared
   Feature.type reducer also proves the pruned-Structure counterexample inapplicable.
   Construction predicates exclude all prospective specialization writers; the
   existing atomic planner audits its final writes against that same set. Predicates
   coupled to those writes remain incomplete and require a mutually stable graph
   proof. Five new private regressions and a ten-context integration test using all
   94 library files pass: 95 Rust workspace tests, 517 selected vendor tests and
   Clippy. Prepared-library replay, mutation, unknown predicates and unchanged
   work limits are covered. This does not close every Step configuration or the
   recorded normative/library spelling discrepancy.
   [Expression-result progress](evidence/expression-results-progress-macos-arm64.json)
   now selects ordinary and literal Expression results through the complete shared
   feature-membership reducer. Function and Expression queries use the same exact
   return-membership selector, preserving inherited order and redefinition.
   Complete absence remains distinct from unavailable evidence and from the
   separate exactly-one validation. Both graph formats pass six literal-kind
   checks against all 94 library files. Five private regressions cover ordering,
   ownership, mutation, budget retry and prepared-library replay; the native trigger
   test now checks Boolean literal result selection. All 95 Rust workspace tests,
   522 selected vendor tests and Clippy pass. General reference/invocation results,
   result-expression bindings and broader source configurations remain open.
   The [reference-result membership review](evidence/reference-result-membership-review.json)
   records thirteen source anchors and a six-case full-library probe. Five reference
   sources have checked generated-result feature domains but still lack checked
   result selection; the Boolean literal control passes. The positional planner's
   stored-source selectors omit generated memberships present in the checked
   semantic view. [Semantic positional progress](evidence/semantic-positional-progress-macos-arm64.json)
   supplies that shared input contract: checked queries now pass their complete
   semantic nodes to the same ordering and reduction used by construction.
   Exact authored redefinitions seed the query; previously generated positional
   edges cannot justify their own omission. Five new regressions cover generated
   returns, stale rows/recorded targets, cycles, work limits and prepared-library
   replay. All 95 Rust workspace tests, 538 selected vendor tests and Clippy pass;
   the selected runner now also includes eleven existing positional tests.
   The [complete vendor model run](evidence/semantic-positional-full-model-macos-arm64.json)
   passes all 891 unit tests with the pinned library and 251 upstream corpus files.
   Its initial two failures were missing-corpus assertions; the same binary passes
   with those fixtures supplied. These test inventories overlap and are not a
   count of closed specification obligations.
   [Reference-result progress](evidence/reference-results-progress-macos-arm64.json)
   now admits registered generated-reference sources through the same checked
   result report and property. Complete foreign-member ownership and positional
   dependencies are scheduled without adding foreign supertypes; generated binding
   ownership/endpoints are checked through a shared reader. Seven new regressions
   cover reference results, foreign alias suppression, malformed ownership/bindings,
   missing/cyclic dependencies, budgets and prepared-library replay. All six
   CanonicalV3 full-library trigger examples now use checked result selection.
   All 95 Rust workspace tests and all 898 vendor model tests pass; Clippy is clean.
   [Trigger-predicate progress](evidence/trigger-predicates-progress-macos-arm64.json)
   replaces the three source-expression guesses with model predicates using checked
   argument/result selection, global library resolution and complete feature
   redefinition/specialization evidence. Six valid and eight invalid full-library
   cases assert verdicts and canonical diagnostic IDs. Five private tests cover
   stale/missing evidence, wrapper ownership, cumulative budgets and prepared replay.
   All 96 Rust workspace tests and all 903 vendor model tests pass; Clippy is clean.
   [Abstract trigger progress](evidence/trigger-abstract-progress-macos-arm64.json)
   distinguishes complete empty arguments from malformed valuations and checks
   first-member ordering, parameter exclusion and non-expression referents.
   Those cases exposed an omitted authored Expression-return domain. The shared
   owner and positional proofs now support parameters of admitted Step/Expression
   families, including definite negative type comparisons and stale-owner refusal.
   Six new regressions pass; all 96 native workspace tests and 909 vendor model
   tests pass, with clean Clippy. Broader result/argument providers, defaults and
   legacy bindings remain open. Conformance mappings, native/WASM integration,
   performance and release qualification remain pending.
   This work is not yet a new qualified engine checkpoint.
   The [invocation-result construction review](evidence/invocation-result-construction-review.json)
   now reproduces missing owned results for ten full-library calls. Canonical
   lowering constructs these outputs and preserves grammar-provided returns.
   Receiver construction inserts its input before the return through the shared
   ownership view. This increment remains unqualified: the first full run exposed
   unresolved library-result dependencies, migration/pairing assumptions and a
   cold-query budget regression. These are active repair work, not waived failures.
   Follow-up work repairs migration and authored identity-pairing expectations,
   and resolves unnamed result members through actual positional targets. The
   original vector-library name now resolves. Header-context handling preserves
   redefinition targets before ordinary lookup hides them. All 78 focused tests,
   including full-library invocation and trigger checks, pass with clean Clippy.
   The cold-query repair now streams redefinition targets into the shared
   membership reducer and collects surviving features in the same ordered pass.
   An independent pairwise-rule comparison covers 96 changing graphs with dense
   and sparse traversal marks, cycles, duplicate memberships and budget retry.
   All 63 focused reduction/input/positional tests pass; the unchanged cold
   full-library fixture uses 9,789,848 canonical steps within the existing
   10,000,000 limit. All 927 vendor model tests and 97 native workspace tests
   pass. The native integration test reads Redefinition endpoints through the
   stored-property API and verifies both exact endpoints. All 55 Node tests pass
   across the public native/WebAssembly, provenance and evidence suites. Reviewed
   implementation fingerprints and generated audits are current without changing
   the 77 bounded assessment records. All 133 conformance cases and the matching
   requirements execution report now pass on macOS arm64. A
   separate cold-analysis profile identifies repeated recorded-redefinition
   scans during construction lookup as a performance review target. No
   completion package closes here.
   [Lookup and effective-default progress](evidence/lookup-default-progress-macos-arm64.json)
   now reuses recorded redefinition source rows across lookups with revision-based
   invalidation and live outcome reads. Three local cold CLI cases return identical
   snapshots in 33–36% less elapsed time; these are small local measurements.
   The same review corrects complete effective-default absence for distinct
   inherited FeatureValue identities (KerML 8.4.4.11), preserving owned overrides,
   shared inherited defaults and incomplete cyclic/unresolved paths. The focused
   lookup/default checks, all 930 vendor model tests and 98 native workspace
   tests and all 55 Node tests pass, along with Clippy, WebAssembly build,
   contract and evidence checks. All 133 conformance cases and the matching
   current-source requirements execution report now pass.
   A full-library experiment selects defaults for five actual calls using the
   same resolver and confirms that supplied inputs suppress only their own
   defaults. Invocation
   default binding construction and complete result selection remain open.
   [Invocation-default construction work](evidence/invocation-default-construction-review.json)
   now shares effective-default selection and binary binding recipes across
   constructors and invocations. It stages invocation bindings after accepted
   argument mappings and preserves exact owned membership and endpoint identities.
   Targeted full-library cold cardinality and post-edit trigger regressions now
   pass within the unchanged work allowance.
   Differential tests cover reuse of current static positions, incomplete
   providers, source edits and replacement of the planning authority. The
   [eligibility identity repair](evidence/eligibility-identity-progress-macos-arm64.json)
   shares successful designated Function identities within the existing checked
   snapshot while retaining fresh argument and path-sensitive expression proofs.
   Both complete mixed-corpus cold queries now return exact 7 at about 6.14 million
   steps under the unchanged 10 million limit. All 945 vendor model tests, 99
   native workspace tests and 55 Node tests pass, with clean Clippy, native/WASM
   builds, contract checks, 133 conformance cases and the matching requirements
   execution report. A rejected no-build evidence run is retained separately from
   the successful official build-and-conformance rerun. The
   [invocation direct-obligation review](evidence/invocation-obligation-decomposition-review.json)
   decomposes nine constraints and two operations, with inherited dependencies.
   It records endpoint and membership formulations requiring reconciliation
   across the metamodel, Table 11 and Feature Values Semantics before the default
   obligation can close. Further disposable probes isolate namespace-owned foreign
   BooleanExpression membership from directed-valuation typing. Shared Step-family
   admission restores exact result selection for Step, Expression and BooleanExpression
   references without making their foreign members into features. Fresh malformed
   ownership/flag/metadata cases are refused below the work limit; several edited
   full-library graphs still exhaust their rebuilding allowance. These are diagnostic
   results, not production admission or complete default-binding semantics. Prepared
   replay and final-source qualification remain required after that next change.
   No package or complete invocation family is closed.
   The subsequent [foreign Step membership repair](evidence/foreign-step-membership-progress-macos-arm64.json)
   admits namespace-owned foreign members through the existing shared Step-family
   and reciprocal ownership checks. Their membership does not add an owned feature
   or positional role. All 51 default-result selections across two calls and three
   trigger kinds now pass through the checked report and property. Four new private
   tests cover exact identities, raw/prepared/serialized library replay, malformed
   evidence, edits and work limits. All 949 vendor model tests, 99 native workspace
   tests, Clippy, native/WASM builds, 39 public regression tests, 133 conformance
   cases and the source-matched requirements report pass. The [FeatureValue review](evidence/feature-value-obligation-review.json)
   decomposes its three direct constraints and separates declared static typing
   from value compatibility and binding semantics. A 17-query full-library baseline
   confirms four unvalued controls and thirteen valued-feature refusals, including
   both directed and explicitly specialized declarations. The implicit-only,
   undirected cases retain their separate result-specialization dependency.
   [Valuation typing progress](evidence/valuation-typing-progress-macos-arm64.json)
   now preserves declared static types when valid direction or an explicit owned
   specialization makes value-result specialization inapplicable. It uses shared
   reciprocal valuation, direction and specialization evidence without evaluating
   the value. Eight private tests cover twelve damaged-evidence partitions,
   incompatible/unresolved/cyclic values, cumulative work and library replay; the
   native test checks ten exact report/property pairs. All 958 vendor model tests,
   100 native workspace tests, 55 public/provenance Node tests, Clippy, native/WASM
   builds, contract checks and 133 source-matched conformance cases pass. Inference,
   value compatibility and complete binding semantics remain separate obligations.
   A [semantic positional type review](evidence/semantic-type-positional-review.json)
   identifies a further adapter gap: generated reference returns have no lexical
   scope, while their complete shared membership proof already supplies exact
   positional targets. The [production positional-type refactor](evidence/semantic-type-positional-progress-macos-arm64.json)
   returns these targets from the same applicability certificate and removes the
   redundant scope-based positional read. Seven private acceptance tests and a
   native API test pass; five of ten selected default-result types are now checked,
   including the more specific UniversalClockLife result and IncomingTransferSort.
   Unresolved type dependencies remain incomplete, while closed circular typing
   closures retain exact types as required by deriveFeatureType. All 965 vendor
   model tests, 101 native workspace tests, 39 public and 16 provenance/evidence
   tests, Clippy, native/WASM builds, contract checks, 133 conformance cases and
   matching requirements evidence pass. The [ordinary Feature context review](evidence/ordinary-feature-context-review.json)
   separately reproduces thirteen owner/inverse refusals in an eighteen-query
   matrix. The [ordinary Feature context repair](evidence/ordinary-feature-context-progress-macos-arm64.json)
   now admits plain noncomposite, undirected, non-end Features under Type owners,
   preserving the separate FlowEnd obligation and all shared ownership, flag,
   valuation and specialization checks. Owned inverse relationships require
   reciprocal carriers and exact endpoints without adding target typing.
   Seven staged tests pass, including thirteen owner/context and eleven inverse
   damage partitions, inherited narrowing, role exclusion, edits, library replay
   and cumulative budgets. Final-source qualification passes 972 vendor model
   tests, 102 native workspace tests, 39 public and 16 provenance/evidence tests,
   Clippy, native/WASM builds, contract checks, 133 conformance cases and matching
   requirements evidence. No complete family or package closes.
   The initial full vendor run found two older unsupported-owner expectations.
   Their reviewed replacements assert exact ordinary-function and nested-feature
   positives in both formats, preserving FlowEnd, portion and incomplete parameter
   negatives. All ten tests in those modules pass; final-source qualification was
   restarted on the corrected tests. A separate [feature-chaining review](evidence/feature-chaining-type-review.json)
   records twelve normative anchors and a seventeen-query public baseline.
   Its isolated prototype reuses the common checked one-hop target for typing and
   required paths. All six acceptance tests pass, including twelve malformed-link
   partitions, edits, repeated links, narrowing, budgets, replay and circular
   typing. Twelve of seventeen diagnostic queries complete; the remaining five
   depend on portion families. The reviewed common chain reader is now admitted
   to the worktree after expanded public and private acceptance. The public test
   preserves the legacy missing-base refusal and checks exact legacy types with
   explicit required paths; canonical inputs retain implicit construction. All 64
   type-relation tests, the feature-type suite and three named-callee guard tests
   pass in the disposable candidate. Final-source qualification passes 978 vendor
   model tests, 103 native workspace tests, 39 public and 16 provenance/evidence
   tests, Clippy, native/WASM builds, contract checks, all 133 conformance cases
   and matching requirements execution. The three named chaining validations
   retain their separate review queue.
   A separate [portion applicability review](evidence/portion-applicability-review.json)
   records six pinned source anchors and fifteen current-source public queries.
   All eleven portion queries remain incomplete, including existing explicit
   library paths. The conditional rule depends on owned Class typing and complete
   effective owning-Feature types; an executed narrowed-owner case demonstrates
   why written typing alone is insufficient. Its shared construction/read proof,
   authority discrepancies and mutation/replay partitions remain open.
   The isolated portion prototype now shares the owned-typing/ownership predicate,
   static Class-owner construction and complete effective Feature-owner predicate
   with the existing atomic planner. An executed dependency trace identified the
   ordinary intersection guard on spaceTimeEnclosedOccurrences; extending the same
   exact-subsetting-witness condition resolves that prerequisite. Seventeen exact
   type probes and all ten selected default-result completeness queries pass.
   Five isolated acceptance tests pass, including ten fresh and post-query damage
   partitions, negative specialization refusal, exact Class-owner negative proof,
   closed owner-typing cycles, complete narrowed-owner predicates, intersection
   witness refusal and raw/prepared/serialized identities with budget retry.
   Exact default-result identities, nonowned typing, predicate exclusion, graph
   formats, broader regressions and Feature-owner negative proof remain open.
   The extended isolated implementation now supplies complete Feature-owner
   certificates to negative specialization proofs, with shared recursion bounds,
   cumulative work and prospective-writer exclusions. Exact conditional paths
   distinguish Class-typed, inherited, narrowed and non-Class owners. All ten
   selected defaults have exact result-type assertions, preserving the Occurrence
   result of portionOfLife separately from that Feature's Life type.
   An owned/inverse typing review repairs false required-base obligations on
   standalone typings. The shared structural index retains conflicting source
   aliases, and the owned-supertype reader validates standalone rows before
   excluding them. Early rejection precedes semantic-plan refresh without changing
   reflexivity or conjugated-receiver normalization. Thirteen portion acceptance
   tests pass, including eight malformed standalone partitions below the work
   limit, plus 36 structural-index, 64 type-relation, 35 type-projection and eight
   recursive-visibility tests. Both graph formats retain exact complete cases;
   legacy owners missing required base paths remain explicitly incomplete.
   The public integration test also passes, including eight exact portion types,
   all ten default-result types and missing-owner refusal. The reviewed candidate
   is now admitted to the working source, with updated patch provenance and
   sixteen passing provenance checks. Final-source qualification now passes all
   992 vendor model tests, 104 native workspace tests, 39 public regression tests,
   Clippy, native/WASM builds, contract checks, 133 conformance cases and matching
   requirements execution in [portion progress](evidence/portion-context-progress-macos-arm64.json).
   Mutually stable conditional-construction proofs and adjacent named validations
   remain open; there is no complete-family claim.
   The [namespace Feature member review](evidence/namespace-feature-members-review.json)
   reproduces incomplete authored TypeFeatureMember typing, Expression results and
   invocation defaults through public queries. Its isolated shared ownership
   certificate preserves ordinary namespace membership without inventing an
   owning Type or positional role. Initial acceptance verifies exact types,
   ownership damage, edits, replay and separate alias roles. Expanded review
   confirms the distinction between local ownedFeature redefinition and inherited
   membership suppression. All eight corrected acceptance tests and one public
   integration test pass; the existing broader Type relation/input/feature/reference
   cases also pass. A post-publication membership-role edit exposed excessive
   repeated graph work. The reviewed repair reuses freshly certified positional
   evidence, indexes current specialization rows and shares safe graph searches
   while preserving physical relationship order and identity. The first edited
   query now returns its exact type within the unchanged work allowance.
   Independent closure-oracle tests cover 2,560 graph/candidate configurations;
   all 1,008 isolated model tests and the public integration pass. The candidate
   is admitted with patch and source-anchor provenance. Final-source qualification
   passes 105 native tests, Clippy, native/WebAssembly builds, 39 public regressions,
   all 133 conformance cases and matching requirements evidence, recorded in
   [namespace-member progress](evidence/namespace-feature-members-progress-macos-arm64.json).
   Contextual default-binding endpoints and complete family closure remain open.
   The subsequent [binding-member review](evidence/binding-namespace-member-review.json)
   preserves authored bindings as namespace members and adds early rejection of
   damaged membership evidence before positional rebuilding. All 85 expanded
   isolated checks and the public integration pass. Canonical source uses implicit
   construction; legacy positives use a fully authored signature with required
   typing paths, while original incomplete literal-result controls remain visible.
   All 1,013 isolated model tests now pass, and the reviewed source is admitted.
   Sixteen provenance checks and the public contract check pass. Final-source
   qualification passes 106 native tests, Clippy, native/WebAssembly builds and
   39 public regressions. The conformance run passes 132 cases, but ACCEPT-003
   exceeds its unchanged 30-second process timeout during overlapping compilation.
   An isolated retry also times out; a subsequent unchanged-source run passes all
   19 checks in 8.533 seconds. A sampled run passes in 18.924 seconds and is
   dominated by initial name resolution. No causal regression is established.
   The full retry passes 128 cases, including ACCEPT-003 in 4.954 seconds. Five
   other cases time out across recorded clamshell/system sleep; matching
   requirements evidence records those failures after two stale audit source
   hashes were refreshed. The subsequent unchanged-source rerun with scoped
   idle-sleep prevention passes all 133 cases and matching requirements evidence;
   ACCEPT-003 completes in 4.272 seconds. Together with the prior source-matched
   gates, this qualifies the binding-member increment on macOS arm64. All attempts
   and their power evidence remain preserved in
   [binding-member progress](evidence/binding-namespace-members-progress-macos-arm64.json).
   The [connector dependency review](evidence/connector-endpoint-obligation-review.json)
   decomposes all 13 direct Connector/BindingConnector constraints and records
   16 crossing/chaining dependencies. Their complete acceptance remains open.
   The [Association end-domain review](evidence/association-end-domain-review.json)
   decomposes eleven direct Association/AssociationStructure constraints. Its
   isolated candidate audits required bases after shared effective-end reduction,
   preserving inherited identities and positional redefinitions. A full-library
   probe now returns exact Link/BinaryLink/SelfLink and object-association end
   lists in both formats. SelfLink required validation of a standalone
   Subclassification without adding it to owned ancestry; malformed source aliases
   exposed a shared structural-index gap, now covered by a passing 28-partition
   regression and all 36 existing index tests. The foreign-alias failure now has
   an isolated shared dependency selector for construction and checked queries,
   including the same standalone-relationship validator in both readers. Exact
   inherited suppression, alias identity, declaration order, damage, budgets and
   full-library replay now pass all nine domain tests. The first complete model
   run passes 1,008 tests and fails fourteen; the public integration is not run.
   Tracing finds exact-metaclass restrictions in the shared foreign positional
   dependency proof that reject valid expression/usage and parameter-membership
   subtypes in the standard library. The isolated correction uses their actual
   roles while preserving ownership, direction, ancestry and cycle guards. Three
   representative invocation/portion/trigger regressions and the nine Association
   tests now pass. The legacy foreign-return expectation is replaced with exact
   two-format result, ownership-damage, budget and model-rebuild identity checks.
   Six additional subtype/format partitions pass after a reserved-name fixture
   error is corrected. The next complete model run passes 1,022 of 1,023 tests:
   all fourteen prior failures pass, but a valid membership-role edit now exceeds
   the unchanged ten-million-step allowance. The failure reproduces alone.
   Profiling locates 9,986,818 steps in the retained-typing rebuild before the
   query completes; borrowed-alias checks account for only 48,704 of those steps.
   A batched membership traversal passes independent exhaustive graph-oracle and
   shared-tail work checks, but adds 22,282 steps on this library rebuild and still
   exceeds the limit. That experiment is preserved and is not selected for this
   repair. A new isolated candidate reuses only the raw joined library-name index
   under its exact name-table revision; semantic proofs remain fresh and external
   overrides bypass reuse. All 22 focused checks pass, including the original
   membership-role edit under the unchanged allowance, all ordinary-feature and
   Association-domain checks, and borrowed subtype/format partitions. Initial
   materialization now uses that same helper. All 1,026 isolated model tests and
   the public integration pass. A subsequent source review finds that stale-map
   retirement cost depends on a sampled reference count; a deterministic test
   reproduces different work charges with and without a retained reader. The
   reviewed successor charges retirement conservatively. All 1,027 isolated model
   tests and public integration pass, including the original membership-edit budget
   regression and reader-independent retirement. The source is admitted with
   reviewed vendor patches and unchanged assessment claims. Final-source
   qualification passes 107 native workspace tests, 39 public API regressions,
   13 provenance checks, Clippy, native/WebAssembly builds, contract verification,
   all 133 conformance cases and source-matched requirements evidence in
   [Association progress](evidence/association-end-domains-progress-macos-arm64.json).
   End typing, derived related/source/target types, complete validation and coupled
   binary-base construction remain open. No complete family closes.
   The [owned-cross construction review](evidence/association-end-typing-review.json)
   now queues 68 pre-fix partitions across both dialects and graph formats. These
   cover dialect-specific metaclasses, prefix/declaration flags, qualified and short
   names, inherited scope, unnamed specialization/conjugation forms, and ordinary
   end declarations that must not acquire a cross feature. Static review locates
   duplicated declaration lookahead and a separate constructor that omits shared
   flags and scope registration. The first baseline attempt fails compilation on
   an unavailable slice helper in a test carrier assertion. After the equivalent
   ordered-iterator correction, all five tests execute: 60 partitions reproduce
   the construction/lookahead gaps and eight ordinary-end controls pass. The
   [pre-fix observations](evidence/owned-cross-construction-baseline-macos-arm64.json)
   preserve exact raw facts and syntax errors separately from the compile failure.
   A shared dialect-aware constructor and declaration-start lookahead repair
   passes all 68 partitions in isolation. The separate 24-partition syntax
   baseline passes 22 controls and reproduces two wrong-dialect acceptance gaps:
   KerML ordinary and cross Features incorrectly accept SysML `defined by`.
   The reviewed candidate restricts that typing keyword to SysML and adds public
   navigation, supported source rebuild and raw/prepared/serialized library replay
   acceptance. Its first expanded run passes syntax and library replay but finds
   an invalid reserved short name in the rebuild fixture. A four-case diagnostic
   proves that quoting the same name restores valid syntax in both dialects.
   With that fixture corrected, all 16 syntax unit tests, seven cross-feature
   tests and public integration pass. The full model suite with 251 pinned
   example/training/validation files is running. No owned-cross repair is admitted
   or semantically qualified.
3. Review the remaining transition bindings and the recorded state derivation
   conflicts together with this slice. Establish public query acceptance criteria
   before adding more projections that could conceal missing model facts. The
   [pre-KERNEL-023 receiver-binding probe](evidence/accept-receiver-binding-before-macos-arm64.json)
   recorded a missing second invocation parameter and binding connector for all
   three ordinary trigger kinds. KERNEL-023 repairs those tested constructions;
   broader inherited/default binding behavior remains open.
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
