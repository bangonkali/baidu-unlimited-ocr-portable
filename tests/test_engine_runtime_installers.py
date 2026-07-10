from __future__ import annotations

import hashlib
import importlib.util
import shutil
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))


def load_script(name: str):
    spec = importlib.util.spec_from_file_location(name, SCRIPTS_DIR / f"{name}.py")
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


install_ppocrv6_runtime = load_script("install_ppocrv6_runtime")
install_paddleocr_vl_runtime = load_script("install_paddleocr_vl_runtime")
install_tesseract_runtime = load_script("install_tesseract_runtime")
trapo_ocr_ffi_build_env = load_script("trapo_ocr_ffi_build_env")
trapo_ocr_ffi_deps = load_script("trapo_ocr_ffi_deps")
build_trapo_ocr_ffi = load_script("build_trapo_ocr_ffi")
test_ctypes_runtime = load_script("test_ctypes_runtime")


class EngineRuntimeInstallerTests(unittest.TestCase):
    def test_trapo_ocr_ffi_cuda_platform_enables_llama_cuda_with_portable_arches(
        self,
    ) -> None:
        with mock.patch.dict(
            trapo_ocr_ffi_build_env.os.environ,
            {"CUDA_ARCHITECTURES": "120a-real"},
            clear=True,
        ):
            env = trapo_ocr_ffi_build_env.portable_build_env("windows-x86_64-cuda13")

        self.assertEqual(env["TRAPO_LLAMA_ENABLE_CUDA"], "1")
        self.assertEqual(env["TRAPO_LLAMA_ENABLE_VULKAN"], "0")
        self.assertEqual(env["TRAPO_LLAMA_ENABLE_OPENCL"], "0")
        self.assertEqual(env["TRAPO_CUDA_ARCHITECTURES"], "120a-real")

    def test_trapo_ocr_ffi_cuda_platform_defaults_portable_cuda_architectures(
        self,
    ) -> None:
        with mock.patch.dict(trapo_ocr_ffi_build_env.os.environ, {}, clear=True):
            env = trapo_ocr_ffi_build_env.portable_build_env("linux-x86_64-cuda13")

        self.assertEqual(env["TRAPO_LLAMA_ENABLE_CUDA"], "1")
        self.assertEqual(
            env["TRAPO_CUDA_ARCHITECTURES"],
            trapo_ocr_ffi_build_env.PORTABLE_CUDA_ARCHITECTURES,
        )

    def test_trapo_ocr_ffi_cpu_platform_keeps_portable_llama_defaults(self) -> None:
        with mock.patch.dict(trapo_ocr_ffi_build_env.os.environ, {}, clear=True):
            env = trapo_ocr_ffi_build_env.portable_build_env("windows-x86_64-cpu")

        self.assertEqual(env["TRAPO_LLAMA_ENABLE_CUDA"], "0")
        self.assertEqual(env["TRAPO_LLAMA_ENABLE_VULKAN"], "0")
        self.assertEqual(env["TRAPO_LLAMA_ENABLE_OPENCL"], "0")
        self.assertNotIn("TRAPO_CUDA_ARCHITECTURES", env)

    def test_trapo_ocr_ffi_cuda_platform_preserves_explicit_backend_override(self) -> None:
        with mock.patch.dict(
            trapo_ocr_ffi_build_env.os.environ,
            {"TRAPO_LLAMA_ENABLE_CUDA": "0"},
            clear=True,
        ):
            env = trapo_ocr_ffi_build_env.portable_build_env("windows-x86_64-cuda13")

        self.assertEqual(env["TRAPO_LLAMA_ENABLE_CUDA"], "0")

    def test_linux_ocr_ffi_configure_enables_native_pipeline(self) -> None:
        args = types.SimpleNamespace(platform="linux-x86_64-cuda13")
        deps = {
            "ort_include": Path("/deps/onnxruntime/include"),
            "ort_lib": Path("/deps/onnxruntime/lib/libonnxruntime.so"),
            "opencv": Path("/deps/opencv/x64"),
        }
        with (
            mock.patch.object(
                build_trapo_ocr_ffi,
                "prepare_onnxruntime_deps",
                return_value={"receipt": True},
            ),
            mock.patch.object(
                build_trapo_ocr_ffi,
                "prepare_linux_deps",
                return_value=deps,
            ),
        ):
            command = build_trapo_ocr_ffi.configure_args(args, Path("/build"))

        self.assertIn("-DTRAPO_ENABLE_DESKTOP_NATIVE_PIPELINE=ON", command)
        self.assertIn(f"-DTRAPO_ORT_INCLUDE_DIR={deps['ort_include']}", command)
        self.assertIn(f"-DTRAPO_ORT_LIB={deps['ort_lib']}", command)
        self.assertIn(f"-DOpenCV_DIR={deps['opencv']}", command)

    def test_linux_opencv_config_uses_archive_cmake_directory(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            package_root = Path(tmp) / "opencv-mobile-4.13.0-ubuntu-2404"
            config_dir = package_root / "lib" / "cmake" / "opencv4"
            config_dir.mkdir(parents=True)
            (config_dir / "OpenCVConfig.cmake").write_text(
                "# synthetic package config\n",
                encoding="utf-8",
            )

            resolved = trapo_ocr_ffi_deps.opencv_config_dir(package_root)

        self.assertEqual(resolved, config_dir)

    def test_trapo_ocr_cuda_capability_guard_allows_compiled_backend_without_device(self) -> None:
        capabilities = {
            "generativeAccelerators": [
                {
                    "backend": 3,
                    "supported": False,
                    "unavailableReason": (
                        "llama.cpp cuda backend is compiled but no compatible device was reported"
                    ),
                }
            ]
        }

        match = test_ctypes_runtime.assert_generative_backend_compiled(capabilities, "cuda")

        self.assertEqual(match["backend"], 3)

    def test_trapo_ocr_cuda_capability_guard_rejects_uncompiled_backend(self) -> None:
        capabilities = {
            "generativeAccelerators": [
                {
                    "backend": 3,
                    "supported": False,
                    "unavailableReason": "llama.cpp cuda backend was not compiled into this build",
                }
            ]
        }

        with self.assertRaisesRegex(SystemExit, "not compiled"):
            test_ctypes_runtime.assert_generative_backend_compiled(capabilities, "cuda")

    def test_trapo_ocr_ffi_resets_cache_generated_from_other_source(self) -> None:
        target_root = build_trapo_ocr_ffi.REPO_ROOT / "target" / "trapo-ocr-ffi"
        target_root.mkdir(parents=True, exist_ok=True)
        build_dir = Path(tempfile.mkdtemp(dir=target_root))
        try:
            stale_source = target_root / "embedded-ocr" / "agus_ocr_core"
            (build_dir / "CMakeFiles").mkdir()
            (build_dir / "CMakeCache.txt").write_text(
                f"CMAKE_HOME_DIRECTORY:INTERNAL={stale_source}\n",
                encoding="utf-8",
            )

            build_trapo_ocr_ffi.reset_stale_cmake_cache(build_dir, {})

            self.assertFalse(build_dir.exists())
        finally:
            if build_dir.exists():
                shutil.rmtree(build_dir)

    def test_trapo_ocr_ffi_resets_cache_when_cuda_backend_mismatches(self) -> None:
        target_root = build_trapo_ocr_ffi.REPO_ROOT / "target" / "trapo-ocr-ffi"
        target_root.mkdir(parents=True, exist_ok=True)
        build_dir = Path(tempfile.mkdtemp(dir=target_root))
        try:
            (build_dir / "CMakeFiles").mkdir()
            (build_dir / "CMakeCache.txt").write_text(
                "\n".join(
                    [
                        f"CMAKE_HOME_DIRECTORY:INTERNAL={build_trapo_ocr_ffi.NATIVE_SOURCE}",
                        "TRAPO_LLAMA_ENABLE_CUDA:BOOL=OFF",
                        "GGML_CUDA:BOOL=OFF",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )

            build_trapo_ocr_ffi.reset_stale_cmake_cache(build_dir, {"TRAPO_LLAMA_ENABLE_CUDA": "1"})

            self.assertFalse(build_dir.exists())
        finally:
            if build_dir.exists():
                shutil.rmtree(build_dir)

    def test_ppocrv6_model_installer_copies_declared_bundle_files(self) -> None:
        original_bundle = install_ppocrv6_runtime.PPOCRV6_BUNDLE
        try:
            install_ppocrv6_runtime.PPOCRV6_BUNDLE = {
                "name": "test-ppocrv6",
                "modules": [
                    {
                        "id": "text_detection",
                        "repo": "PaddlePaddle/test",
                        "revision": "test",
                        "files": [{"name": "inference.onnx", "sizeBytes": 5}],
                    }
                ],
            }
            with tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                source = root / "source"
                output = root / "runtime"
                (source / "text_detection").mkdir(parents=True)
                (source / "text_detection" / "inference.onnx").write_bytes(b"model")

                args = types.SimpleNamespace(models_source=[source])
                install_ppocrv6_runtime.install_models(output, args)

                installed = output / "models" / "text_detection" / "inference.onnx"
                self.assertEqual(installed.read_bytes(), b"model")
                self.assertTrue((output / "models" / "manifest.json").is_file())
        finally:
            install_ppocrv6_runtime.PPOCRV6_BUNDLE = original_bundle

    def test_ppocrv6_installer_removes_stale_python_runtime_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "ppocrv6"
            stale = [
                output / "trapo_ppocrv6_engine.py",
                output / ".venv" / "pyvenv.cfg",
                output / ".paddlex" / "temp" / "cache.json",
                output / "build" / "trapo_ppocrv6_engine" / "localpycs" / "struct.pyc",
                output / "ppocrv6" / "build" / "trapo_ppocrv6_engine" / "app.spec",
                output / "ppocrv6" / "__pycache__" / "old.pyc",
            ]
            for path in stale:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"stale")

            install_ppocrv6_runtime.remove_stale_python_assets(output)

            self.assertFalse(any(path.exists() for path in stale))
            self.assertFalse((output / ".venv").exists())
            self.assertFalse((output / ".paddlex").exists())
            self.assertFalse((output / "build").exists())
            self.assertFalse((output / "ppocrv6").exists())

    def test_paddleocr_vl_installer_copies_layout_bundle_files(self) -> None:
        original_bundle = install_paddleocr_vl_runtime.PADDLEOCR_VL_BUNDLE
        onnx = b"layout-model"
        yml = b"layout-yml"
        try:
            install_paddleocr_vl_runtime.PADDLEOCR_VL_BUNDLE = {
                "name": "paddleocr_vl_1_6",
                "modules": [
                    {
                        "id": "layout_detection",
                        "repo": "PaddlePaddle/test-layout",
                        "revision": "test",
                        "files": [
                            {
                                "name": "inference.onnx",
                                "sizeBytes": len(onnx),
                                "sha256": hashlib.sha256(onnx).hexdigest(),
                            },
                            {
                                "name": "inference.yml",
                                "sizeBytes": len(yml),
                                "sha256": hashlib.sha256(yml).hexdigest(),
                            },
                        ],
                    },
                    {"id": "vl", "files": [{"name": "not-staged.gguf"}]},
                ],
            }
            with tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                source = root / "source"
                output = root / "runtime"
                (source / "layout_detection").mkdir(parents=True)
                (source / "layout_detection" / "inference.onnx").write_bytes(onnx)
                (source / "layout_detection" / "inference.yml").write_bytes(yml)

                args = types.SimpleNamespace(models_source=[source])
                install_paddleocr_vl_runtime.install_layout_bundle(output, args)

                self.assertEqual(
                    (output / "layout_detection" / "inference.onnx").read_bytes(), onnx
                )
                self.assertEqual((output / "layout_detection" / "inference.yml").read_bytes(), yml)
                manifest = (output / "manifest.json").read_text(encoding="utf-8")
                self.assertIn("layout_detection", manifest)
                self.assertFalse((output / "vl" / "not-staged.gguf").exists())
        finally:
            install_paddleocr_vl_runtime.PADDLEOCR_VL_BUNDLE = original_bundle

    def test_paddleocr_vl_installer_has_ci_safe_default_manifest(self) -> None:
        original_bundle = install_paddleocr_vl_runtime.PADDLEOCR_VL_BUNDLE
        try:
            install_paddleocr_vl_runtime.PADDLEOCR_VL_BUNDLE = None
            manifest = install_paddleocr_vl_runtime.bundle_manifest()
            layout_modules = [
                module for module in manifest["modules"] if module["id"] == "layout_detection"
            ]

            self.assertEqual(len(layout_modules), 1)
            self.assertEqual(layout_modules[0]["repo"], "PaddlePaddle/PP-DocLayoutV3_onnx")
            self.assertEqual(
                [file_info["name"] for file_info in layout_modules[0]["files"]],
                ["inference.onnx", "inference.yml"],
            )
        finally:
            install_paddleocr_vl_runtime.PADDLEOCR_VL_BUNDLE = original_bundle

    def test_tesseract_installer_stages_eng_tessdata_from_source_dir(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / "tessdata-source"
            output = root / "runtime"
            source.mkdir()
            (source / "eng.traineddata").write_bytes(b"eng")

            args = types.SimpleNamespace(tessdata_source=[source])
            installed = install_tesseract_runtime.stage_tessdata(output, args)

            self.assertEqual(installed, output / "tessdata" / "eng.traineddata")
            self.assertEqual(installed.read_bytes(), b"eng")


if __name__ == "__main__":
    unittest.main()
