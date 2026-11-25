import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("release_version", ROOT / "tools/release_version.py")
versioning = importlib.util.module_from_spec(spec)
spec.loader.exec_module(versioning)


class ReleaseVersionTests(unittest.TestCase):
    def test_development_releases_are_always_prereleases(self):
        for version in ["0.0.1", "0.0.250"]:
            for tag in [None, "v" + version]:
                self.assertTrue(versioning.release_policy(version, tag, True)["prerelease"])

    def test_official_publication_requires_a_matching_explicit_tag(self):
        with self.assertRaises(ValueError):
            versioning.release_policy("0.1.0", publish=True)
        with self.assertRaises(ValueError):
            versioning.release_policy("0.0.1", "v1.0.0", True)
        self.assertFalse(versioning.release_policy("0.1.0", "v0.1.0", True)["prerelease"])

    def test_invalid_versions_are_rejected(self):
        for version in ["0.04.0", "v0.0.1", "0.0.1\n", "1.0", "0.0.1-beta"]:
            with self.assertRaises(ValueError):
                versioning.release_policy(version)

    def test_version_update_synchronizes_manifests_and_preserves_dependencies(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            paths = set(k.split(":")[0] for k in versioning.versions())
            for path in paths:
                (root / path).parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(ROOT / path, root / path)
            before = json.loads((root / "package-lock.json").read_text())["packages"]["node_modules/typescript"]
            versioning.set_version("0.0.99", root)
            self.assertEqual(versioning.check(root), "0.0.99")
            self.assertEqual(before, json.loads((root / "package-lock.json").read_text())["packages"]["node_modules/typescript"])
            data = json.loads((root / "src-tauri/tauri.conf.json").read_text())
            data["version"] = "0.1.0"
            (root / "src-tauri/tauri.conf.json").write_text(json.dumps(data))
            with self.assertRaises(ValueError):
                versioning.check(root)


if __name__ == "__main__":
    unittest.main()
