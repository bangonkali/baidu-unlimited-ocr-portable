from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path


PORTABLE_ROOT = Path(__file__).resolve().parents[2]
REPO_ROOT = PORTABLE_ROOT.parent
PORTABLE_MODELS = PORTABLE_ROOT / "models"
PORTABLE_THIRDPARTY = PORTABLE_ROOT / "thirdparty"
LEGACY_THIRDPARTY = REPO_ROOT / "thirdparty"


def _exe_suffix() -> str:
    return ".exe" if os.name == "nt" else ""


def _ffi_library_names() -> list[str]:
    if os.name == "nt":
        return ["uocr-ffi.dll", "libuocr-ffi.dll"]
    if os.uname().sysname == "Darwin":
        return ["libuocr-ffi.dylib"]
    return ["libuocr-ffi.so"]


def default_llama_binary() -> Path:
    if os.name == "nt":
        candidates = [
            *sorted(PORTABLE_THIRDPARTY.glob("uocr-runtime/*/bin/llama-uocr-parity.exe")),
            PORTABLE_THIRDPARTY / "llama.cpp" / "build" / "bin" / "Release" / "llama-uocr-parity.exe",
            PORTABLE_THIRDPARTY / "llama.cpp" / "build" / "bin" / "llama-uocr-parity.exe",
            LEGACY_THIRDPARTY / "llama.cpp" / "build" / "bin" / "Release" / "llama-uocr-parity.exe",
            LEGACY_THIRDPARTY / "llama.cpp" / "build" / "bin" / "llama-uocr-parity.exe",
        ]
    else:
        candidates = [
            *sorted(PORTABLE_THIRDPARTY.glob("uocr-runtime/*/bin/llama-uocr-parity")),
            PORTABLE_THIRDPARTY / "llama.cpp" / "build" / "bin" / f"llama-uocr-parity{_exe_suffix()}",
            LEGACY_THIRDPARTY / "llama.cpp" / "build" / "bin" / f"llama-uocr-parity{_exe_suffix()}",
        ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[0]


def default_ffi_library() -> Path:
    candidates: list[Path] = []
    names = _ffi_library_names()
    for name in names:
        candidates.extend(sorted(PORTABLE_THIRDPARTY.glob(f"uocr-runtime/*/bin/{name}")))
    for name in names:
        if os.name == "nt":
            candidates.extend(
                [
                    PORTABLE_THIRDPARTY / "llama.cpp" / "build" / "bin" / "Release" / name,
                    PORTABLE_THIRDPARTY / "llama.cpp" / "build" / "bin" / name,
                    LEGACY_THIRDPARTY / "llama.cpp" / "build" / "bin" / "Release" / name,
                    LEGACY_THIRDPARTY / "llama.cpp" / "build" / "bin" / name,
                ]
            )
        else:
            candidates.extend(
                [
                    PORTABLE_THIRDPARTY / "llama.cpp" / "build" / "bin" / name,
                    LEGACY_THIRDPARTY / "llama.cpp" / "build" / "bin" / name,
                ]
            )
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[0]


def default_llama_server_binary() -> Path:
    if os.name == "nt":
        candidates = [
            *sorted(PORTABLE_THIRDPARTY.glob("uocr-runtime/*/bin/llama-server.exe")),
            PORTABLE_THIRDPARTY / "llama.cpp" / "build" / "bin" / "Release" / "llama-server.exe",
            PORTABLE_THIRDPARTY / "llama.cpp" / "build" / "bin" / "llama-server.exe",
            LEGACY_THIRDPARTY / "llama.cpp" / "build" / "bin" / "Release" / "llama-server.exe",
            LEGACY_THIRDPARTY / "llama.cpp" / "build" / "bin" / "llama-server.exe",
        ]
    else:
        candidates = [
            *sorted(PORTABLE_THIRDPARTY.glob("uocr-runtime/*/bin/llama-server")),
            PORTABLE_THIRDPARTY / "llama.cpp" / "build" / "bin" / f"llama-server{_exe_suffix()}",
            LEGACY_THIRDPARTY / "llama.cpp" / "build" / "bin" / f"llama-server{_exe_suffix()}",
        ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[0]


def resolve_repo_path(value: str | os.PathLike[str] | None, default: Path) -> Path:
    raw = Path(value).expanduser() if value else default
    if raw.is_absolute():
        return raw
    portable_candidate = PORTABLE_ROOT / raw
    if portable_candidate.exists():
        return portable_candidate
    return REPO_ROOT / raw


def default_gguf_file(name: str) -> Path:
    candidates = [
        PORTABLE_MODELS / name,
        PORTABLE_THIRDPARTY / "uocr-gguf" / name,
        LEGACY_THIRDPARTY / "uocr-gguf" / name,
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[0]


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
        label="Practical zero-empty Q4",
        engine_name="llamacpp-q4_k_m-uocr-rswa-eos-origin-ngram-default-full",
        description="Current R-SWA Q4 demo default: 54/104 pass, zero empty rows, avg similarity 0.678.",
        force_prompt_eos=True,
        no_image_end=False,
    ),
    "experimental-exact-prefill-q4": CandidateProfile(
        key="experimental-exact-prefill-q4",
        label="Experimental exact-prefill Q4",
        engine_name="llamacpp-q4_k_m-uocr-rswa-noimgend-noeos-full",
        description="Higher avg similarity 0.719, but had 5 empty rows in full validation.",
        force_prompt_eos=False,
        no_image_end=True,
    ),
}


DEFAULT_PROMPT_PROFILE = "document_parsing"
DEFAULT_CANDIDATE_PROFILE = os.environ.get("UOCR_DEFAULT_PROFILE", "best-zero-empty-q4")

DEFAULT_MODEL = default_gguf_file("Unlimited-OCR-Q4_K_M.gguf")
DEFAULT_MMPROJ = default_gguf_file("mmproj-Unlimited-OCR-F16.gguf")
