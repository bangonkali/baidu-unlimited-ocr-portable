from __future__ import annotations

import csv
import hashlib
import json
from pathlib import Path
from typing import Any

from PIL import Image

from .preprocess import inspect_image_preprocessing
from .profiles import PROMPT_PROFILES
from .util import ensure_dir, read_json, read_jsonl, resolve_path, utc_now, write_json, write_jsonl


IMAGE_MODE_PRESETS: dict[str, dict[str, Any]] = {
    "tiny": {"base_size": 512, "image_size": 512, "crop_mode": False},
    "small": {"base_size": 640, "image_size": 640, "crop_mode": False},
    "base": {"base_size": 1024, "image_size": 1024, "crop_mode": False},
    "large": {"base_size": 1280, "image_size": 1280, "crop_mode": False},
    "gundam": {"base_size": 1024, "image_size": 640, "crop_mode": True},
}

BOUNDARY_NEWLINE_TOKEN_ID = 201
EOS_TOKEN_ID = 1
SPECIAL_TOKENS = [
    "<image>",
    "<|det|>",
    "<|/det|>",
    "<|grounding|>",
    "<|ref|>",
    "<|/ref|>",
    "<|User|>",
    "<|Assistant|>",
]


def inspect_sglang_processor(
    *,
    manifest_path: Path,
    results_dir: Path,
    profile_names: list[str],
    model_dir: Path,
    image_mode: str,
    media_placement: str,
    output_path: Path,
    limit: int | None = None,
    case_id: str | None = None,
    force: bool = False,
) -> list[dict[str, Any]]:
    """Use the installed SGLang Unlimited-OCR processor as a template oracle."""
    mode_kwargs = _image_mode_kwargs(image_mode)
    processor, tokenizer, processor_config, model_config = _load_sglang_processor(model_dir)
    rows = _filter_rows(read_jsonl(manifest_path), limit=limit, case_id=case_id)
    summary_rows: list[dict[str, Any]] = []

    for row in rows:
        image_path = resolve_path(row["prepared_rel"])
        for profile_name in profile_names:
            profile = PROMPT_PROFILES[profile_name]
            artifact_path = _processor_artifact_path(
                results_dir=results_dir,
                case_id=row["case_id"],
                profile_name=profile_name,
            )
            if artifact_path.exists() and not force:
                artifact = read_json(artifact_path)
            else:
                processor_prompt = _processor_prompt(profile.prompt, media_placement)
                artifact = _build_processor_artifact(
                    row=row,
                    profile_name=profile_name,
                    raw_prompt=profile.prompt,
                    processor_prompt=processor_prompt,
                    image_path=image_path,
                    image_mode=image_mode,
                    media_placement=media_placement,
                    mode_kwargs=mode_kwargs,
                    processor=processor,
                    tokenizer=tokenizer,
                    processor_config=processor_config,
                    model_config=model_config,
                    model_dir=model_dir,
                )
                write_json(artifact_path, artifact)
            summary_rows.append(_processor_summary_row(artifact_path=artifact_path, artifact=artifact))

    write_jsonl(output_path, summary_rows)
    return summary_rows


