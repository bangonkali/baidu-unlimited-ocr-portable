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


def compare_generation_artifacts(
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
    step_rows: list[dict[str, Any]] = []
    for row in rows:
        for profile_name in profile_names:
            ref_result_path = results_dir / "reference" / "sglang" / row["case_id"] / f"{profile_name}.json"
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
            metric, rows_for_pair = _compare_generation_one(
                row=row,
                profile_name=profile_name,
                reference_artifact_path=ref_artifact_path,
                candidate_artifact_path=cand_artifact_path,
            )
            metrics.append(metric)
            step_rows.extend(rows_for_pair)

    metrics_dir = ensure_dir(results_dir / "compare")
    metrics_name = (
        "generation-artifact-metrics.csv"
        if summary_path.stem == "SUMMARY-generation-artifacts"
        else f"{summary_path.stem}.csv"
    )
    metrics_path = metrics_dir / metrics_name
    steps_path = metrics_dir / f"{summary_path.stem}-steps.csv"
    _write_metrics_csv(metrics_path, metrics)
    _write_metrics_csv(steps_path, step_rows)
    _write_generation_summary(
        summary_path=summary_path,
        metrics_path=metrics_path,
        steps_path=steps_path,
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
        "reference_hidden_shape": ref_view.get("hidden_shape", ""),
        "reference_hidden_count": ref_view.get("hidden_count", ""),
        "reference_hidden_mean": ref_view.get("hidden_mean", ""),
        "candidate_output_embedding_count": cand_view.get("output_embedding_count", ""),
        "candidate_prefill_output_embedding_n_embd": cand_view.get("prefill_output_embedding_n_embd", ""),
        "candidate_prefill_output_embedding_mean": cand_view.get("prefill_output_embedding_mean", ""),
        "candidate_generation_output_embedding_count": cand_view.get("generation_output_embedding_count", ""),
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


def _compare_generation_one(
    *,
    row: dict[str, Any],
    profile_name: str,
    reference_artifact_path: Path,
    candidate_artifact_path: Path,
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    reference = read_json(reference_artifact_path) if reference_artifact_path.exists() else None
    candidate = read_json(candidate_artifact_path) if candidate_artifact_path.exists() else None
    ref_steps = _sglang_generation_steps(reference)
    cand_steps = _llamacpp_generation_steps(candidate)
    comparable = min(len(ref_steps), len(cand_steps))
    matching_prefix = 0
    first_divergence = ""
    first_divergence_detail: dict[str, Any] = {}
    step_rows: list[dict[str, Any]] = []
    overlaps: list[float] = []

    for index in range(comparable):
        ref_step = ref_steps[index]
        cand_step = cand_steps[index]
        token_match = ref_step["token_id"] == cand_step["token_id"]
        overlap = _overlap_ratio(ref_step["top_token_ids"], cand_step["top_token_ids"])
        ref_candidate_score = _score_for_token(ref_step["top_items"], cand_step["token_id"])
        cand_reference_score = _score_for_token(cand_step["top_items"], ref_step["token_id"])
        ref_candidate_rank = _rank_of(ref_step["top_token_ids"], cand_step["token_id"])
        cand_reference_rank = _rank_of(cand_step["top_token_ids"], ref_step["token_id"])
        ref_margin = _score_margin(ref_step["token_score"], ref_candidate_score)
        cand_margin = _score_margin(cand_step["token_score"], cand_reference_score)
        if isinstance(overlap, float):
            overlaps.append(overlap)
        if token_match and first_divergence == "":
            matching_prefix += 1
        elif first_divergence == "":
            first_divergence = index
            first_divergence_detail = {
                "first_divergence_reference_token_id": ref_step["token_id"],
                "first_divergence_reference_piece": ref_step["piece"],
                "first_divergence_candidate_token_id": cand_step["token_id"],
                "first_divergence_candidate_piece": cand_step["piece"],
                "first_divergence_ref_score": ref_step["token_score"],
                "first_divergence_candidate_score_in_reference_top": ref_candidate_score,
                "first_divergence_reference_margin_vs_candidate": ref_margin,
                "first_divergence_candidate_score": cand_step["token_score"],
                "first_divergence_reference_score_in_candidate_top": cand_reference_score,
                "first_divergence_candidate_margin_vs_reference": cand_margin,
                "first_divergence_candidate_rank_in_reference_top": ref_candidate_rank,
                "first_divergence_reference_rank_in_candidate_top": cand_reference_rank,
            }
        step_rows.append(
            {
                "case_id": row["case_id"],
                "source_path": row["source_rel"],
                "page_index": row.get("page_index") or "",
                "prompt_profile": profile_name,
                "step_index": index,
                "token_match": token_match,
                "reference_token_id": ref_step["token_id"],
                "reference_piece": ref_step["piece"],
                "candidate_token_id": cand_step["token_id"],
                "candidate_piece": cand_step["piece"],
                "top_overlap": overlap,
                "reference_score": ref_step["token_score"],
                "candidate_token_score_in_reference_top": ref_candidate_score,
                "reference_margin_vs_candidate": ref_margin,
                "candidate_score": cand_step["token_score"],
                "reference_token_score_in_candidate_top": cand_reference_score,
                "candidate_margin_vs_reference": cand_margin,
                "candidate_token_rank_in_reference_top": ref_candidate_rank,
                "reference_token_rank_in_candidate_top": cand_reference_rank,
                "reference_top_token_ids": ref_step["top_token_ids"],
                "candidate_top_token_ids": cand_step["top_token_ids"],
            }
        )

    if first_divergence == "" and len(ref_steps) != len(cand_steps):
        first_divergence = comparable

    if reference is None:
        status = "missing_reference_artifact"
    elif candidate is None:
        status = "missing_candidate_artifact"
    elif not ref_steps:
        status = "reference_no_generation_steps"
    elif not cand_steps:
        status = "candidate_no_generation_steps"
    elif first_divergence == "":
        status = "generation_steps_match"
    elif matching_prefix > 0:
        status = "generation_diverged_after_prefix"
    else:
        status = "generation_diverged_at_first_token"

    avg_overlap = sum(overlaps) / len(overlaps) if overlaps else ""
    return (
        {
            "case_id": row["case_id"],
            "source_path": row["source_rel"],
            "page_index": row.get("page_index") or "",
            "prompt_profile": profile_name,
            "status": status,
            "reference_artifact_exists": bool(reference),
            "candidate_artifact_exists": bool(candidate),
            "reference_steps": len(ref_steps),
            "candidate_steps": len(cand_steps),
            "comparable_steps": comparable,
            "matching_prefix_tokens": matching_prefix,
            "first_divergence_step": first_divergence,
            "average_top_overlap": avg_overlap,
            "reference_first_ids": [step["token_id"] for step in ref_steps[:16]],
            "candidate_first_ids": [step["token_id"] for step in cand_steps[:16]],
            "reference_artifact": str(reference_artifact_path) if reference_artifact_path.exists() else "",
            "candidate_artifact": str(candidate_artifact_path) if candidate_artifact_path.exists() else "",
            **first_divergence_detail,
        },
        step_rows,
    )


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
        **_sglang_hidden_summary(meta.get("hidden_states")),
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
    output_embedding_summary = _llamacpp_output_embedding_summary(artifact.get("output_embeddings"))
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
        **output_embedding_summary,
        "stop_reason": artifact.get("stop_reason", ""),
    }


def _sglang_hidden_summary(hidden_states: Any) -> dict[str, Any]:
    if not isinstance(hidden_states, dict):
        return {}
    return {
        "hidden_shape": hidden_states.get("shape", ""),
        "hidden_count": hidden_states.get("count", ""),
        "hidden_mean": hidden_states.get("mean", ""),
        "hidden_abs_sum": hidden_states.get("abs_sum", ""),
    }


def _llamacpp_output_embedding_summary(output_embeddings: Any) -> dict[str, Any]:
    if not isinstance(output_embeddings, list):
        return {}
    prefill = next(
        (item for item in output_embeddings if isinstance(item, dict) and item.get("phase") == "prefill_last"),
        {},
    )
    generation = [
        item
        for item in output_embeddings
        if isinstance(item, dict) and item.get("phase") == "generation"
    ]
    return {
        "output_embedding_count": len(output_embeddings),
        "prefill_output_embedding_n_embd": prefill.get("n_embd", "") if isinstance(prefill, dict) else "",
        "prefill_output_embedding_mean": _embedding_mean(prefill),
        "prefill_output_embedding_abs_sum": prefill.get("abs_sum", "") if isinstance(prefill, dict) else "",
        "generation_output_embedding_count": len(generation),
    }


def _embedding_mean(item: Any) -> float | str:
    if not isinstance(item, dict):
        return ""
    total = item.get("sum")
    n_embd = item.get("n_embd")
    if isinstance(total, (int, float)) and isinstance(n_embd, int) and n_embd > 0:
        return float(total) / n_embd
    return ""


def _sglang_generation_steps(artifact: dict[str, Any] | None) -> list[dict[str, Any]]:
    if artifact is None:
        return []
    response = artifact.get("response")
    if isinstance(response, list) and response:
        response = response[0]
    if not isinstance(response, dict):
        response = {}
    openai_logprobs = _openai_chat_logprobs(response)
    meta = response.get("meta_info") or {}
    output_token_logprobs = meta.get("output_token_logprobs") or []
    output_top_logprobs = meta.get("output_top_logprobs") or []
    output_ids = response.get("output_ids") if isinstance(response.get("output_ids"), list) else []
    steps: list[dict[str, Any]] = []
    count = max(len(output_token_logprobs), len(output_ids), len(openai_logprobs))
    for index in range(count):
        logprob_item = output_token_logprobs[index] if index < len(output_token_logprobs) else None
        token_id = _token_id_from_logprob_item(logprob_item)
        if token_id is None and index < len(output_ids) and isinstance(output_ids[index], int):
            token_id = output_ids[index]
        piece = _piece_from_logprob_item(logprob_item)
        if not piece and index < len(openai_logprobs):
            piece = openai_logprobs[index].get("token", "") if isinstance(openai_logprobs[index], dict) else ""
        top_items = output_top_logprobs[index] if index < len(output_top_logprobs) else []
        top_item_views = _top_items_from_logprob_items(top_items)
        steps.append(
            {
                "token_id": token_id,
                "piece": piece,
                "token_score": _score_from_logprob_item(logprob_item),
                "top_items": top_item_views,
                "top_token_ids": [item["token_id"] for item in top_item_views],
                "top_pieces": [item["piece"] for item in top_item_views],
            }
        )
    return [step for step in steps if isinstance(step["token_id"], int)]


def _llamacpp_generation_steps(artifact: dict[str, Any] | None) -> list[dict[str, Any]]:
    if artifact is None:
        return []
    steps: list[dict[str, Any]] = []
    for item in artifact.get("generation") or []:
        token_id = item.get("token_id")
        if not isinstance(token_id, int):
            continue
        top_items = item.get("top_logits") or []
        top_item_views = _top_items_from_llamacpp_logits(top_items)
        steps.append(
            {
                "token_id": token_id,
                "piece": item.get("piece", "") if isinstance(item.get("piece"), str) else "",
                "token_score": _score_for_token(top_item_views, token_id),
                "top_items": top_item_views,
                "top_token_ids": [top["token_id"] for top in top_item_views],
                "top_pieces": [top["piece"] for top in top_item_views],
            }
        )
    return steps


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


def _top_items_from_logprob_items(items: Any) -> list[dict[str, Any]]:
    if not isinstance(items, list):
        return []
    out: list[dict[str, Any]] = []
    for item in items:
        token_id = _token_id_from_logprob_item(item)
        if token_id is None:
            continue
        out.append(
            {
                "token_id": token_id,
                "piece": _piece_from_logprob_item(item),
                "score": _score_from_logprob_item(item),
            }
        )
    return out


def _top_items_from_llamacpp_logits(items: Any) -> list[dict[str, Any]]:
    if not isinstance(items, list):
        return []
    out: list[dict[str, Any]] = []
    for item in items:
        if not isinstance(item, dict):
            continue
        token_id = item.get("token_id")
        if not isinstance(token_id, int):
            continue
        out.append(
            {
                "token_id": token_id,
                "piece": item.get("piece", "") if isinstance(item.get("piece"), str) else "",
                "score": _float_or_blank(item.get("logit")),
            }
        )
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


def _piece_from_logprob_item(item: Any) -> str:
    if isinstance(item, (list, tuple)) and len(item) >= 3 and isinstance(item[2], str):
        return item[2]
    if isinstance(item, dict):
        value = item.get("token") or item.get("piece")
        if isinstance(value, str):
            return value
    return ""


def _score_from_logprob_item(item: Any) -> float | str:
    if isinstance(item, (list, tuple)) and item:
        return _float_or_blank(item[0])
    if isinstance(item, dict):
        for key in ("logprob", "logit", "score"):
            score = _float_or_blank(item.get(key))
            if score != "":
                return score
    return ""


def _float_or_blank(value: Any) -> float | str:
    if isinstance(value, (int, float)):
        return float(value)
    return ""


def _rank_of(token_ids: list[int], token_id: int) -> int | str:
    try:
        return token_ids.index(token_id) + 1
    except ValueError:
        return ""


def _score_for_token(items: list[dict[str, Any]], token_id: int) -> float | str:
    for item in items:
        if item.get("token_id") == token_id:
            score = item.get("score")
            if isinstance(score, (int, float)):
                return float(score)
    return ""


def _score_margin(selected_score: Any, alternate_score: Any) -> float | str:
    if isinstance(selected_score, (int, float)) and isinstance(alternate_score, (int, float)):
        return float(selected_score) - float(alternate_score)
    return ""


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
    fieldnames = _csv_fieldnames(metrics)
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(metrics)


def _csv_fieldnames(metrics: list[dict[str, Any]]) -> list[str]:
    if not metrics:
        return ["status"]
    fieldnames: list[str] = []
    seen: set[str] = set()
    for metric in metrics:
        for key in metric:
            if key in seen:
                continue
            seen.add(key)
            fieldnames.append(key)
    return fieldnames


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
            _finding(counts, metrics),
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


def _write_generation_summary(
    *,
    summary_path: Path,
    metrics_path: Path,
    steps_path: Path,
    metrics: list[dict[str, Any]],
    reference_engine: str,
    candidate_engine: str,
) -> None:
    ensure_dir(summary_path.parent)
    counts: dict[str, int] = {}
    for metric in metrics:
        counts[metric["status"]] = counts.get(metric["status"], 0) + 1

    review_rows = [m for m in metrics if m["status"] != "generation_steps_match"][:10]
    overlaps = [m["average_top_overlap"] for m in metrics if isinstance(m["average_top_overlap"], float)]
    avg_overlap = sum(overlaps) / len(overlaps) if overlaps else None

    lines = [
        "# Unlimited-OCR Generation Step Artifact Summary",
        "",
        f"Generated: {utc_now()}",
        "",
        "## Engines",
        "",
        f"- Reference: `{reference_engine}`",
        f"- Candidate: `{candidate_engine}`",
        f"- Metrics CSV: `{_display_path(metrics_path, summary_path.parent)}`",
        f"- Steps CSV: `{_display_path(steps_path, summary_path.parent)}`",
        "",
        "## Status Counts",
        "",
    ]
    for status in sorted(counts):
        lines.append(f"- `{status}`: {counts[status]}")
    if not counts:
        lines.append("- No generation artifact rows generated.")

    matching_prefix_values = [
        int(m["matching_prefix_tokens"])
        for m in metrics
        if isinstance(m.get("matching_prefix_tokens"), int)
    ]
    first_divergences = [
        int(m["first_divergence_step"])
        for m in metrics
        if isinstance(m.get("first_divergence_step"), int)
    ]
    lines.extend(
        [
            "",
            "## Aggregate Findings",
            "",
            f"- Rows compared: {len(metrics)}",
            f"- Average matching prefix tokens: {_fmt_float(_avg(matching_prefix_values))}",
            f"- Earliest divergence step: {min(first_divergences) if first_divergences else ''}",
            f"- Average top-k overlap: {_fmt_float(avg_overlap)}",
            "",
            "## Review Queue",
            "",
        ]
    )
    if review_rows:
        lines.append(
            "| Status | Case | Profile | Ref Steps | Candidate Steps | Matching Prefix | First Divergence | Ref Token | Cand Token | Cand Rank In Ref Top | Ref Rank In Cand Top | Ref Margin | Cand Margin | Avg Top Overlap |"
        )
        lines.append("|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
        for row in review_rows:
            lines.append(
                "| "
                + " | ".join(
                    [
                        str(row["status"]),
                        str(row["case_id"]),
                        str(row["prompt_profile"]),
                        str(row["reference_steps"]),
                        str(row["candidate_steps"]),
                        str(row["matching_prefix_tokens"]),
                        str(row["first_divergence_step"]),
                        _token_display(
                            row.get("first_divergence_reference_token_id"),
                            row.get("first_divergence_reference_piece"),
                        ),
                        _token_display(
                            row.get("first_divergence_candidate_token_id"),
                            row.get("first_divergence_candidate_piece"),
                        ),
                        str(row.get("first_divergence_candidate_rank_in_reference_top", "")),
                        str(row.get("first_divergence_reference_rank_in_candidate_top", "")),
                        _fmt_score(row.get("first_divergence_reference_margin_vs_candidate")),
                        _fmt_score(row.get("first_divergence_candidate_margin_vs_reference")),
                        _fmt_overlap(row["average_top_overlap"]),
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
            "- This compares generated token IDs and top-k token ID overlap step by step.",
            "- Steps after the first token mismatch are no longer conditioned on the same prefix, so the first divergence is the primary debugging signal.",
            "- Use native SGLang `/generate` artifacts for token IDs and input/output logprobs.",
            "",
        ]
    )
    summary_path.write_text("\n".join(lines), encoding="utf-8")


def _reference_endpoint_note(reference_engine: str) -> str:
    return "/generate" if reference_engine == "sglang-native" else "/v1/chat/completions"


def _finding(counts: dict[str, int], metrics: list[dict[str, Any]]) -> str:
    if not counts:
        return "- No artifact comparisons were available."
    if counts.get("candidate_leading_whitespace_token"):
        return "- The candidate emits a raw leading whitespace token after prefill before the first visible OCR token."
    if counts.get("first_output_token_mismatch"):
        return "- The first API-visible output token diverges; prioritize prefill logits, sampling/logit processor, and tokenization parity."
    if counts.get("reference_no_output_token_ids"):
        return "- SGLang did not expose output token IDs through the debug request; deeper SGLang instrumentation is needed."
    if counts.get("api_visible_tokens_aligned") == sum(counts.values()):
        if any(m.get("reference_hidden_shape") and m.get("candidate_output_embedding_count") for m in metrics):
            return "- API-visible output tokens align and hidden/output-embedding summaries are present for this set; remaining drift is beyond the first-token visible/logit/embedding smoke."
        return "- API-visible output tokens align for this set; move deeper to image embeddings, attention/SWA, or hidden-state instrumentation."
    return "- Artifact rows show mixed results; inspect the review queue before patching model/runtime code."


def _fmt_overlap(value: Any) -> str:
    if isinstance(value, float):
        return f"{value:.3f}"
    return ""


def _fmt_score(value: Any) -> str:
    if isinstance(value, float):
        return f"{value:.6g}"
    return ""


def _fmt_float(value: float | None) -> str:
    return "" if value is None else f"{value:.3f}"


def _token_display(token_id: Any, piece: Any) -> str:
    if token_id is None or token_id == "":
        return ""
    text = str(piece) if piece not in (None, "") else ""
    if text:
        text = text.replace("|", "\\|").replace("\n", "\\n")
        return f"{token_id}:{text}"
    return str(token_id)


def _avg(values: list[int]) -> float | None:
    return (sum(values) / len(values)) if values else None


def _display_path(path: Path, base: Path) -> str:
    try:
        return path.resolve().relative_to(base.resolve()).as_posix()
    except ValueError:
        return path.as_posix()
