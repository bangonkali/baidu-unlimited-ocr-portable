from __future__ import annotations

import json
import os
import re
import shutil
import time
from pathlib import Path

from quality_core import Annotation, CommandSpec, GateResult, run_step, synthetic_step

REPO_ROOT = Path(__file__).resolve().parents[1]
SCC_PACKAGE = "github.com/boyter/scc/v3@v3.7.0"
SCC_IGNORE_RE = re.compile(
    r"(^|/)(\.deps|\.logs|\.skylos|build|cache|data|models|target|thirdparty)(/|$)|"
    r"node_modules|dist|generated|\.json$|\.md$|\.csv$|(^|/)LICENSE$|trapo/migrations|tests/|"
    r"scripts/(install_runtime\.py|package_runtime\.py|package_trapo_workbench\.py|"
    r"test_ctypes_runtime\.py|windows/setup-build\.ps1|mac/setup-build\.sh|linux/setup-build\.sh)"
)
SCC_LOW_COMPLEXITY_LIST_RE = re.compile(
    r"src/trapo-server/src/catalog/models\.rs|"
    r"src/trapo-server/src/storage/records\.rs|"
    r"src/trapo-server/src/storage/migration_sql\.rs|"
    r"src/trapo-server/src/storage/migrations\.rs|"
    r"src/trapo-server/src/routes\.rs|"
    r"src/trapo-server/src/workbench_types\.rs|"
    r"src/trapo-client/src/api/types\.ts|"
    r"\.github/workflows/build-runtime\.yml"
)


def scc_files(payload: object) -> list[dict[str, object]]:
    if isinstance(payload, dict):
        return list(payload.get("Files", []))
    if isinstance(payload, list):
        return [
            item for group in payload if isinstance(group, dict) for item in group.get("Files", [])
        ]
    return []


def scc_offenders(payload: object) -> list[dict[str, object]]:
    offenders = [item for item in scc_files(payload) if is_scc_offender(item)]
    return sorted(offenders, key=lambda item: int(item.get("Lines", 0)), reverse=True)


def is_scc_offender(item: dict[str, object]) -> bool:
    if int(item.get("Lines", 0)) <= 300:
        return False
    location = str(item.get("Location", "")).replace("\\", "/")
    if SCC_IGNORE_RE.search(location):
        return False
    complexity = int(item.get("Complexity", 0))
    return not (complexity <= 2 and SCC_LOW_COMPLEXITY_LIST_RE.search(location))


def ensure_scc(report_dir: Path) -> list:
    if shutil.which("scc"):
        return []
    return [
        run_step(
            "scc",
            CommandSpec("Install SCC", ["go", "install", SCC_PACKAGE], REPO_ROOT),
            report_dir,
        )
    ]


def gate_scc(report_dir: Path, _: object) -> GateResult:
    start = time.monotonic()
    steps = ensure_scc(report_dir)
    if steps and not steps[-1].passed:
        return GateResult("scc", tuple(steps), time.monotonic() - start)
    scc = shutil.which("scc") or str(
        Path.home() / "go" / "bin" / ("scc.exe" if os.name == "nt" else "scc")
    )
    steps.append(
        run_step(
            "scc",
            CommandSpec("SCC scan", [scc, "--by-file", "--format", "json"], REPO_ROOT),
            report_dir,
        )
    )
    if steps[-1].passed:
        raw = (
            steps[-1]
            .log_path.read_text(encoding="utf-8")
            .split("## stdout", 1)[1]
            .split("## stderr", 1)[0]
            .strip()
        )
        (report_dir / "scc" / "scc.json").write_text(raw + "\n", encoding="utf-8")
        offenders = scc_offenders(json.loads(raw))
        lines = [
            (
                f"{o.get('Location')} lines={o.get('Lines')} "
                f"code={o.get('Code')} complexity={o.get('Complexity')}"
            )
            for o in offenders
        ]
        annotations = [
            Annotation(line, "SCC complexity", str(o.get("Location", "")))
            for line, o in zip(lines, offenders, strict=False)
        ]
        steps.append(
            synthetic_step(
                "scc",
                "SCC threshold",
                report_dir,
                not offenders,
                "\n".join(lines) or "No gated SCC offenders.",
                annotations,
            )
        )
    return GateResult("scc", tuple(steps), time.monotonic() - start)
