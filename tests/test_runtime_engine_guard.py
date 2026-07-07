from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))

spec = importlib.util.spec_from_file_location(
    "runtime_engine_guard", SCRIPTS_DIR / "runtime_engine_guard.py"
)
assert spec is not None
runtime_engine_guard = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = runtime_engine_guard
spec.loader.exec_module(runtime_engine_guard)


class RuntimeEngineGuardTests(unittest.TestCase):
    def test_manifest_guard_accepts_current_runtime_matrix(self) -> None:
        self.assertEqual(runtime_engine_guard.manifest_errors(REPO_ROOT), [])

    def test_engine_executable_names_follow_platform_suffixes(self) -> None:
        self.assertEqual(
            runtime_engine_guard.executable_name("trapo-tesseract-rs-runner", "windows-x86_64-cpu"),
            "trapo-tesseract-rs-runner.exe",
        )
        self.assertEqual(
            runtime_engine_guard.executable_name("trapo-tesseract-rs-runner", "linux-x86_64-cpu"),
            "trapo-tesseract-rs-runner",
        )

    def test_build_runtime_matrix_matches_supported_targets(self) -> None:
        platforms = runtime_engine_guard.load_platforms(REPO_ROOT)
        entries = runtime_engine_guard.workflow_matrix_entries(
            REPO_ROOT / ".github" / "workflows" / "build-runtime.yml"
        )

        self.assertEqual(set(entries), runtime_engine_guard.supported_targets(platforms))

    def test_supported_targets_declare_engine_asset_dirs(self) -> None:
        platforms = runtime_engine_guard.load_platforms(REPO_ROOT)
        for platform_id in runtime_engine_guard.supported_targets(platforms):
            with self.subTest(platform_id=platform_id):
                asset_dirs = set(platforms["targets"][platform_id]["engine_asset_dirs"])
                self.assertIn("ppocrv6", asset_dirs)
                self.assertIn("tesseract", asset_dirs)


if __name__ == "__main__":
    unittest.main()
