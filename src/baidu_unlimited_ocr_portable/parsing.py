from __future__ import annotations

import ast
import base64
import io
import re
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

from PIL import Image, ImageDraw, ImageFont


@dataclass(frozen=True)
class Box:
    label: str
    x1: float
    y1: float
    x2: float
    y2: float


def _is_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def _coerce_boxes(value: Any) -> list[list[float]]:
    if isinstance(value, (list, tuple)) and len(value) == 4 and all(_is_number(item) for item in value):
        return [[float(item) for item in value]]
    boxes: list[list[float]] = []
    if isinstance(value, (list, tuple)):
        for item in value:
            if isinstance(item, (list, tuple)) and len(item) == 4 and all(_is_number(point) for point in item):
                boxes.append([float(point) for point in item])
    return boxes


def _parse_box_literal(raw: str) -> list[list[float]]:
    try:
        return _coerce_boxes(ast.literal_eval(raw))
    except Exception:
        return []


def _append_boxes(target: list[Box], label: str, raw_points: str) -> bool:
    points_list = _parse_box_literal(raw_points)
    if not points_list:
        nested = re.search(r"(\[.*\])", raw_points, re.DOTALL)
        if nested:
            points_list = _parse_box_literal(nested.group(1))
    for points in points_list:
        if len(points) == 4:
            target.append(Box(label=label.strip() or "det", x1=points[0], y1=points[1], x2=points[2], y2=points[3]))
    return bool(points_list)


def extract_boxes(text: str) -> list[Box]:
    boxes: list[Box] = []
    consumed_spans: list[tuple[int, int]] = []
    ref_pattern = re.compile(
        r"<\|ref\|>(.*?)<\|/ref\|>\s*<\|det\|>\s*(.*?)\s*<\|/det\|>",
        re.DOTALL,
    )
    for match in ref_pattern.finditer(text):
        if _append_boxes(boxes, match.group(1).strip(), match.group(2).strip()):
            consumed_spans.append(match.span())

    det_pattern = re.compile(r"<\|det\|>\s*(.*?)\s*<\|/det\|>", re.DOTALL)
    for match in det_pattern.finditer(text):
        if any(match.start() >= start and match.end() <= end for start, end in consumed_spans):
            continue
        content = match.group(1).strip()
        bracket_at = content.find("[")
        if bracket_at < 0:
            continue
        label = content[:bracket_at].strip() or "det"
        _append_boxes(boxes, label, content[bracket_at:])
    return boxes


def boxes_as_dicts(boxes: list[Box]) -> list[dict[str, float | str]]:
    return [asdict(box) for box in boxes]


def preview_image(image_path: str | Path, max_size: int = 900) -> Image.Image | None:
    try:
        image = Image.open(image_path).convert("RGB")
    except Exception:
        return None
    if max(image.size) > max_size:
        resample = getattr(Image, "Resampling", Image).LANCZOS
        image.thumbnail((max_size, max_size), resample)
    return image


def build_preview_image(image_path: str | Path, text: str, max_size: int = 900) -> Image.Image | None:
    image = preview_image(image_path, max_size=max_size)
    if image is None:
        return None
    boxes = extract_boxes(text)
    if not boxes:
        return image

    annotated = image.copy()
    draw = ImageDraw.Draw(annotated)
    font = ImageFont.load_default()
    width, height = annotated.size

    for box in boxes:
        x1, y1, x2, y2 = [float(value) for value in (box.x1, box.y1, box.x2, box.y2)]
        x1 = max(0, min(width, int(x1 / 999 * width)))
        y1 = max(0, min(height, int(y1 / 999 * height)))
        x2 = max(0, min(width, int(x2 / 999 * width)))
        y2 = max(0, min(height, int(y2 / 999 * height)))
        if x2 < x1:
            x1, x2 = x2, x1
        if y2 < y1:
            y1, y2 = y2, y1
        draw.rectangle([x1, y1, x2, y2], outline=(255, 107, 53), width=3)
        text_bbox = draw.textbbox((0, 0), box.label, font=font)
        text_width = text_bbox[2] - text_bbox[0]
        text_height = text_bbox[3] - text_bbox[1]
        tx = max(0, min(x1, width - text_width))
        ty = max(0, y1 - text_height - 4)
        draw.rectangle([tx, ty, tx + text_width, ty + text_height + 2], fill=(255, 107, 53))
        draw.text((tx, ty), box.label, font=font, fill=(255, 255, 255))
    return annotated


def preview_data_url(image_path: str | Path, text: str, max_size: int = 900) -> str | None:
    image = build_preview_image(image_path, text, max_size=max_size)
    if image is None:
        return None
    preview_bytes = io.BytesIO()
    image.save(preview_bytes, format="PNG")
    return "data:image/png;base64," + base64.b64encode(preview_bytes.getvalue()).decode("utf-8")

