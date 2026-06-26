from __future__ import annotations

import csv
from pathlib import Path
from typing import Any

from .util import ensure_dir, read_json, read_jsonl, utc_now


def compare_debug_artifacts(
    *,
    manifest_path: Path,
    results_dir: Path,
    profile_names: list[str],
    reference_engine: str,
    candidate_engine: str,
    summary_path: Path,
    limit: int | None = None,
    case_id: str | None = None,
) -> list[dict[str, Any]]:
    rows = _filter_rows(read_jsonl(manifest_path), limit=limit, case_id=case_id)
    metrics: list[dict[str, Any]] = []
    for row in rows:
        for profile_name in profile_names:
            ref_result_path = results_dir / "reference" / reference_engine / row["case_id"] / f"{profile_name}.json"
            cand_result_path = results_dir / "candidate" / candidate_engine / row["case_id"] / f"{profile_name}.json"
            ref_artifact_path = _artifact_path_from_result(
                ref_result_path,
                fallback=results_dir
                / "artifacts"
                / "reference"
                / reference_engine
                / row["case_id"]
                / f"{profile_name}.{_reference_artifact_suffix(reference_engine)}.json",
                keys=("native_debug_artifact_path", "debug_artifact_path")
                if reference_engine == "sglang-native"
                else ("debug_artifact_path",),
            )
            cand_artifact_path = _artifact_path_from_result(
                cand_result_path,
                fallback=results_dir
                / "artifacts"
                / "candidate"
                / candidate_engine
                / row["case_id"]
                / f"{profile_name}.llamacpp.json",
                keys=("debug_artifact_path",),
            )
            metrics.append(
                _compare_one(
                    row=row,
                    profile_name=profile_name,
                    reference_artifact_path=ref_artifact_path,
                    candidate_artifact_path=cand_artifact_path,
                )
            )

    metrics_dir = ensure_dir(results_dir / "compare")
    metrics_name = "artifact-metrics.csv" if summary_path.stem == "SUMMARY-parity-artifacts" else f"{summary_path.stem}.csv"
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


def _reference_artifact_suffix(reference_engine: str) -> str:
    return "sglang-native" if reference_engine == "sglang-native" else "sglang"


def _artifact_path_from_result(result_path: Path, *, fallback: Path, keys: tuple[str, ...]) -> Path:
    if result_path.exists():
        try:
            record = read_json(result_path)
            for key in keys:
                value = record.get(key)
                if value:
                    return Path(value)
        except Exception:  # noqa: BLE001 - comparison should fall back to conventional paths
            pass
    return fallback


def _compare_one(
    *,
    row: dict[str, Any],
    profile_name: str,
    reference_artifact_path: Path,
    candidate_artifact_path: Path,
) -> dict[str, Any]:
    reference = read_json(reference_artifact_path) if reference_artifact_path.exists() else None
    candidate = read_json(candidate_artifact_path) if candidate_artifact_path.exists() else None
    ref_view = _sglang_view(reference)
    cand_view = _llamacpp_view(candidate)
    status = _status(ref_view, cand_view)

    ref_first_visible = _first_visible_token(ref_view)
    cand_first_visible = _first_visible_token(cand_view)
    cand_first_raw = _first_raw_token(cand_view)

    return {
        "case_id": row["case_id"],
        "source_path": row["source_rel"],
        "page_index": row.get("page_index") or "",
        "prompt_profile": profile_name,
        "status": status,
        "reference_artifact_exists": bool(reference),
        "candidate_artifact_exists": bool(candidate),
        "reference_prompt_tokens": ref_view.get("prompt_tokens", ""),
        "candidate_prefill_n_past": cand_view.get("prefill_n_past", ""),
        "candidate_text_tokens": cand_view.get("text_token_count", ""),
        "candidate_media_tokens": cand_view.get("media_token_count", ""),
        "reference_output_tokens": _output_token_count(ref_view),
        "candidate_output_tokens": _output_token_count(cand_view),
        "reference_first_token": _first_display_token(ref_view),
        "candidate_first_token": _first_display_token(cand_view),
        "reference_first_visible_token": ref_first_visible,
        "candidate_first_visible_token": cand_first_visible,
        "candidate_first_raw_token": cand_first_raw,
        "first_visible_token_match": ref_first_visible == cand_first_visible,
        "prefill_top_overlap": _top_overlap_ratio(
            reference=ref_view,
            candidate=cand_view,
            candidate_prefix="prefill_top",
        ),
        "first_output_top_overlap": _top_overlap_ratio(
            reference=ref_view,
            candidate=cand_view,
            candidate_prefix="first_output_top",
        ),
        "candidate_stop_reason": cand_view.get("stop_reason", ""),
        "reference_finish_reason": ref_view.get("finish_reason", ""),
        "reference_artifact": str(reference_artifact_path) if reference_artifact_path.exists() else "",
        "candidate_artifact": str(candidate_artifact_path) if candidate_artifact_path.exists() else "",
    }