def compare_runtime_parity(
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
            reference_artifact_path = (
                results_dir
                / "artifacts"
                / "reference"
                / reference_engine
                / row["case_id"]
                / f"{profile_name}.processor.json"
            )
            candidate_artifact_path = _candidate_artifact_path(
                results_dir=results_dir,
                candidate_engine=candidate_engine,
                row=row,
                profile_name=profile_name,
            )
            metrics.append(
                _compare_one_runtime(
                    row=row,
                    profile_name=profile_name,
                    reference_artifact_path=reference_artifact_path,
                    candidate_artifact_path=candidate_artifact_path,
                )
            )

    metrics_dir = ensure_dir(results_dir / "compare")
    metrics_name = (
        "runtime-parity-metrics.csv"
        if summary_path.stem == "SUMMARY-runtime-parity"
        else f"{summary_path.stem}.csv"
    )
    metrics_path = metrics_dir / metrics_name
    _write_metrics_csv(metrics_path, metrics)
    _write_runtime_summary(
        summary_path=summary_path,
        metrics_path=metrics_path,
        metrics=metrics,
        reference_engine=reference_engine,
        candidate_engine=candidate_engine,
    )
    return metrics


def _load_sglang_processor(model_dir: Path) -> tuple[Any, Any, dict[str, Any], dict[str, Any]]:
    try:
        from sglang.srt.configs.unlimited_ocr import UnlimitedOCRHFProcessor
        from transformers import AutoTokenizer
    except Exception as exc:  # noqa: BLE001 - turn dependency failures into a direct harness error
        raise SystemExit(
            "inspect-sglang-processor must run with the SGLang environment, for example "
            "`uv run --project unlimited-ocr-portable --python .venv/bin/python ...`."
        ) from exc

    processor_config = json.loads((model_dir / "processor_config.json").read_text(encoding="utf-8"))
    model_config = json.loads((model_dir / "config.json").read_text(encoding="utf-8"))
    tokenizer = AutoTokenizer.from_pretrained(model_dir, trust_remote_code=True)
    kwargs = {k: v for k, v in processor_config.items() if k != "processor_class"}
    processor = UnlimitedOCRHFProcessor(tokenizer=tokenizer, **kwargs)
    return processor, tokenizer, processor_config, model_config


def _build_processor_artifact(
    *,
    row: dict[str, Any],
    profile_name: str,
    raw_prompt: str,
    processor_prompt: str,
    image_path: Path,
    image_mode: str,
    media_placement: str,
    mode_kwargs: dict[str, Any],
    processor: Any,
    tokenizer: Any,
    processor_config: dict[str, Any],
    model_config: dict[str, Any],
    model_dir: Path,
) -> dict[str, Any]:
    with Image.open(image_path) as image:
        rgb_image = image.convert("RGB")
        processed = processor(text=[processor_prompt], images=[rgb_image], **mode_kwargs)

    input_ids = _flatten_ints(processed.input_ids)
    seq_mask = _bool_list(getattr(processed, "images_seq_mask", []))
    image_token_id = int(processor.image_token_id)
    non_image_ids = [token_id for token_id in input_ids if token_id != image_token_id]
    image_mask_runs = _runs(seq_mask, target=True)
    image_id_runs = _runs([token_id == image_token_id for token_id in input_ids], target=True)
    preprocessing = inspect_image_preprocessing(image_path)

    return {
        "schema_version": 1,
        "engine": "sglang-processor",
        "generated_at": utc_now(),
        "model_dir": str(model_dir),
        "model_config": _model_config_summary(model_config),
        "processor_config": {
            "processor_class": processor_config.get("processor_class"),
            "image_token": processor_config.get("image_token"),
            "patch_size": processor_config.get("patch_size"),
            "downsample_ratio": processor_config.get("downsample_ratio"),
            "candidate_resolutions": processor_config.get("candidate_resolutions"),
            "add_special_token": processor_config.get("add_special_token"),
            "mask_prompt": processor_config.get("mask_prompt"),
        },
        "tokenizer": _tokenizer_summary(tokenizer, image_token_id=image_token_id),
        "case": {
            "case_id": row["case_id"],
            "source_path": row["source_rel"],
            "prepared_path": row["prepared_rel"],
            "page_index": row.get("page_index"),
            "profile": profile_name,
        },
        "prompt": {
            "raw_prompt": raw_prompt,
            "processor_prompt": processor_prompt,
            "media_placement": media_placement,
            "image_token_at_prefix": media_placement in {"separate", "prefix-tight", "prefix-newline"},
            "conversation_template": "unlimited-ocr",
            "roles": ["", ""],
            "separators": ["", ""],
        },
        "image": {
            **_image_digest(image_path),
            "mode": image_mode,
            "processor_kwargs": mode_kwargs,
            "preprocessing": preprocessing,
        },
        "input_ids": {
            "count": len(input_ids),
            "first_ids": input_ids[:16],
            "last_ids": input_ids[-16:],
            "image_token_id": image_token_id,
            "image_token_count": sum(1 for token_id in input_ids if token_id == image_token_id),
            "non_image_token_count": len(non_image_ids),
            "non_image_token_ids": non_image_ids,
            "non_image_tokens": _token_entries(tokenizer, non_image_ids),
            "image_token_runs": image_id_runs,
        },
        "images_seq_mask": {
            "count": len(seq_mask),
            "true_count": sum(1 for item in seq_mask if item),
            "true_runs": image_mask_runs,
            "matches_image_token_count": sum(1 for item in seq_mask if item)
            == sum(1 for token_id in input_ids if token_id == image_token_id),
        },
        "tensors": {
            "pixel_values": _tensor_summary(getattr(processed, "pixel_values", None)),
            "images_crop": _tensor_summary(getattr(processed, "images_crop", None)),
            "images_spatial_crop": _tensor_summary(getattr(processed, "images_spatial_crop", None)),
            "images_spatial_crop_values": _to_builtin(getattr(processed, "images_spatial_crop", None)),
            "has_images": _to_builtin(getattr(processed, "has_images", None)),
            "has_local_crops": _to_builtin(getattr(processed, "has_local_crops", None)),
        },
        "notes": [
            "This artifact comes from the custom SGLang UnlimitedOCRHFProcessor and does not run model generation.",
            "It captures tokenizer/template/media-token layout so llama.cpp artifacts can be compared offline.",
        ],
    }


def _processor_summary_row(*, artifact_path: Path, artifact: dict[str, Any]) -> dict[str, Any]:
    input_ids = artifact.get("input_ids", {})
    image = artifact.get("image", {})
    tensors = artifact.get("tensors", {})
    case = artifact.get("case", {})
    return {
        "case_id": case.get("case_id", ""),
        "source_path": case.get("source_path", ""),
        "page_index": case.get("page_index") or "",
        "prompt_profile": case.get("profile", ""),
        "processor_prompt": artifact.get("prompt", {}).get("processor_prompt", ""),
        "image_mode": image.get("mode", ""),
        "input_token_count": input_ids.get("count", ""),
        "image_token_count": input_ids.get("image_token_count", ""),
        "non_image_token_ids": input_ids.get("non_image_token_ids", []),
        "spatial_crop": tensors.get("images_spatial_crop_values"),
        "preprocessing_total_image_tokens": image.get("preprocessing", {}).get("sglang_total_image_tokens", ""),
        "artifact": str(artifact_path),
    }


def _compare_one_runtime(
    *,
    row: dict[str, Any],
    profile_name: str,
    reference_artifact_path: Path,
    candidate_artifact_path: Path,
) -> dict[str, Any]:
    reference = read_json(reference_artifact_path) if reference_artifact_path.exists() else None
    candidate = read_json(candidate_artifact_path) if candidate_artifact_path.exists() else None

    ref_input = (reference or {}).get("input_ids", {})
    ref_non_image = _int_list(ref_input.get("non_image_token_ids", []))
    ref_total = _optional_int(ref_input.get("count"))
    ref_image_count = _optional_int(ref_input.get("image_token_count"))

    cand_text_tokens = _candidate_text_tokens(candidate)
    cand_stripped = _strip_candidate_boundary_tokens(cand_text_tokens, ref_non_image)
    cand_media_count = _optional_int((candidate or {}).get("media_token_count"))
    cand_prefill = _optional_int((candidate or {}).get("prefill_n_past"))
    expected_prefill_delta = (
        cand_prefill - ref_total if cand_prefill is not None and ref_total is not None else ""
    )

    status = _runtime_status(
        reference=reference,
        candidate=candidate,
        ref_non_image=ref_non_image,
        cand_text_tokens=cand_text_tokens,
        cand_stripped=cand_stripped,
        ref_image_count=ref_image_count,
        cand_media_count=cand_media_count,
        expected_prefill_delta=expected_prefill_delta,
    )

    return {
        "case_id": row["case_id"],
        "source_path": row["source_rel"],
        "page_index": row.get("page_index") or "",
        "prompt_profile": profile_name,
        "status": status,
        "reference_artifact_exists": bool(reference),
        "candidate_artifact_exists": bool(candidate),
        "reference_processor_prompt": (reference or {}).get("prompt", {}).get("processor_prompt", ""),
        "candidate_formatted_prompt": (candidate or {}).get("formatted_prompt", ""),
        "reference_total_tokens": ref_total if ref_total is not None else "",
        "candidate_prefill_n_past": cand_prefill if cand_prefill is not None else "",
        "prefill_delta_vs_processor": expected_prefill_delta,
        "reference_image_tokens": ref_image_count if ref_image_count is not None else "",
        "candidate_media_tokens": cand_media_count if cand_media_count is not None else "",
        "image_token_count_match": ref_image_count == cand_media_count
        if ref_image_count is not None and cand_media_count is not None
        else "",
        "reference_non_image_tokens": ref_non_image,
        "candidate_text_tokens": cand_text_tokens,
        "candidate_text_tokens_boundary_stripped": cand_stripped,
        "text_tokens_exact_match": ref_non_image == cand_text_tokens,
        "text_tokens_boundary_stripped_match": ref_non_image == cand_stripped,
        "candidate_has_extra_newline": BOUNDARY_NEWLINE_TOKEN_ID in _extra_tokens(cand_text_tokens, cand_stripped),
        "candidate_has_extra_eos": EOS_TOKEN_ID in _extra_tokens(cand_text_tokens, cand_stripped),
        "reference_sliding_window": (reference or {}).get("model_config", {}).get("sliding_window_size", ""),
        "reference_prefill_aware_swa": (reference or {})
        .get("model_config", {})
        .get("prefill_aware_swa", ""),
        "candidate_stop_reason": (candidate or {}).get("stop_reason", ""),
        "reference_artifact": str(reference_artifact_path) if reference_artifact_path.exists() else "",
        "candidate_artifact": str(candidate_artifact_path) if candidate_artifact_path.exists() else "",
    }


def _runtime_status(
    *,
    reference: dict[str, Any] | None,
    candidate: dict[str, Any] | None,
    ref_non_image: list[int],
    cand_text_tokens: list[int],
    cand_stripped: list[int],
    ref_image_count: int | None,
    cand_media_count: int | None,
    expected_prefill_delta: int | str,
) -> str:
    if reference is None:
        return "missing_reference_processor_artifact"
    if candidate is None:
        return "missing_candidate_artifact"
    if ref_image_count != cand_media_count:
        return "image_token_count_mismatch"
    if ref_non_image == cand_text_tokens and expected_prefill_delta in (0, ""):
        return "runtime_sequence_match"
    if ref_non_image == cand_stripped:
        return "candidate_extra_boundary_tokens"
    if ref_non_image != cand_text_tokens:
        return "text_token_sequence_mismatch"
    if expected_prefill_delta not in (0, ""):
        return "prefill_length_mismatch"
    return "review"


def _candidate_artifact_path(
    *,
    results_dir: Path,
    candidate_engine: str,
    row: dict[str, Any],
    profile_name: str,
) -> Path:
    result_path = results_dir / "candidate" / candidate_engine / row["case_id"] / f"{profile_name}.json"
    fallback = (
        results_dir
        / "artifacts"
        / "candidate"
        / candidate_engine
        / row["case_id"]
        / f"{profile_name}.llamacpp.json"
    )
    if result_path.exists():
        try:
            value = read_json(result_path).get("debug_artifact_path")
            if value:
                return Path(value)
        except Exception:  # noqa: BLE001 - conventional artifact path is enough for comparison
            pass
    return fallback


def _processor_artifact_path(*, results_dir: Path, case_id: str, profile_name: str) -> Path:
    return results_dir / "artifacts" / "reference" / "sglang-processor" / case_id / f"{profile_name}.processor.json"


def _image_mode_kwargs(image_mode: str) -> dict[str, Any]:
    try:
        return dict(IMAGE_MODE_PRESETS[image_mode])
    except KeyError as exc:
        raise SystemExit(f"Unknown SGLang image mode: {image_mode}") from exc


def _processor_prompt(prompt: str, media_placement: str) -> str:
    if media_placement in {"separate", "prefix-tight"}:
        return f"<image>{prompt}"
    if media_placement == "prefix-newline":
        return f"<image>\n{prompt}"
    if media_placement == "suffix-newline":
        return f"{prompt}\n<image>"
    raise SystemExit(f"Unknown SGLang media placement: {media_placement}")


def _filter_rows(rows: list[dict[str, Any]], *, limit: int | None, case_id: str | None) -> list[dict[str, Any]]:
    if case_id:
        requested = {item.strip() for item in case_id.split(",") if item.strip()}
        rows = [row for row in rows if row["case_id"] in requested]
    if limit is not None:
        rows = rows[:limit]
    return rows


def _flatten_ints(value: Any) -> list[int]:
    value = _to_builtin(value)
    if isinstance(value, list):
        return [int(item) for item in _flatten(value)]
    return []


def _flatten(value: list[Any]) -> list[Any]:
    out: list[Any] = []
    for item in value:
        if isinstance(item, list):
            out.extend(_flatten(item))
        else:
            out.append(item)
    return out


def _bool_list(value: Any) -> list[bool]:
    value = _to_builtin(value)
    if not isinstance(value, list):
        return []
    return [bool(item) for item in _flatten(value)]


def _runs(values: list[bool], *, target: bool) -> list[dict[str, int]]:
    runs: list[dict[str, int]] = []
    start: int | None = None
    for index, value in enumerate(values):
        if value == target and start is None:
            start = index
        elif value != target and start is not None:
            runs.append({"start": start, "end_exclusive": index, "length": index - start})
            start = None
    if start is not None:
        runs.append({"start": start, "end_exclusive": len(values), "length": len(values) - start})
    return runs


def _tokenizer_summary(tokenizer: Any, *, image_token_id: int) -> dict[str, Any]:
    return {
        "class": type(tokenizer).__name__,
        "name_or_path": getattr(tokenizer, "name_or_path", ""),
        "bos_token_id": tokenizer.bos_token_id,
        "eos_token_id": tokenizer.eos_token_id,
        "pad_token_id": tokenizer.pad_token_id,
        "image_token_id": image_token_id,
        "special_token_ids": {
            token: {
                "convert_tokens_to_ids": _safe_convert_token_to_id(tokenizer, token),
                "encode": tokenizer.encode(token, add_special_tokens=False),
            }
            for token in SPECIAL_TOKENS
        },
    }


def _safe_convert_token_to_id(tokenizer: Any, token: str) -> int | str | None:
    try:
        value = tokenizer.convert_tokens_to_ids(token)
        if isinstance(value, int):
            return value
        return str(value) if value is not None else None
    except Exception:  # noqa: BLE001
        return None


def _token_entries(tokenizer: Any, token_ids: list[int]) -> list[dict[str, Any]]:
    entries = []
    for token_id in token_ids:
        try:
            token = tokenizer.convert_ids_to_tokens(token_id)
        except Exception:  # noqa: BLE001
            token = ""
        try:
            text = tokenizer.decode([token_id], skip_special_tokens=False)
        except Exception:  # noqa: BLE001
            text = ""
        entries.append({"id": token_id, "token": token, "text": text})
    return entries


def _model_config_summary(config: dict[str, Any]) -> dict[str, Any]:
    language = config.get("language_config") or {}
    return {
        "model_type": config.get("model_type"),
        "architectures": config.get("architectures"),
        "torch_dtype": config.get("torch_dtype"),
        "hidden_size": config.get("hidden_size"),
        "num_hidden_layers": config.get("num_hidden_layers"),
        "vocab_size": config.get("vocab_size"),
        "sliding_window_size": config.get("sliding_window_size")
        or language.get("sliding_window_size"),
        "sliding_window": config.get("sliding_window") or language.get("sliding_window"),
        "language_config": {
            "hidden_size": language.get("hidden_size"),
            "num_hidden_layers": language.get("num_hidden_layers"),
            "vocab_size": language.get("vocab_size"),
            "sliding_window_size": language.get("sliding_window_size"),
        },
        "prefill_aware_swa": True,
        "prefill_aware_swa_source": "sglang.srt.models.unlimited_ocr.UnlimitedOCRForCausalLM.is_prefill_aware_swa",
    }


def _tensor_summary(value: Any) -> dict[str, Any] | None:
    if value is None:
        return None
    try:
        tensor = value.detach().cpu() if hasattr(value, "detach") else value
        shape = list(tensor.shape) if hasattr(tensor, "shape") else None
        dtype = str(tensor.dtype) if hasattr(tensor, "dtype") else type(tensor).__name__
        numel = int(tensor.numel()) if hasattr(tensor, "numel") else None
        summary: dict[str, Any] = {"shape": shape, "dtype": dtype, "numel": numel}
        if numel:
            float_tensor = tensor.float() if hasattr(tensor, "float") else None
            if float_tensor is not None:
                summary.update(
                    {
                        "min": float(float_tensor.min().item()),
                        "max": float(float_tensor.max().item()),
                        "mean": float(float_tensor.mean().item()),
                    }
                )
        return summary
    except Exception as exc:  # noqa: BLE001
        return {"error": f"{type(exc).__name__}: {exc}"}


def _to_builtin(value: Any) -> Any:
    if hasattr(value, "detach"):
        value = value.detach().cpu()
    if hasattr(value, "tolist"):
        return value.tolist()
    if isinstance(value, tuple):
        return [_to_builtin(item) for item in value]
    if isinstance(value, list):
        return [_to_builtin(item) for item in value]
    if isinstance(value, dict):
        return {str(k): _to_builtin(v) for k, v in value.items()}
    if hasattr(value, "item"):
        return value.item()
    return value


def _image_digest(image_path: Path) -> dict[str, Any]:
    data = image_path.read_bytes()
    with Image.open(image_path) as image:
        width, height = image.size
    return {
        "path": str(image_path),
        "width": width,
        "height": height,
        "bytes": len(data),
        "sha256": hashlib.sha256(data).hexdigest(),
    }


def _candidate_text_tokens(candidate: dict[str, Any] | None) -> list[int]:
    if not candidate:
        return []
    tokens: list[int] = []
    for chunk in candidate.get("chunks") or []:
        if chunk.get("type") == "text":
            tokens.extend(_int_list(chunk.get("text_tokens", [])))
    return tokens


def _strip_candidate_boundary_tokens(candidate_tokens: list[int], reference_tokens: list[int]) -> list[int]:
    stripped = list(candidate_tokens)
    if (
        reference_tokens
        and stripped
        and reference_tokens[0] == stripped[0]
        and len(stripped) > 1
        and stripped[1] == BOUNDARY_NEWLINE_TOKEN_ID
    ):
        del stripped[1]
    if reference_tokens and stripped and reference_tokens[-1:] != [EOS_TOKEN_ID] and stripped[-1] == EOS_TOKEN_ID:
        stripped = stripped[:-1]
    return stripped


def _extra_tokens(original: list[int], stripped: list[int]) -> list[int]:
    extra = list(original)
    for token_id in stripped:
        if token_id in extra:
            extra.remove(token_id)
    return extra


def _int_list(value: Any) -> list[int]:
    if not isinstance(value, list):
        return []
    return [int(item) for item in value if isinstance(item, int) or str(item).lstrip("-").isdigit()]


def _optional_int(value: Any) -> int | None:
    if isinstance(value, int):
        return value
    if isinstance(value, str) and value.strip().lstrip("-").isdigit():
        return int(value)
    return None


def _write_metrics_csv(path: Path, metrics: list[dict[str, Any]]) -> None:
    ensure_dir(path.parent)
    fieldnames = list(metrics[0].keys()) if metrics else ["status"]
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(metrics)


def _write_runtime_summary(
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

    image_matches = sum(1 for metric in metrics if metric["image_token_count_match"] is True)
    boundary_matches = sum(1 for metric in metrics if metric["text_tokens_boundary_stripped_match"] is True)
    exact_text = sum(1 for metric in metrics if metric["text_tokens_exact_match"] is True)
    deltas = [
        int(metric["prefill_delta_vs_processor"])
        for metric in metrics
        if isinstance(metric["prefill_delta_vs_processor"], int)
    ]
    avg_delta = sum(deltas) / len(deltas) if deltas else None

    lines = [
        "# Runtime Parity Summary",
        "",
        f"Generated: {utc_now()}",
        "",
        "## Engines",
        "",
        f"- Reference artifact engine: `{reference_engine}`",
        f"- Candidate artifact engine: `{candidate_engine}`",
        f"- Metrics CSV: `{_display_path(metrics_path, summary_path.parent)}`",
        "",
        "## Status Counts",
        "",
    ]
    for status in sorted(counts):
        lines.append(f"- `{status}`: {counts[status]}")
    if not counts:
        lines.append("- No metrics generated.")

    lines.extend(
        [
            "",
            "## Aggregate Findings",
            "",
            f"- Rows compared: {len(metrics)}",
            f"- Image/media token count matches: {image_matches} / {len(metrics)}",
            f"- Exact non-image text token matches: {exact_text} / {len(metrics)}",
            f"- Matches after stripping candidate newline/EOS boundary tokens: {boundary_matches} / {len(metrics)}",
            f"- Average candidate prefill delta vs SGLang processor input length: {_fmt_float(avg_delta)}",
            "",
            "## Interpretation",
            "",
            "- `runtime_sequence_match` means llama.cpp's text chunks, media-token count, and prefill length match the SGLang processor artifact.",
            "- `candidate_extra_boundary_tokens` means image tokens match and the prompt text matches only after removing candidate-only newline/EOS boundary tokens.",
            "- This check does not run generation; it isolates tokenizer/template/media-token parity before logits and decoding differences.",
        ]
    )

    mismatches = [metric for metric in metrics if metric["status"] != "runtime_sequence_match"][:10]
    if mismatches:
        lines.extend(["", "## Review Rows", "", "| Status | Case | Profile | Prefill Delta | Ref Text Tokens | Candidate Text Tokens |"])
        lines.append("|---|---|---|---:|---|---|")
        for metric in mismatches:
            lines.append(
                "| {status} | {case_id} | {prompt_profile} | {prefill_delta_vs_processor} | `{reference_non_image_tokens}` | `{candidate_text_tokens}` |".format(
                    **metric
                )
            )

    summary_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def _display_path(path: Path, base: Path) -> str:
    try:
        return path.resolve().relative_to(base.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def _fmt_float(value: float | None) -> str:
    return "" if value is None else f"{value:.3f}"
