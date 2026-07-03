from __future__ import annotations

import dataclasses
import os
import re
import shutil
import subprocess
import time
from collections.abc import Sequence
from pathlib import Path


@dataclasses.dataclass(frozen=True)
class Annotation:
    message: str
    title: str
    file: str = ""
    line: int | None = None


@dataclasses.dataclass(frozen=True)
class StepResult:
    gate: str
    name: str
    command: str
    passed: bool
    exit_code: int
    duration: float
    log_path: Path
    message: str = ""
    annotations: tuple[Annotation, ...] = ()


@dataclasses.dataclass(frozen=True)
class CommandSpec:
    name: str
    command: list[str]
    cwd: Path
    env: dict[str, str] | None = None


@dataclasses.dataclass(frozen=True)
class GateResult:
    name: str
    steps: tuple[StepResult, ...]
    duration: float

    @property
    def passed(self) -> bool:
        return all(step.passed for step in self.steps)


def slug(value: str) -> str:
    return re.sub(r"[^A-Za-z0-9_.-]+", "-", value).strip("-").lower() or "step"


def tool(name: str) -> list[str]:
    found = shutil.which(name)
    if found:
        return [found]
    if shutil.which("uv"):
        return ["uv", "run", name]
    return [name]


def command_text(command: Sequence[str]) -> str:
    return " ".join(command)


def _env(extra: dict[str, str] | None = None) -> dict[str, str]:
    env = os.environ.copy()
    env.setdefault("DUCKDB_DOWNLOAD_LIB", "1")
    if extra:
        env.update(extra)
    return env


def _last_line(stdout: str, stderr: str) -> str:
    lines = (stderr or stdout or "").strip().splitlines()
    return lines[-1] if lines else ""


def _write_log(
    path: Path, command: Sequence[str], cwd: Path, result: subprocess.CompletedProcess[str]
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        "\n".join(
            [
                f"$ {command_text(command)}",
                f"cwd: {cwd}",
                f"exit_code: {result.returncode}",
                "",
                "## stdout",
                result.stdout or "",
                "",
                "## stderr",
                result.stderr or "",
            ]
        ),
        encoding="utf-8",
    )


def run_step(gate: str, spec: CommandSpec, report_dir: Path) -> StepResult:
    start = time.monotonic()
    result = subprocess.run(
        spec.command,
        cwd=spec.cwd,
        env=_env(spec.env),
        text=True,
        capture_output=True,
        encoding="utf-8",
        errors="replace",
    )
    log_path = report_dir / gate / f"{slug(spec.name)}.log"
    _write_log(log_path, spec.command, spec.cwd, result)
    return StepResult(
        gate=gate,
        name=spec.name,
        command=command_text(spec.command),
        passed=result.returncode == 0,
        exit_code=result.returncode,
        duration=time.monotonic() - start,
        log_path=log_path,
        message=_last_line(result.stdout, result.stderr),
    )


def run_commands(gate: str, specs: Sequence[CommandSpec], report_dir: Path) -> list[StepResult]:
    results = []
    for spec in specs:
        result = run_step(gate, spec, report_dir)
        results.append(result)
        if not result.passed:
            break
    return results


def synthetic_step(
    gate: str,
    name: str,
    report_dir: Path,
    passed: bool,
    message: str,
    annotations: Sequence[Annotation] = (),
) -> StepResult:
    log_path = report_dir / gate / f"{slug(name)}.log"
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.write_text(message + "\n", encoding="utf-8")
    return StepResult(
        gate,
        name,
        name,
        passed,
        0 if passed else 1,
        0.0,
        log_path,
        message,
        tuple(annotations),
    )


def gate_to_dict(gate: GateResult, root: Path) -> dict[str, object]:
    return {
        "name": gate.name,
        "passed": gate.passed,
        "duration_seconds": round(gate.duration, 3),
        "steps": [
            {
                "name": step.name,
                "passed": step.passed,
                "exit_code": step.exit_code,
                "duration_seconds": round(step.duration, 3),
                "log": step.log_path.relative_to(root).as_posix(),
                "message": step.message,
            }
            for step in gate.steps
        ],
    }


def markdown_report(gates: Sequence[GateResult]) -> str:
    lines = [
        "# Quality Gate Report",
        "",
        "| Gate | Status | Seconds | Failed step |",
        "| --- | --- | ---: | --- |",
    ]
    for gate in gates:
        failed = next((step.name for step in gate.steps if not step.passed), "")
        status = "PASS" if gate.passed else "FAIL"
        lines.append(f"| {gate.name} | {status} | {gate.duration:.1f} | {failed} |")
    failures = [step for gate in gates for step in gate.steps if not step.passed]
    if failures:
        lines += ["", "## Failures", ""]
        lines += [
            f"- `{step.gate}` / `{step.name}`: {step.message or step.log_path}" for step in failures
        ]
    return "\n".join(lines) + "\n"


def gha_escape(value: object) -> str:
    return str(value).replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")


def emit_github(gates: Sequence[GateResult], markdown: str) -> None:
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with Path(summary).open("a", encoding="utf-8") as fh:
            fh.write(markdown + "\n")
    for step in (step for gate in gates for step in gate.steps if not step.passed):
        annotations = step.annotations or (
            Annotation(step.message or str(step.log_path), step.name),
        )
        for ann in annotations[:50]:
            props = [f"title={gha_escape(ann.title)}"]
            if ann.file:
                props.append(f"file={gha_escape(ann.file)}")
            if ann.line:
                props.append(f"line={ann.line}")
            print(f"::error {','.join(props)}::{gha_escape(ann.message)}")


def selected_gates(defaults: Sequence[str], only: Sequence[str]) -> list[str]:
    return list(dict.fromkeys(list(only) or list(defaults)))