def _status(reference: dict[str, Any], candidate: dict[str, Any]) -> str:
    if not reference.get("exists"):
        return "missing_reference_artifact"
    if not candidate.get("exists"):
        return "missing_candidate_artifact"
    ref_tokens = reference.get("output_token_ids", [])
    cand_tokens = candidate.get("output_token_ids", [])
    ref_cmp = _first_comparable_token(reference)
    cand_cmp = _first_comparable_token(candidate)
    ref_visible = _first_visible_token(reference)
    cand_visible = _first_visible_token(candidate)
    cand_raw = _first_raw_piece(candidate)
    if not ref_cmp:
        return "reference_no_output_tokens"
    if not cand_cmp:
        return "candidate_no_output_token_ids"
    if ref_visible and cand_visible and ref_visible == cand_visible and cand_raw and not _normalize_piece(cand_raw):
        return "candidate_leading_whitespace_token"
    if ref_cmp != cand_cmp:
        return "first_output_token_mismatch"
    overlap = _top_overlap_ratio(
        reference=reference,
        candidate=candidate,
        candidate_prefix="first_output_top",
    )
    if overlap != "" and float(overlap) < 0.5:
        return "first_output_topk_mismatch"
    ref_count = _output_token_count(reference)
    cand_count = _output_token_count(candidate)
    if abs(ref_count - cand_count) > max(8, int(ref_count * 0.2)):
        return "output_length_mismatch"
    return "api_visible_tokens_aligned"


def _sglang_view(artifact: dict[str, Any] | None) -> dict[str, Any]:
    if artifact is None:
        return {"exists": False}
    response = artifact.get("response")
    if isinstance(response, list) and response:
        response = response[0]
    if not isinstance(response, dict):
        response = {}
    openai_logprobs = _openai_chat_logprobs(response)
    meta = response.get("meta_info") or {}
    output_token_logprobs = meta.get("output_token_logprobs") or []
    output_top_logprobs = meta.get("output_top_logprobs") or []
    output_token_ids = [_token_id_from_logprob_item(item) for item in output_token_logprobs]
    output_token_ids = [token for token in output_token_ids if token is not None]
    output_tokens = _openai_output_tokens(openai_logprobs) or _pieces_from_logprob_items(output_token_logprobs)
    first_output_top = _top_ids_from_logprob_items(output_top_logprobs[0]) if output_top_logprobs else []
    first_output_top_tokens = _openai_top_tokens(openai_logprobs[0]) if openai_logprobs else []
    if not first_output_top_tokens and output_top_logprobs:
        first_output_top_tokens = _pieces_from_logprob_items(output_top_logprobs[0])
    finish_reason = meta.get("finish_reason")
    return {
        "exists": True,
        "prompt_tokens": meta.get("prompt_tokens", ""),
        "completion_tokens": meta.get("completion_tokens", ""),
        "output_token_ids": output_token_ids,
        "output_tokens": output_tokens,
        "first_output_top_token_ids": first_output_top,
        "first_output_top_tokens": first_output_top_tokens,
        "finish_reason": finish_reason.get("type") if isinstance(finish_reason, dict) else finish_reason or "",
    }


def _llamacpp_view(artifact: dict[str, Any] | None) -> dict[str, Any]:
    if artifact is None:
        return {"exists": False}
    generation = artifact.get("generation") or []
    output_token_ids = [step.get("token_id") for step in generation if isinstance(step.get("token_id"), int)]
    output_tokens = [step.get("piece") for step in generation if isinstance(step.get("piece"), str)]
    first_output_top = []
    first_output_top_tokens = []
    if generation:
        first_output_top = [item.get("token_id") for item in generation[0].get("top_logits", [])]
        first_output_top = [token for token in first_output_top if isinstance(token, int)]
        first_output_top_tokens = [
            _normalize_piece(item.get("piece", ""))
            for item in generation[0].get("top_logits", [])
            if isinstance(item.get("piece"), str)
        ]
    prefill_items = artifact.get("prefill_top_logits", [])
    prefill_top = [item.get("token_id") for item in prefill_items]
    prefill_top = [token for token in prefill_top if isinstance(token, int)]
    prefill_top_tokens = [
        _normalize_piece(item.get("piece", ""))
        for item in prefill_items
        if isinstance(item.get("piece"), str)
    ]
    return {
        "exists": True,
        "prefill_n_past": artifact.get("prefill_n_past", ""),
        "text_token_count": artifact.get("text_token_count", ""),
        "media_token_count": artifact.get("media_token_count", ""),
        "output_token_ids": output_token_ids,
        "output_tokens": output_tokens,
        "first_output_top_token_ids": first_output_top,
        "first_output_top_tokens": first_output_top_tokens,
        "prefill_top_token_ids": prefill_top,
        "prefill_top_tokens": prefill_top_tokens,
        "stop_reason": artifact.get("stop_reason", ""),
    }


