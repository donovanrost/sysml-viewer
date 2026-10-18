# Standards requirements inventory

The inventory covers the full numbered clause structure of **SysML 2.0 Part 1:
Language** and **KerML 1.0**, their published abstract-syntax metamodels, and the
normative library archives and JSON schemas listed with those specifications.
It makes remaining review work visible. It does **not** yet constitute a fully
decomposed, independently reviewed set of atomic requirements.

Open the [searchable inventory](requirements.html) for source links, filters,
implementation evidence and test mappings. The [generated summary](requirements-summary.md)
contains the extraction counts and current assessments. The [executable test
matrix](conformance.md) continues to hold runnable acceptance cases.

## What is inventoried

- Every numbered PDF clause, including context and annex clauses, reconciled against
  both the printed table of contents and PDF bookmarks. Numbered body headings are
  also located. Substantive unnumbered bookmarks are retained separately.
- Named grammar productions, including lexical terminals. Five headers that omit
  an equals sign in the printed specification are included as explicit extraction
  exceptions. Production alternatives, synthesis assignments and contextual notes
  still require individual review.
- Every metaclass, property (including association-owned ends), association,
  generalization, operation, parameter, constraint, enumeration/literal and package
  declaration of the indexed XMI types. Constraint records include named validation,
  derivation and semantic checks plus unnamed operation conditions. They are not
  all independent well-formedness rules.
- All ten published normative library archives: each model file and project
  metadata file is indexed with its exact content hash. OS packaging metadata is
  counted and explicitly excluded. Every definition in the three normative JSON
  schemas is indexed, with property names, required-property names and a digest.
- Modal prose passages are located by page, containing clause and digest. These
  are **review candidates**, not a reliable automatic count of obligations. Some
  normative rules have no modal verb; others express several obligations in one
  paragraph. All clause review remains open to catch both cases.

The catalogue contains identifiers, structural facts and source locators. Original
PDF text, OCL bodies, model source and schema bodies remain in their upstream
documents. Downloads stay in the ignored `target/standards/` cache.

## Source authority and scope

The document baseline is `formal/2026-03-02` for SysML and `formal/2026-03-01` for
KerML. Their OMG publication pages list the companion artifacts, whose date-based
URIs differ from the PDFs' publication dates. Each artifact is independently pinned
by URL, document ID, byte length and SHA-256 in `standards/sources.json`.
Clause 2 gives normative machine-readable artifacts precedence over the PDF when
they conflict. Apparent discrepancies require a recorded review; an extraction
exception does not silently amend the standard.

