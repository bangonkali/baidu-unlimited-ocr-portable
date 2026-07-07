#!/usr/bin/env python3
from __future__ import annotations

import argparse
import importlib
import importlib.util
import json
import os
import sys
from pathlib import Path
from typing import Any


def configure_cache(home: Path) -> None:
    os.environ.setdefault("TRAPO_PPOCRV6_HOME", str(home))
    os.environ.setdefault("HF_HOME", str(home / "cache" / "huggingface"))
    os.environ.setdefault("PADDLE_HOME", str(home / "cache" / "paddle"))
    os.environ.setdefault("PADDLEX_HOME", str(home / ".paddlex"))


def local_model_dirs(home: Path) -> dict[str, str]:
    models = home / "models"
    candidates = {
        "doc_orientation_classify_model_dir": models / "doc_orientation",
        "doc_unwarping_model_dir": models / "doc_unwarping",
        "text_detection_model_dir": models / "text_detection",
        "textline_orientation_model_dir": models / "textline_orientation",
        "text_recognition_model_dir": models / "text_recognition",
    }
    return {key: str(path) for key, path in candidates.items() if path.is_dir()}


def patch_paddlex_frozen_extras() -> None:
    if not getattr(sys, "frozen", False) and os.environ.get("TRAPO_PPOCRV6_PATCH_EXTRAS") != "1":
        return
    try:
        from paddlex.utils import deps as paddlex_deps
    except Exception:
        return
    original = paddlex_deps.is_extra_available
    original_dep = paddlex_deps.is_dep_available

    def is_extra_available(extra: str) -> bool:
        if extra in {"ocr", "ocr-core"}:
            return True
        return original(extra)

    def is_dep_available(dep: str, /, check_version: bool = False) -> bool:
        import_names = {
            "opencv-contrib-python": "cv2",
            "pillow": "PIL",
            "pyclipper": "pyclipper",
            "pypdfium2": "pypdfium2",
            "python-bidi": "bidi",
            "shapely": "shapely",
        }
        if dep in import_names:
            try:
                importlib.import_module(import_names[dep])
            except Exception:
                return False
            return True
        return original_dep(dep, check_version=check_version)

    paddlex_deps.is_extra_available = is_extra_available
    paddlex_deps.is_dep_available = is_dep_available
    try:
        cv2 = importlib.import_module("cv2")
    except Exception:
        return
    for module_name, module in list(sys.modules.items()):
        if module_name.startswith("paddlex.") and module is not None and not hasattr(module, "cv2"):
            module.cv2 = cv2


def create_ocr() -> Any:
    patch_paddlex_frozen_extras()
    from paddleocr import PaddleOCR

    home = Path(os.environ["TRAPO_PPOCRV6_HOME"])
    return PaddleOCR(
        **local_model_dirs(home),
        use_doc_orientation_classify=False,
        use_doc_unwarping=False,
        use_textline_orientation=False,
        engine=os.environ.get("TRAPO_PPOCRV6_ENGINE", "onnxruntime"),
    )


def collect_text(value: Any, out: list[str]) -> None:
    if value is None:
        return
    if isinstance(value, str):
        text = value.strip()
        if text:
            out.append(text)
        return
    if isinstance(value, dict):
        for key in ("rec_texts", "texts", "text", "transcription", "label"):
            if key in value:
                collect_text(value[key], out)
        for nested in value.values():
            if isinstance(nested, dict | list | tuple):
                collect_text(nested, out)
        return
    if isinstance(value, list | tuple):
        for item in value:
            collect_text(item, out)
        return
    for attr in ("json", "res", "data"):
        if hasattr(value, attr):
            collect_text(getattr(value, attr), out)
            return
    if hasattr(value, "to_json"):
        collect_text(value.to_json(), out)


def normalize_result(result: Any) -> str:
    texts: list[str] = []
    collect_text(result, texts)
    deduped = list(dict.fromkeys(texts))
    return "\n".join(deduped).strip()


def run_ocr(image: Path) -> str:
    ocr = create_ocr()
    result = ocr.predict(str(image))
    text = normalize_result(result)
    if text:
        return text
    return json.dumps(result, ensure_ascii=False, default=str)


def main() -> int:
    parser = argparse.ArgumentParser(description="Trapo PP-OCRv6 ONNXRuntime adapter")
    parser.add_argument("--image", type=Path)
    parser.add_argument("--self-check", action="store_true")
    args = parser.parse_args()

    home = Path(os.environ.get("TRAPO_PPOCRV6_HOME") or Path(__file__).resolve().parent)
    configure_cache(home)
    if args.self_check:
        create_ocr()
        print("PP-OCRv6 engine self-check passed")
        return 0
    if args.image is None:
        parser.error("--image is required unless --self-check is used")
    if not args.image.is_file():
        raise SystemExit(f"image does not exist: {args.image}")
    sys.stdout.write(run_ocr(args.image))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
