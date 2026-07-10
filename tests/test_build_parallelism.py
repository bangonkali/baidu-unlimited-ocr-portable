from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS_DIR))

spec = importlib.util.spec_from_file_location(
    "build_parallelism", SCRIPTS_DIR / "build_parallelism.py"
)
assert spec is not None
build_parallelism = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = build_parallelism
spec.loader.exec_module(build_parallelism)


class BuildParallelismTests(unittest.TestCase):
    def test_prefers_build_parallel_env(self) -> None:
        self.assertEqual(
            build_parallelism.resolve_build_jobs(
                {"BUILD_PARALLEL": "12", "CMAKE_BUILD_PARALLEL_LEVEL": "3"}
            ),
            12,
        )

    def test_falls_back_to_cmake_build_parallel_level(self) -> None:
        self.assertEqual(
            build_parallelism.resolve_build_jobs({"CMAKE_BUILD_PARALLEL_LEVEL": "6"}),
            6,
        )

    def test_defaults_to_cpu_count(self) -> None:
        with mock.patch.object(build_parallelism.os, "cpu_count", return_value=16):
            self.assertEqual(build_parallelism.resolve_build_jobs({}), 16)

    def test_rejects_non_positive_and_invalid_values(self) -> None:
        with mock.patch.object(build_parallelism.os, "cpu_count", return_value=8):
            self.assertEqual(
                build_parallelism.resolve_build_jobs(
                    {"BUILD_PARALLEL": "0", "CMAKE_BUILD_PARALLEL_LEVEL": "nope"}
                ),
                8,
            )

    def test_cmake_args(self) -> None:
        self.assertEqual(
            build_parallelism.cmake_build_parallel_args({"BUILD_PARALLEL": "4"}),
            ["--parallel", "4"],
        )


if __name__ == "__main__":
    unittest.main()
