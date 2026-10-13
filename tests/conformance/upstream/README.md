# Official example provenance

These two files are unchanged copies from SysML-v2-Release 2026-03, the same
revision as the bundled library. `provenance.json` records their source paths and
SHA-256 hashes. The conformance runner verifies those hashes before every run.

Original repository: https://github.com/Systems-Modeling/SysML-v2-Release

Revision: `cd99f7ca70b96abb38f09dfd25725e3cf259baa3`

The pinned release carries LGPL-3.0 notices. The unchanged release
[README](../../../vendor/sysml-library/README.md),
[LGPL text](../../../vendor/sysml-library/LICENSE), and
[GPL text](../../../vendor/sysml-library/LICENSE-GPL) are retained in the repository.

To update, use `git show <pinned-revision>:<upstreamPath>` from a verified checkout,
copy bytes without line-ending conversion, and recompute the recorded hashes.
Updating this provenance also requires updating `dependencies.json` and reviewing
the expectations; see `docs/dependency-upgrades.md`. Do not edit these examples
to make a test pass. Add an independently authored regression fixture instead.
