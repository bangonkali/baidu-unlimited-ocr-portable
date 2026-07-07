from __future__ import annotations

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
