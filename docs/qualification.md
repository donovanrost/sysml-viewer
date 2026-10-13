# Initial kernel qualification

This record describes the schema 1 baseline. See [foundation qualification](foundation-qualification.md) for the schema 2 changes and current evidence.

Recorded on 2026-10-12 in Hawaii (2026-10-13 UTC). This milestone establishes a
reviewed public contract and reproducible acceptance evidence. It does not certify
full SysML 2.0 or KerML 1.0 conformance.

## Verified behavior

- **21 Rust tests** pass: 11 analysis acceptance tests, nine contract tests and one
  deterministic malformed-input property sweep.
- **13 Node tests** pass: six kernel/viewer integration tests, four worker lifecycle
  tests and three coverage-harness tests. Native and WASM snapshots agree exactly
  for the shared integration request.
- **29 executable standards cases** pass. Two additional cases are explicitly
  planned and unexecuted; there are no waived failures in this run. Each case runs
  in a fresh native process with a 30-second deadline.
- Rust formatting, Clippy with warnings denied, TypeScript checking, Prettier,
  generated wire-contract consistency, matrix-document consistency and all 270
  vendored-file hashes pass.
- The production web build and Apple Silicon macOS application bundle build pass.
  The refreshed local app is `release/SysML Viewer.app` (11.86 MiB). This milestone
  adds automated worker checks; it does not repeat the prior milestone's interactive
  file-picker and navigation checks.

See the [public contract](kernel-contract.md), [coverage matrix](conformance.md)
and [saved matrix report](evidence/conformance-macos-arm64.json). The report records
fixture hashes, engine/library versions and each assertion's result. The matrix
links expectations to specification clauses and identifies uncovered scope.
Two fixtures are unchanged official examples from the pinned library release;
this run does not execute the Java pilot implementation as a differential oracle.

The review resulted in checked native query APIs and a syntax-only operation.
It also fixed two defects with retained regressions: a matching worker envelope
could carry a stale snapshot revision, and unresolved heritage could leave member
queries incorrectly marked complete. The latter fix propagates uncertainty through
dependent definitions and usages without changing vendored upstream code.

## Performance observations

Environment: Apple M1, eight logical CPUs, 16 GiB RAM, macOS/Darwin 24.6.0, arm64.
The [saved benchmark report](evidence/benchmark-macos-arm64.json) contains all 18
samples, input hashes, versions and environment metadata.

| Workload | Native median | WASM median | Native maximum peak RSS | WASM maximum peak RSS |
| --- | --- | --- | --- | --- |
| Bundled viewer example | 1,395 ms | 1,650 ms | 306.9 MiB | 289.4 MiB |
| Synthetic 100 usages | 1,362 ms | 1,667 ms | 281.6 MiB | 289.0 MiB |
| Synthetic 1,000 usages | 1,514 ms | 1,757 ms | 301.8 MiB | 301.7 MiB |

Each cell uses three fresh processes. Wall time includes process startup, WASM
initialization where applicable, complete analysis and JSON serialization/output.
Peak RSS covers the entire process, including Node/V8 for WASM. It is not the WASM
heap size. Builds finish before timing starts. These runs occurred on an ordinary
desktop, not a dedicated performance runner, and establish no hard latency budget.
All repeated and native/WASM snapshots matched exactly; the workloads produced no
diagnostics.

These small inputs have similar total times. That suggests fixed analysis costs
deserve profiling before introducing caching; it does not establish scalability
for larger or more complex models. Run `npm run benchmark` to reproduce the
workloads. Broader real-model measurements are required before making performance
commitments or selecting an incremental-analysis design.

## Platform and remaining qualification

The CI workflow runs checks, tests, the matrix and desktop packaging on macOS,
Windows and Linux, retaining coverage reports as artifacts. Only local Apple
Silicon macOS execution is verified here. No remote is configured, so Windows and
Linux jobs have not run.

Next qualification work is to run that CI matrix, add pilot differential cases,
exercise larger real models and establish continuous fuzzing. The two planned API
areas are namespace membership/alias enumeration and expression-valued multiplicity
with explicit completeness. They need consumer-driven contracts before implementation.

Use the [dependency upgrade procedure](dependency-upgrades.md) to preserve these
checks and evidence across future engine and standard-library changes.
