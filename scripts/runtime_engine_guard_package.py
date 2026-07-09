from __future__ import annotations

import json
import tarfile
import zipfile
from pathlib import Path
from typing import Any

from runtime_engine_guard_manifest import load_platforms
from runtime_engine_guard_runtime import (
    archived_forbidden_asset_paths,
    archived_forbidden_runtime_paths,
    forbidden_asset_files,
    required_asset_files,
)


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


def packaged_runtime(args: Any) -> None:
    repo_root = args.repo_root.resolve()
    platforms = load_platforms(repo_root)
    target = platforms["targets"][args.platform]
    archive_name = (
        f"{platforms['asset_prefix']}-{args.platform}-{args.version}.{target['archive_ext']}"
    )
    archive = args.dist_dir.resolve() / archive_name
    sidecar = args.dist_dir.resolve() / f"{archive_name}.runtime.json"
    if not archive.is_file():
        die(f"runtime archive was not produced: {archive}")
    if not sidecar.is_file():
        die(f"runtime sidecar was not produced: {sidecar}")
    manifest = json.loads(sidecar.read_text(encoding="utf-8"))
    validate_packaged_runtime_layout(args.platform, target, manifest, archive_members(archive))
    print(f"{args.platform}: packaged runtime archive guard passed")


def validate_packaged_runtime_layout(
    platform_id: str,
    target: dict[str, Any],
    manifest: dict[str, Any],
    archived: set[str],
) -> None:
    listed = set(manifest.get("layout", {}).get("executables", {}))
    expected = set(target["executables"])
    if listed != expected:
        die(f"{platform_id} sidecar executable mismatch: {sorted(expected - listed)}")
    missing = [
        name
        for name in target["executables"]
        if not any(member.endswith(f"/bin/{name}") for member in archived)
    ]
    if missing:
        die(f"{platform_id} archive is missing executables: {', '.join(missing)}")
    validate_required_libraries(platform_id, target, manifest, archived)
    layout_files = set(manifest.get("layout", {}).get("files", []))
    validate_no_python_runtime_files(platform_id, layout_files, archived)
    validate_notice_files(platform_id, target, layout_files, archived)
    validate_engine_assets(platform_id, target, layout_files, archived)


def validate_required_libraries(
    platform_id: str,
    target: dict[str, Any],
    manifest: dict[str, Any],
    archived: set[str],
) -> None:
    listed = set(manifest.get("layout", {}).get("required_libraries", {}))
    expected = set(target.get("required_libraries", []))
    if listed != expected:
        die(
            f"{platform_id} sidecar required library mismatch: "
            + ", ".join(sorted(expected - listed))
        )
    missing = [
        name
        for name in target.get("required_libraries", [])
        if not any(member.endswith(f"/bin/{name}") for member in archived)
    ]
    if missing:
        die(f"{platform_id} archive is missing required libraries: {', '.join(missing)}")


def validate_no_python_runtime_files(
    platform_id: str,
    layout_files: set[str],
    archived: set[str],
) -> None:
    forbidden_layout = archived_forbidden_runtime_paths(layout_files)
    if forbidden_layout:
        die(
            f"{platform_id} sidecar contains forbidden Python runtime files: "
            + ", ".join(forbidden_layout[:10])
        )
    forbidden_archive = archived_forbidden_runtime_paths(archived)
    if forbidden_archive:
        die(
            f"{platform_id} archive contains forbidden Python runtime files: "
            + ", ".join(forbidden_archive[:10])
        )


def validate_notice_files(
    platform_id: str,
    target: dict[str, Any],
    layout_files: set[str],
    archived: set[str],
) -> None:
    for notice in target.get("bundled_notice_files", []):
        packaged_notice = f"bin/{notice}"
        if packaged_notice not in layout_files:
            die(f"{platform_id} sidecar is missing bundled notice {packaged_notice}")
        if not any(member.endswith(f"/{packaged_notice}") for member in archived):
            die(f"{platform_id} archive is missing bundled notice {packaged_notice}")


def validate_engine_assets(
    platform_id: str,
    target: dict[str, Any],
    layout_files: set[str],
    archived: set[str],
) -> None:
    for asset_dir in target.get("engine_asset_dirs", []):
        for required_file in required_asset_files(platform_id, asset_dir):
            if required_file not in layout_files:
                die(f"{platform_id} sidecar is missing engine asset {required_file}")
            if not any(member.endswith(f"/{required_file}") for member in archived):
                die(f"{platform_id} archive is missing engine asset {required_file}")
        for forbidden_file in forbidden_asset_files(asset_dir):
            if any(member.endswith(f"/{forbidden_file}") for member in archived):
                die(f"{platform_id} archive contains forbidden engine asset {forbidden_file}")
        forbidden_paths = archived_forbidden_asset_paths(archived, asset_dir)
        if forbidden_paths:
            die(
                f"{platform_id} archive contains forbidden engine assets: "
                + ", ".join(forbidden_paths[:10])
            )


def archive_members(archive: Path) -> set[str]:
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as zipf:
            return {f"/{name}" for name in zipf.namelist()}
    with tarfile.open(archive, "r:gz") as tar:
        return {f"/{member.name}" for member in tar.getmembers()}
