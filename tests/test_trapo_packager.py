from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))

spec = importlib.util.spec_from_file_location(
    "package_trapo_workbench", SCRIPTS_DIR / "package_trapo_workbench.py"
)
assert spec is not None
package_trapo_workbench = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = package_trapo_workbench
spec.loader.exec_module(package_trapo_workbench)


class TrapoPackagerTests(unittest.TestCase):
    def test_windows_arm64_platform_uses_arm64_pdfium_asset(self) -> None:
        platform = package_trapo_workbench.PLATFORMS["windows-arm64"]

        self.assertEqual(platform["archive_ext"], "zip")
        self.assertEqual(platform["server"], "trapo-server.exe")
        self.assertEqual(platform["duckdb"], "duckdb.dll")
        self.assertEqual(platform["pdfium_asset"], "pdfium-win-arm64.tgz")
        self.assertEqual(platform["pdfium_lib"], "pdfium.dll")
        self.assertEqual(platform["pdfium_dir"], ("thirdparty", "pdfium", "bin"))

    def test_linux_arm64_platform_remains_packaged(self) -> None:
        platform = package_trapo_workbench.PLATFORMS["linux-arm64"]

        self.assertEqual(platform["archive_ext"], "tar.gz")
        self.assertEqual(platform["server"], "trapo-server")
        self.assertEqual(platform["duckdb"], "libduckdb.so")
        self.assertEqual(platform["pdfium_asset"], "pdfium-linux-arm64.tgz")


if __name__ == "__main__":
    unittest.main()
