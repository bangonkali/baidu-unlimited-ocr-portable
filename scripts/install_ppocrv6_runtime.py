#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import urllib.request
from pathlib import Path
from urllib.parse import urlparse

REPO_ROOT = Path(__file__).resolve().parents[1]
ENGINE_SCRIPT = REPO_ROOT / "scripts" / "ppocrv6_engine.py"
EMBEDDED_OCR_MODEL_ROOT = (
    REPO_ROOT.parent.parent / "embedded-ocr" / "assets" / "models" / "ppocrv6_medium_full"
)
DEFAULT_PACKAGES = (
    "paddleocr>=3.7,<4",
    "onnxruntime>=1.23,<2",
)
PYINSTALLER_PACKAGE = "pyinstaller>=6,<7"
HUGGINGFACE_HOSTS = {"huggingface.co"}
USER_AGENT = "trapo-ppocrv6-runtime-installer"

PPOCRV6_BUNDLE = {
    "name": "ppocrv6_medium_full",
    "description": "Full PaddleOCR General OCR pipeline using PP-OCRv6 medium ONNX models.",
    "version": "2026-06-18",
    "layoutVersion": 1,
    "source": "embedded-ocr/assets/models/ppocrv6_medium_full",
    "modules": [
        {
            "id": "doc_orientation",
            "modelName": "PP-LCNet_x1_0_doc_ori",
            "repo": "PaddlePaddle/PP-LCNet_x1_0_doc_ori_onnx",
            "revision": "7330ab7039123e46af2dc03154b9969aa412c61d",
            "files": [
                {"name": "inference.onnx", "sizeBytes": 6_788_069},
                {"name": "inference.yml", "sizeBytes": 766},
            ],
        },
        {
            "id": "doc_unwarping",
            "modelName": "UVDoc",
            "repo": "PaddlePaddle/UVDoc_onnx",
            "revision": "3bcf535371727d11e783101f79a504c68848aae3",
            "files": [
                {"name": "inference.onnx", "sizeBytes": 31_684_150},
                {"name": "inference.yml", "sizeBytes": 330},
            ],
        },
        {
            "id": "textline_orientation",
            "modelName": "PP-LCNet_x1_0_textline_ori",
            "repo": "PaddlePaddle/PP-LCNet_x1_0_textline_ori_onnx",
            "revision": "7fdcf3cf7061163eda7183b224aa334bd33068f7",
            "files": [
                {"name": "inference.onnx", "sizeBytes": 6_777_816},
                {"name": "inference.yml", "sizeBytes": 735},
            ],
        },
        {
            "id": "text_detection",
            "modelName": "PP-OCRv6_medium_det",
            "repo": "PaddlePaddle/PP-OCRv6_medium_det_onnx",
            "revision": "61323801669c338b7891481ec7bac61ce31b576a",
            "files": [
                {"name": "inference.onnx", "sizeBytes": 62_032_837},
                {"name": "inference.yml", "sizeBytes": 886},
                {"name": "inference.json", "sizeBytes": 312_150},
            ],
        },
        {
            "id": "text_recognition",
            "modelName": "PP-OCRv6_medium_rec",
            "repo": "PaddlePaddle/PP-OCRv6_medium_rec_onnx",
            "revision": "50c7eacafc52fa7bcf4194e8cd08e46f8558504b",
            "files": [
                {"name": "inference.onnx", "sizeBytes": 76_554_979},
                {"name": "inference.yml", "sizeBytes": 150_580},
                {"name": "inference.json", "sizeBytes": 221_814},
            ],
        },
    ],
}


def run(command: list[str], *, cwd: Path = REPO_ROOT, env: dict[str, str] | None = None) -> None:
    print("+ " + " ".join(command), flush=True)
    subprocess.run(command, cwd=cwd, env=env, check=True)


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


def python_exe(venv: Path) -> Path:
    if sys.platform == "win32":
        return venv / "Scripts" / "python.exe"
    return venv / "bin" / "python"


def ensure_venv(output_dir: Path, packages: tuple[str, ...]) -> Path:
    venv = output_dir / ".venv"
    py = python_exe(venv)
    if not py.is_file():
        run([sys.executable, "-m", "venv", str(venv)])
    run([str(py), "-m", "pip", "install", "--upgrade", "pip"])
    run([str(py), "-m", "pip", "install", *packages])
    return py


def freeze_engine(output_dir: Path, py: Path) -> Path:
    run([str(py), "-m", "pip", "install", "--only-binary=:all:", PYINSTALLER_PACKAGE])
    dist_dir = output_dir / "bin"
    build_dir = output_dir / "build"
    run(
        [
            str(py),
            "-m",
            "PyInstaller",
            "--clean",
            "--noconfirm",
            "--onefile",
            "--name",
            "trapo_ppocrv6_engine",
            "--distpath",
            str(dist_dir),
            "--workpath",
            str(build_dir),
            "--specpath",
            str(build_dir),
            "--collect-all",
            "paddleocr",
            "--collect-all",
            "paddlex",
            "--copy-metadata",
            "paddleocr",
            "--copy-metadata",
            "paddlex",
            str(output_dir / "trapo_ppocrv6_engine.py"),
        ]
    )
    name = "trapo_ppocrv6_engine.exe" if sys.platform == "win32" else "trapo_ppocrv6_engine"
    binary = dist_dir / name
    if not binary.is_file():
        raise SystemExit(f"PyInstaller did not produce {binary}")
    return binary


