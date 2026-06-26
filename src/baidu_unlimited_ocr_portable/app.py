from __future__ import annotations

import argparse
import os
import sys
import time
from pathlib import Path
from typing import Any, Iterator

import gradio as gr

from .config import (
    CANDIDATE_PROFILES,
    DEFAULT_CANDIDATE_PROFILE,
    DEFAULT_PROMPT_PROFILE,
    PROMPT_PROFILES,
)
from .native_runner import RuntimePaths, clean_generated_text, profile_by_key, stream_ocr
from .parsing import boxes_as_dicts, build_preview_image, extract_boxes, preview_image
from .pdf import pdf_to_images


IMAGE_SUFFIXES = {".bmp", ".jpeg", ".jpg", ".png", ".tif", ".tiff", ".webp"}


def _profile_label(profile_key: str) -> str:
    profile = CANDIDATE_PROFILES[profile_key]
    return f"{profile.label} ({profile.key})"


def _prompt_label(prompt_key: str) -> str:
    profile = PROMPT_PROFILES[prompt_key]
    return f"{profile.label} ({profile.key})"


CANDIDATE_CHOICES = [_profile_label(key) for key in CANDIDATE_PROFILES]
CANDIDATE_LABEL_TO_KEY = {_profile_label(key): key for key in CANDIDATE_PROFILES}
PROMPT_CHOICES = [_prompt_label(key) for key in PROMPT_PROFILES]
PROMPT_LABEL_TO_KEY = {_prompt_label(key): key for key in PROMPT_PROFILES}


def _default_candidate_label() -> str:
    key = DEFAULT_CANDIDATE_PROFILE if DEFAULT_CANDIDATE_PROFILE in CANDIDATE_PROFILES else "best-zero-empty-q4"
    return _profile_label(key)


def _default_prompt_label() -> str:
    return _prompt_label(DEFAULT_PROMPT_PROFILE)


def _coerce_file_path(file_value: Any) -> str | None:
    if file_value is None:
        return None
    if isinstance(file_value, dict):
        value = file_value.get("path")
        return str(value) if value else None
    value = getattr(file_value, "path", None)
    if value:
        return str(value)
    return str(file_value)


def _state_for_image(path: str) -> dict[str, Any]:
    return {"kind": "image", "source_path": path, "pages": [], "active_image": path}


def _state_for_pdf(path: str, pages: list[Path]) -> dict[str, Any]:
    return {
        "kind": "pdf",
        "source_path": path,
        "pages": [str(page) for page in pages],
        "active_image": str(pages[0]) if pages else None,
    }


def load_input(file_path: Any) -> tuple[dict[str, Any] | None, Any, Any, str]:
    file_path = _coerce_file_path(file_path)
    if not file_path:
        return None, None, gr.update(choices=[], value=None, visible=False), "No input loaded."
    path = Path(file_path)
    suffix = path.suffix.lower()
    if suffix == ".pdf":
        pages = pdf_to_images(path, dpi=200)
        if not pages:
            return None, None, gr.update(choices=[], value=None, visible=False), "PDF rendered no pages."
        choices = [f"Page {index + 1}" for index in range(len(pages))]
        state = _state_for_pdf(str(path), pages)
        return (
            state,
            preview_image(pages[0]),
            gr.update(choices=choices, value=choices[0], visible=True),
            f"Loaded PDF with {len(pages)} page(s).",
        )
    if suffix in IMAGE_SUFFIXES:
        return (
            _state_for_image(str(path)),
            preview_image(path),
            gr.update(choices=[], value=None, visible=False),
            f"Loaded image: {path.name}",
        )
    return None, None, gr.update(choices=[], value=None, visible=False), f"Unsupported file type: {suffix}"


def select_pdf_page(page_label: str | None, state: dict[str, Any] | None) -> tuple[dict[str, Any] | None, Any, str]:
    if not state or state.get("kind") != "pdf":
        return state, None, "No PDF loaded."
    pages = state.get("pages") or []
    if not pages:
        return state, None, "PDF has no rendered pages."
    index = 0
    if page_label and page_label.startswith("Page "):
        try:
            index = max(0, min(len(pages) - 1, int(page_label.removeprefix("Page ")) - 1))
        except ValueError:
            index = 0
    state = dict(state)
    state["active_image"] = pages[index]
    return state, preview_image(pages[index]), f"Selected {page_label or 'Page 1'}."


def apply_prompt_profile(prompt_profile_label: str) -> str:
    key = PROMPT_LABEL_TO_KEY.get(prompt_profile_label, DEFAULT_PROMPT_PROFILE)
    return PROMPT_PROFILES[key].prompt


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


