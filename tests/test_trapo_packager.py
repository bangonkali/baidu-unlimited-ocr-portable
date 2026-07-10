from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))

staging_spec = importlib.util.spec_from_file_location(
    "onnxruntime_staging", SCRIPTS_DIR / "onnxruntime_staging.py"
)
assert staging_spec is not None
onnxruntime_staging = importlib.util.module_from_spec(staging_spec)
assert staging_spec.loader is not None
sys.modules[staging_spec.name] = onnxruntime_staging
staging_spec.loader.exec_module(onnxruntime_staging)

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

    def test_windows_launcher_adds_runtime_bins_to_path(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            stage_root = Path(tmp)

            package_trapo_workbench.make_launcher(stage_root, "windows-x64")

            launcher = (stage_root / "trapo-server.cmd").read_text(encoding="ascii")
            self.assertIn("thirdparty\\uocr-runtime\\*cpu*", launcher)
            self.assertIn("thirdparty\\uocr-runtime\\*", launcher)
            self.assertIn('set "PATH=%%~fD\\bin;%PATH%"', launcher)
            self.assertIn('findstr /I /C:"cuda"', launcher)

    def test_unix_launcher_adds_runtime_bins_to_library_path(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            stage_root = Path(tmp)

            package_trapo_workbench.make_launcher(stage_root, "linux-x64")

            launcher = (stage_root / "trapo-server.sh").read_text(encoding="utf-8")
            self.assertIn("thirdparty/uocr-runtime", launcher)
            self.assertIn("-name bin", launcher)
            self.assertIn("LD_LIBRARY_PATH", launcher)
            self.assertIn("runtime_lib_path", launcher)
            self.assertIn("cuda", launcher)

    def test_native_runner_names_match_platform_suffixes(self) -> None:
        self.assertEqual(
            package_trapo_workbench.native_runner_names("windows-x86_64-cpu"),
            [
                "trapo-tesseract-rs-runner.exe",
            ],
        )
        self.assertEqual(
            package_trapo_workbench.native_runner_names("linux-x86_64-cpu"),
            [
                "trapo-tesseract-rs-runner",
            ],
        )

    def test_cuda_runtime_uses_cpu_safe_ort_core_for_ocr_ffi(self) -> None:
        self.assertEqual(
            onnxruntime_staging.ocr_ffi_ort_platform("windows-x86_64-cuda13"),
            "windows-x86_64-cpu",
        )
        self.assertEqual(
            onnxruntime_staging.provider_ort_platforms("windows-x86_64-cuda13"),
            ["windows-x86_64-cuda13"],
        )
        self.assertTrue(
            onnxruntime_staging.is_provider_runtime_library(Path("onnxruntime_providers_cuda.dll"))
        )
        self.assertFalse(onnxruntime_staging.is_provider_runtime_library(Path("onnxruntime.dll")))

    def test_incomplete_runtime_redistributables_are_repaired(self) -> None:
        with (
            tempfile.TemporaryDirectory() as tmp,
            mock.patch.object(
                package_trapo_workbench,
                "missing_staged_nvidia_redist",
                return_value=["cudnn64_9.dll"],
            ),
            mock.patch.object(
                package_trapo_workbench,
                "missing_staged_windows_runtime",
                return_value=["vcruntime140.dll"],
            ),
            mock.patch.object(package_trapo_workbench, "stage_nvidia_redist") as stage_nvidia,
            mock.patch.object(package_trapo_workbench, "stage_windows_runtime") as stage_windows,
            mock.patch.object(package_trapo_workbench, "validate_staged_nvidia_redist"),
            mock.patch.object(package_trapo_workbench, "validate_staged_windows_runtime"),
        ):
            runtime_dir = Path(tmp)
            package_trapo_workbench.ensure_runtime_redistributables(
                runtime_dir,
                "windows-x86_64-cuda13",
            )

        stage_nvidia.assert_called_once_with(
            runtime_dir / "bin",
            "windows-x86_64-cuda13",
        )
        stage_windows.assert_called_once_with(
            runtime_dir / "bin",
            "windows-x86_64-cuda13",
        )

    def test_complete_runtime_skips_local_redistributable_staging(self) -> None:
        with (
            tempfile.TemporaryDirectory() as tmp,
            mock.patch.object(
                package_trapo_workbench,
                "missing_staged_nvidia_redist",
                return_value=[],
            ),
            mock.patch.object(
                package_trapo_workbench,
                "missing_staged_windows_runtime",
                return_value=[],
            ),
            mock.patch.object(package_trapo_workbench, "stage_nvidia_redist") as stage_nvidia,
            mock.patch.object(package_trapo_workbench, "stage_windows_runtime") as stage_windows,
            mock.patch.object(package_trapo_workbench, "validate_staged_nvidia_redist"),
            mock.patch.object(package_trapo_workbench, "validate_staged_windows_runtime"),
        ):
            package_trapo_workbench.ensure_runtime_redistributables(
                Path(tmp),
                "windows-x86_64-cuda13",
            )

        stage_nvidia.assert_not_called()
        stage_windows.assert_not_called()

    def test_paddleocr_vl_engine_is_accepted_when_layout_bundle_exists(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            runtime_dir = Path(tmp)
            engine_dir = runtime_dir / "paddleocr_vl_1_6"
            (engine_dir / "layout_detection").mkdir(parents=True)
            (engine_dir / "manifest.json").write_text("{}", encoding="utf-8")
            (engine_dir / "layout_detection" / "inference.onnx").write_bytes(b"onnx")
            (engine_dir / "layout_detection" / "inference.yml").write_text(
                "model: layout", encoding="utf-8"
            )

            package_trapo_workbench.ensure_paddleocr_vl_engine(runtime_dir)

    def test_paddleocr_vl_engine_installer_runs_when_layout_bundle_is_missing(self) -> None:
        calls: list[list[str]] = []
        original_run = package_trapo_workbench.run

        def capture_run(command: list[str]) -> None:
            calls.append(command)

        try:
            package_trapo_workbench.run = capture_run
            with tempfile.TemporaryDirectory() as tmp:
                package_trapo_workbench.ensure_paddleocr_vl_engine(Path(tmp))
        finally:
            package_trapo_workbench.run = original_run

        self.assertEqual(len(calls), 1)
        self.assertIn("install_paddleocr_vl_runtime.py", calls[0][1])

    def test_workbench_package_rejects_python_runtime_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            stage_root = Path(tmp)
            forbidden = stage_root / "thirdparty" / "uocr-runtime" / "windows-x86_64-cpu"
            forbidden.mkdir(parents=True)
            (forbidden / "runtime.py").write_text("print('not shipped')", encoding="utf-8")

            with self.assertRaises(SystemExit):
                package_trapo_workbench.ensure_no_python_runtime_files(stage_root)

    def test_workbench_package_allows_native_runtime_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            stage_root = Path(tmp)
            allowed = stage_root / "thirdparty" / "uocr-runtime" / "windows-x86_64-cpu" / "bin"
            allowed.mkdir(parents=True)
            (allowed / "trapo-ocr-ffi.dll").write_bytes(b"native")

            package_trapo_workbench.ensure_no_python_runtime_files(stage_root)


if __name__ == "__main__":
    unittest.main()