def model_sources(args: argparse.Namespace) -> list[Path]:
    sources = [Path(item).resolve() for item in args.models_source]
    if EMBEDDED_OCR_MODEL_ROOT.is_dir():
        sources.append(EMBEDDED_OCR_MODEL_ROOT)
    env_source = os.environ.get("TRAPO_PPOCRV6_MODEL_SOURCE")
    if env_source:
        sources.append(Path(env_source).resolve())
    return list(dict.fromkeys(sources))


def install_models(output_dir: Path, args: argparse.Namespace) -> None:
    models_dir = output_dir / "models"
    models_dir.mkdir(parents=True, exist_ok=True)
    sources = model_sources(args)
    for module in PPOCRV6_BUNDLE["modules"]:
        module_id = module["id"]
        module_dir = models_dir / module_id
        module_dir.mkdir(parents=True, exist_ok=True)
        for file_info in module["files"]:
            name = file_info["name"]
            destination = module_dir / name
            if not valid_sized_file(destination, int(file_info["sizeBytes"])):
                copy_or_download_model_file(destination, sources, module, file_info)
            verify_model_file(destination, file_info)
    manifest = dict(PPOCRV6_BUNDLE)
    manifest["installed_from"] = [str(source) for source in sources if source.is_dir()]
    manifest["sha256"] = bundle_sha256(models_dir)
    (models_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


def valid_sized_file(path: Path, expected_size: int) -> bool:
    return path.is_file() and path.stat().st_size == expected_size


def copy_or_download_model_file(
    destination: Path,
    sources: list[Path],
    module: dict[str, object],
    file_info: dict[str, object],
) -> None:
    module_id = str(module["id"])
    name = str(file_info["name"])
    for source_root in sources:
        source = source_root / module_id / name
        if source.is_file():
            shutil.copy2(source, destination)
            return
    download_model_file(destination, module, name)


def download_model_file(destination: Path, module: dict[str, object], name: str) -> None:
    url = f"https://huggingface.co/{module['repo']}/resolve/{module['revision']}/{name}"
    parsed = urlparse(url)
    if parsed.scheme != "https" or (parsed.hostname or "").lower() not in HUGGINGFACE_HOSTS:
        die(f"refusing non-Hugging Face model URL: {url}")
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with (
        urllib.request.urlopen(
            request, timeout=300
        ) as response,  # skylos: ignore[SKY-D216] fixed Hugging Face host allowlist.
        destination.open(
            "wb"
        ) as handle,  # skylos: ignore[SKY-D324] model path is installer output.
    ):
        shutil.copyfileobj(response, handle)


def verify_model_file(path: Path, file_info: dict[str, object]) -> None:
    expected_size = int(file_info["sizeBytes"])
    actual_size = path.stat().st_size if path.is_file() else -1
    if actual_size != expected_size:
        die(f"PP-OCRv6 model file has unexpected size: {path} {actual_size} != {expected_size}")


def bundle_sha256(models_dir: Path) -> dict[str, str]:
    hashes: dict[str, str] = {}
    for path in sorted(models_dir.rglob("*")):
        if not path.is_file() or path.name == "manifest.json":
            continue
        digest = hashlib.sha256()
        with path.open("rb") as handle:
            for chunk in iter(lambda: handle.read(1024 * 1024), b""):
                digest.update(chunk)
        hashes[str(path.relative_to(models_dir)).replace("\\", "/")] = digest.hexdigest()
    return hashes


def self_check_env(output_dir: Path) -> dict[str, str]:
    env = os.environ.copy()
    env["TRAPO_PPOCRV6_HOME"] = str(output_dir)
    env["HF_HOME"] = str(output_dir / "cache" / "huggingface")
    env["PADDLE_HOME"] = str(output_dir / "cache" / "paddle")
    env["PADDLEX_HOME"] = str(output_dir / ".paddlex")
    return env


def install(args: argparse.Namespace) -> None:
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy2(ENGINE_SCRIPT, output_dir / "trapo_ppocrv6_engine.py")
    if not args.no_models:
        install_models(output_dir, args)
    packages = tuple(args.package or DEFAULT_PACKAGES)
    py = ensure_venv(output_dir, packages)
    frozen_binary = freeze_engine(output_dir, py) if args.frozen else None
    manifest = {
        "schema_version": 1,
        "engine": "pp-ocrv6",
        "backend": "onnxruntime",
        "python": str(py.relative_to(output_dir)),
        "binary": str(frozen_binary.relative_to(output_dir)) if frozen_binary else "",
        "script": "trapo_ppocrv6_engine.py",
        "packages": list(packages),
        "prefetched": bool(args.prefetch),
        "frozen": bool(frozen_binary),
    }
    if args.prefetch:
        command = (
            [str(frozen_binary)]
            if frozen_binary
            else [str(py), str(output_dir / "trapo_ppocrv6_engine.py")]
        )
        run([*command, "--self-check"], env=self_check_env(output_dir))
    (output_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(output_dir)


def main() -> None:
    parser = argparse.ArgumentParser(description="Install the packaged PP-OCRv6 engine.")
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--models-source", type=Path, action="append", default=[])
    parser.add_argument("--package", action="append")
    parser.add_argument("--prefetch", action="store_true")
    parser.add_argument("--frozen", action="store_true")
    parser.add_argument("--no-models", action="store_true")
    install(parser.parse_args())


if __name__ == "__main__":
    main()
