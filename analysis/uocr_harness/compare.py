from __future__ import annotations

import csv
import re
from difflib import SequenceMatcher
from pathlib import Path
from typing import Any

from .util import ensure_dir, read_json, read_jsonl, utc_now

DET_RE = re.compile(r"<\|det\|>.*?<\|/det\|>", re.DOTALL)
REF_RE = re.compile(r"<\|ref\|>.*?<\|/ref\|>", re.DOTALL)
SPECIAL_RE = re.compile(r"<\|[^>]+?\|>")
COORD_RE = re.compile(r"\[[0-9,\s.-]+\]")


def compare_results(
    *,
    manifest_path: Path,
    results_dir: Path,
    profile_names: list[str],
    reference_engine: str = "sglang",
    candidate_engine: str = "llamacpp-q4_k_m",
    summary_path: Path,
    limit: int | None = None,
    case_id: str | None = None,
) -> list[dict[str, Any]]:
    rows = _filter_rows(read_jsonl(manifest_path), limit=limit, case_id=case_id)
    metrics: list[dict[str, Any]] = []
    for row in rows:
        for profile_name in profile_names:
            reference_path = results_dir / "reference" / reference_engine / row["case_id"] / f"{profile_name}.json"
            candidate_path = results_dir / "candidate" / candidate_engine / row["case_id"] / f"{profile_name}.json"
            metrics.append(
                _compare_one(
                    row=row,
                    profile_name=profile_name,
                    reference_path=reference_path,
                    candidate_path=candidate_path,
                )
            )

    metrics_dir = ensure_dir(results_dir / "compare")
    metrics_name = "metrics.csv" if summary_path.stem == "SUMMARY" else f"{summary_path.stem}.csv"
    metrics_path = metrics_dir / metrics_name
    _write_metrics_csv(metrics_path, metrics)
    _write_summary(
        summary_path=summary_path,
        metrics_path=metrics_path,
        metrics=metrics,
        reference_engine=reference_engine,
        candidate_engine=candidate_engine,
    )
    return metrics


def _filter_rows(rows: list[dict[str, Any]], *, limit: int | None, case_id: str | None) -> list[dict[str, Any]]:
    if case_id:
        requested = {item.strip() for item in case_id.split(",") if item.strip()}
        rows = [row for row in rows if row["case_id"] in requested]
    if limit is not None:
        rows = rows[:limit]
    return rows


def _compare_one(
    *,
    row: dict[str, Any],
    profile_name: str,
    reference_path: Path,
    candidate_path: Path,
) -> dict[str, Any]:
    reference = read_json(reference_path) if reference_path.exists() else None
    candidate = read_json(candidate_path) if candidate_path.exists() else None
    ref_text = (reference or {}).get("output_text", "")
    cand_text = (candidate or {}).get("output_text", "")
    ref_stats = _text_stats(ref_text)
    cand_stats = _text_stats(cand_text)

    similarity = ""
    if reference and candidate and ref_stats["normalized_text"] and cand_stats["normalized_text"]:
        similarity = SequenceMatcher(None, ref_stats["normalized_text"], cand_stats["normalized_text"]).ratio()

    status = _status(reference, candidate, similarity, ref_stats, cand_stats)
    return {
        "case_id": row["case_id"],
        "source_path": row["source_rel"],
        "page_index": row.get("page_index") or "",
        "prompt_profile": profile_name,
        "status": status,
        "reference_exists": bool(reference),
        "candidate_exists": bool(candidate),
        "reference_exit_code": (reference or {}).get("exit_code", ""),
        "candidate_exit_code": (candidate or {}).get("exit_code", ""),
        "reference_elapsed_ms": (reference or {}).get("elapsed_ms", ""),
        "candidate_elapsed_ms": (candidate or {}).get("elapsed_ms", ""),
        "reference_gpu_after_mb": (reference or {}).get("gpu_memory_after_mb", ""),
        "candidate_gpu_after_mb": (candidate or {}).get("gpu_memory_after_mb", ""),
        "similarity": similarity,
        "char_error_proxy": (1 - similarity) if isinstance(similarity, float) else "",
        "reference_text_len": ref_stats["normalized_len"],
        "candidate_text_len": cand_stats["normalized_len"],
        "reference_bbox_count": ref_stats["bbox_count"],
        "candidate_bbox_count": cand_stats["bbox_count"],
        "reference_malformed_marker_count": ref_stats["malformed_marker_count"],
        "candidate_malformed_marker_count": cand_stats["malformed_marker_count"],
        "candidate_repetition_ratio": cand_stats["repetition_ratio"],
        "reference_result": str(reference_path) if reference_path.exists() else "",
        "candidate_result": str(candidate_path) if candidate_path.exists() else "",
    }


