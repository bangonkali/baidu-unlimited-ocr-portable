#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path

from runtime_engine_guard_manifest import (
    REPO_ROOT,
    executable_name,
    load_native_deps,
    load_platforms,
    manifest_errors,
    supported_targets,
    workflow_matrix_entries,
)
from runtime_engine_guard_package import packaged_runtime
from runtime_engine_guard_runtime import (
    is_forbidden_asset_path,
    is_forbidden_runtime_path,
    local_smoke,
    smoke_runners,
)

__all__ = [
    "REPO_ROOT",
    "executable_name",
    "load_platforms",
    "load_native_deps",
    "manifest_errors",
    "is_forbidden_asset_path",
    "is_forbidden_runtime_path",
    "supported_targets",
    "workflow_matrix_entries",
]


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


def validate_manifest(args: argparse.Namespace) -> None:
    errors = manifest_errors(args.repo_root.resolve())
    if errors:
        die("\n".join(errors))
    print("runtime manifest guard passed")


def main() -> None:
    parser = argparse.ArgumentParser(description="Validate Trapo runtime engine coverage.")
    parser.add_argument("--repo-root", type=Path, default=REPO_ROOT)
    subparsers = parser.add_subparsers(dest="command", required=True)

    manifest_parser = subparsers.add_parser("manifest")
    manifest_parser.set_defaults(func=validate_manifest)

    smoke_parser = subparsers.add_parser("smoke-runners")
    smoke_parser.add_argument("--platform", required=True)
    smoke_parser.add_argument("--build-dir", type=Path, required=True)
    smoke_parser.set_defaults(func=smoke_runners)

    local_parser = subparsers.add_parser("local-smoke")
    local_parser.add_argument("--optional", action="store_true")
    local_parser.set_defaults(func=local_smoke)

    package_parser = subparsers.add_parser("packaged-runtime")
    package_parser.add_argument("--platform", required=True)
    package_parser.add_argument("--version", required=True)
    package_parser.add_argument("--dist-dir", type=Path, default=REPO_ROOT / "dist")
    package_parser.set_defaults(func=packaged_runtime)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