Source listings: [OMG SysML 2.0](https://www.omg.org/spec/SysML/2.0) and
[OMG KerML 1.0](https://www.omg.org/spec/KerML/1.0). The separate SysML v1-to-v2
transformation specification and Systems Modeling API and Services are outside this
language-kernel inventory. Referenced external standards are dependencies to assess
where applicable, rather than documents claimed to be exhaustively inventoried here.

The current product is a read-only textual model viewer. Full tool conformance
additionally has abstract-syntax editing/validation and model-interchange obligations;
textual, graphical and semantic conformance have their own conditions. Those
obligations remain explicit in the assessment ledger. A private JSON snapshot does
not meet normative interchange requirements. Domain library support and informative
annex material must not be silently treated as universally mandatory requirements.
Applicability decisions belong to the intended conformance claim and its dependency
closure, not to which features happen to exist today.

## How evidence is represented

`standards/catalogue.json` is generated source metadata. `standards/assessments.json`
is the human-maintained ledger. Each bounded assessment records:

1. A concrete obligation or product guarantee, its category, applicability and priority.
2. Exact source-unit IDs. These are anchors, not a claim that a whole clause is satisfied.
3. Separate implementation and public API statuses, with source-file hashes and symbols.
4. Explicit test-case and assertion IDs; test polarity comes from the executable matrix.
5. Remaining scope and limitations.

Unmapped units default to **unassessed** for implementation and API, and **unmapped**
for tests. This is different from **unsupported**, which records a reviewed gap in
the product's public surface. **Candidate** identifies code worth evaluating;
**partial** identifies bounded behavior with remaining scope. Filename or symbol
presence is never sufficient to prove normative correctness.

The initial schema deliberately has no `qualified` status. Every clause, prose
candidate, library model and schema definition retains pending decomposition.
Before adding that status, define a reviewed closure record containing the complete
obligation set, applicability rationale, dependencies and acceptance criteria.
Review positive and negative cases independently; explain when either is inapplicable.
Do not infer parent-clause completion from a tested child. Do not count robustness
tests as proof of a semantic rule.

**No conformance percentage is calculated.** Clauses, productions, XMI declarations,
schema definitions and prose passages overlap. Counting their sum, or dividing
passing examples by these counts, would produce a misleading denominator.

## Commands and checks

```sh
npm run requirements:check    # Offline integrity, evidence edges and generated-file checks
npm run requirements:docs     # Regenerate the summary and standalone HTML browser
npm run conformance          # Build and execute the native acceptance matrix
npm run requirements:report  # Join that execution to the inventory
node --test tests/requirements.test.mjs
```

The execution report is saved to `target/requirements/report.json`. It distinguishes
mapped, planned, not-run, failed and passing evidence without promoting source-unit
coverage. It validates matrix/fixture hashes, engine/library pins, current kernel
input fingerprints and whether the CLI was built for the run. The conformance report
also records the tested binary's hash. These are qualification provenance checks,
not a promise of reproducible binary builds. CI runs the offline inventory gate and
uploads the execution report alongside its conformance report on each platform.

The [recorded macOS execution](evidence/requirements-macos-arm64.json) maps the
current executable and planned cases without promoting known gaps to passing.
The foundation update adds direct named-rule mappings; see the
[qualification record](foundation-qualification.md) for its evidence and limits.
Extraction reproduces exactly from all 17 pinned source artifacts. Windows/Linux
execution still awaits their CI runners.

The gate rejects missing source units, dangling test/assertion edges, unmapped new
assertions, changed implementation evidence, stale execution evidence and generated
document drift. A changed code hash requires reviewing the mapping and expectations
before manually updating its recorded digest. Refreshing a hash is not a correctness
review. The generated browser shows mappings, not live execution results; use the
execution report for outcomes from a particular run.

To reproduce the extraction, download the exact files in `standards/sources.json`
to `target/standards/` using each entry's `cacheFile` name. Use Python with
`pip install -r tools/requirements-extraction.txt` and Poppler `pdftotext` 26.01.0:

```sh
python tools/import-standards.py --check  # Verify pinned downloads and exact reproduction
python tools/import-standards.py         # Regenerate after a reviewed source/tool change
npm run requirements:docs
npm run requirements:check
```

Normal builds and CI do not download standards or require PDF tooling. The extractor
fails on changed source bytes, missing clause headings or unknown grammar-header
exceptions. Tool versions are pinned because text normalization affects passage
digests. Changes to source editions require reviewing IDs, mappings, counts and
normative differences before adopting a new baseline.

## Review order

1. Define the intended kernel and tool-conformance profiles, keeping viewer usability
   distinct from editing, interchange, execution and graphical conformance.
2. Decompose lexical/name-resolution, namespace/membership and identity rules first.
   Establish reusable positive/negative fixtures and exact semantic expectations.
3. Review the type system, implicit relationships, specialization, subsetting,
   redefinition, multiplicity and library dependencies together.
4. Expand through structural and behavioral families: ports, interfaces, connections,
   flows, actions, states, calculations, constraints, requirements, cases, views,
   metadata, variation and individuals. The index retains their syntax, abstract
   syntax, semantics and library anchors even where no test exists yet.
5. Decompose library function/model bodies and interchange schema properties, then
   qualify the selected editing/interchange/notation conformance points. Compare
   with the pilot implementation as corroborating evidence; the standard remains
   the authority for expected results.

This review queue is substantial remaining engineering work. The completed artifact
is the reproducible standards-wide inventory and traceability mechanism; atomic
requirement review and implementation of uncovered behavior remain open.

## Breadth audit: declaration and explicit-typing baseline

KERNEL-016 starts the family audit with 24 SysML definition/usage pairs. Ten
`FAMILY-*` cases exercise nominal construction, written type identity, exact source
extents and an incompatible explicit type for each kind. `TYPE-005` adds permitted
KerML classifier targets; `TYPE-006/007` exercise nearby valid and invalid concern
and state typing. Each family's assessment names only the tested obligation.
The tests are representative partitions, not a complete grammar or constraint audit.

| Family group | Executable baseline | Next review needed |
| --- | --- | --- |
| Attributes, enumerations, occurrences, items, parts, ports | `FAMILY-DATA-001/002` | Directed parameters, enum variants, individuals/portions, implicit bases, conjugation and feature derivations |
| Connections, interfaces, allocations, flows | `FAMILY-CONNECTIONS-001/002` | End construction, typing/direction compatibility, binding/succession constraints and payloads |
| Actions, states, calculations, constraints | `FAMILY-BEHAVIORS-001/002`, `TYPE-005/007` | Body alternatives, parameter/result memberships, control nodes, transitions, temporal rules and expression semantics |
| Requirements, concerns, cases, analysis, verification, use cases | `FAMILY-CASES-001/002`, `TYPE-006` | Subject/actor/stakeholder/objective memberships, requirement constraints, satisfaction, verification and inclusion |
| Views, viewpoints, renderings, metadata | `FAMILY-VIEWS-001/002` | Expose/filter/render memberships, annotation targets, semantic metadata and graphical notation |

Comparison with the normative XMI found two implementation defects: states were
restricted to StateDefinition despite the Behavior range of
`StateUsage::stateDefinition`; concerns accepted general RequirementDefinition
despite the narrowed range of `ConcernUsage::concernDefinition`. These are fixed
in the semantic dependency, with public Rust and native/WASM regressions. The
pre-fix execution is preserved in `docs/evidence/families-before-macos-arm64.json`.

The distinction between redefinition and subsetting matters in this audit.
`ItemUsage::itemDefinition` selects Structures from occurrence types, and
`PartUsage::partDefinition` selects PartDefinitions from item definitions. Their
property ranges alone do not establish that every explicit type must have those
metaclasses. No blanket narrowing was added for those properties.

The whole-language audit remains open. Root elements/dependencies/annotations,
the complete KerML type/feature/relationship and expression grammar, all family
body productions, inferred/derived metamodel construction, normative libraries,
interchange, authoring and graphical conformance still need obligation-level
review. Existing upstream code in these areas remains a candidate until its
public behavior and positive/negative acceptance criteria are independently
checked. The next audit step is the body/member construction and derived-property
boundary, followed by full validation-rule mapping; more nominal declaration
examples alone cannot close these areas.

## Validation-rule audit

The [generated rule audit](validation-audit.md) enumerates all 180 named
`validate...` constraints in the pinned metamodels. It scans the complete Rust
source trees of the syntax library, model library and public kernel, fingerprints
each file, and joins only direct assessment source edges. `requirements:check`
rejects generated drift and registry names outside the pinned catalogue.

After KERNEL-019, the registries name 91 rules and source text mentions 92. The
other 88 names were not located by this search. Nine rules have direct bounded
assessments; 171 still need direct obligation review. These are separate measures:
comments, private tests and registration lists are search hints, and a missing
name does not establish missing behavior. The 36 labels outside the catalogue
include deliberate compatibility aliases and checks needing further classification.

The first behavioral review found two concrete issues:

- **KERNEL-017, corrected:** state subaction uniqueness already rejected repeated
  owned entry/do/exit actions, but the public codes omitted `State` before
  `SubactionKind`. `STATE-001/002` retain all three kinds on definitions and usages,
  named and anonymous actions, valid peers, and local actions sharing an inherited
  kind. Canonical codes now follow the XMI; legacy message text stays compatible.
- **KERNEL-018, corrected for VIEW-001–006:** view rendering validation now
  counts owned and inherited ViewRenderingMemberships using the shared membership
  projection. Actual/transitive redefinition, visibility, public re-export and
  diamond identity are retained. Same-name compatibility shadowing no longer
  removes a view rendering: the required implicit RenderingUsage redefinition
  targets the library's `Views::View::viewRendering`, not every inherited user
  rendering. Missing/ambiguous references, recovered contributing documents,
  cycles and other incomplete dependencies cannot certify an inherited-count
  violation. Two owned renderings remain a definite error. Complete peer views
  are still validated beside unresolved models. The assessments retain the
  remaining metadata, filter/import-all and general view-semantics limits.

Diagnostic assertions support an exact qualified owner and require the
finding's byte range to be within that owner's source in the same document.
Absence checks can select a specific rule, severity and message, so a cyclic
model may retain its cycle error while proving that no rendering-count error was
invented from an incomplete candidate set.
A failure on a peer cannot satisfy a missing finding. Known gaps are restricted
to named assertions; crashes, syntax errors and unrelated failures are not waived.
The [pre-fix probes](evidence/validation-before-macos-arm64.json) retain the actual
earlier codes and missed findings. They used the existing `.17` executable and
are explicitly distinguished from the fresh qualification run.

KERNEL-019 corrects parallel-state validation using transition incidence on direct
owned actions, rather than the ownership of the transition itself. `STATE-003–007`
cover plain successions, external and cross-file transitions, aliases, inherited
and nested actions, performed actions, state subactions, missing/ambiguous endpoints
and independently known opposite endpoints. The same endpoint proof prevents an
unresolved target chain from producing a false `validateTransitionUsageSuccession`
kind error. A ReferenceUsage referencing an action retains its non-action kind;
a PerformActionUsage retains its own ActionUsage identity.

These three additional assessments remain bounded. The target check uses the
current lowering representation's first succession and positional target end;
missing successions, nonbinary/raw targets, implied connector construction and
all transition source/target derivations remain open. The next body/member review
must also expose state action slots and membership kinds through a considered
public API. Trigger, guard and effect constraints, other action forms and the
remaining family obligations still need independent review.

The broader audit should classify complete obligation sets by implementation, API
exposure and verified behavior before refining the remaining-work forecast.
Closing the last named failure in the current matrix does not close the standards
inventory.

For each remaining validation rule, review its XMI expression and prose, locate
the actual execution path, derive positive/negative partitions, and map exact
public assertions before making any implementation claim. Derivations, `check...`
semantic constraints, operation conditions, grammar, libraries, interchange and
prose remain separate queues. This audit does not close those areas or establish
a full-spec completion percentage.
