# SysML command-line tool

The CLI uses the viewer's Rust kernel and bundled SysML/KerML libraries. It reads
models without modifying them and performs no runtime downloads. No Node.js,
browser, desktop application, or account is needed to run the binaries.

## Install or build

The **CLI builds and releases** GitHub Actions workflow produces versioned archives for
Apple Silicon macOS, Intel macOS, Linux x86-64, and Windows x86-64. Download the
asset for your platform from [GitHub Releases](https://github.com/donovanrost/sysml-viewer/releases) and extract its
`.tar.gz` or `.zip`. Add the extracted `bin` directory to PATH, or call its `sysml`
executable directly (`sysml.exe` on Windows). Each archive includes both commands,
this guide, third-party notices and a build manifest; a companion SHA-256 file
checks the archive. These are unsigned builds, not notarized installers.

To build from this repository, install Rust through rustup and a native C linker
(Xcode Command Line Tools on macOS, C build tools on Linux, MSVC Build Tools on
Windows). The repository's `rust-toolchain.toml` selects the toolchain:

```sh
cargo build --locked --release -p sysml-cli
./target/release/sysml --version
```

For a source installation into Cargo's executable directory:

```sh
cargo install --locked --path crates/sysml-cli
```

`npm run cli -- <arguments>` is a development convenience. Existing
`npm run inspect -- <arguments>` still invokes the compatibility inspector.
Use the executable directly when consuming JSON; npm prints its own banners.

## Commands

```sh
sysml parse model.sysml
sysml validate models/
sysml validate --format json models/ > diagnostics.json
sysml inspect models/ > model.json
sysml inspect --strict models/ > model.json
```

| Command | Work performed | Default output | Error diagnostics |
| --- | --- | --- | --- |
| `parse` | Syntax and body-structure checking, no libraries or name resolution | Text | Exit 1 |
| `validate` | Parsing, cross-file references and implemented semantic checks | Text | Exit 1 |
| `inspect` | Same full analysis, with elements, memberships, references and connections | Snapshot JSON | Exit 0; use `--strict` for exit 1 |

`parse` currently reports parsing diagnostics; it does **not** export an AST.
Use `inspect` for structured semantic model data. Validation coverage is the same
as the viewer's: a clean report means the implemented checks found no errors,
not proof of complete SysML conformance. See [coverage](conformance.md).

`parse` and `validate` support `--format text` and `--format json` (also
`--format=json`). All commands support `--warnings-as-errors`, which fails on
either errors or warnings. `sysml --help`, `sysml <command> --help` and
`sysml --version` need no models.
Unresolved references currently produce kernel warnings; use
`--warnings-as-errors` when they must fail your validation gate.

## Inputs and discovery

- Supply one or more files or directories. All discovered files form one analysis
  workspace, so include the definitions needed by imports and type references.
- Directories are walked recursively in sorted filename order. Only `.sysml` and
  `.kerml` files are included; extension matching is case insensitive. Hidden
  directories and symlink entries encountered while walking are skipped. Explicit
  paths can name hidden directories or symlinked files/directories.
- Files reached through multiple paths are deduplicated by their canonical path.
  The first supplied/discovered path remains the document ID and display name.
- An explicit missing, unreadable, non-model or non-regular file is an error.
  An empty discovery result is an error. Use `--` before paths beginning with `-`.
- `-` reads a single source from stdin, optionally together with files. Its name
  defaults to `stdin.sysml`; `--stdin-name draft.kerml` chooses KerML syntax.
- `--request-json` instead reads an `AnalysisRequest` from stdin and cannot be
  combined with file, directory or `-` inputs. Caller-supplied IDs and revision
  are preserved. IDs must be unique, nonempty and outside `library://`.

```sh
printf 'package Demo { part def Vessel; }' | sysml parse -
printf 'package Demo { class Vessel; }' | sysml parse --stdin-name draft.kerml -
sysml validate definitions.sysml - < draft.sysml
sysml inspect --request-json < request.json
```

Example `request.json`:

```json
{"revision":1,"documents":[{"id":"draft","name":"draft.sysml","text":"package Demo;"}]}
```

The kernel currently permits at most **32 documents**, **5 MB per source**,
**20 MB of source text per workspace**, and **500,000 lexical tokens**. Semantic
analysis also limits written declarations to 20,000. CLI file/stdin reads are
bounded before allocation; encoded JSON requests have a 25 MB host limit. Large
projects must be supplied as smaller, self-contained workspaces for now.

## Output and exit status

Text diagnostics go to **stderr**, one per line:

```text
models/Mission.sysml:3:18: error[resolution]: ...
```

The text summary goes to stdout. Terminal control characters in filenames and
messages are escaped. Locations use one-based lines and UTF-16 columns from the
kernel. Reports are sorted by document ID, source offset, code and message.

JSON mode emits one complete JSON value to **stdout**, including all diagnostics;
there is no duplicate diagnostic stream on stderr. On an input/engine failure,
stderr explains the problem and no report is emitted. Output failures may leave
partial output. JSON ends with a newline.

| Exit | Meaning |
| --- | --- |
| `0` | Command completed and its selected diagnostic policy passed |
| `1` | Analysis completed but diagnostics failed the selected policy |
| `2` | Usage, input, engine or output failure |

`parse`/`validate` JSON has its own **report schema version 1**: `command`,
`revision`, `engineVersion`, `libraryVersion` (null for syntax-only), `documents`
(IDs and names, without source text), `diagnostics`, `summary` (files, errors,
warnings), and `passed`. Diagnostics preserve the kernel's codes, severity,
stage and exact source ranges. `passed` reflects `--warnings-as-errors` too.

`inspect` emits the existing **snapshot schema version 4**, including its source
documents and bundled-library records. It is a query snapshot, not a full SysML
interchange format. `sysml-inspect` retains the previous file/`--request-json`
interface, snapshot schema, and opt-in `--strict` behavior.

## CI and release builds

Once the executable is on PATH, a CI validation step can simply run:

```sh
sysml validate --warnings-as-errors models/
```

For JSON evidence, redirect `--format json` output to a file; retain its exit
status so errors still fail the job. Avoid suppressing failures with `|| true`.

The repository's `.github/workflows/cli.yml` tests both command interfaces and
builds archives on each platform. Relevant branch pushes/pull requests and manual
dispatch build workflow artifacts. Pushing a matching `vMAJOR.MINOR.PATCH` tag
publishes a GitHub Release only after all four platform jobs pass and the complete
set of archives, manifests and checksums is verified.

Development releases are **0.0.x**, always marked as prereleases and never marked
Latest. Official releases require an explicit tag outside 0.0.x. A moved tag,
version mismatch, failed build or incomplete asset set blocks publication. Existing
releases are never overwritten by reruns. See [release instructions](releases.md).

To package a local native build (Python 3.11+):

```sh
python3 tools/package-cli.py --target aarch64-apple-darwin --bin-dir target/release
```

Archives and checksums are written to `release/cli/`. Packaging does not build
or sign the binaries. Run `cargo test --locked --release -p sysml-cli` and the
kernel contract tests before packaging.
