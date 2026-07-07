from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
ENGINE_EXECUTABLE_BASES = (
    "llama-mtmd-cli",
    "trapo-tesseract-rs-runner",
    "trapo-pp-ocrv6-runner",
)
REQUIRED_ENGINE_ASSET_DIRS = ("ppocrv6", "tesseract")


def load_platforms(repo_root: Path) -> dict[str, Any]:
    path = repo_root / "runtime" / "platforms.json"
    return json.loads(path.read_text(encoding="utf-8"))


def executable_name(base: str, platform_id: str) -> str:
    return f"{base}.exe" if platform_id.startswith("windows-") else base


def workflow_matrix_entries(path: Path) -> dict[str, dict[str, str]]:
    entries: dict[str, dict[str, str]] = {}
    current: dict[str, str] | None = None
    platform_pattern = re.compile(r"^\s*-\s+platform:\s*([^\s#]+)\s*$")
    field_pattern = re.compile(r"^\s+([A-Za-z_]+):\s*(.*?)\s*$")
    for line in path.read_text(encoding="utf-8").splitlines():
        platform_match = platform_pattern.match(line)
        if platform_match:
            platform_id = clean_yaml_scalar(platform_match.group(1))
            current = {"platform": platform_id}
            entries[platform_id] = current
            continue
        if current is None:
            continue
        field_match = field_pattern.match(line)
        if field_match:
            current[field_match.group(1)] = clean_yaml_scalar(field_match.group(2))
    return entries


def clean_yaml_scalar(value: str) -> str:
    value = value.strip()
    if " #" in value:
        value = value.split(" #", 1)[0].strip()
    return value.strip("\"'")


def release_runtime_coverage(path: Path) -> set[str]:
    text = path.read_text(encoding="utf-8")
    covered = {
        clean_yaml_scalar(match)
        for match in re.findall(r"^\s+runtime_platform:\s*(.*?)\s*$", text, re.MULTILINE)
    }
    for raw in re.findall(r"^\s+additional_runtime_platforms:\s*(.*?)\s*$", text, re.MULTILINE):
        clean = clean_yaml_scalar(raw)
        covered.update(item for item in re.split(r"[\s,]+", clean) if item)
    return covered


def supported_targets(platforms: dict[str, Any]) -> set[str]:
    return {
        platform_id
        for platform_id, target in platforms["targets"].items()
        if target.get("support_status") == "supported"
    }


def planned_targets(platforms: dict[str, Any]) -> set[str]:
    return {
        platform_id
        for platform_id, target in platforms["targets"].items()
        if target.get("support_status") == "planned"
    }


def manifest_errors(repo_root: Path) -> list[str]:
    platforms = load_platforms(repo_root)
    targets = platforms["targets"]
    supported = supported_targets(platforms)
    planned = planned_targets(platforms)
    build_entries = workflow_matrix_entries(
        repo_root / ".github" / "workflows" / "build-runtime.yml"
    )
    release_coverage = release_runtime_coverage(
        repo_root / ".github" / "workflows" / "release-workbench.yml"
    )
    errors: list[str] = []
    build_platforms = set(build_entries)
    if build_platforms != supported:
        errors.append(
            "build-runtime matrix must equal supported runtime targets; "
            f"missing={sorted(supported - build_platforms)} "
            f"extra={sorted(build_platforms - supported)}"
        )
    if planned & build_platforms:
        errors.append(
            f"planned runtime targets must not be built: {sorted(planned & build_platforms)}"
        )
    if release_coverage != supported:
        errors.append(
            "release workbench runtime coverage must equal supported targets; "
            f"missing={sorted(supported - release_coverage)} "
            f"extra={sorted(release_coverage - supported)}"
        )
    for platform_id in sorted(supported):
        target = targets[platform_id]
        matrix = build_entries.get(platform_id, {})
        if matrix.get("runner") != target.get("runner"):
            errors.append(
                f"{platform_id} runner mismatch: workflow={matrix.get('runner')} "
                f"manifest={target.get('runner')}"
            )
        errors.extend(target_metadata_errors(platform_id, target))
    return errors


def target_metadata_errors(platform_id: str, target: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    executables = set(target.get("executables", []))
    if target.get("primary_binary") not in executables:
        errors.append(f"{platform_id} primary_binary is not listed in executables")
    for base in ENGINE_EXECUTABLE_BASES:
        expected = executable_name(base, platform_id)
        if expected not in executables:
            errors.append(f"{platform_id} is missing engine executable {expected}")
    asset_dirs = set(target.get("engine_asset_dirs", []))
    for asset_dir in REQUIRED_ENGINE_ASSET_DIRS:
        if asset_dir not in asset_dirs:
            errors.append(f"{platform_id} is missing engine asset directory {asset_dir}")
    expected_ext = "zip" if platform_id.startswith("windows-") else "tar.gz"
    if target.get("archive_ext") != expected_ext:
        errors.append(f"{platform_id} archive_ext must be {expected_ext}")
    return errors
