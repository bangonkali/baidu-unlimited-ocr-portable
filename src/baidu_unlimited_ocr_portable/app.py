from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
import time
import uuid
from pathlib import Path
from typing import Any, Iterator

import uvicorn
from fastapi import FastAPI, File, Form, HTTPException, UploadFile
from fastapi.responses import HTMLResponse, StreamingResponse

from .config import (
    CANDIDATE_PROFILES,
    DEFAULT_CANDIDATE_PROFILE,
    DEFAULT_PROMPT_PROFILE,
    PROMPT_PROFILES,
)
from .native_runner import RuntimePaths, clean_generated_text, profile_by_key, stream_ocr
from .parsing import boxes_as_dicts, extract_boxes, preview_data_url
from .pdf import pdf_to_images


IMAGE_SUFFIXES = {".bmp", ".jpeg", ".jpg", ".png", ".tif", ".tiff", ".webp"}
WEB_ROOT = Path(__file__).resolve().parent / "web"
PDF_SESSIONS: dict[str, list[str]] = {}


def _profile_label(profile_key: str) -> str:
    profile = CANDIDATE_PROFILES[profile_key]
    return f"{profile.label} ({profile.key})"


def _prompt_label(prompt_key: str) -> str:
    profile = PROMPT_PROFILES[prompt_key]
    return f"{profile.label} ({profile.key})"


def _default_candidate_key() -> str:
    if DEFAULT_CANDIDATE_PROFILE in CANDIDATE_PROFILES:
        return DEFAULT_CANDIDATE_PROFILE
    return "best-zero-empty-q4"


def _metadata(
    *,
    profile_key: str,
    text: str,
    image_path: str | None,
    native: dict | None = None,
) -> dict[str, Any]:
    boxes = extract_boxes(text)
    payload: dict[str, Any] = {
        "candidate_profile": CANDIDATE_PROFILES[profile_key].engine_name,
        "text_chars": len(text),
        "boxes": len(boxes),
        "box_preview": boxes_as_dicts(boxes[:50]),
        "active_image": image_path,
    }
    if native:
        payload.update(native)
    return payload


def _config_payload() -> dict[str, Any]:
    default_candidate = _default_candidate_key()
    return {
        "prompt_profiles": [
            {
                "key": key,
                "label": _prompt_label(key),
                "name": profile.label,
                "prompt": profile.prompt,
                "description": profile.description,
            }
            for key, profile in PROMPT_PROFILES.items()
        ],
        "candidate_profiles": [
            {
                "key": key,
                "label": _profile_label(key),
                "name": profile.label,
                "engine_name": profile.engine_name,
                "description": profile.description,
                "default_max_tokens": profile.default_max_tokens,
            }
            for key, profile in CANDIDATE_PROFILES.items()
        ],
        "defaults": {
            "prompt_profile": DEFAULT_PROMPT_PROFILE,
            "prompt": PROMPT_PROFILES[DEFAULT_PROMPT_PROFILE].prompt,
            "candidate_profile": default_candidate,
            "max_tokens": CANDIDATE_PROFILES[default_candidate].default_max_tokens,
        },
        "limits": {"min_tokens": 64, "max_tokens": 8192, "token_step": 64},
    }


def _ndjson(payload: dict[str, Any]) -> bytes:
    return (json.dumps(payload, ensure_ascii=False) + "\n").encode("utf-8")


def _payload(
    *,
    text: str,
    done: bool,
    profile_key: str,
    image_path: str,
    native: dict | None = None,
    include_preview: bool = False,
    error: str | None = None,
) -> dict[str, Any]:
    boxes = extract_boxes(text)
    payload: dict[str, Any] = {
        "text": text,
        "done": done,
        "boxes": boxes_as_dicts(boxes),
        "metadata": _metadata(profile_key=profile_key, text=text, image_path=image_path, native=native),
    }
    if include_preview:
        preview = preview_data_url(image_path, text, max_size=900)
        if preview:
            payload["preview"] = preview
    if error:
        payload["error"] = error
    return payload


def _candidate_key(value: str | None) -> str:
    key = value or _default_candidate_key()
    if key not in CANDIDATE_PROFILES:
        raise HTTPException(status_code=400, detail=f"Unknown candidate profile: {key}")
    return key


