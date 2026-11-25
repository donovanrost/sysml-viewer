#!/usr/bin/env python3
"""Keep product versions synchronized and enforce development/release tag policy."""
import argparse
import json
import os
from pathlib import Path
import re
import tomllib

ROOT = Path(__file__).resolve().parent.parent
VERSION = re.compile(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\Z")
LOCK_PACKAGES = {
    "Cargo.lock": ["sysml-cli", "sysml-kernel", "sysml-wasm"],
    "src-tauri/Cargo.lock": ["sysml-viewer"],
}


def versions(root=ROOT):
    result = {
        "Cargo.toml": tomllib.loads((root / "Cargo.toml").read_text())["workspace"]["package"]["version"],
        "src-tauri/Cargo.toml": tomllib.loads((root / "src-tauri/Cargo.toml").read_text())["package"]["version"],
    }
    for path in ["package.json", "package-lock.json", "src-tauri/tauri.conf.json"]:
        data = json.loads((root / path).read_text())
        result[path] = data["version"]
        if path == "package-lock.json":
            result[path + ":root"] = data["packages"][""]["version"]
    for path, names in LOCK_PACKAGES.items():
        packages = tomllib.loads((root / path).read_text())["package"]
        for name in names:
            result[f"{path}:{name}"] = next(p["version"] for p in packages if p["name"] == name)
    return result


def check(root=ROOT):
    found = versions(root)
    version = found["Cargo.toml"]
    if not VERSION.fullmatch(version) or len(set(found.values())) != 1:
        raise ValueError(f"Product versions must match: {found}")
    return version


def release_policy(version, tag=None, publish=False):
    if not VERSION.fullmatch(version):
        raise ValueError("Use a numeric major.minor.patch version")
    if tag is not None and tag != f"v{version}":
        raise ValueError(f"Tag must match the product version: v{version}")
    prerelease = version.startswith("0.0.")
    if publish and not prerelease and tag is None:
        raise ValueError("An official release requires an explicitly pushed version tag")
    return {"version": version, "tag": f"v{version}", "prerelease": prerelease}


def set_version(version, root=ROOT):
    if not VERSION.fullmatch(version):
        raise ValueError("Use a numeric major.minor.patch version")
    # Read/prepare everything before writing; unrelated dependency versions stay intact.
    files = {}
    for path, section in [("Cargo.toml", "workspace.package"), ("src-tauri/Cargo.toml", "package")]:
        text = (root / path).read_text()
        # Both manifests have version in this section; constrain the replacement
        # to that section instead of changing dependency or tool versions.
        start = text.index(f"[{section}]")
        end = text.find("\n[", start + 1)
        end = len(text) if end == -1 else end
        block, count = re.subn(r'(?m)^version = "[^"]+"$', f'version = "{version}"', text[start:end])
        if count != 1:
            raise ValueError(f"Expected one version in {path}")
        files[path] = text[:start] + block + text[end:]
    for path in ["package.json", "package-lock.json", "src-tauri/tauri.conf.json"]:
        data = json.loads((root / path).read_text())
        data["version"] = version
        if path == "package-lock.json":
            data["packages"][""]["version"] = version
        files[path] = json.dumps(data, indent=2) + "\n"
    for path, names in LOCK_PACKAGES.items():
        text = (root / path).read_text()
        for name in names:
            pattern = rf'(\[\[package\]\]\nname = "{re.escape(name)}"\nversion = ")[^"]+("\n)'
            text, count = re.subn(pattern, lambda m: m[1] + version + m[2], text)
            if count != 1:
                raise ValueError(f"Expected one {name} package in {path}")
        files[path] = text
    for path, text in files.items():
        (root / path).write_text(text)
    check(root)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--set", metavar="VERSION")
    parser.add_argument("--tag", help="Require this exact pushed tag to match the version")
    parser.add_argument("--publish", action="store_true")
    parser.add_argument("--github-output", action="store_true")
    args = parser.parse_args()
    try:
        if args.set:
            set_version(args.set)
        result = release_policy(check(), args.tag, args.publish)
        if args.github_output:
            with open(os.environ["GITHUB_OUTPUT"], "a") as output:
                for key, value in result.items():
                    output.write(f"{key}={str(value).lower() if isinstance(value, bool) else value}\n")
        print(json.dumps(result))
    except (ValueError, KeyError, StopIteration) as error:
        parser.error(str(error))


if __name__ == "__main__":
    main()
