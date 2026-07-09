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

package_spec = importlib.util.spec_from_file_location(
    "runtime_engine_guard_package", SCRIPTS_DIR / "runtime_engine_guard_package.py"
)
assert package_spec is not None
runtime_engine_guard_package = importlib.util.module_from_spec(package_spec)
assert package_spec.loader is not None
sys.modules[package_spec.name] = runtime_engine_guard_package
package_spec.loader.exec_module(runtime_engine_guard_package)

validate_spec = importlib.util.spec_from_file_location(
    "validate_trapo_ocr_ffi", SCRIPTS_DIR / "validate_trapo_ocr_ffi.py"
)
assert validate_spec is not None
validate_trapo_ocr_ffi = importlib.util.module_from_spec(validate_spec)
assert validate_spec.loader is not None
sys.modules[validate_spec.name] = validate_trapo_ocr_ffi
validate_spec.loader.exec_module(validate_trapo_ocr_ffi)


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
                self.assertIn("paddleocr_vl_1_6", asset_dirs)
                self.assertIn("tesseract", asset_dirs)

    def test_native_deps_cover_supported_targets_and_onnx_pin(self) -> None:
        platforms = runtime_engine_guard.load_platforms(REPO_ROOT)
        native_deps = runtime_engine_guard.load_native_deps(REPO_ROOT)
        supported = runtime_engine_guard.supported_targets(platforms)

        self.assertLessEqual(supported, set(native_deps["onnxruntime"]["targets"]))
        self.assertEqual(native_deps["onnx"]["required_tag"], "v1.21.0")
        self.assertEqual(
            native_deps["onnx"]["required_commit"],
            "be2b5fde82d9c8874f3d19328bdfe3b6962dc67b",
        )
        self.assertEqual(
            native_deps["onnxruntime"]["compatible_onnx_tag"],
            native_deps["onnx"]["required_tag"],
        )

    def test_ppocrv6_python_artifacts_are_forbidden_at_any_depth(self) -> None:
        self.assertTrue(
            runtime_engine_guard.is_forbidden_asset_path(
                "ppocrv6",
                "bundle/thirdparty/uocr-runtime/windows-x86_64-cpu/ppocrv6/build/"
                "trapo_ppocrv6_engine/localpycs/struct.pyc",
            )
        )
        self.assertTrue(
            runtime_engine_guard.is_forbidden_asset_path(
                "ppocrv6", "ppocrv6/ppocrv6/.venv/Scripts/python.exe"
            )
        )
        self.assertTrue(
            runtime_engine_guard.is_forbidden_asset_path(
                "ppocrv6", "ppocrv6/.paddlex/temp/cache.json"
            )
        )
        self.assertTrue(
            runtime_engine_guard.is_forbidden_asset_path(
                "ppocrv6", "ppocrv6/bin/trapo_ppocrv6_engine.exe"
            )
        )
        self.assertFalse(
            runtime_engine_guard.is_forbidden_asset_path(
                "ppocrv6", "ppocrv6/models/text_detection/inference.onnx"
            )
        )

    def test_ppocrv6_required_assets_cover_native_model_layout(self) -> None:
        required = runtime_engine_guard.required_asset_files("windows-x86_64-cpu", "ppocrv6")

        self.assertIn("ppocrv6/manifest.json", required)
        self.assertIn("ppocrv6/models/manifest.json", required)
        self.assertIn("ppocrv6/models/doc_orientation/inference.onnx", required)
        self.assertIn("ppocrv6/models/text_detection/inference.json", required)
        self.assertIn("ppocrv6/models/text_recognition/inference.yml", required)

    def test_cuda_ffi_validation_requires_backend_cache_flags(self) -> None:
        valid_cache = "\n".join(
            [
                "TRAPO_LLAMA_ENABLE_CUDA:BOOL=ON",
                "GGML_CUDA:BOOL=ON",
            ]
        )
        invalid_cache = "\n".join(
            [
                "TRAPO_LLAMA_ENABLE_CUDA:BOOL=ON",
                "GGML_CUDA:BOOL=OFF",
            ]
        )

        self.assertTrue(validate_trapo_ocr_ffi.cache_bool(valid_cache, "GGML_CUDA"))
        self.assertFalse(validate_trapo_ocr_ffi.cache_bool(invalid_cache, "GGML_CUDA"))

    def test_packaged_runtime_guard_requires_runtime_libraries(self) -> None:
        target = {
            "executables": ["runner.exe"],
            "required_libraries": ["trapo-ocr-ffi.dll"],
            "engine_asset_dirs": [],
        }
        manifest = {"layout": {"executables": {"runner.exe": "bin/runner.exe"}}}

        with self.assertRaisesRegex(SystemExit, "required library mismatch"):
            runtime_engine_guard_package.validate_packaged_runtime_layout(
                "windows-x86_64-cpu",
                target,
                manifest,
                {"/runtime/bin/runner.exe"},
            )

        manifest["layout"]["required_libraries"] = {"trapo-ocr-ffi.dll": "bin/trapo-ocr-ffi.dll"}
        with self.assertRaisesRegex(SystemExit, "missing required libraries"):
            runtime_engine_guard_package.validate_packaged_runtime_layout(
                "windows-x86_64-cpu",
                target,
                manifest,
                {"/runtime/bin/runner.exe"},
            )

    def test_paddleocr_vl_python_artifacts_are_forbidden_at_any_depth(self) -> None:
        self.assertTrue(
            runtime_engine_guard.is_forbidden_asset_path(
                "paddleocr_vl_1_6", "paddleocr_vl_1_6/.venv/Scripts/python.exe"
            )
        )
        self.assertTrue(
            runtime_engine_guard.is_forbidden_asset_path(
                "paddleocr_vl_1_6", "bundle/paddleocr_vl_1_6/__pycache__/old.pyc"
            )
        )
        self.assertFalse(
            runtime_engine_guard.is_forbidden_asset_path(
                "paddleocr_vl_1_6", "paddleocr_vl_1_6/layout_detection/inference.onnx"
            )
        )

    def test_python_runtime_files_are_forbidden_globally(self) -> None:
        for path in [
            "bin/python.exe",
            "lib/python3.14/os.py",
            "runtime/.venv/pyvenv.cfg",
            "paddleocr_vl_1_6/__pycache__/old.pyc",
            "plugins/native_extension.pyd",
            "ppocrv6/app.spec",
        ]:
            with self.subTest(path=path):
                self.assertTrue(runtime_engine_guard.is_forbidden_runtime_path(path))

        for path in [
            "bin/trapo-ocr-ffi.dll",
            "paddleocr_vl_1_6/layout_detection/inference.onnx",
            "ppocrv6/models/manifest.json",
        ]:
            with self.subTest(path=path):
                self.assertFalse(runtime_engine_guard.is_forbidden_runtime_path(path))


if __name__ == "__main__":
    unittest.main()
