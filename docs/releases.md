# Publishing releases

All development releases use **0.0.x**. The initial release is `0.0.1`.
The workflow marks every `0.0.x` release as a GitHub prerelease with Latest disabled.
Only an explicitly pushed version tag outside that series (for example `v0.1.0`)
produces an official release. Branch builds and manual workflow runs do not publish.

## Prepare and publish a development release

1. Run `python3 tools/release_version.py --set 0.0.2` (use the next unused patch).
   This updates the Rust workspace, desktop crate/config, npm manifests and
   lockfiles together without changing dependency versions.
2. Write `docs/releases/0.0.2.md` with the features, fixes and known limits.
3. Run the release-version tests and the CLI/kernel contract checks. Commit the
   complete source used to build the release and push that commit to `origin`.
4. Create an annotated tag on that commit and push it:

   ```sh
   git tag -a v0.0.2 -m 'SysML 0.0.2 development prerelease'
   git push origin v0.0.2
   ```

The **CLI builds and releases** workflow builds/tests Apple Silicon macOS, Intel
macOS, Linux x86-64, and Windows x86-64 binaries. Its publishing job verifies all
four archives and their binary/manifest checksums before creating a draft and
uploading assets. Only the completed draft becomes a published release. It checks
that the remote tag still identifies the tested source commit. Publishing uses
the repository's `GITHUB_TOKEN`; write permission is limited to the publish job.

Build or upload failures do not publish a partial release. If an upload fails
after draft creation, inspect the draft and fix it or remove that unpublished
draft before retrying. Reruns never replace a published release or its assets.
Do not move published tags; fix problems in the next patch release.

## Official release

When the project is ready, deliberately set an official version, write its release
notes, commit it, and push the matching annotated `v...` tag. For example `v0.1.0`
is an official early release; it need not imply a 1.0 compatibility guarantee.
No untagged build or manual workflow run promotes a development build to official.

## Checks and local packaging

```sh
python3 tools/release_version.py
python3 -m unittest discover -s tests -p 'test_release*.py'
cargo test --locked --release -p sysml-cli -p sysml-kernel --test cli --test contracts
cargo build --locked --release -p sysml-cli
python3 tools/package-cli.py --target aarch64-apple-darwin --bin-dir target/release
```

The attached release assets currently contain the CLI and compatibility inspector,
their guide, notices and manifest. The viewer shares the same product version and
its desktop CI continues to build viewer packages as workflow artifacts. Versions
of vendored dependencies, snapshot schemas and internal evaluation tools are
independent of the product release version.
