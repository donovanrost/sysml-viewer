# SysML Viewer

A local SysML v2 viewer with a reusable Rust language kernel, a TypeScript interface,
and a Tauri desktop host. The same kernel runs natively in a command-line tool and
as WebAssembly in the desktop/browser viewer. Models stay on your device; analysis
has no runtime server, account, telemetry or network dependency and never writes
model files.

The first kernel migration milestone is implemented. Open-MBEE's syntax and model
libraries provide lexing, parsing, semantic construction, resolution and validation
behind our own versioned facade. This is **not a claim of complete SysML conformance**.
Read [the architecture](docs/architecture.md) and [the engine evaluation](docs/kernel-evaluation.md)
for the boundaries, evidence and remaining work.

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
- Single-click a card to inspect it; double-click or choose **Explore** to view its
  effective members, including inherited members and redefinitions.
- The tree shows written ownership. Breadcrumbs and **Up a level** navigate scopes.
- **Connections** draws resolved binary connections between visible owner cards.
  **Types & specialization** shows references; unresolved, ambiguous and
  out-of-view targets remain explicit.
- **Source** shows the original document and highlights the declaration. Reference
  navigation can cross files and open the source of bundled library definitions.
- **Diagnostics** reports syntax, resolution and semantic findings. An empty list
  means the implemented checks found no problem, not that the model is certified.
- `⌘O` / `Ctrl+O` opens files; `/` focuses search. `⌘R` / `Ctrl+R` reloads selected
  desktop files. In a browser, select the files again to read external changes.

The built-in Coastal Explorer example contains four subsystems and five connections.
Diagrams are a structural overview, not full standard SysML graphical notation.

## Reuse the kernel without the viewer

```sh
npm run inspect -- examples/CoastalExplorer.sysml
node tools/rust.mjs build --locked --release -p sysml-cli
./target/release/sysml-inspect --strict tests/fixtures/kernel/Definitions.sysml tests/fixtures/kernel/Mission.sysml
```

The executable writes a schema-versioned JSON snapshot. `--strict` exits 1 when
there are error diagnostics; invalid input exits 2. Windows uses `sysml-inspect.exe`.
For programmatic use, `sysml-inspect --request-json` accepts an `AnalysisRequest`
on stdin. The Rust library accepts supplied source text and returns the same
snapshot without depending on a UI or filesystem. Native callers must use the
exported 16 MiB analysis-thread stack size.

| Component | Responsibility |
| --- | --- |
| `vendor/sysml-toolkit` | Pinned upstream syntax and semantic libraries |
| `vendor/sysml-library` | Pinned SysML 2.0 / KerML 1.0 model libraries |
| `crates/sysml-kernel` | Source identities/ranges, analysis policy, diagnostics and query snapshots |
| `crates/sysml-cli` | Native headless consumer |
| `crates/sysml-wasm` | Thin WebAssembly binding |
| `src/kernel` | Generated TypeScript contract and disposable worker transport |
| `src/viewer` | Display projection, diagram relationships and layout |
| `src/platform`, `src-tauri` | File access and desktop packaging |
| `src/main.ts`, `src/ui` | Interaction and rendering |

Rust types are authoritative. Run `npm run contract:generate` after changing
public types. Do not edit `src/kernel/generated` by hand. There is no second
TypeScript parser or name resolver.

## Checks

```sh
python3 tools/verify-vendor.py
node tools/rust.mjs fmt -p sysml-kernel -p sysml-cli -p sysml-wasm -- --check
node tools/rust.mjs clippy --locked --workspace --all-targets --all-features -- -D warnings
npm run format:check
npm run contract:check
npm run kernel:test
npm test
npm run build
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

Verified locally on 2026-10-12: 12 Rust tests, 6 Node/WASM integration tests,
formatting, Clippy, generated contract consistency, vendor hashes, production web
build and Apple Silicon macOS bundle. Interactive checks covered browser multi-file
imports, inheritance and source navigation, distinct ambiguity/missing-name
diagnostics, and native file selection with successful bundled WASM analysis.
Windows/Linux execution and packaging remain for their CI runners.

## Current limits

Analysis rebuilds the workspace for each load; incremental parsing/analysis and a
lossless editable CST are deferred. IDs last for one snapshot. Library records
support inspection and source navigation but not full member exploration. Expression
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
