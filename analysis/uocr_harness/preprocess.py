from __future__ import annotations

from pathlib import Path
from typing import Any

from PIL import Image

from .util import ensure_dir, read_jsonl, resolve_path, write_jsonl


SGLANG_BASE_SIZE = 1024
SGLANG_TILE_SIZE = 640
SGLANG_MIN_TILES = 2
SGLANG_MAX_TILES = 32
PATCH_SIZE = 16
DOWNSAMPLE_RATIO = 4


def inspect_manifest_preprocessing(*, manifest_path: Path, output_path: Path) -> list[dict[str, Any]]:
    rows = []
    for row in read_jsonl(manifest_path):
        image_path = resolve_path(row["prepared_rel"])
        rows.append(
            {
                "case_id": row["case_id"],
                "prepared_path": row["prepared_rel"],
                "source_kind": row.get("source_kind"),
                **inspect_image_preprocessing(image_path),
            }
        )
    ensure_dir(output_path.parent)
    write_jsonl(output_path, rows)
    return rows


def inspect_image_preprocessing(image_path: Path) -> dict[str, Any]:
    with Image.open(image_path) as img:
        width, height = img.size
    return sglang_gundam_metadata(width=width, height=height)


def sglang_gundam_metadata(*, width: int, height: int) -> dict[str, Any]:
    crop_grid = (1, 1)
    has_local_crops = width > SGLANG_TILE_SIZE or height > SGLANG_TILE_SIZE
    if has_local_crops:
        crop_grid = _find_closest_aspect_ratio(
            aspect_ratio=width / height,
            target_ratios=_target_ratios(),
            width=width,
            height=height,
            image_size=SGLANG_TILE_SIZE,
        )

    grid_w, grid_h = crop_grid
    base_queries = _num_queries(SGLANG_BASE_SIZE)
    tile_queries = _num_queries(SGLANG_TILE_SIZE)
    global_tokens = base_queries * (base_queries + 1) + 1

    if has_local_crops and (grid_w > 1 or grid_h > 1):
        sglang_local_tokens = (tile_queries * grid_w + 1) * (tile_queries * grid_h)
        llamacpp_independent_local_tokens = grid_w * grid_h * tile_queries * (tile_queries + 1)
    else:
        sglang_local_tokens = 0
        llamacpp_independent_local_tokens = 0

    composed_local_tokens = sglang_local_tokens
    independent_extra_newline_tokens = llamacpp_independent_local_tokens - sglang_local_tokens
    return {
        "image_width": width,
        "image_height": height,
        "sglang_mode": "gundam",
        "sglang_base_size": SGLANG_BASE_SIZE,
        "sglang_tile_size": SGLANG_TILE_SIZE,
        "sglang_crop_grid": {"width": grid_w, "height": grid_h},
        "sglang_crop_count": grid_w * grid_h if has_local_crops else 0,
        "sglang_global_tokens": global_tokens,
        "sglang_local_tokens": sglang_local_tokens,
        "sglang_total_image_tokens": global_tokens + sglang_local_tokens,
        "llamacpp_native_image_tokens": global_tokens,
        "llamacpp_gundam_composed_local_tokens": composed_local_tokens,
        "llamacpp_gundam_composed_total_image_tokens": global_tokens + composed_local_tokens,
        "llamacpp_gundam_independent_local_tokens": llamacpp_independent_local_tokens,
        "llamacpp_gundam_independent_total_image_tokens": global_tokens + llamacpp_independent_local_tokens,
        "llamacpp_gundam_independent_extra_newline_tokens": independent_extra_newline_tokens,
        "llamacpp_gundam_total_image_tokens": global_tokens + composed_local_tokens,
        "llamacpp_gundam_extra_newline_tokens": 0,
        "llamacpp_gundam_layout_exact": True,
    }


def _num_queries(image_size: int) -> int:
    return -(-((image_size // PATCH_SIZE)) // DOWNSAMPLE_RATIO)


def _target_ratios() -> list[tuple[int, int]]:
    ratios = {
        (w, h)
        for n in range(SGLANG_MIN_TILES, SGLANG_MAX_TILES + 1)
        for w in range(1, n + 1)
        for h in range(1, n + 1)
        if SGLANG_MIN_TILES <= w * h <= SGLANG_MAX_TILES
    }
    return sorted(ratios, key=lambda ratio: ratio[0] * ratio[1])


def _find_closest_aspect_ratio(
    *,
    aspect_ratio: float,
    target_ratios: list[tuple[int, int]],
    width: int,
    height: int,
    image_size: int,
) -> tuple[int, int]:
    best_ratio = (1, 1)
    best_ratio_diff = float("inf")
    area = width * height
    for ratio in target_ratios:
        target_aspect_ratio = ratio[0] / ratio[1]
        ratio_diff = abs(aspect_ratio - target_aspect_ratio)
        if ratio_diff < best_ratio_diff:
            best_ratio_diff = ratio_diff
            best_ratio = ratio
        elif ratio_diff == best_ratio_diff:
            target_area = image_size * image_size * ratio[0] * ratio[1]
            if area > 0.5 * target_area:
                best_ratio = ratio
    return best_ratio
