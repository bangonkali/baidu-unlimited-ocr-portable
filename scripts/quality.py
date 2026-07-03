#!/usr/bin/env python3
from __future__ import annotations

import argparse
import concurrent.futures
import json
from collections.abc import Sequence
from datetime import UTC, datetime
from pathlib import Path

from quality_core import (
    GateResult,
    emit_github,
    gate_to_dict,
    markdown_report,
    selected_gates,
)
from quality_gates import DEFAULT_GATES, GATES


def run_gates(names: Sequence[str], args: argparse.Namespace) -> list[GateResult]:
    report_dir = args.report_dir.resolve()
    report_dir.mkdir(parents=True, exist_ok=True)
    if args.parallel and not args.fail_fast and len(names) > 1:
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs or len(names)) as pool:
            return list(pool.map(lambda name: GATES[name](report_dir, args), names))

    results = []
    for name in names:
        results.append(GATES[name](report_dir, args))
        if args.fail_fast and not results[-1].passed:
            break
    return results


def main() -> int:
    parser = argparse.ArgumentParser(description="Run Trapo quality gates.")
    parser.add_argument("--profile", default="ci", choices=("ci",))
    parser.add_argument("--only", action="append", choices=sorted(GATES))
    parser.add_argument("--parallel", action="store_true")
    parser.add_argument("--jobs", type=int, default=0)
    parser.add_argument("--github", action="store_true")
    parser.add_argument("--report-dir", type=Path, default=Path(".logs/quality"))
    parser.add_argument("--update-skylos-state", action="store_true")
    parser.add_argument("--fail-fast", action="store_true")
    args = parser.parse_args()

    gates = run_gates(selected_gates(DEFAULT_GATES, args.only or ()), args)
    markdown = markdown_report(gates)
    report_dir = args.report_dir.resolve()
    payload = {
        "generated_at": datetime.now(UTC).isoformat(),
        "passed": all(gate.passed for gate in gates),
        "gates": [gate_to_dict(gate, report_dir) for gate in gates],
    }
    report_dir.mkdir(parents=True, exist_ok=True)
    (report_dir / "quality-report.json").write_text(
        json.dumps(payload, indent=2) + "\n", encoding="utf-8"
    )
    (report_dir / "quality-report.md").write_text(markdown, encoding="utf-8")
    print(markdown)
    if args.github:
        emit_github(gates, markdown)
    return 0 if payload["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
