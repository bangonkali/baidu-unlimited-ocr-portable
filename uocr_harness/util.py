from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


PACKAGE_ROOT = Path(__file__).resolve().parents[1]
PORTABLE_ROOT = PACKAGE_ROOT
REPO_ROOT = PORTABLE_ROOT.parent
DEFAULT_RESULTS_DIR = PORTABLE_ROOT / "results"
DEFAULT_MANIFEST = DEFAULT_RESULTS_DIR / "manifest.jsonl"
DEFAULT_DATASET = REPO_ROOT / "dataset"


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def monotonic_ms() -> int:
    return int(time.monotonic() * 1000)


def ensure_dir(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    return path


def slugify(value: str, *, max_len: int = 64) -> str:
    slug = re.sub(r"[^A-Za-z0-9._-]+", "-", value.strip()).strip("-._")
    slug = re.sub(r"-{2,}", "-", slug)
    if not slug:
        slug = "case"
    return slug[:max_len].strip("-._") or "case"


def stable_hash(value: str, length: int = 8) -> str:
    return hashlib.sha1(value.encode("utf-8")).hexdigest()[:length]


def case_id_for(path: Path, suffix: str | None = None) -> str:
    base = slugify(path.stem)
    if suffix:
        base = f"{base}-{suffix}"
    return f"{base}-{stable_hash(str(path.resolve()) + (suffix or ''))}"


def portable_rel(path: Path) -> str:
    try:
        return path.resolve().relative_to(PORTABLE_ROOT).as_posix()
    except ValueError:
        return path.resolve().as_posix()


def repo_rel(path: Path) -> str:
    try:
        return path.resolve().relative_to(REPO_ROOT).as_posix()
    except ValueError:
        return path.resolve().as_posix()


def resolve_path(value: str | Path) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    portable_candidate = PORTABLE_ROOT / path
    if portable_candidate.exists():
        return portable_candidate
    return REPO_ROOT / path


def write_json(path: Path, data: dict[str, Any]) -> None:
    ensure_dir(path.parent)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    ensure_dir(path.parent)
    with path.open("w", encoding="utf-8") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n")


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    return rows


def command_text(argv: list[str]) -> str:
    return " ".join(sh_quote(part) for part in argv)


def sh_quote(value: str) -> str:
    if re.fullmatch(r"[A-Za-z0-9_./:=,@+-]+", value):
        return value
    return "'" + value.replace("'", "'\"'\"'") + "'"


def gpu_snapshot() -> dict[str, Any]:
    try:
        proc = subprocess.run(
            [
                "nvidia-smi",
                "--query-gpu=name,memory.total,memory.used,memory.free",
                "--format=csv,noheader,nounits",
            ],
            check=True,
            capture_output=True,
            text=True,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError):
        return {"available": False}

    gpus = []
    for line in proc.stdout.splitlines():
        parts = [part.strip() for part in line.split(",")]
        if len(parts) != 4:
            continue
        name, total, used, free = parts
        gpus.append(
            {
                "name": name,
                "memory_total_mb": int(total),
                "memory_used_mb": int(used),
                "memory_free_mb": int(free),
            }
        )
    return {"available": bool(gpus), "gpus": gpus}


def first_gpu_used_mb(snapshot: dict[str, Any]) -> int | None:
    try:
        return int(snapshot["gpus"][0]["memory_used_mb"])
    except (KeyError, IndexError, TypeError, ValueError):
        return None


def env_with_no_proxy() -> dict[str, str]:
    env = os.environ.copy()
    env.setdefault("NO_PROXY", "127.0.0.1,localhost")
    env.setdefault("no_proxy", "127.0.0.1,localhost")
    return env