def _token_limit(value: int | str | None, profile_key: str) -> int:
    profile = CANDIDATE_PROFILES[profile_key]
    if value in (None, ""):
        return profile.default_max_tokens
    try:
        parsed = int(value)
    except (TypeError, ValueError) as exc:
        raise HTTPException(status_code=400, detail=f"Invalid max token value: {value}") from exc
    return max(1, min(32768, parsed))


def _stream_native_ocr(
    *,
    image_path: str,
    prompt: str,
    candidate_profile: str,
    max_tokens: int,
) -> Iterator[bytes]:
    profile_key = _candidate_key(candidate_profile)
    profile = profile_by_key(profile_key)
    token_limit = _token_limit(max_tokens, profile_key)
    paths = RuntimePaths.from_env()
    missing = paths.missing()
    if missing:
        yield _ndjson(
            {
                "text": "",
                "done": True,
                "boxes": [],
                "error": "Native runtime files are missing.",
                "metadata": {"missing": missing, "active_image": image_path},
            }
        )
        return

    accumulated = ""
    last_emit = 0.0
    last_text_len = 0
    last_preview_box_count = -1
    yield _ndjson(
        _payload(
            text="",
            done=False,
            profile_key=profile_key,
            image_path=image_path,
        )
    )

    for event in stream_ocr(
        paths=paths,
        image_path=Path(image_path),
        prompt=prompt,
        profile=profile,
        max_tokens=token_limit,
    ):
        if event.kind == "token":
            accumulated += event.text
            cleaned = clean_generated_text(accumulated)
            boxes = extract_boxes(cleaned)
            now = time.monotonic()
            boxes_changed = bool(boxes) and len(boxes) != last_preview_box_count
            should_emit = now - last_emit >= 0.15 or len(cleaned) - last_text_len >= 120 or boxes_changed
            if should_emit:
                last_emit = now
                last_text_len = len(cleaned)
                if boxes_changed:
                    last_preview_box_count = len(boxes)
                yield _ndjson(
                    _payload(
                        text=cleaned,
                        done=False,
                        profile_key=profile_key,
                        image_path=image_path,
                        include_preview=boxes_changed,
                    )
                )
            continue

        native_meta = event.metadata or {}
        final_text = clean_generated_text(event.text or accumulated)
        if event.kind == "error":
            if not final_text and native_meta.get("stderr_tail"):
                final_text = str(native_meta["stderr_tail"])
            yield _ndjson(
                _payload(
                    text=final_text,
                    done=True,
                    profile_key=profile_key,
                    image_path=image_path,
                    native=native_meta,
                    include_preview=True,
                    error=str(native_meta.get("error") or "Native OCR failed."),
                )
            )
            return

        yield _ndjson(
            _payload(
                text=final_text,
                done=True,
                profile_key=profile_key,
                image_path=image_path,
                native=native_meta,
                include_preview=True,
            )
        )


async def _save_upload(upload: UploadFile, *, allowed_suffixes: set[str], prefix: str) -> Path:
    suffix = Path(upload.filename or "").suffix.lower()
    if suffix not in allowed_suffixes:
        raise HTTPException(status_code=400, detail=f"Unsupported file type: {suffix or 'unknown'}")

    target_dir = Path(tempfile.mkdtemp(prefix=prefix))
    target = target_dir / f"input{suffix}"
    try:
        with target.open("wb") as handle:
            while True:
                chunk = await upload.read(1024 * 1024)
                if not chunk:
                    break
                handle.write(chunk)
    finally:
        await upload.close()
    return target