def run_ocr(
    state: dict[str, Any] | None,
    prompt: str,
    candidate_profile_label: str,
    max_tokens: int,
) -> Iterator[tuple[str, Any, dict[str, Any], str]]:
    if not state or not state.get("active_image"):
        yield "", None, {}, "Load an image or PDF page before running OCR."
        return

    profile_key = CANDIDATE_LABEL_TO_KEY.get(candidate_profile_label, "best-zero-empty-q4")
    profile = profile_by_key(profile_key)
    token_limit = int(max_tokens or profile.default_max_tokens)
    image_path = str(state["active_image"])
    paths = RuntimePaths.from_env()
    missing = paths.missing()
    if missing:
        yield "", preview_image(image_path), {"missing": missing}, "Native runtime files are missing."
        return

    accumulated = ""
    last_emit = 0.0
    last_preview_box_count = -1
    last_preview = preview_image(image_path)
    yield "", last_preview, _metadata(profile_key=profile_key, text="", image_path=image_path), "Starting native OCR..."

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
            now = time.monotonic()
            boxes = extract_boxes(cleaned)
            if boxes and len(boxes) != last_preview_box_count:
                last_preview = build_preview_image(image_path, cleaned)
                last_preview_box_count = len(boxes)
            if now - last_emit >= 0.18 or len(event.text) >= 32:
                last_emit = now
                yield (
                    cleaned,
                    last_preview,
                    _metadata(profile_key=profile_key, text=cleaned, image_path=image_path),
                    "Running native OCR...",
                )
            continue

        native_meta = event.metadata or {}
        final_text = clean_generated_text(event.text or accumulated)
        final_preview = build_preview_image(image_path, final_text)
        if event.kind == "error":
            if not final_text and native_meta.get("stderr_tail"):
                final_text = str(native_meta["stderr_tail"])
            yield (
                final_text,
                final_preview or last_preview,
                _metadata(profile_key=profile_key, text=final_text, image_path=image_path, native=native_meta),
                "Native OCR failed.",
            )
            return
        yield (
            final_text,
            final_preview or last_preview,
            _metadata(profile_key=profile_key, text=final_text, image_path=image_path, native=native_meta),
            f"Done in {native_meta.get('elapsed_ms', 0)} ms.",
        )


def clear_all() -> tuple[None, None, Any, str, None, str, dict[str, Any], None]:
    return (
        None,
        None,
        gr.update(choices=[], value=None, visible=False),
        "",
        None,
        "Cleared.",
        {},
        None,
    )


def build_demo() -> gr.Blocks:
    css = """
    .uocr-status textarea { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
    .uocr-output textarea { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; line-height: 1.45; }
    """
    with gr.Blocks(title="Unlimited-OCR Portable Candidate", css=css) as demo:
        input_state = gr.State(None)
        gr.Markdown("# Unlimited-OCR Portable Candidate")
        with gr.Row():
            with gr.Column(scale=1):
                file_input = gr.File(
                    label="Image or PDF",
                    file_types=[".png", ".jpg", ".jpeg", ".bmp", ".webp", ".tif", ".tiff", ".pdf"],
                    type="filepath",
                )
                page_select = gr.Dropdown(label="PDF page", choices=[], visible=False)
                input_preview = gr.Image(label="Input", type="pil", height=420)
                prompt_profile = gr.Dropdown(
                    label="Prompt profile",
                    choices=PROMPT_CHOICES,
                    value=_default_prompt_label(),
                )
                prompt = gr.Textbox(label="Prompt", value=PROMPT_PROFILES[DEFAULT_PROMPT_PROFILE].prompt, lines=2)
                candidate_profile = gr.Dropdown(
                    label="Candidate profile",
                    choices=CANDIDATE_CHOICES,
                    value=_default_candidate_label(),
                )
                max_tokens = gr.Slider(
                    label="Max tokens",
                    minimum=64,
                    maximum=8192,
                    value=CANDIDATE_PROFILES["best-zero-empty-q4"].default_max_tokens,
                    step=64,
                )
                with gr.Row():
                    run_button = gr.Button("Run OCR", variant="primary")
                    stop_button = gr.Button("Stop")
                    clear_button = gr.Button("Clear")
            with gr.Column(scale=1):
                status = gr.Textbox(label="Status", value="Idle.", lines=1, elem_classes=["uocr-status"])
                output_text = gr.Textbox(label="OCR output", lines=18, elem_classes=["uocr-output"])
                overlay_preview = gr.Image(label="Bounding-box preview", type="pil", height=420)
                metadata = gr.JSON(label="Run metadata")

        file_input.change(load_input, inputs=file_input, outputs=[input_state, input_preview, page_select, status])
        page_select.change(select_pdf_page, inputs=[page_select, input_state], outputs=[input_state, input_preview, status])
        prompt_profile.change(apply_prompt_profile, inputs=prompt_profile, outputs=prompt)
        run_event = run_button.click(
            run_ocr,
            inputs=[input_state, prompt, candidate_profile, max_tokens],
            outputs=[output_text, overlay_preview, metadata, status],
        )
        stop_button.click(fn=None, cancels=[run_event])
        clear_button.click(
            clear_all,
            outputs=[file_input, input_state, page_select, output_text, overlay_preview, status, metadata, input_preview],
        )
    return demo


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
    parser.add_argument("--smoke", action="store_true", help="Run a CLI smoke test instead of launching Gradio.")
    parser.add_argument("--image", default="dataset/sc-02.png", help="Image path for --smoke.")
    parser.add_argument("--prompt", default=PROMPT_PROFILES[DEFAULT_PROMPT_PROFILE].prompt)
    parser.add_argument("--profile", default="best-zero-empty-q4", choices=sorted(CANDIDATE_PROFILES))
    parser.add_argument("--max-tokens", type=int, default=64)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.smoke:
        return run_smoke(args)
    demo = build_demo()
    demo.queue(default_concurrency_limit=1).launch(
        server_name=args.host,
        server_port=args.port,
        show_error=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
