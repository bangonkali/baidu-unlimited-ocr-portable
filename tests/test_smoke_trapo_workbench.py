from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))

spec = importlib.util.spec_from_file_location(
    "smoke_trapo_workbench_engines",
    SCRIPTS_DIR / "smoke_trapo_workbench_engines.py",
)
assert spec is not None
smoke = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = smoke
spec.loader.exec_module(smoke)


def _ppocr(*, available: bool = True, availability: str = "ready") -> dict:
    return {
        "engine_id": "pp-ocrv6",
        "available": available,
        "availability": availability,
        "runner_status": "wired",
        "requires_model": False,
        "download_model_ids": [],
    }


def _paddle(
    *,
    available: bool = False,
    availability: str = "missing_model",
    runner_status: str = "wired",
    requires_model: bool = True,
    download_model_ids: list[str] | None = None,
) -> dict:
    return {
        "engine_id": "paddleocr-vl-1.6-gguf",
        "available": available,
        "availability": availability,
        "runner_status": runner_status,
        "requires_model": requires_model,
        "download_model_ids": download_model_ids
        if download_model_ids is not None
        else ["paddleocr-vl-1-6-gguf"],
    }


class SmokeTrapoWorkbenchTests(unittest.TestCase):
    def test_accepts_missing_gguf_for_paddleocr_vl(self) -> None:
        smoke.validate_packaged_ingest_engines([_ppocr(), _paddle()])

    def test_accepts_ready_paddleocr_vl_when_models_present(self) -> None:
        smoke.validate_packaged_ingest_engines(
            [_ppocr(), _paddle(available=True, availability="ready")]
        )

    def test_rejects_unwired_paddleocr_vl(self) -> None:
        with self.assertRaisesRegex(SystemExit, "runner is not wired"):
            smoke.validate_packaged_ingest_engines([_ppocr(), _paddle(runner_status="planned")])

    def test_rejects_native_runner_missing(self) -> None:
        with self.assertRaisesRegex(SystemExit, "missing native runner"):
            smoke.validate_packaged_ingest_engines(
                [
                    _ppocr(),
                    _paddle(availability="native_runner_missing", runner_status="wired"),
                ]
            )

    def test_rejects_unready_ppocr(self) -> None:
        with self.assertRaisesRegex(SystemExit, "pp-ocrv6"):
            smoke.validate_packaged_ingest_engines(
                [_ppocr(available=False, availability="missing_model"), _paddle()]
            )


if __name__ == "__main__":
    unittest.main()
