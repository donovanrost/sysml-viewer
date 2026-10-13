# Maintaining the language dependency

The application owns its facade and acceptance expectations. Upstream owns the
syntax and semantic implementation. Vendor updates are reviewed changes, not an
automatic background update or a replacement of failing expected results.

## Upgrade procedure

1. Start from a committed passing baseline. Choose exact toolkit and standard
   library revisions independently; check which normative language editions the
   new library targets. Review upstream release notes, API changes and license
   notices. Keep a summary of why the upgrade is needed.
2. Obtain clean checkouts, verify their commit IDs, and update `dependencies.json`.
   Run `python3 tools/vendor-kernel.py --toolkit CHECKOUT --library CHECKOUT`.
   The importer verifies tracked cleanliness and revision equality. Inventory/hash
   checking detects leftover files: remove only files verified to have been removed
   upstream, or import into a clean staging checkout and review the diff.
3. Review all imported changes. Our only current toolkit packaging adaptation is
   its reduced workspace manifest. Preserve source and notices; do not make silent
   edits inside `vendor/`. If a source patch is unavoidable, first add a failing
   regression, document its upstream revision and rationale, and introduce an
   explicit patch/provenance mechanism rather than hand-editing the hash manifest.
4. If the library revision changes, refresh the official conformance fixtures from
   the same revision using their recorded upstream paths. Preserve bytes and update
   `tests/conformance/upstream/provenance.json`. Verify the expectations against the
   relevant specification instead of copying the new engine's output as truth.
5. Update `ENGINE_VERSION`/`LIBRARY_VERSION`, toolchain pins and Cargo locks as
   needed. Keep the WASM binding generator exactly aligned with its crate version.
   Regenerate TypeScript only if public Rust wire types changed. Reassess the
   schema version when the meaning or shape of an output changes.
6. Run vendor verification, formatting, Clippy, contract and matrix-document
   checks, Rust/Node tests, the executable matrix, benchmarks and platform builds.
   Preserve matrix/benchmark reports for comparison. CI must run on all three OSes
   before calling an upgrade cross-platform verified.
7. Investigate every new failure and unexpected pass. A known gap is tied to a
   named assertion, issue identifier and explanation; a crash, timeout or unrelated
   failure cannot inherit that exemption. Remove a resolved gap after reviewing the
   fix and retaining its regression case.
   Run `npm run requirements:check` and review changed implementation mappings
   before refreshing their hashes in `standards/assessments.json`. Run
   `npm run requirements:report` against the new matrix execution. Specification
   source pins are separate from toolkit/library pins; follow the
   [inventory extraction procedure](requirements.md) if changing the normative baseline.
8. Update `public/third-party-notices.txt`, qualification evidence and decision
   notes, then commit the upgrade as a reviewable change. Public release additionally
   requires choosing the project's own license and completing distribution setup.

## Reporting an engine defect

Reduce the model to the smallest reproducer while preserving original behavior.
Record the pinned engine/library versions, dialect, source files, expected result,
actual result and the relevant specification clause. Determine whether the fault
belongs to source mapping, our projection, a display rule or the upstream engine.
Add a regression before implementing a fix. Keep local issue IDs in the matrix
until an upstream issue exists; record the upstream link when filed.

Submitting an upstream issue is an external communication and is a separate action;
this procedure does not automatically file issues or send user models anywhere.

## Evidence and operational limits

`target/conformance/report.json` records inputs by hash, version pins, per-assertion
results and process durations. `target/benchmarks/latest.json` records repeatable
synthetic workloads, environment metadata, three samples, and whole-process peak
RSS where available. Benchmark values are observations, not pass/fail latency
thresholds. Use dedicated, otherwise idle runners before adopting regression budgets.

The current repository has no remote configured. CI is ready to run after it is
pushed to an authorized repository; local macOS results do not substitute for
Windows/Linux runs. No pilot-implementation differential run or continuous fuzzing
campaign is claimed by this milestone.
