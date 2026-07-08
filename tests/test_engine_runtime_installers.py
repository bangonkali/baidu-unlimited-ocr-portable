from __future__ import annotations

import hashlib
import importlib.util
import sys
import tempfile
import types
import unittest
from pathlib import Path

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


class EngineRuntimeInstallerTests(unittest.TestCase):
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
