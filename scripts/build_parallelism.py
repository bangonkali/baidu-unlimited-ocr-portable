from __future__ import annotations

import os


def resolve_build_jobs(env: dict[str, str] | None = None) -> int:
    """Return cmake/cargo-friendly parallel job count (at least 1)."""
    source = os.environ if env is None else env
    for name in ("BUILD_PARALLEL", "CMAKE_BUILD_PARALLEL_LEVEL"):
        raw = source.get(name, "").strip()
        if not raw:
            continue
        try:
            value = int(raw)
        except ValueError:
            continue
        if value > 0:
            return value
    return max(1, os.cpu_count() or 1)


def cmake_build_parallel_args(env: dict[str, str] | None = None) -> list[str]:
    return ["--parallel", str(resolve_build_jobs(env))]
