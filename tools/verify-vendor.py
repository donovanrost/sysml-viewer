"""Compatibility entry point for the shared source-and-patch verifier."""
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
subprocess.run(["node", str(root / "tools/verify-vendor.mjs"), *sys.argv[1:]], check=True)
