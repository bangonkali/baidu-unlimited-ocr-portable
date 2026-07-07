#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import stat
import subprocess
import sys
import urllib.request
from pathlib import Path
from urllib.parse import urlparse

REPO_ROOT = Path(__file__).resolve().parents[1]
TESSERACT_SOURCE = REPO_ROOT / "thirdparty" / "tesseract"
TESSDATA_URL = "https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/main/eng.traineddata"
DOWNLOAD_HOSTS = {"raw.githubusercontent.com", "github.com"}
USER_AGENT = "trapo-tesseract-runtime-installer"


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


def run(command: list[str], *, cwd: Path = REPO_ROOT) -> None:
    print("+ " + " ".join(command), flush=True)
    subprocess.run(command, cwd=cwd, check=True)


def tesseract_name() -> str:
    return "tesseract.exe" if sys.platform == "win32" else "tesseract"


def candidate_commands(args: argparse.Namespace, output_dir: Path) -> list[Path]:
    candidates: list[Path] = []
    for raw in [args.command, os.environ.get("TRAPO_TESSERACT_COMMAND")]:
        if raw:
            candidates.append(Path(raw))
    path_match = shutil.which(tesseract_name()) or shutil.which("tesseract")
    if path_match:
        candidates.append(Path(path_match))
    if sys.platform == "win32":
        for root in (os.environ.get("PROGRAMFILES"), os.environ.get("PROGRAMFILES(X86)")):
            if root:
                candidates.append(Path(root) / "Tesseract-OCR" / "tesseract.exe")
    install_dir = output_dir / "_build" / "install"
    candidates.append(install_dir / "bin" / tesseract_name())
    return [path.resolve() for path in candidates if path and path.is_file()]


def build_from_source(output_dir: Path) -> Path:
    if not (TESSERACT_SOURCE / "CMakeLists.txt").is_file():
        die(f"Tesseract source submodule is missing: {TESSERACT_SOURCE}")
    build_dir = output_dir / "_build" / "tesseract"
    install_dir = output_dir / "_build" / "install"
    generator = ["-G", "Ninja"] if shutil.which("ninja") else []
    run(
        [
            "cmake",
            "-S",
            str(TESSERACT_SOURCE),
            "-B",
            str(build_dir),
            *generator,
            "-DCMAKE_BUILD_TYPE=Release",
            f"-DCMAKE_INSTALL_PREFIX={install_dir}",
            "-DBUILD_TRAINING_TOOLS=OFF",
            "-DBUILD_TESTS=OFF",
        ]
    )
    run(["cmake", "--build", str(build_dir), "--config", "Release", "--target", "install"])
    built = install_dir / "bin" / tesseract_name()
    if not built.is_file():
        die(f"Tesseract source build did not produce {built}")
    return built


def stage_binary(source: Path, output_dir: Path) -> Path:
    bin_dir = output_dir / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    destination = bin_dir / tesseract_name()
    shutil.copy2(source, destination)
    if sys.platform != "win32":
        destination.chmod(destination.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    copy_adjacent_libraries(source.parent, bin_dir)
    return destination


def copy_adjacent_libraries(source_dir: Path, bin_dir: Path) -> None:
    patterns = ("*.dll",) if sys.platform == "win32" else ("*.dylib", "*.so", "*.so.*")
    for pattern in patterns:
        for source in source_dir.glob(pattern):
            if source.is_file():
                shutil.copy2(source, bin_dir / source.name)


def tessdata_sources(args: argparse.Namespace) -> list[Path]:
    sources = [Path(item).resolve() for item in args.tessdata_source]
    env_source = os.environ.get("TESSDATA_PREFIX")
    if env_source:
        sources.append(Path(env_source).resolve())
    sources.append(TESSERACT_SOURCE / "tessdata")
    if sys.platform == "win32":
        for root in (os.environ.get("PROGRAMFILES"), os.environ.get("PROGRAMFILES(X86)")):
            if root:
                sources.append(Path(root) / "Tesseract-OCR" / "tessdata")
    sources.extend(
        [
            Path("/usr/share/tesseract-ocr/5/tessdata"),
            Path("/usr/share/tesseract-ocr/4.00/tessdata"),
            Path("/opt/homebrew/share/tessdata"),
            Path("/usr/local/share/tessdata"),
        ]
    )
    return list(dict.fromkeys(sources))


def stage_tessdata(output_dir: Path, args: argparse.Namespace) -> Path:
    tessdata_dir = output_dir / "tessdata"
    tessdata_dir.mkdir(parents=True, exist_ok=True)
    destination = tessdata_dir / "eng.traineddata"
    for source_dir in tessdata_sources(args):
        source = source_dir / "eng.traineddata"
        if source.is_file():
            shutil.copy2(source, destination)
            return destination
    download_tessdata(destination)
    return destination


def download_tessdata(destination: Path) -> None:
    parsed = urlparse(TESSDATA_URL)
    if parsed.scheme != "https" or (parsed.hostname or "").lower() not in DOWNLOAD_HOSTS:
        die(f"refusing non-GitHub tessdata URL: {TESSDATA_URL}")
    request = urllib.request.Request(TESSDATA_URL, headers={"User-Agent": USER_AGENT})
    with (
        urllib.request.urlopen(
            request, timeout=300
        ) as response,  # skylos: ignore[SKY-D216] fixed GitHub raw host allowlist.
        destination.open("wb") as handle,  # skylos: ignore[SKY-D324] fixed tessdata output path.
    ):
        shutil.copyfileobj(response, handle)


def self_check(binary: Path, output_dir: Path) -> None:
    env = os.environ.copy()
    env["TESSDATA_PREFIX"] = str(output_dir / "tessdata")
    result = subprocess.run(
        [str(binary), "--list-langs"],
        text=True,
        capture_output=True,
        env=env,
        timeout=60,
        check=False,
    )
    if result.returncode != 0 or "eng" not in f"{result.stdout}\n{result.stderr}":
        detail = (result.stderr or result.stdout).strip()
        die(f"Tesseract self-check failed: {detail}")


def install(args: argparse.Namespace) -> None:
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    candidates = candidate_commands(args, output_dir)
    source = (
        candidates[0]
        if candidates and not args.build_from_source
        else build_from_source(output_dir)
    )
    binary = stage_binary(source, output_dir)
    tessdata = stage_tessdata(output_dir, args)
    if args.prefetch:
        self_check(binary, output_dir)
    manifest = {
        "schema_version": 1,
        "engine": "tesseract-rs",
        "binary": str(binary.relative_to(output_dir)),
        "tessdata": str(tessdata.relative_to(output_dir)),
        "source_binary": str(source),
        "prefetched": bool(args.prefetch),
    }
    (output_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(output_dir)


def main() -> None:
    parser = argparse.ArgumentParser(description="Install the packaged Tesseract engine.")
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--command")
    parser.add_argument("--tessdata-source", type=Path, action="append", default=[])
    parser.add_argument("--build-from-source", action="store_true")
    parser.add_argument("--prefetch", action="store_true")
    install(parser.parse_args())


if __name__ == "__main__":
    main()
