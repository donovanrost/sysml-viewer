#!/usr/bin/env python3
"""Reject incomplete or mismatched CLI release assets before publication."""
import argparse
import hashlib
import json
from pathlib import Path
import tarfile
import zipfile
from release_version import release_policy

TARGETS = (
    "aarch64-apple-darwin", "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc",
)


def verify(directory, version):
    release_policy(version)
    expected = set()
    for target in TARGETS:
        base = f"sysml-{version}-{target}"
        name = base + (".zip" if "windows" in target else ".tar.gz")
        expected.update([name, name + ".sha256"])
    actual = {p.name for p in directory.iterdir()}
    if actual != expected:
        raise ValueError(f"Incomplete release: missing {sorted(expected - actual)}, unexpected {sorted(actual - expected)}")
    for target in TARGETS:
        base = f"sysml-{version}-{target}"
        name = base + (".zip" if "windows" in target else ".tar.gz")
        archive = directory / name
        checksum = (directory / (name + ".sha256")).read_text().split()
        if checksum != [hashlib.sha256(archive.read_bytes()).hexdigest(), name]:
            raise ValueError(f"Checksum mismatch: {name}")
        if archive.suffix == ".zip":
            with zipfile.ZipFile(archive) as bundle:
                manifest = json.loads(bundle.read(base + "/manifest.json"))
                check_binaries(manifest, bundle.read, base, target)
        else:
            with tarfile.open(archive) as bundle:
                def read(path):
                    member = bundle.getmember(path)
                    if not member.isfile():
                        raise ValueError(f"Expected a regular archive member: {path}")
                    return bundle.extractfile(member).read()
                manifest = json.loads(read(base + "/manifest.json"))
                check_binaries(manifest, read, base, target)
        if manifest["version"] != version or manifest["target"] != target:
            raise ValueError(f"Manifest version/target mismatch: {name}")


def check_binaries(manifest, read, base, target):
    suffix = ".exe" if "windows" in target else ""
    names = {"sysml" + suffix, "sysml-inspect" + suffix}
    if set(manifest["binaries"]) != names:
        raise ValueError(f"Unexpected binaries for {target}")
    for name, digest in manifest["binaries"].items():
        if hashlib.sha256(read(base + "/bin/" + name)).hexdigest() != digest:
            raise ValueError(f"Binary checksum mismatch: {name}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    verify(args.directory, args.version)
    print(f"Verified four CLI archives and checksums for {args.version}")