def _status(
    reference: dict[str, Any] | None,
    candidate: dict[str, Any] | None,
    similarity: float | str,
    reference_stats: dict[str, Any],
    candidate_stats: dict[str, Any],
) -> str:
    if reference is None:
        if candidate is not None:
            if candidate.get("exit_code") not in (0, None):
                return "candidate_error_no_reference"
            if not candidate_stats["normalized_text"]:
                return "candidate_empty_no_reference"
            if candidate_stats["repetition_ratio"] >= 0.35:
                return "candidate_repetition_no_reference"
        return "missing_reference"
    if candidate is None:
        return "missing_candidate"
    if reference.get("exit_code") not in (0, None):
        return "reference_error"
    if candidate.get("exit_code") not in (0, None):
        return "candidate_error"
    if not candidate_stats["normalized_text"]:
        return "candidate_empty"
    if candidate_stats["repetition_ratio"] >= 0.35:
        return "candidate_repetition"
    if not isinstance(similarity, float):
        return "review"
    if similarity < 0.35:
        return "low_similarity"
    if candidate_stats["malformed_marker_count"]:
        return "candidate_malformed_markers"
    if _bbox_delta(reference_stats, candidate_stats) > 0.30:
        return "bbox_count_mismatch"
    if similarity < 0.75:
        return "review"
    return "pass"


def _bbox_delta(reference_stats: dict[str, Any], candidate_stats: dict[str, Any]) -> float:
    ref_count = int(reference_stats["bbox_count"])
    cand_count = int(candidate_stats["bbox_count"])
    if ref_count == 0 and cand_count == 0:
        return 0.0
    return abs(ref_count - cand_count) / max(ref_count, 1)


def _text_stats(text: str) -> dict[str, Any]:
    det_open = text.count("<|det|>")
    det_close = text.count("<|/det|>")
    bbox_count = len(DET_RE.findall(text))
    malformed = abs(det_open - det_close)
    malformed += sum(1 for match in DET_RE.findall(text) if "[" not in match or "]" not in match)
    normalized = REF_RE.sub("", text)
    normalized = DET_RE.sub("", normalized)
    normalized = SPECIAL_RE.sub("", normalized)
    normalized = COORD_RE.sub("", normalized)
    normalized = " ".join(normalized.split())
    return {
        "normalized_text": normalized,
        "normalized_len": len(normalized),
        "bbox_count": bbox_count,
        "malformed_marker_count": malformed,
        "repetition_ratio": _repetition_ratio(normalized),
    }


def _repetition_ratio(text: str) -> float:
    tokens = text.split()
    if len(tokens) < 8:
        return 0.0
    ngrams = [tuple(tokens[i : i + 4]) for i in range(len(tokens) - 3)]
    if not ngrams:
        return 0.0
    return 1.0 - (len(set(ngrams)) / len(ngrams))


def _write_metrics_csv(path: Path, metrics: list[dict[str, Any]]) -> None:
    ensure_dir(path.parent)
    fieldnames = list(metrics[0].keys()) if metrics else ["status"]
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(metrics)


