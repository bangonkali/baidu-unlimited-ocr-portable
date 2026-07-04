from __future__ import annotations

import argparse
import json
import os
import sys
import time
from collections.abc import Callable
from pathlib import Path

from quality_core import (
    Annotation,
    CommandSpec,
    GateResult,
    run_commands,
    run_step,
    synthetic_step,
    tool,
)
from quality_scc import gate_scc

REPO_ROOT = Path(__file__).resolve().parents[1]
CLIENT_ROOT = REPO_ROOT / "src" / "trapo-client"
DEFAULT_GATES = ("frontend", "rust", "python", "scc", "skylos")
PY_COMPILE_FILES = (
    "scripts/package_runtime.py",
    "scripts/install_runtime.py",
    "scripts/test_ctypes_runtime.py",
    "scripts/package_trapo_workbench.py",
    "scripts/package_runtime_macos.py",
    "scripts/skylos_triage.py",
    "scripts/quality_core.py",
    "scripts/quality_scc.py",
    "scripts/quality_gates.py",
    "scripts/quality.py",
)


def gate_frontend(report_dir: Path, _: argparse.Namespace) -> GateResult:
    start = time.monotonic()
    specs = [
        CommandSpec(
            "Install dependencies",
            ["bun", "install", "--frozen-lockfile", "--ignore-scripts"],
            CLIENT_ROOT,
        ),
        CommandSpec("Format check", ["bun", "run", "format:check"], CLIENT_ROOT),
        CommandSpec("Lint", ["bun", "run", "lint"], CLIENT_ROOT),
        CommandSpec("Tests", ["bun", "run", "test"], CLIENT_ROOT),
        CommandSpec("Typecheck", ["bun", "run", "typecheck"], CLIENT_ROOT),
        CommandSpec("Build", ["bun", "run", "build"], CLIENT_ROOT),
        CommandSpec("Build Storybook", ["bun", "run", "build-storybook"], CLIENT_ROOT),
    ]
    return GateResult(
        "frontend",
        tuple(run_commands("frontend", specs, report_dir)),
        time.monotonic() - start,
    )


def gate_python(report_dir: Path, _: argparse.Namespace) -> GateResult:
    start = time.monotonic()
    specs = [
        CommandSpec(
            "Ruff format check",
            [*tool("ruff"), "format", "--check", "scripts", "tests"],
            REPO_ROOT,
        ),
        CommandSpec("Ruff lint", [*tool("ruff"), "check", "scripts", "tests"], REPO_ROOT),
        CommandSpec(
            "py_compile",
            [sys.executable, "-m", "py_compile", *PY_COMPILE_FILES],
            REPO_ROOT,
        ),
        CommandSpec(
            "unittest",
            [sys.executable, "-m", "unittest", "discover", "-s", "tests", "-p", "test_*.py"],
            REPO_ROOT,
        ),
    ]
    return GateResult(
        "python",
        tuple(run_commands("python", specs, report_dir)),
        time.monotonic() - start,
    )


def gate_rust(report_dir: Path, _: argparse.Namespace) -> GateResult:
    start = time.monotonic()
    include_linux_only = os.environ.get("TRAPO_RUST_PLATFORM", "") in {"", "linux-x64"}
    specs = [
        CommandSpec("Cargo fmt", ["cargo", "fmt", "--all", "--", "--check"], REPO_ROOT),
        CommandSpec(
            "Clippy",
            [
                "cargo",
                "clippy",
                "-p",
                "trapo-server",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
            REPO_ROOT,
        ),
        CommandSpec("Tests", ["cargo", "test", "-p", "trapo-server"], REPO_ROOT),
    ]
    steps = run_commands("rust", specs if include_linux_only else specs[1:], report_dir)
    if steps and steps[-1].passed and include_linux_only:
        output = report_dir / "rust" / "trapo.openapi.json"
        output.parent.mkdir(parents=True, exist_ok=True)
        steps.append(
            run_step(
                "rust",
                CommandSpec(
                    "Export OpenAPI",
                    [
                        "cargo",
                        "run",
                        "-p",
                        "trapo-server",
                        "--bin",
                        "export-openapi",
                        "--",
                        str(output),
                    ],
                    REPO_ROOT,
                ),
                report_dir,
            )
        )
        if steps[-1].passed:
            expected = REPO_ROOT / "src" / "trapo-server" / "openapi" / "trapo.openapi.json"
            passed = output.read_bytes() == expected.read_bytes()
            message = (
                "OpenAPI export matches tracked schema."
                if passed
                else "OpenAPI export differs from tracked schema."
            )
            annotations = (
                [Annotation(message, "OpenAPI drift", str(expected))] if not passed else ()
            )
            steps.append(
                synthetic_step(
                    "rust",
                    "Compare OpenAPI",
                    report_dir,
                    passed,
                    message,
                    annotations,
                )
            )
    return GateResult("rust", tuple(steps), time.monotonic() - start)


def gate_skylos(report_dir: Path, args: argparse.Namespace) -> GateResult:
    start = time.monotonic()
    target = Path("docs/skylos/issues") if args.update_skylos_state else report_dir / "skylos"
    raw, filtered, summary = (
        report_dir / "skylos" / "skylos.raw.json",
        target / "current.json",
        target / "current.md",
    )
    raw.parent.mkdir(parents=True, exist_ok=True)
    specs = [
        CommandSpec(
            "Skylos raw scan",
            [
                *tool("skylos"),
                ".",
                "-a",
                "--format",
                "json",
                "--no-upload",
                "--output",
                str(raw),
            ],
            REPO_ROOT,
        ),
        CommandSpec(
            "Skylos triage",
            [
                sys.executable,
                "scripts/skylos_triage.py",
                "--raw",
                str(raw),
                "--filtered",
                str(filtered),
                "--summary",
                str(summary),
                "--fail-on-open",
            ],
            REPO_ROOT,
        ),
    ]
    steps = run_commands("skylos", specs, report_dir)
    if steps and not steps[-1].passed and filtered.exists():
        data = json.loads(filtered.read_text(encoding="utf-8"))
        anns = [
            Annotation(
                str(f.get("message") or f.get("name") or "Open Skylos finding"),
                "Skylos",
                f.get("_path") or f.get("file") or "",
                f.get("line"),
            )
            for findings in data.get("open", {}).values()
            for f in findings
        ]
        steps.append(
            synthetic_step(
                "skylos",
                "Skylos open findings",
                report_dir,
                False,
                f"{len(anns)} open first-party findings.",
                anns,
            )
        )
    return GateResult("skylos", tuple(steps), time.monotonic() - start)


GATES: dict[str, Callable[[Path, argparse.Namespace], GateResult]] = {
    "frontend": gate_frontend,
    "rust": gate_rust,
    "python": gate_python,
    "scc": gate_scc,
    "skylos": gate_skylos,
}
