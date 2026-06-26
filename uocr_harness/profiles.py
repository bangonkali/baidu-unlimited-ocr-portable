from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class PromptProfile:
    name: str
    prompt: str
    description: str


PROMPT_PROFILES: dict[str, PromptProfile] = {
    "grounding": PromptProfile(
        name="grounding",
        prompt="<|grounding|>Convert the document to markdown.",
        description="Layout-aware markdown with detection/bounding-box markers.",
    ),
    "plain_text": PromptProfile(
        name="plain_text",
        prompt="Free OCR.",
        description="Plain OCR text without explicit grounding request.",
    ),
    "ocr_boxes": PromptProfile(
        name="ocr_boxes",
        prompt="<|grounding|>OCR this image.",
        description="Model-card OCR prompt with bounding boxes.",
    ),
    "document_parsing": PromptProfile(
        name="document_parsing",
        prompt="document parsing.",
        description="Native Unlimited-OCR / DeepSeek-OCR parse prompt from the GGUF model card.",
    ),
}

DEFAULT_PROFILE_NAMES = ["grounding", "plain_text"]


def parse_profile_names(value: str | None) -> list[str]:
    if not value:
        return list(DEFAULT_PROFILE_NAMES)
    names = [name.strip() for name in value.split(",") if name.strip()]
    unknown = [name for name in names if name not in PROMPT_PROFILES]
    if unknown:
        known = ", ".join(sorted(PROMPT_PROFILES))
        raise SystemExit(f"Unknown prompt profile(s): {', '.join(unknown)}. Known: {known}")
    return names