def _write_summary(
    *,
    summary_path: Path,
    metrics_path: Path,
    metrics: list[dict[str, Any]],
    reference_engine: str,
    candidate_engine: str,
) -> None:
    ensure_dir(summary_path.parent)
    counts: dict[str, int] = {}
    for metric in metrics:
        counts[metric["status"]] = counts.get(metric["status"], 0) + 1

    comparable = [m for m in metrics if isinstance(m["similarity"], float)]
    avg_similarity = (
        sum(float(m["similarity"]) for m in comparable) / len(comparable) if comparable else None
    )
    avg_candidate_ms = _avg_int([m["candidate_elapsed_ms"] for m in metrics])
    avg_reference_ms = _avg_int([m["reference_elapsed_ms"] for m in metrics])
    avg_candidate_gpu = _avg_int([m["candidate_gpu_after_mb"] for m in metrics])
    avg_reference_gpu = _avg_int([m["reference_gpu_after_mb"] for m in metrics])
    avg_candidate_bbox = _avg_float([m["candidate_bbox_count"] for m in metrics])
    avg_reference_bbox = _avg_float([m["reference_bbox_count"] for m in metrics])
    reference_files = sum(1 for m in metrics if m["reference_exists"])
    candidate_files = sum(1 for m in metrics if m["candidate_exists"])
    candidate_empty = sum(1 for m in metrics if int(m["candidate_text_len"]) == 0 and m["candidate_exists"])
    candidate_nonempty = sum(1 for m in metrics if int(m["candidate_text_len"]) > 0 and m["candidate_exists"])
    candidate_repeated = sum(1 for m in metrics if float(m["candidate_repetition_ratio"]) >= 0.35)
    candidate_malformed = sum(1 for m in metrics if int(m["candidate_malformed_marker_count"]) > 0)
    bbox_mismatched = sum(1 for m in metrics if _metric_bbox_delta(m) > 0.30)
    review_rows = [m for m in metrics if m["status"] not in {"pass"}][:10]
    sglang_log = summary_path.parent / "results" / "logs" / "sglang_server.log"
    sglang_error = _summarize_sglang_log(sglang_log)

    metrics_display = _display_path(metrics_path, summary_path.parent)
    procedure_path = summary_path.parent / "TEST-PROCEDURE.md"
    procedure_display = _display_path(procedure_path, summary_path.parent)
    related_summaries = _related_summaries(summary_path)
    lines = [
        "# Unlimited-OCR Portable Validation Summary",
        "",
        f"Generated: {utc_now()}",
        "",
        "## Engines",
        "",
        f"- Reference: `{reference_engine}`",
        f"- Candidate: `{candidate_engine}`",
        f"- Metrics CSV: `{metrics_display}`",
        f"- Test procedure: `{procedure_display}`",
        "",
        "## Status Counts",
        "",
    ]
    for status in sorted(counts):
        lines.append(f"- `{status}`: {counts[status]}")
    if not counts:
        lines.append("- No metrics generated.")

    if related_summaries:
        lines.extend(["", "## Related Strategy Summaries", ""])
        for path in related_summaries:
            lines.append(f"- `{_display_path(path, summary_path.parent)}`")

    lines.extend(
        [
            "",
            "## Aggregate Metrics",
            "",
            f"- Reference result files: {reference_files} / {len(metrics)}",
            f"- Candidate result files: {candidate_files} / {len(metrics)}",
            f"- Comparable pairs: {len(comparable)} / {len(metrics)}",
            f"- Candidate non-empty outputs: {candidate_nonempty} / {len(metrics)}",
            f"- Candidate empty outputs: {candidate_empty} / {len(metrics)}",
            f"- Candidate high-repetition rows: {candidate_repeated} / {len(metrics)}",
            f"- Candidate malformed-marker rows: {candidate_malformed} / {len(metrics)}",
            f"- Rows with >30% bbox-count delta: {bbox_mismatched} / {len(metrics)}",
            f"- Average text similarity: {_fmt_float(avg_similarity)}",
            f"- Average reference elapsed: {_fmt_ms(avg_reference_ms)}",
            f"- Average candidate elapsed: {_fmt_ms(avg_candidate_ms)}",
            f"- Average reference GPU after request: {_fmt_mb(avg_reference_gpu)}",
            f"- Average candidate GPU after request: {_fmt_mb(avg_candidate_gpu)}",
            f"- Average reference bbox markers: {_fmt_float(avg_reference_bbox)}",
            f"- Average candidate bbox markers: {_fmt_float(avg_candidate_bbox)}",
            "",
            "## Quality Finding",
            "",
            _quality_finding(
                candidate_engine=candidate_engine,
                total=len(metrics),
                candidate_empty=candidate_empty,
                candidate_repeated=candidate_repeated,
                bbox_mismatched=bbox_mismatched,
            ),
            "",
            "## Reference Runner Status",
            "",
            sglang_error or "No SGLang startup error log found.",
            "",
            "## Review Queue",
            "",
        ]
    )
    if review_rows:
        lines.append(
            "| Status | Case | Profile | Similarity | Ref boxes | Cand boxes | Cand chars | Cand repetition | Candidate ms | Source |"
        )
        lines.append("|---|---|---|---:|---:|---:|---:|---:|---:|---|")
        for row in review_rows:
            lines.append(
                "| "
                + " | ".join(
                    [
                        str(row["status"]),
                        str(row["case_id"]),
                        str(row["prompt_profile"]),
                        _fmt_float(row["similarity"] if isinstance(row["similarity"], float) else None),
                        str(row["reference_bbox_count"]),
                        str(row["candidate_bbox_count"]),
                        str(row["candidate_text_len"]),
                        _fmt_float(float(row["candidate_repetition_ratio"])),
                        str(row["candidate_elapsed_ms"] or ""),
                        str(row["source_path"]).replace("|", "\\|"),
                    ]
                )
                + " |"
            )
    else:
        lines.append("No review items.")

    lines.extend(
        [
            "",
            "## Notes",
            "",
            "- `candidate_empty` means the process completed but normalized output text was empty.",
            "- `candidate_repetition` means the normalized output had a repeated 4-gram ratio of at least 0.35.",
            "- `bbox_count_mismatch` means the candidate bbox marker count differs from the reference by more than 30%.",
            "- Similarity is computed after removing detection/ref markers and coordinates.",
            "- Bounding-box quality still needs visual review for cases with marker count mismatches.",
            "",
        ]
    )
    summary_path.write_text("\n".join(lines), encoding="utf-8")


