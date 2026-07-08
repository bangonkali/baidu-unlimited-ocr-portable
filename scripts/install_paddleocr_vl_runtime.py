#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import urllib.request
from pathlib import Path
from urllib.parse import urlparse

REPO_ROOT = Path(__file__).resolve().parents[1]
MODEL_NAME = "paddleocr_vl_1_6"
MANIFEST_PATH = (
    REPO_ROOT / "thirdparty" / "embedded-ocr" / "assets" / "models" / MODEL_NAME / "manifest.json"
)
EMBEDDED_OCR_MODEL_ROOTS = (
    REPO_ROOT / "thirdparty" / "embedded-ocr" / "assets" / "models" / MODEL_NAME,
    REPO_ROOT.parent.parent / "embedded-ocr" / "assets" / "models" / MODEL_NAME,
)
HUGGINGFACE_HOSTS = {"huggingface.co"}
USER_AGENT = "trapo-paddleocr-vl-runtime-installer"


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


def load_bundle_manifest() -> dict[str, object]:
    if not MANIFEST_PATH.is_file():
        die(f"PaddleOCR-VL asset manifest is missing: {MANIFEST_PATH}")
    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


PADDLEOCR_VL_BUNDLE: dict[str, object] | None = None


def bundle_manifest() -> dict[str, object]:
    global PADDLEOCR_VL_BUNDLE
    if PADDLEOCR_VL_BUNDLE is None:
        PADDLEOCR_VL_BUNDLE = load_bundle_manifest()
    return PADDLEOCR_VL_BUNDLE


def layout_module() -> dict[str, object]:
    for module in bundle_manifest()["modules"]:
        if module["id"] == "layout_detection":
            return module
    die("PaddleOCR-VL manifest does not declare layout_detection")


def model_sources(args: argparse.Namespace) -> list[Path]:
    sources = [Path(item).resolve() for item in args.models_source]
    sources.extend(path for path in EMBEDDED_OCR_MODEL_ROOTS if path.is_dir())
    env_source = os.environ.get("TRAPO_PADDLEOCR_VL_MODEL_SOURCE")
    if env_source:
        sources.append(Path(env_source).resolve())
    return list(dict.fromkeys(sources))


def install_layout_bundle(output_dir: Path, args: argparse.Namespace) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    sources = model_sources(args)
    module = layout_module()
    module_dir = output_dir / str(module["id"])
    module_dir.mkdir(parents=True, exist_ok=True)
    for file_info in module["files"]:
        name = str(file_info["name"])
        destination = module_dir / name
        if not valid_model_file(destination, file_info):
            copy_or_download_model_file(destination, sources, module, file_info)
        verify_model_file(destination, file_info)
    write_manifest(output_dir, sources)


def valid_model_file(path: Path, file_info: dict[str, object]) -> bool:
    if not path.is_file():
        return False
    expected_size = int(file_info.get("sizeBytes", -1))
    return expected_size < 0 or path.stat().st_size == expected_size


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
    expected_size = int(file_info.get("sizeBytes", -1))
    actual_size = path.stat().st_size if path.is_file() else -1
    if expected_size >= 0 and actual_size != expected_size:
        die(
            f"PaddleOCR-VL layout file has unexpected size: {path} {actual_size} != {expected_size}"
        )
    expected_sha = str(file_info.get("sha256", ""))
    if expected_sha and sha256_file(path) != expected_sha:
        die(f"PaddleOCR-VL layout file has unexpected SHA-256: {path}")


def write_manifest(output_dir: Path, sources: list[Path]) -> None:
    manifest = dict(PADDLEOCR_VL_BUNDLE)
    manifest["installed_from"] = [str(source) for source in sources if source.is_dir()]
    manifest["staged_modules"] = ["layout_detection"]
    manifest["sha256"] = bundle_sha256(output_dir)
    (output_dir / "manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def bundle_sha256(output_dir: Path) -> dict[str, str]:
    hashes: dict[str, str] = {}
    for path in sorted(output_dir.rglob("*")):
        if not path.is_file() or path.name == "manifest.json":
            continue
        hashes[str(path.relative_to(output_dir)).replace("\\", "/")] = sha256_file(path)
    return hashes


def install(args: argparse.Namespace) -> None:
    output_dir = args.output_dir.resolve()
    install_layout_bundle(output_dir, args)
    print(output_dir)


def main() -> None:
    parser = argparse.ArgumentParser(description="Install the packaged PaddleOCR-VL layout bundle.")
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--models-source", type=Path, action="append", default=[])
    parser.add_argument("--package", action="append")
    parser.add_argument("--prefetch", action="store_true")
    parser.add_argument("--frozen", action="store_true")
    install(parser.parse_args())


if __name__ == "__main__":
    main()
