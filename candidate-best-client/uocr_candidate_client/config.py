from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path


CLIENT_ROOT = Path(__file__).resolve().parents[1]
PORTABLE_ROOT = CLIENT_ROOT.parent
REPO_ROOT = PORTABLE_ROOT.parent


def _exe_suffix() -> str:
    return ".exe" if os.name == "nt" else ""


def default_llama_binary() -> Path:
    if os.name == "nt":
        candidates = [
            REPO_ROOT / "thirdparty" / "llama.cpp" / "build" / "bin" / "Release" / "llama-uocr-parity.exe",
            REPO_ROOT / "thirdparty" / "llama.cpp" / "build" / "bin" / "llama-uocr-parity.exe",
        ]
    else:
        candidates = [
            REPO_ROOT / "thirdparty" / "llama.cpp" / "build" / "bin" / f"llama-uocr-parity{_exe_suffix()}",
        ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[0]


def resolve_repo_path(value: str | os.PathLike[str] | None, default: Path) -> Path:
    raw = Path(value).expanduser() if value else default
    return raw if raw.is_absolute() else (REPO_ROOT / raw)


@dataclass(frozen=True)
class PromptProfile:
    key: str
    label: str
    prompt: str
    description: str


PROMPT_PROFILES: dict[str, PromptProfile] = {
    "document_parsing": PromptProfile(
        key="document_parsing",
        label="Document parsing",
        prompt="document parsing.",
        description="Native Unlimited-OCR / DeepSeek-OCR parse prompt.",
    ),
    "grounding": PromptProfile(
        key="grounding",
        label="Grounding markdown",
        prompt="<|grounding|>Convert the document to markdown.",
        description="Layout-aware markdown with detection markers.",
    ),
    "plain_text": PromptProfile(
        key="plain_text",
        label="Plain OCR",
        prompt="Free OCR.",
        description="Plain OCR text without explicit grounding markers.",
    ),
    "ocr_boxes": PromptProfile(
        key="ocr_boxes",
        label="OCR boxes",
        prompt="<|grounding|>OCR this image.",
        description="Model-card OCR prompt with bounding boxes.",
    ),
}


@dataclass(frozen=True)
class CandidateProfile:
    key: str
    label: str
    engine_name: str
    description: str
    force_prompt_eos: bool
    no_image_end: bool
    deepseek_ocr_mode: str = "gundam"
    media_placement: str = "prefix-tight"
    no_repeat_ngram: bool = True
    ngram_size: int = 30
    ngram_window: int = 90
    ngram_whitelist: tuple[int, ...] = (128821, 128822)
    prefill_aware_swa: bool = True
    decode_window: int = 128
    ctx_size: int = 32768
    default_max_tokens: int = 8192


CANDIDATE_PROFILES: dict[str, CandidateProfile] = {
    "best-zero-empty-q4": CandidateProfile(
        key="best-zero-empty-q4",
        label="Best zero-empty Q4",
        engine_name="llamacpp-q4_k_m-uocr-parity-eos-origin-ngram-default-swa128-full",
        description="Current best demo default: 56/104 pass, zero empty rows, avg similarity 0.688.",
        force_prompt_eos=True,
        no_image_end=False,
    ),
    "experimental-exact-prefill-q4": CandidateProfile(
        key="experimental-exact-prefill-q4",
        label="Experimental exact-prefill Q4",
        engine_name="llamacpp-q4_k_m-uocr-parity-noimgend-noeos-swa128-full",
        description="Higher avg similarity 0.717, but had 5 empty rows in full validation.",
        force_prompt_eos=False,
        no_image_end=True,
    ),
}


DEFAULT_PROMPT_PROFILE = "document_parsing"
DEFAULT_CANDIDATE_PROFILE = os.environ.get("UOCR_DEFAULT_PROFILE", "best-zero-empty-q4")

DEFAULT_MODEL = REPO_ROOT / "thirdparty" / "uocr-gguf" / "Unlimited-OCR-Q4_K_M.gguf"
DEFAULT_MMPROJ = REPO_ROOT / "thirdparty" / "uocr-gguf" / "mmproj-Unlimited-OCR-F16.gguf"

