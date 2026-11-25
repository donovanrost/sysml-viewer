import hashlib
import io
import json
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "tools"))
from verify_cli_release import TARGETS, verify


def assets(directory, manifest_version="0.0.1"):
    for target in TARGETS:
        base = f"sysml-0.0.1-{target}"
        windows = "windows" in target
        suffix = ".exe" if windows else ""
        binaries = {"sysml" + suffix: b"test executable", "sysml-inspect" + suffix: b"compatibility executable"}
        manifest = {"version": manifest_version, "target": target,
                    "binaries": {k: hashlib.sha256(v).hexdigest() for k, v in binaries.items()}}
        files = {base + "/bin/" + k: v for k, v in binaries.items()}
        files[base + "/manifest.json"] = json.dumps(manifest).encode()
        archive = directory / (base + (".zip" if windows else ".tar.gz"))
        if windows:
            with zipfile.ZipFile(archive, "w") as bundle:
                for name, data in files.items():
                    bundle.writestr(name, data)
        else:
            with tarfile.open(archive, "w:gz") as bundle:
                for name, data in files.items():
                    info = tarfile.TarInfo(name)
                    info.size = len(data)
                    bundle.addfile(info, io.BytesIO(data))
        archive.with_name(archive.name + ".sha256").write_text(
            hashlib.sha256(archive.read_bytes()).hexdigest() + "  " + archive.name + "\n")


class ReleaseAssetTests(unittest.TestCase):
    def test_complete_release_and_missing_platform(self):
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            assets(directory)
            verify(directory, "0.0.1")
            next(directory.glob("*.zip")).unlink()
            with self.assertRaisesRegex(ValueError, "Incomplete release"):
                verify(directory, "0.0.1")

    def test_corrupt_archive_is_rejected_before_publication(self):
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            assets(directory)
            next(directory.glob("*.tar.gz")).write_bytes(b"corrupt")
            with self.assertRaisesRegex(ValueError, "Checksum mismatch"):
                verify(directory, "0.0.1")

    def test_wrong_version_is_rejected_even_with_valid_archive_checksums(self):
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            assets(directory, "0.4.0")
            with self.assertRaisesRegex(ValueError, "Manifest version/target mismatch"):
                verify(directory, "0.0.1")


if __name__ == "__main__":
    unittest.main()
