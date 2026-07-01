from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))

spec = importlib.util.spec_from_file_location("install_runtime", SCRIPTS_DIR / "install_runtime.py")
assert spec is not None
install_runtime = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = install_runtime
spec.loader.exec_module(install_runtime)


class RuntimeInstallerTests(unittest.TestCase):
    def test_skip_accelerator_probe_keeps_os_arch_validation(self) -> None:
        original_normalize_os = install_runtime.normalize_os
        original_machine = install_runtime.platform.machine
        try:
            install_runtime.normalize_os = lambda: "linux"
            install_runtime.platform.machine = lambda: "x86_64"
            detected = install_runtime.detect_platform(
                REPO_ROOT,
                "linux-x86_64-cuda13",
                skip_accelerator_probe=True,
            )
            self.assertTrue(detected.supported)
            self.assertEqual(detected.platform_id, "linux-x86_64-cuda13")
            self.assertEqual(detected.accelerator_detail, "accelerator probe skipped for requested platform")

            install_runtime.normalize_os = lambda: "windows"
            mismatch = install_runtime.detect_platform(
                REPO_ROOT,
                "linux-x86_64-cuda13",
                skip_accelerator_probe=True,
            )
            self.assertFalse(mismatch.supported)
            self.assertIn("detected windows/x86_64", mismatch.reason)
        finally:
            install_runtime.normalize_os = original_normalize_os
            install_runtime.platform.machine = original_machine

    def test_release_has_platform_asset_matches_runtime_archives_only(self) -> None:
        app_release = {
            "assets": [
                {"name": "uocr-workbench-linux-x64-v0.0.30.tar.gz"},
                {"name": "uocr-runtime-linux-x86_64-cuda13-v0.0.7.tar.gz.sha256"},
            ]
        }
        runtime_release = {"assets": [{"name": "uocr-runtime-linux-arm64-cpu-v0.0.34.tar.gz"}]}

        self.assertFalse(install_runtime.release_has_platform_asset(REPO_ROOT, app_release, "linux-arm64-cpu"))
        self.assertTrue(install_runtime.release_has_platform_asset(REPO_ROOT, runtime_release, "linux-arm64-cpu"))

    def test_windows_arm64_runtime_can_be_requested(self) -> None:
        original_normalize_os = install_runtime.normalize_os
        original_machine = install_runtime.platform.machine
        try:
            install_runtime.normalize_os = lambda: "windows"
            install_runtime.platform.machine = lambda: "ARM64"
            detected = install_runtime.detect_platform(
                REPO_ROOT,
                "windows-arm64-cpu",
                skip_accelerator_probe=True,
            )

            self.assertTrue(detected.supported)
            self.assertEqual(detected.platform_id, "windows-arm64-cpu")
            self.assertEqual(detected.arch, "arm64")
        finally:
            install_runtime.normalize_os = original_normalize_os
            install_runtime.platform.machine = original_machine


if __name__ == "__main__":
    unittest.main()