def _openai_chat_logprobs(response: dict[str, Any]) -> list[dict[str, Any]]:
    try:
        content = response["choices"][0]["logprobs"]["content"]
    except (KeyError, IndexError, TypeError):
        return []
    return content if isinstance(content, list) else []


def _openai_output_tokens(logprobs: list[dict[str, Any]]) -> list[str]:
    out: list[str] = []
    for item in logprobs:
        token = item.get("token") if isinstance(item, dict) else None
        if isinstance(token, str):
            out.append(token)
    return out


def _openai_top_tokens(item: dict[str, Any]) -> list[str]:
    top = item.get("top_logprobs") if isinstance(item, dict) else None
    if not isinstance(top, list):
        return []
    out: list[str] = []
    for top_item in top:
        token = top_item.get("token") if isinstance(top_item, dict) else None
        if isinstance(token, str):
            out.append(_normalize_piece(token))
    return out


def _token_id_from_logprob_item(item: Any) -> int | None:
    if isinstance(item, (list, tuple)) and len(item) >= 2 and isinstance(item[1], int):
        return item[1]
    if isinstance(item, dict):
        for key in ("token_id", "id"):
            value = item.get(key)
            if isinstance(value, int):
                return value
    return None


def _top_ids_from_logprob_items(items: Any) -> list[int]:
    if not isinstance(items, list):
        return []
    out: list[int] = []
    for item in items:
        token_id = _token_id_from_logprob_item(item)
        if token_id is not None:
            out.append(token_id)
    return out


def _pieces_from_logprob_items(items: Any) -> list[str]:
    if not isinstance(items, list):
        return []
    out: list[str] = []
    for item in items:
        piece = None
        if isinstance(item, (list, tuple)) and len(item) >= 3 and isinstance(item[2], str):
            piece = item[2]
        elif isinstance(item, dict):
            value = item.get("token") or item.get("piece")
            if isinstance(value, str):
                piece = value
        if piece is not None:
            out.append(piece)
    return out


def _first(values: list[Any]) -> Any:
    return values[0] if values else ""


def _output_token_count(view: dict[str, Any]) -> int:
    ids = view.get("output_token_ids") or []
    if ids:
        return len(ids)
    return len(view.get("output_tokens") or [])


def _first_comparable_token(view: dict[str, Any]) -> Any:
    ids = view.get("output_token_ids") or []
    if ids:
        return ids[0]
    for piece in view.get("output_tokens") or []:
        normalized = _normalize_piece(piece)
        if normalized:
            return normalized
    return ""


def _first_visible_token(view: dict[str, Any]) -> str:
    for piece in view.get("output_tokens") or []:
        normalized = _normalize_piece(piece)
        if normalized:
            return normalized
    first = _first_comparable_token(view)
    return str(first) if first != "" else ""


def _first_raw_piece(view: dict[str, Any]) -> str:
    for piece in view.get("output_tokens") or []:
        if isinstance(piece, str):
            return piece
    return ""


def _first_raw_token(view: dict[str, Any]) -> Any:
    ids = view.get("output_token_ids") or []
    if ids:
        return ids[0]
    return _first_raw_piece(view)


def _first_display_token(view: dict[str, Any]) -> str:
    raw = _first_raw_token(view)
    visible = _first_visible_token(view)
    if raw == "":
        return ""
    if isinstance(raw, int):
        raw_piece = _first_raw_piece(view)
        if raw_piece and not _normalize_piece(raw_piece) and visible:
            return f"{raw} -> {visible}"
        if raw_piece and _normalize_piece(raw_piece):
            return f"{raw}:{visible}"
        return str(raw)
    raw_text = str(raw)
    if raw_text and not _normalize_piece(raw_text) and visible:
        return f"{raw_text!r} -> {visible}"
    return visible or raw_text


