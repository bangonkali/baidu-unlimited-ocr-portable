from __future__ import annotations

from baidu_unlimited_ocr_portable.native_runner import (
    clean_generated_text,
    detect_recoverable_output_issue,
)


def assert_issue(text: str) -> None:
    issue = detect_recoverable_output_issue(text)
    assert issue, f"expected recoverable output issue for: {text[:120]!r}"


def assert_no_issue(text: str) -> None:
    issue = detect_recoverable_output_issue(text)
    assert not issue, f"unexpected recoverable output issue {issue!r} for: {text[:120]!r}"


def main() -> int:
    assert_issue(", " + ", ".join(str(value) for value in range(2015, 2070)))
    assert_issue(", " + ", ".join(f"{value}: 1" for value in range(4, 70)))
    assert_issue(", " + " ".join("10." for _ in range(70)))

    assert_no_issue(
        "Nataniel Ruiz et al. Dreambooth. In CVPR, pp. 22500-22510, 2023. "
        "Chitwan Saharia et al. Advances in Neural Information Processing Systems, 2022."
    )

    assert clean_generated_text("[0, 0, 999, 999]<|/det|>Nike Air Jordan 1").startswith("Nike")
    assert clean_generated_text(", 10. 10. 10. 10. 10. 10.\n<|det|>ref_text [1, 2, 3, 4]<|/det|>x").startswith(
        "<|det|>ref_text"
    )
    assert clean_generated_text(", , , , , , , <|det|>text [1, 2, 3, 4]<|/det|>x").startswith("<|det|>text")
    assert clean_generated_text(", and the <|det|>text [1, 2, 3, 4]<|/det|>x").startswith("<|det|>text")
    assert clean_generated_text(", the <|det|>image [1, 2, 3, 4]<|/det|>x").startswith("<|det|>image")
    assert clean_generated_text(
        ", and the image is too blurry to recognize any text content. <|det|>image [1, 2, 3, 4]<|/det|>x"
    ).startswith("<|det|>image")
    assert clean_generated_text(
        ", the image contains no text. The horizontal line is visual content. <|det|>image [1, 2, 3, 4]<|/det|>x"
    ).startswith("<|det|>image")
    assert clean_generated_text("00 <|det|>image [1, 2, 3, 4]<|/det|>x").startswith("<|det|>image")
    assert clean_generated_text(", 2:10.16818 v2 [cs.CV] 26 Oct 2023\n<|det|>title [1, 2, 3, 4]<|/det|>x").startswith(
        ", 2:10.16818"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
