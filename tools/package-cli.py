#!/usr/bin/env python3
"""Package already-tested native CLI binaries without desktop dependencies."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import tarfile
import tempfile
import zipfile
from release_version import check


def main():
    root = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, choices=[
        "aarch64-apple-darwin", "x86_64-apple-darwin",
        "x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc",
    ])
    parser.add_argument("--bin-dir", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, default=root / "release" / "cli")
    args = parser.parse_args()
    version = check(root)
    windows = "windows" in args.target
    suffix = ".exe" if windows else ""
    binaries = [args.bin_dir / f"{name}{suffix}" for name in ("sysml", "sysml-inspect")]
    for binary in binaries:
        if not binary.is_file():
            parser.error(f"Missing binary: {binary}; build sysml-cli first")
    name = f"sysml-{version}-{args.target}"
    args.output_dir.mkdir(parents=True, exist_ok=True)
    archive = args.output_dir / (name + (".zip" if windows else ".tar.gz"))
    with tempfile.TemporaryDirectory(prefix="sysml-package-") as directory:
        stage = Path(directory) / name
        (stage / "bin").mkdir(parents=True)
        for binary in binaries:
            shutil.copy2(binary, stage / "bin" / binary.name)
            if not windows:
                (stage / "bin" / binary.name).chmod(0o755)
        shutil.copy2(root / "docs" / "cli.md", stage / "README.md")
        shutil.copy2(root / "public" / "third-party-notices.txt", stage / "THIRD-PARTY-NOTICES.txt")
        (stage / "licenses").mkdir()
        for source, destination in [
            ("vendor/sysml-toolkit/LICENSE", "sysml-toolkit-LICENSE.txt"),
            ("vendor/sysml-library/LICENSE", "sysml-library-LICENSE.txt"),
            ("vendor/sysml-library/LICENSE-GPL", "sysml-library-LICENSE-GPL.txt"),
        ]:
            shutil.copy2(root / source, stage / "licenses" / destination)
        manifest = {
            "version": version, "target": args.target,
            "binaries": {
                binary.name: hashlib.sha256(binary.read_bytes()).hexdigest()
                for binary in binaries
            },
        }
        (stage / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        if windows:
            with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as output:
                for file in sorted(stage.rglob("*")):
                    if file.is_file():
                        output.write(file, file.relative_to(directory))
        else:
            with tarfile.open(archive, "w:gz") as output:
                output.add(stage, arcname=name)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_name(archive.name + ".sha256").write_text(f"{digest}  {archive.name}\n")
    print(archive)


if __name__ == "__main__":
    main()