def _first_top_comparable(view: dict[str, Any]) -> list[Any]:
    ids = view.get("first_output_top_token_ids") or []
    if ids:
        return ids
    return [item for item in (view.get("first_output_top_tokens") or []) if item]


def _top_overlap_ratio(
    *,
    reference: dict[str, Any],
    candidate: dict[str, Any],
    candidate_prefix: str,
) -> float | str:
    ref_ids = reference.get("first_output_top_token_ids") or []
    cand_ids = candidate.get(f"{candidate_prefix}_token_ids") or []
    if ref_ids and cand_ids:
        return _overlap_ratio(ref_ids, cand_ids)
    ref_tokens = [item for item in (reference.get("first_output_top_tokens") or []) if item]
    cand_tokens = [item for item in (candidate.get(f"{candidate_prefix}_tokens") or []) if item]
    return _overlap_ratio(ref_tokens, cand_tokens)


def _normalize_piece(piece: str) -> str:
    return "".join(piece.split())


def _overlap_ratio(a: list[Any], b: list[Any]) -> float | str:
    if not a or not b:
        return ""
    aset = set(a)
    bset = set(b)
    return len(aset & bset) / max(len(aset), 1)


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
    review_rows = [m for m in metrics if m["status"] != "api_visible_tokens_aligned"][:10]

    lines = [
        "# Unlimited-OCR Parity Artifact Summary",
        "",
        f"Generated: {utc_now()}",
        "",
        "## Engines",
        "",
        f"- Reference: `{reference_engine}`",
        f"- Candidate: `{candidate_engine}`",
        f"- Metrics CSV: `{_display_path(metrics_path, summary_path.parent)}`",
        "",
        "## Status Counts",
        "",
    ]
    for status in sorted(counts):
        lines.append(f"- `{status}`: {counts[status]}")
    if not counts:
        lines.append("- No artifact rows generated.")

    lines.extend(
        [
            "",
            "## Finding",
            "",
            _finding(counts),
            "",
            "## Review Queue",
            "",
        ]
    )
    if review_rows:
        lines.append(
            "| Status | Case | Profile | Ref first | Cand first | Cand raw | Prefill top overlap | Output top overlap | Ref tokens | Cand tokens |"
        )
        lines.append("|---|---|---|---:|---:|---:|---:|---:|---:|---:|")
        for row in review_rows:
            lines.append(
                "| "
                + " | ".join(
                    [
                        str(row["status"]),
                        str(row["case_id"]),
                        str(row["prompt_profile"]),
                        str(row["reference_first_token"]),
                        str(row["candidate_first_token"]),
                        str(row["candidate_first_raw_token"]),
                        _fmt_overlap(row["prefill_top_overlap"]),
                        _fmt_overlap(row["first_output_top_overlap"]),
                        str(row["reference_output_tokens"]),
                        str(row["candidate_output_tokens"]),
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
            f"- SGLang artifacts use `{_reference_endpoint_note(reference_engine)}` logprob metadata.",
            "- llama.cpp artifacts use `LLAMA_UOCR_PARITY_DUMP` from the patched native MTMD path.",
            "- Native SGLang `/generate` artifacts can include input logprobs; OpenAI chat artifacts only cover output logprobs.",
            "",
        ]
    )
    summary_path.write_text("\n".join(lines), encoding="utf-8")


def _reference_endpoint_note(reference_engine: str) -> str:
    return "/generate" if reference_engine == "sglang-native" else "/v1/chat/completions"


def _finding(counts: dict[str, int]) -> str:
    if not counts:
        return "- No artifact comparisons were available."
    if counts.get("candidate_leading_whitespace_token"):
        return "- The candidate emits a raw leading whitespace token after prefill before the first visible OCR token."
    if counts.get("first_output_token_mismatch"):
        return "- The first API-visible output token diverges; prioritize prefill logits, sampling/logit processor, and tokenization parity."
    if counts.get("reference_no_output_token_ids"):
        return "- SGLang did not expose output token IDs through the debug request; deeper SGLang instrumentation is needed."
    if counts.get("api_visible_tokens_aligned") == sum(counts.values()):
        return "- API-visible output tokens align for this set; move deeper to image embeddings, attention/SWA, or hidden-state instrumentation."
    return "- Artifact rows show mixed results; inspect the review queue before patching model/runtime code."


def _fmt_overlap(value: Any) -> str:
    if isinstance(value, float):
        return f"{value:.3f}"
    return ""


def _display_path(path: Path, base: Path) -> str:
    try:
        return path.resolve().relative_to(base.resolve()).as_posix()
    except ValueError:
        return path.as_posix()
