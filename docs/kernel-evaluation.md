# Kernel decision: Open-MBEE behind a versioned facade

Decision date: 2026-10-12. This is a bounded integration evaluation, not a complete
correctness or security audit of either implementation.

| Candidate | Revision | Evidence |
| --- | --- | --- |
| Open-MBEE sysml-toolkit 0.10.2 | `821221767c3c56cb1ebe7da22666197a47c9c645` | Compiled; exercised imports, standard-library names, effective members, redefinition, connection chains, ambiguity and missing names |
| Syster 0.5.1-alpha | `6a355413ee4ac6dcd8d2110c29646746e3e1e1e7` | Compiled; exercised the same written sources through its public analysis/index APIs |
| SysML v2 Pilot Implementation | Not executed | Reviewed as a future comparison implementation; JVM packaging and full differential testing deferred |

Both Rust candidates parsed the authored fixture without errors. Open-MBEE returned
the expected imported type, inherited `mass`, redefining `SpecialDevice::signal`,
and the two connection chains through `vehicle::a` and `vehicle::b`. It reported
`Duplicate` as ambiguous and `Absent` as unresolved.

Syster resolved the imported type and `ScalarValues::Real`, and found inherited
`mass` and `serial`. Its queried resolver returned `is_ambiguous() == false` for
the two wildcard-imported `Duplicate` definitions. Its member lookup returned the
redefined port under the synthetic name `Components::SpecialDevice::<:Signal#1@L11>`.
These are observations of this probe/API, not claims that every Syster API has the
same behavior. Connection-chain enumeration was not evaluated through Syster.

Open-MBEE is selected because its semantic queries satisfy the first viewer
milestone directly, and its model libraries compile for native and WASM targets.
Syster's lossless CST and incremental design remain attractive, but they do not
outweigh the observed semantic/API differences for this milestone.

## Reproduce

```sh
node tools/rust.mjs run --locked --manifest-path tools/kernel-evaluation/Cargo.toml -- \
  tests/fixtures/kernel vendor/sysml-library/sources
```

The evaluation program is separate from product dependencies. Syster is fetched
only when running it. The result is recorded in `kernel-evaluation-results.json`.
Product acceptance tests assert expectations rather than just printing results.

## Packaging and provenance

The desired Open-MBEE release is not published under these crate versions in the
public registry. A Cargo git dependency also fetches large, unrelated corpus
submodules. We therefore vendor only the used syntax/model crates, their required
ambient model, README and license. Upstream source files are unchanged. The single
packaging adaptation is a reduced Cargo workspace manifest.

`dependencies.json` records source pins. `tools/vendor-kernel.py` verifies checkout
revisions and clean tracked files before copying. `vendor/SHA256SUMS.json` records
every imported file; `tools/verify-vendor.py` rejects modified or extra files.
Changing the hashes is not a substitute for rerunning the importer and tests.

The standard library is pinned separately to SysML-v2-Release `2026-03`, commit
`cd99f7ca70b96abb38f09dfd25725e3cf259baa3`. This preserves the SysML 2.0 / KerML 1.0
baseline instead of implicitly adopting newer beta libraries. This release's
models carry LGPL-3.0 notices; the toolkit carries Apache-2.0. Sources and original
license texts are retained under `vendor/`, with notices also in web/app assets.

## Known gaps and adaptation choices

- Validation is incomplete upstream. A clean analysis is not certification.
- Syntax is a source-faithful AST plus original text, not a lossless CST API.
- Our workspace currently rebuilds; no incremental editing claim is made.
- Querying ambiguity currently requires a second graph build through the public API.
- A failed connector chain can retain a resolved prefix upstream. The facade
  conservatively marks its ends unsupported if the declaration contains unresolved
  references, rather than presenting a partial prefix as a complete endpoint.
- Library records in a snapshot support inspection/source navigation; expanding
  their member queries is deferred.
- Full source-level coverage, an official semantic-rule matrix, and execution of
  the pilot implementation remain future evaluation work.

## Upstream references

- https://github.com/Open-MBEE/sysml-toolkit
- https://github.com/jade-codes/syster-base
- https://github.com/Systems-Modeling/SysML-v2-Pilot-Implementation
- https://www.omg.org/spec/SysML/2.0
- https://www.omg.org/spec/KerML/1.0
