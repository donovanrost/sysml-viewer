"""Fail if a vendored source changed or an unrecorded source was added."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
hashes = json.loads((root / "vendor/SHA256SUMS.json").read_text())
actual = {str(p.relative_to(root)) for base in ["sysml-toolkit", "sysml-library"] for p in (root / "vendor" / base).rglob("*") if p.is_file()}
assert actual == set(hashes), "Vendored file inventory changed; use the importer."
for name, expected in hashes.items():
    assert hashlib.sha256((root / name).read_bytes()).hexdigest() == expected, f"Modified vendor file: {name}"
print(f"Verified {len(hashes)} vendored files.")
