# SysML Viewer

A local SysML v2 viewer with a reusable Rust language kernel, a TypeScript interface,
and a Tauri desktop host. The same kernel runs natively in a command-line tool and
as WebAssembly in the desktop/browser viewer. Models stay on your device; analysis
has no runtime server, account, telemetry or network dependency and never writes
model files.

Development builds use **0.0.x** and are published as
[GitHub prereleases](https://github.com/donovanrost/sysml-viewer/releases).
Official releases require an explicit version tag outside 0.0.x.
See [publishing releases](docs/releases.md) for the version and release process.

The first kernel migration milestone is implemented. Open-MBEE's syntax and model
libraries provide lexing, parsing, semantic construction, resolution and validation
behind our own versioned facade. This is **not a claim of complete SysML conformance**.
Read [the architecture](docs/architecture.md) and [the engine evaluation](docs/kernel-evaluation.md)
for the boundaries, evidence and remaining work.

Kernel qualification adds a [public contract review](docs/kernel-contract.md),
[executable standards coverage](docs/conformance.md), checked native queries and
syntax-only checking. The [dependency upgrade process](docs/dependency-upgrades.md)
defines how to preserve that evidence across engine/library changes.

The [standards requirements inventory](docs/requirements.md) indexes the full clause
structure, grammar, metamodel rules and normative companion artifacts. Its
[searchable browser](docs/requirements.html) separates indexed source units from
bounded implementation/API evidence; it makes no full-conformance percentage claim.
The [named validation-rule audit](docs/validation-audit.md) separates source search
hits, registry declarations and directly assessed behavior, and records open defects.

## Build and run

Development requires Node.js 24, Rust installed through rustup, and the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.
Python 3 is needed only for vendor verification/import scripts. Install the pinned
Rust toolchain and matching WebAssembly binding generator:

```sh
rustup toolchain install 1.97.0 --profile minimal --component rustfmt,clippy --target wasm32-unknown-unknown
cargo +1.97.0 install wasm-bindgen-cli --version 0.2.122 --locked
npm ci
npm run dev
```

Open `http://127.0.0.1:1473`. `npm run dev` builds the kernel first. Rust commands
in the npm scripts use the pinned rustup toolchain even if another Cargo is on PATH.
After changing Rust while the dev server is running, run `npm run kernel:build`
and reload the page.

```sh
npm run desktop                              # desktop development
npm run build                                # WASM + TypeScript + web production build
npm run desktop:build -- --bundles app --ci    # macOS application
```

The macOS bundle is `src-tauri/target/release/bundle/macos/SysML Viewer.app`.
A local Apple Silicon build is copied to `release/SysML Viewer.app`; refresh that
copy after rebuilding. Local builds are not Developer ID signed or notarized.
On Windows/Linux, `npm run desktop:build -- --ci` builds the platform's packages.
The CI workflow has runners for all three platforms; only macOS has been verified
in this workspace.

## Use

- **Open models** selects `.sysml` and `.kerml` files together. Include the files
  needed by imports; the viewer does not crawl folders or fetch dependencies.
  The standard SysML/KerML libraries are bundled. Drag-and-drop also works.
- **System map** fills the workspace with package regions and grouped cross-package relationships.
  View tabs float at the upper left; navigation and zoom controls sit in the lower corners.
  Zoom progressively reveals written members in stable positions. Drag or scroll
  to pan; pinch, Ctrl/⌘-scroll, or use the zoom buttons. Double-click an element or
  choose **Explore structure** to enter its effective structure; **Focus selection**
  approaches it without switching views. **Back** restores the previous location.
- Hover or Tab to a relationship to reveal its label; click or press Enter to inspect
  its underlying source records. The relationship menu at the upper right switches
  between types and connections; **Show labels** keeps labels visible where they fit.
  Selecting an element emphasizes its links while other links recede. TALA arranges linked ownership regions locally in a WebAssembly worker;
  unrelated members use deterministic packing. Layout stays fixed while zooming.
  TALA routes connect the regions it arranged; links into expanded regions use
  the viewer's obstacle-aware router. An unavailable layout leaves a simplified
  map with a **Retry** action.
  Use the relationship inspector to
  navigate to either endpoint. The map separates type relationships from binary
  connections. **Unmapped relationships** appears in that menu when unresolved or
  unsupported relationships exist; library
  references and annotation bodies are omitted from the map. **Browse relationships**
  opens a floating list of all groups at the current detail level, including those
  beyond the drawing limit. Escape closes the list or relationship menu.
- In **Diagram**, single-click a card to inspect it; double-click or choose **Explore** to view its
  effective members, including inherited members and redefinitions.
- In **System map**, selecting a structured element in the tree or choosing
  **Explore structure** opens its effective parts and connections. A usage such as
  `researchVessel : SurveyVessel` includes the inherited structure supplied by the
  kernel. Repeated uses have separate canvas positions and keep links to their
  original declarations; **From …** badges identify inherited members.
- Type names on cards are navigable labels. **Specializes …** or **Show type hierarchy**
  opens a separate definition hierarchy, with general types above specific types
  and hollow triangular arrows pointing to the general type. Usage typing does
  not become a specialization arrow. **Back** restores the previous view and camera.
- Leaf attribute definitions such as `Percentage` appear as clickable property
  types, without separate system-map nodes or arrows to a shared type. Attributes
  show their declared value at detail zoom (for example, `charge : Percentage = 100`).
  The explorer and type links still open those definitions and their hierarchy.
  Attribute definitions with structure, and leaf parts or ports, remain on the map.
- The tree shows written ownership. Breadcrumbs and **Up a level** navigate scopes.
- **Connections** draws resolved binary connections between visible owner cards.
  **Types & specialization** shows references; unresolved, ambiguous and
  out-of-view targets remain explicit.
- **Source** shows the original document and highlights the declaration. Reference
  navigation can cross files and open the source of bundled library definitions.
- A header **diagnostics badge** appears when analysis reports findings and opens
  their details. An empty list
  means the implemented checks found no problem, not that the model is certified.
- `⌘O` / `Ctrl+O` opens files; `/` focuses search. `⌘R` / `Ctrl+R` reloads selected
  desktop files. In a browser, select the files again to read external changes.

The built-in Coastal Explorer example contains four subsystems and five connections.
`SurveyVessel` specializes `ResearchVessel`, and `researchVessel` uses `SurveyVessel`.
Diagrams are a structural overview, not full standard SysML graphical notation.

## Command-line parsing and validation

The standalone CLI needs no browser, Node.js, Tauri, or runtime library downloads.
Build it with the repository's pinned Rust toolchain:

```sh
cargo build --locked --release -p sysml-cli
./target/release/sysml parse examples/CoastalExplorer.sysml
./target/release/sysml validate examples/
./target/release/sysml validate --format json examples/ > diagnostics.json
./target/release/sysml inspect examples/ > model.json
```

`parse` checks syntax and body structure without name resolution. `validate`
analyzes all supplied files together with the bundled standard libraries. Both
exit **1** on error diagnostics; `--warnings-as-errors` also fails on warnings.
`inspect` exports the versioned semantic snapshot and accepts `--strict` to fail
on error diagnostics. Input, usage, engine, or output failures exit **2**.

Files and directories can be mixed; directories are searched recursively for
`.sysml` and `.kerml` files. `-` reads source text from stdin, and `--request-json`
reads the programmatic `AnalysisRequest`. Windows uses `sysml.exe`.
The original `sysml-inspect` binary and `npm run inspect -- ...` remain compatible.
`npm run cli -- validate examples/` is also available during development; invoke
the binary directly when piping JSON so npm's banners do not enter the output.

See the [CLI guide](docs/cli.md) for installation, CI use, JSON contracts, discovery
rules, input limits, and release packaging. The Rust library remains independently
reusable; native callers must use its exported 16 MiB analysis-thread stack size.

| Component                   | Responsibility                                                             |
| --------------------------- | -------------------------------------------------------------------------- |
| `vendor/sysml-toolkit`      | Pinned upstream syntax and semantic libraries                              |
| `vendor/sysml-library`      | Pinned SysML 2.0 / KerML 1.0 model libraries                               |
| `crates/sysml-kernel`       | Source identities/ranges, analysis policy, diagnostics and query snapshots |
| `crates/sysml-cli`          | Native headless consumer                                                   |
| `crates/sysml-wasm`         | Thin WebAssembly binding                                                   |
| `src/kernel`                | Generated TypeScript contract and disposable worker transport              |
| `src/viewer`                | Display projection, diagram relationships and layout                       |
| `src/platform`, `src-tauri` | File access and desktop packaging                                          |
| `src/main.ts`, `src/ui`     | Interaction and rendering                                                  |

Rust types are authoritative. Run `npm run contract:generate` after changing
public types. Do not edit `src/kernel/generated` by hand. There is no second
TypeScript parser or name resolver.

## Checks

```sh
python3 tools/verify-vendor.py
npm run vendor:test
node tools/rust.mjs fmt -p sysml-kernel -p sysml-cli -p sysml-wasm -- --check
node tools/rust.mjs clippy --locked --workspace --all-targets --all-features -- -D warnings
npm run format:check
npm run contract:check
npm run conformance:check
npm run requirements:check
npm run kernel:test
npm test
npm run conformance
npm run requirements:report
npm run build
npm run benchmark
```

Format only our named Rust packages: `cargo fmt --all` also traverses vendored
path dependencies and would change the pinned upstream source.

Tests cover imports, standard-library resolution, effective members, redefinition,
connection chains and failed endpoints, ambiguous versus absent names, malformed
syntax recovery, Unicode/CRLF source positions, conjugated typing, valid handles
and revision lifetimes. A deterministic malformed-input sweep checks lexer coverage
and diagnostic ranges. Node tests compare native and WASM snapshots exactly and
check the viewer projection, source slicing and bundled example. This is the initial
acceptance suite, not a substitute for an official semantic conformance matrix or
continuous fuzzing.

The coverage matrix includes nominal declaration and incompatible-type cases for
24 SysML definition/usage kinds, plus cross-dialect classifier boundaries. The
[generated inventory summary](docs/requirements-summary.md) gives current counts. The matrix
checks standards-linked expectations, including two unchanged official examples,
with a fresh process and deadline per case. Its harness rejects new failures and
unexpected passes of known gaps. Reports are saved to `target/conformance/` and
uploaded by CI. The matrix document is generated from its JSON catalogue and checked
for drift. This is bounded coverage, not full standards certification.

Native/WASM benchmarks use three fresh processes for the bundled example, literal,
arithmetic and conditional providers, compound ranges, isolated expressions alongside missing bases,
imported membership closure, and two synthetic sizes. They record
whole-process time and peak RSS where available;
these are observations rather than production-model performance promises. Results
are in `target/benchmarks/latest.json`. See [qualification evidence](docs/foundation-qualification.md)
for the recorded run and remaining platform verification.

## Current limits

Analysis rebuilds the workspace for each load; incremental parsing/analysis and a
lossless editable CST are deferred. IDs last for one snapshot. Library records
support inspection and source navigation but not full member exploration. General expression
execution, simulation and complete graphical notation are outside this milestone.

Input limits: 32 files, 5 MB per file, 20 MB combined, 500,000 lexical tokens and
20,000 written declarations. The UI runs analysis in a disposable worker with a
60-second deadline. Superseded work is terminated and failed loads retain the
previous displayed model. The WASM memory ceiling is 1 GiB. These limits do not
promise acceptable performance for every model below them. Diagrams show up to
150 immediate members and search shows up to 300 matches.

## Dependency provenance

`dependencies.json` pins the engine and library independently. Imported source
files, original licenses and a hash manifest are checked in. The toolkit uses
Apache-2.0; the pinned March 2026 standard models carry LGPL-3.0 notices. Notices
are also included in the built web/app assets at `third-party-notices.txt`.
See the evaluation document and `tools/vendor-kernel.py` for the update procedure.

The [foundation qualification record](docs/foundation-qualification.md) describes
schema 4 membership/import/cardinality/state-subaction queries, namespace validation and the remaining
checked-evaluator gap. Full language conformance remains unestablished.