def create_app() -> FastAPI:
    api = FastAPI(title="Unlimited-OCR Portable Candidate")

    @api.get("/", response_class=HTMLResponse)
    async def homepage() -> HTMLResponse:
        html_path = WEB_ROOT / "index.html"
        return HTMLResponse(html_path.read_text(encoding="utf-8"))

    @api.get("/api/config")
    async def config() -> dict[str, Any]:
        return _config_payload()

    @api.post("/api/explode_pdf")
    async def explode_pdf(pdf_file: UploadFile = File(...)) -> dict[str, Any]:
        pdf_path = await _save_upload(pdf_file, allowed_suffixes={".pdf"}, prefix="uocr_pdf_upload_")
        pages = pdf_to_images(pdf_path, dpi=200)
        session_id = uuid.uuid4().hex
        PDF_SESSIONS[session_id] = [str(page) for page in pages]
        return {
            "session_id": session_id,
            "pages": [
                {
                    "session_id": session_id,
                    "page_index": index,
                    "orig_name": Path(page).name,
                }
                for index, page in enumerate(pages)
            ],
        }

    @api.post("/api/run_ocr")
    async def run_ocr_image(
        image_file: UploadFile = File(...),
        prompt: str = Form(PROMPT_PROFILES[DEFAULT_PROMPT_PROFILE].prompt),
        candidate_profile: str = Form(_default_candidate_key()),
        max_tokens: int = Form(CANDIDATE_PROFILES[_default_candidate_key()].default_max_tokens),
    ) -> StreamingResponse:
        profile_key = _candidate_key(candidate_profile)
        token_limit = _token_limit(max_tokens, profile_key)
        image_path = await _save_upload(image_file, allowed_suffixes=IMAGE_SUFFIXES, prefix="uocr_image_upload_")
        return StreamingResponse(
            _stream_native_ocr(
                image_path=str(image_path),
                prompt=prompt,
                candidate_profile=profile_key,
                max_tokens=token_limit,
            ),
            media_type="application/x-ndjson",
        )

    @api.post("/api/run_ocr_page")
    async def run_ocr_page(
        session_id: str = Form(...),
        page_index: int = Form(...),
        prompt: str = Form(PROMPT_PROFILES[DEFAULT_PROMPT_PROFILE].prompt),
        candidate_profile: str = Form(_default_candidate_key()),
        max_tokens: int = Form(CANDIDATE_PROFILES[_default_candidate_key()].default_max_tokens),
    ) -> StreamingResponse:
        profile_key = _candidate_key(candidate_profile)
        token_limit = _token_limit(max_tokens, profile_key)
        pages = PDF_SESSIONS.get(session_id)
        if not pages:
            raise HTTPException(status_code=404, detail="PDF session not found.")
        if page_index < 0 or page_index >= len(pages):
            raise HTTPException(status_code=400, detail=f"PDF page index out of range: {page_index}")
        return StreamingResponse(
            _stream_native_ocr(
                image_path=pages[page_index],
                prompt=prompt,
                candidate_profile=profile_key,
                max_tokens=token_limit,
            ),
            media_type="application/x-ndjson",
        )

    return api


def run_smoke(args: argparse.Namespace) -> int:
    paths = RuntimePaths.from_env()
    profile = profile_by_key(args.profile)
    image_path = Path(args.image)
    if not image_path.is_absolute():
        image_path = Path.cwd() / image_path
    accumulated = ""
    last_printed = 0
    for event in stream_ocr(
        paths=paths,
        image_path=image_path,
        prompt=args.prompt,
        profile=profile,
        max_tokens=args.max_tokens,
    ):
        if event.kind == "token":
            accumulated += event.text
            cleaned = clean_generated_text(accumulated)
            delta = cleaned[last_printed:]
            if delta:
                sys.stdout.write(delta)
                sys.stdout.flush()
                last_printed = len(cleaned)
        elif event.kind == "done":
            final_text = clean_generated_text(event.text or accumulated)
            if len(final_text) > last_printed:
                sys.stdout.write(final_text[last_printed:])
            sys.stdout.write("\n")
            sys.stderr.write(f"metadata={event.metadata}\n")
            return 0
        else:
            sys.stderr.write(f"error={event.metadata}\n")
            return 1
    return 1


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run the Unlimited-OCR portable candidate demo.")
    parser.add_argument("--host", default=os.environ.get("UOCR_CLIENT_HOST", "127.0.0.1"))
    parser.add_argument("--port", type=int, default=int(os.environ.get("UOCR_CLIENT_PORT", "7861")))
    parser.add_argument("--smoke", action="store_true", help="Run a CLI smoke test instead of launching the web app.")
    parser.add_argument("--image", default="dataset/sc-02.png", help="Image path for --smoke.")
    parser.add_argument("--prompt", default=PROMPT_PROFILES[DEFAULT_PROMPT_PROFILE].prompt)
    parser.add_argument("--profile", default="best-zero-empty-q4", choices=sorted(CANDIDATE_PROFILES))
    parser.add_argument("--max-tokens", type=int, default=64)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.smoke:
        return run_smoke(args)
    uvicorn.run(create_app(), host=args.host, port=args.port)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
