"""Copy pinned upstream sources from verified local checkouts, then apply recorded patches.

Only the two used Rust crates and the library sources are copied. Cargo git
dependencies otherwise fetch the toolkit's large, unrelated corpus submodules.
Run verify-vendor.py after importing. Updating a pin requires rerunning acceptance.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
pins = json.loads((ROOT / "dependencies.json").read_text())
parser = argparse.ArgumentParser()
parser.add_argument("--toolkit", type=Path, required=True)
parser.add_argument("--library", type=Path, required=True)
args = parser.parse_args()
hashes = {}

def copy(source, destination):
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)
    hashes[str(destination.relative_to(ROOT))] = hashlib.sha256(destination.read_bytes()).hexdigest()

for key, checkout in [("toolkit", args.toolkit), ("library", args.library)]:
    actual = subprocess.check_output(["git", "-C", str(checkout), "rev-parse", "HEAD"], text=True).strip()
    if actual != pins[key]["revision"]:
        raise SystemExit(f"{key}: expected {pins[key]['revision']}, got {actual}")
    dirty = subprocess.check_output(["git", "-C", str(checkout), "status", "--porcelain", "--untracked-files=no"], text=True)
    if dirty:
        raise SystemExit(f"{key}: tracked files are modified")

vendor = ROOT / "vendor"
for name in ["sysmlv2-syntax", "sysmlv2-model"]:
    base = args.toolkit / "crates" / name
    for file in sorted(base.rglob("*")):
        rel = file.relative_to(base)
        if file.is_file() and (rel.parts[0] == "src" or str(rel) in ["Cargo.toml", "build.rs"]):
            copy(file, vendor / "sysml-toolkit" / "crates" / name / rel)
for name in ["LICENSE", "README.md"]:
    copy(args.toolkit / name, vendor / "sysml-toolkit" / name)
copy(args.toolkit / "local-packages/TransformMeta.sysml", vendor / "sysml-toolkit/local-packages/TransformMeta.sysml")

# The sole packaging adaptation: a two-crate workspace. All inherited values
# are copied verbatim from the pinned root manifest, including feature flags.
manifest = (args.toolkit / "Cargo.toml").read_text()
sections = manifest.split("[workspace.package]", 1)[1].split("# The wasi CLI", 1)[0]
minimal = '[workspace]\nresolver = "2"\nmembers = ["crates/sysmlv2-syntax", "crates/sysmlv2-model"]\n\n[workspace.package]' + sections
manifest_path = vendor / "sysml-toolkit" / "Cargo.toml"
manifest_path.write_text(minimal)
hashes[str(manifest_path.relative_to(ROOT))] = hashlib.sha256(manifest_path.read_bytes()).hexdigest()

for file in sorted((args.library / "sysml.library").rglob("*")):
    if file.is_file():
        copy(file, vendor / "sysml-library" / "sources" / file.relative_to(args.library / "sysml.library"))
for name in ["LICENSE", "LICENSE-GPL", "README.md"]:
    copy(args.library / name, vendor / "sysml-library" / name)
(vendor / "SHA256SUMS.json").write_text(json.dumps(hashes, indent=2, sort_keys=True) + "\n")
print(f"Vendored {len(hashes)} files from verified revisions.")
subprocess.run(["node", str(ROOT / "tools/verify-vendor.mjs"), "--apply"], check=True)
