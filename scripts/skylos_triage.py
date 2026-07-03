#!/usr/bin/env python3
from __future__ import annotations

import argparse
import fnmatch
import json
from pathlib import Path
from typing import Any

FINDING_KEYS = (
    "unused_functions",
    "unused_imports",
    "unused_variables",
    "unused_classes",
    "unused_parameters",
    "unused_files",
    "danger",
    "secrets",
    "quality",
    "dependency_vulnerabilities",
    "custom_rules",
    "circular_dependencies",
)
PERSISTED_FINDING_FIELDS = (
    "category",
    "col",
    "dead_code_classification",
    "dead_code_reason",
    "kind",
    "line",
    "message",
    "name",
    "rule_id",
    "severity",
    "simple_name",
    "threshold",
    "type",
    "value",
)


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(
        path.read_text(encoding="utf-8")
    )  # skylos: ignore[SKY-D325] operator-selected local Skylos artifact.


def normalized_relative_path(raw_path: str | None, repo_root: Path) -> str:
    if not raw_path:
        return "<unknown>"
    value = raw_path.replace("\\", "/")
    try:
        path = Path(raw_path)
        if path.is_absolute():
            value = path.resolve(strict=False).relative_to(repo_root).as_posix()
    except OSError, ValueError:
        pass
    if value in {".", ""}:
        return "."
    return value.lstrip("./")


def finding_path(finding: dict[str, Any], repo_root: Path) -> str:
    raw = finding.get("file") or finding.get("path") or finding.get("file_path")
    return normalized_relative_path(str(raw) if raw else None, repo_root)


def path_matches(path: str, pattern: str) -> bool:
    normalized = path.replace("\\", "/")
    pattern = pattern.replace("\\", "/")
    return fnmatch.fnmatch(normalized, pattern) or normalized.startswith(pattern.rstrip("/") + "/")


def is_excluded(path: str, exceptions: dict[str, Any]) -> tuple[bool, str]:
    for entry in exceptions.get("excluded_paths", []):
        if path_matches(path, entry["path_glob"]):
            return True, entry["reason"]
    return False, ""


def is_included(path: str, exceptions: dict[str, Any]) -> bool:
    if path == ".":
        return True
    return any(path_matches(path, pattern) for pattern in exceptions["included_paths"])


def accepts_finding(
    finding: dict[str, Any], category: str, path: str, exceptions: dict[str, Any]
) -> tuple[bool, str, str]:
    rule_id = str(finding.get("rule_id") or "")
    name = str(finding.get("name") or "")
    for entry in exceptions.get("accepted_findings", []):
        rule_ok = entry["rule_id"] == "*" or entry["rule_id"] == rule_id
        category_ok = entry.get("category", "*") in {"*", category}
        path_ok = path_matches(path, entry["path_glob"])
        name_glob = entry.get("name_glob")
        name_ok = not name_glob or fnmatch.fnmatch(name, name_glob)
        if rule_ok and category_ok and path_ok and name_ok:
            return True, entry["id"], entry["reason"]
    return False, "", ""


def iter_findings(raw: dict[str, Any], repo_root: Path):
    for category in FINDING_KEYS:
        for finding in raw.get(category, []) or []:
            if isinstance(finding, dict):
                yield category, finding, finding_path(finding, repo_root)


def sanitized_payload(finding: dict[str, Any], path: str) -> dict[str, Any]:
    payload = {key: finding[key] for key in PERSISTED_FINDING_FIELDS if key in finding}
    payload["_path"] = path
    payload["file"] = path
    return payload


def filtered_result(
    raw: dict[str, Any], exceptions: dict[str, Any], repo_root: Path
) -> tuple[dict[str, Any], dict[str, int]]:
    result: dict[str, Any] = {"open": {}, "accepted": {}, "excluded": []}
    counts = {"open": 0, "accepted": 0, "excluded": 0}
    seen: set[tuple[str, str, int, str, str]] = set()
    for category, finding, path in iter_findings(raw, repo_root):
        key = (
            category,
            path,
            int(finding.get("line") or 0),
            str(finding.get("rule_id") or ""),
            str(finding.get("name") or finding.get("message") or ""),
        )
        if key in seen:
            continue
        seen.add(key)

        excluded, exclude_reason = is_excluded(path, exceptions)
        if excluded or not is_included(path, exceptions):
            counts["excluded"] += 1
            result["excluded"].append(
                {"category": category, "path": path, "reason": exclude_reason}
            )
            continue

        accepted, exception_id, reason = accepts_finding(finding, category, path, exceptions)
        payload = sanitized_payload(finding, path)
        if accepted:
            payload["_exception_id"] = exception_id
            payload["_exception_reason"] = reason
            result["accepted"].setdefault(category, []).append(payload)
            counts["accepted"] += 1
        else:
            result["open"].setdefault(category, []).append(payload)
            counts["open"] += 1
    result["counts"] = counts
    return result, counts


def write_summary(path: Path, result: dict[str, Any], raw_path: Path) -> None:
    lines = [
        "# Current Skylos Triage",
        "",
        f"Raw input: `{raw_path.as_posix()}`",
        "",
        "| State | Count |",
        "| --- | ---: |",
        f"| Open | {result['counts']['open']} |",
        f"| Accepted exception | {result['counts']['accepted']} |",
        f"| Excluded by scope | {result['counts']['excluded']} |",
        "",
        "## Open Findings",
        "",
    ]
    if not result["open"]:
        lines.append("No open first-party findings remain.")
    else:
        lines.extend(markdown_findings(result["open"]))
    lines.extend(["", "## Accepted Findings", ""])
    if not result["accepted"]:
        lines.append("No accepted findings.")
    else:
        lines.extend(markdown_findings(result["accepted"]))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        "\n".join(lines) + "\n", encoding="utf-8"
    )  # skylos: ignore[SKY-D324] summary path is a repo-local operator-selected report.


def markdown_findings(grouped: dict[str, list[dict[str, Any]]]) -> list[str]:
    lines = [
        "| Category | Rule | Path | Line | Name | Exception |",
        "| --- | --- | --- | ---: | --- | --- |",
    ]
    for category, findings in sorted(grouped.items()):
        for finding in findings:
            lines.append(
                "| {category} | {rule} | `{path}` | {line} | {name} | {exception} |".format(
                    category=category,
                    rule=finding.get("rule_id") or "",
                    path=finding.get("_path") or "",
                    line=finding.get("line") or "",
                    name=str(finding.get("name") or "")[:80],
                    exception=finding.get("_exception_id") or "",
                )
            )
    return lines


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Filter Skylos output to tracked first-party triage."
    )
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument(
        "--exceptions", type=Path, default=Path("docs/skylos/issues/exceptions.json")
    )
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--filtered", type=Path, default=Path("docs/skylos/issues/current.json"))
    parser.add_argument("--summary", type=Path, default=Path("docs/skylos/issues/current.md"))
    parser.add_argument("--fail-on-open", action="store_true")
    args = parser.parse_args()

    repo_root = args.repo_root.resolve()
    raw = load_json(args.raw)
    exceptions = load_json(args.exceptions)
    result, counts = filtered_result(raw, exceptions, repo_root)
    args.filtered.parent.mkdir(parents=True, exist_ok=True)
    args.filtered.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )  # skylos: ignore[SKY-D324] filtered path is a repo-local operator-selected report.
    write_summary(args.summary, result, args.raw)
    print(json.dumps(counts, sort_keys=True))
    return 1 if args.fail_on_open and counts["open"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
