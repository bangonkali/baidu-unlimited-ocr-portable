from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
QUALITY_PATH = REPO_ROOT / "scripts" / "quality.py"
QUALITY_CORE_PATH = REPO_ROOT / "scripts" / "quality_core.py"
QUALITY_SCC_PATH = REPO_ROOT / "scripts" / "quality_scc.py"
sys.path.insert(0, str(SCRIPTS_DIR))

core_spec = importlib.util.spec_from_file_location("quality_core", QUALITY_CORE_PATH)
assert core_spec is not None
quality_core = importlib.util.module_from_spec(core_spec)
assert core_spec.loader is not None
sys.modules["quality_core"] = quality_core
core_spec.loader.exec_module(quality_core)

scc_spec = importlib.util.spec_from_file_location("quality_scc", QUALITY_SCC_PATH)
assert scc_spec is not None
quality_scc = importlib.util.module_from_spec(scc_spec)
assert scc_spec.loader is not None
sys.modules["quality_scc"] = quality_scc
scc_spec.loader.exec_module(quality_scc)

spec = importlib.util.spec_from_file_location("quality", QUALITY_PATH)
assert spec is not None
quality = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules["quality"] = quality
spec.loader.exec_module(quality)


class QualityRunnerTests(unittest.TestCase):
    def test_selected_gates_deduplicates_without_reordering(self) -> None:
        self.assertEqual(
            quality_core.selected_gates(("frontend",), ["scc", "python", "scc"]),
            ["scc", "python"],
        )

    def test_scc_offenders_apply_ci_exclusions(self) -> None:
        payload = {
            "Files": [
                {"Location": ".logs/quality/skylos/skylos-raw-scan.log", "Lines": 600},
                {"Location": "scripts/install_runtime.py", "Lines": 900},
                {"Location": "src/trapo-server/src/main.rs", "Lines": 301, "Code": 250},
                {"Location": "tests/test_large.py", "Lines": 500},
            ]
        }

        offenders = quality_scc.scc_offenders(payload)

        self.assertEqual([item["Location"] for item in offenders], ["src/trapo-server/src/main.rs"])

    def test_markdown_report_lists_failed_step(self) -> None:
        step = quality_core.StepResult(
            gate="python",
            name="Ruff lint",
            command="ruff check",
            passed=False,
            exit_code=1,
            duration=0.1,
            log_path=Path(".logs/quality/python/ruff-lint.log"),
            message="lint failed",
        )
        report = quality_core.markdown_report([quality_core.GateResult("python", (step,), 0.1)])

        self.assertIn("| python | FAIL |", report)
        self.assertIn("Ruff lint", report)

    def test_github_escape_handles_command_delimiters(self) -> None:
        self.assertEqual(quality_core.gha_escape("a%b\r\nc"), "a%25b%0D%0Ac")

    def test_run_step_writes_log_and_exit_code(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            result = quality_core.run_step(
                "sample",
                quality_core.CommandSpec(
                    "fail",
                    [sys.executable, "-c", "import sys; print('no'); sys.exit(3)"],
                    REPO_ROOT,
                ),
                Path(tmp),
            )

            self.assertFalse(result.passed)
            self.assertEqual(result.exit_code, 3)
            self.assertIn("no", result.log_path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