def _avg_int(values: list[Any]) -> float | None:
    ints = [int(v) for v in values if isinstance(v, int) or (isinstance(v, str) and v.isdigit())]
    return sum(ints) / len(ints) if ints else None


def _avg_float(values: list[Any]) -> float | None:
    floats = []
    for value in values:
        try:
            floats.append(float(value))
        except (TypeError, ValueError):
            pass
    return sum(floats) / len(floats) if floats else None


def _metric_bbox_delta(metric: dict[str, Any]) -> float:
    try:
        ref_count = int(metric["reference_bbox_count"])
        cand_count = int(metric["candidate_bbox_count"])
    except (KeyError, TypeError, ValueError):
        return 0.0
    if ref_count == 0 and cand_count == 0:
        return 0.0
    return abs(ref_count - cand_count) / max(ref_count, 1)


def _quality_finding(
    *,
    candidate_engine: str,
    total: int,
    candidate_empty: int,
    candidate_repeated: int,
    bbox_mismatched: int,
) -> str:
    if not total:
        return "- No rows were compared."
    issues = candidate_empty + candidate_repeated + bbox_mismatched
    if candidate_empty > total // 2:
        return (
            f"- `{candidate_engine}` is not production-ready in this harness: most candidate rows "
            "are empty even though the process exits successfully."
        )
    if issues:
        return (
            f"- `{candidate_engine}` needs more validation before packaging: non-empty outputs still "
            "show repetition or layout-marker mismatches."
        )
    return "- Candidate outputs passed the automated checks in this harness."


def _fmt_float(value: float | None) -> str:
    return "n/a" if value is None else f"{value:.3f}"


def _fmt_ms(value: float | None) -> str:
    return "n/a" if value is None else f"{value:.0f} ms"


def _fmt_mb(value: float | None) -> str:
    return "n/a" if value is None else f"{value:.0f} MB"


def _summarize_sglang_log(path: Path) -> str | None:
    if not path.exists():
        return None
    text = path.read_text(encoding="utf-8", errors="replace")
    if "ImportError" in text and "sgl_kernel" in text:
        return (
            "- SGLang reference startup failed before serving requests. "
            "The log reports `ImportError` from `sgl_kernel` while loading `common_ops`, "
            "with an undefined symbol in `sm100/common_ops.abi3.so`."
        )
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    if not lines:
        return "- SGLang log exists but is empty."
    return f"- SGLang log exists. Last line: `{lines[-1][:180]}`"


def _display_path(path: Path, base: Path) -> str:
    try:
        return path.resolve().relative_to(base.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def _related_summaries(summary_path: Path) -> list[Path]:
    parent = summary_path.parent
    summaries = sorted(parent.glob("SUMMARY-*.md"))
    return [path for path in summaries if path.resolve() != summary_path.resolve()]
