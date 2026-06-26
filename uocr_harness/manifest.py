from __future__ import annotations

from pathlib import Path
from typing import Any

from PIL import Image, ImageOps

from .util import case_id_for, ensure_dir, portable_rel, repo_rel, write_jsonl

IMAGE_EXTS = {".png", ".jpg", ".jpeg", ".webp", ".bmp"}
PDF_EXTS = {".pdf"}


def prepare_dataset(
    *,
    dataset_dir: Path,
    results_dir: Path,
    manifest_path: Path,
    pdf_dpi: int = 300,
    force: bool = False,
) -> list[dict[str, Any]]:
    dataset_dir = dataset_dir.resolve()
    prepared_dir = ensure_dir(results_dir / "prepared")
    rows: list[dict[str, Any]] = []

    for source in sorted(dataset_dir.iterdir(), key=lambda p: p.name.lower()):
        if not source.is_file():
            continue
        ext = source.suffix.lower()
        if ext in IMAGE_EXTS:
            rows.append(_prepare_image(source, prepared_dir / "images", force=force))
        elif ext in PDF_EXTS:
            rows.extend(_prepare_pdf(source, prepared_dir / "pdf_pages", pdf_dpi=pdf_dpi, force=force))

    write_jsonl(manifest_path, rows)
    return rows


def _prepare_image(source: Path, out_dir: Path, *, force: bool) -> dict[str, Any]:
    case_id = case_id_for(source)
    out_path = ensure_dir(out_dir) / f"{case_id}.png"
    if force or not out_path.exists():
        with Image.open(source) as img:
            img = ImageOps.exif_transpose(img)
            if img.mode not in ("RGB", "RGBA"):
                img = img.convert("RGB")
            if img.mode == "RGBA":
                background = Image.new("RGB", img.size, (255, 255, 255))
                background.paste(img, mask=img.getchannel("A"))
                img = background
            img.save(out_path, "PNG")
    with Image.open(out_path) as prepared:
        width, height = prepared.size
    return {
        "case_id": case_id,
        "source_kind": "image",
        "source_rel": repo_rel(source),
        "prepared_rel": portable_rel(out_path),
        "page_index": None,
        "source_format": source.suffix.lower().lstrip("."),
        "width": width,
        "height": height,
    }


def _prepare_pdf(source: Path, out_dir: Path, *, pdf_dpi: int, force: bool) -> list[dict[str, Any]]:
    import fitz

    rows: list[dict[str, Any]] = []
    doc = fitz.open(source)
    target_dir = ensure_dir(out_dir / case_id_for(source))
    matrix = fitz.Matrix(pdf_dpi / 72.0, pdf_dpi / 72.0)
    try:
        for index, page in enumerate(doc, start=1):
            suffix = f"page-{index:04d}"
            case_id = case_id_for(source, suffix)
            out_path = target_dir / f"{case_id}.png"
            if force or not out_path.exists():
                pixmap = page.get_pixmap(matrix=matrix, alpha=False)
                pixmap.save(out_path)
            with Image.open(out_path) as prepared:
                width, height = prepared.size
            rows.append(
                {
                    "case_id": case_id,
                    "source_kind": "pdf_page",
                    "source_rel": repo_rel(source),
                    "prepared_rel": portable_rel(out_path),
                    "page_index": index,
                    "source_format": "pdf",
                    "width": width,
                    "height": height,
                    "pdf_dpi": pdf_dpi,
                }
            )
    finally:
        doc.close()
    return rows
