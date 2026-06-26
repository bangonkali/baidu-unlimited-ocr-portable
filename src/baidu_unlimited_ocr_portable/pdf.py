from __future__ import annotations

import tempfile
from pathlib import Path

import fitz


def pdf_to_images(pdf_path: str | Path, dpi: int = 200) -> list[Path]:
    path = Path(pdf_path)
    doc = fitz.open(path)
    tmp_dir = Path(tempfile.mkdtemp(prefix="uocr_pdf_"))
    mat = fitz.Matrix(dpi / 72, dpi / 72)
    pages: list[Path] = []
    try:
        for index, page in enumerate(doc):
            out_path = tmp_dir / f"page_{index + 1:04d}.png"
            page.get_pixmap(matrix=mat).save(out_path)
            pages.append(out_path)
    finally:
        doc.close()
    return pages

