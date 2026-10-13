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
29 passing cases and two planned cases to the current assessments. The 19-test Node
suite, including six inventory integrity tests, passes. Extraction reproduces
exactly from all 17 pinned source artifacts. Browser checks exercised rule search,
unassessed constraint details and the separate positive/negative port-test mapping.
Windows/Linux execution still awaits their CI runners.

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
