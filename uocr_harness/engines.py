from __future__ import annotations

import base64
import hashlib
import json
import subprocess
import time
from pathlib import Path
from typing import Any

import requests

from .preprocess import inspect_image_preprocessing
from .profiles import PROMPT_PROFILES
from .util import (
    command_text,
    ensure_dir,
    env_with_no_proxy,
    first_gpu_used_mb,
    gpu_snapshot,
    read_jsonl,
    resolve_path,
    utc_now,
    write_json,
)


def run_llamacpp(
    *,
    manifest_path: Path,
    results_dir: Path,
    profile_names: list[str],
    binary: Path,
    model: Path,
    mmproj: Path,
    limit: int | None,
    case_id: str | None,
    force: bool,
    ctx_size: int,
    max_tokens: int,
    timeout_s: int,
    candidate_engine: str,
    quantization: str | None,
    repeat_penalty: float,
    image_min_tokens: int | None,
    image_max_tokens: int | None,
    media_placement: str,
    deepseek_ocr_mode: str,
    deepseek_ocr_force_prompt_eos: bool,
    deepseek_ocr_no_repeat_ngram: bool,
    deepseek_ocr_ngram_size: int,
    deepseek_ocr_ngram_window: int,
    deepseek_ocr_ngram_whitelist: list[int],
    deepseek_ocr_prefill_aware_swa: bool,
    deepseek_ocr_decode_window: int,
    deepseek_ocr_no_image_end: bool,
    deepseek_ocr_min_new_tokens: int,
    debug_artifacts: bool,
    debug_top_k: int,
    debug_output_embeddings: bool,
) -> list[Path]:
    rows = _filter_rows(read_jsonl(manifest_path), limit=limit, case_id=case_id)
    quantization = quantization or _quantization_from_model(model)
    written: list[Path] = []
    for row in rows:
        image_path = resolve_path(row["prepared_rel"])
        preprocessing = inspect_image_preprocessing(image_path)
        for profile_name in profile_names:
            profile = PROMPT_PROFILES[profile_name]
            prompt = _llamacpp_cli_prompt(profile.prompt, media_placement)
            out_dir = ensure_dir(results_dir / "candidate" / candidate_engine / row["case_id"])
            result_path = out_dir / f"{profile_name}.json"
            if result_path.exists() and not force:
                written.append(result_path)
                continue

            stdout_path = out_dir / f"{profile_name}.stdout.txt"
            stderr_path = out_dir / f"{profile_name}.stderr.txt"
            debug_artifact_path = _debug_artifact_path(
                results_dir=results_dir,
                side="candidate",
                engine=candidate_engine,
                case_id=row["case_id"],
                profile_name=profile_name,
                suffix="llamacpp",
            ) if debug_artifacts else None
            argv = [
                str(binary),
                "-m",
                str(model),
                "--mmproj",
                str(mmproj),
                "--image",
                str(image_path),
                "-p",
                prompt,
                "--chat-template",
                "deepseek-ocr",
                "--temp",
                "0",
                "--top-k",
                "1",
                "-n",
                str(max_tokens),
                "-c",
                str(ctx_size),
                "-ngl",
                "all",
                "--log-verbosity",
                "2",
            ]
            if repeat_penalty != 1.0:
                argv.extend(["--repeat-penalty", str(repeat_penalty)])
            if image_min_tokens is not None:
                argv.extend(["--image-min-tokens", str(image_min_tokens)])
            if image_max_tokens is not None:
                argv.extend(["--image-max-tokens", str(image_max_tokens)])
            if deepseek_ocr_force_prompt_eos:
                argv.extend(["--override-kv", "tokenizer.ggml.add_eos_token=bool:true"])
            before = gpu_snapshot()
            started = utc_now()
            t0 = time.monotonic()
            error = None
            try:
                env = env_with_no_proxy()
                _apply_deepseek_ocr_mode(env, deepseek_ocr_mode)
                _apply_deepseek_ocr_no_repeat_ngram(
                    env,
                    enabled=deepseek_ocr_no_repeat_ngram,
                    ngram_size=deepseek_ocr_ngram_size,
                    ngram_window=deepseek_ocr_ngram_window,
                    ngram_whitelist=deepseek_ocr_ngram_whitelist,
                )
                _apply_deepseek_ocr_prefill_aware_swa(
                    env,
                    enabled=deepseek_ocr_prefill_aware_swa,
                    decode_window=deepseek_ocr_decode_window,
                )
                _apply_deepseek_ocr_no_image_end(env, enabled=deepseek_ocr_no_image_end)
                _apply_deepseek_ocr_min_new_tokens(env, n_tokens=deepseek_ocr_min_new_tokens)
                if debug_artifact_path is not None:
                    ensure_dir(debug_artifact_path.parent)
                    env["LLAMA_UOCR_PARITY_DUMP"] = str(debug_artifact_path)
                    env["LLAMA_UOCR_PARITY_TOPK"] = str(debug_top_k)
                    if debug_output_embeddings:
                        env["LLAMA_UOCR_PARITY_OUTPUT_EMBEDDINGS"] = "1"
                proc = subprocess.run(
                    argv,
                    capture_output=True,
                    timeout=timeout_s,
                    env=env,
                )
                exit_code = proc.returncode
                stdout = _decode_process_output(proc.stdout)
                stderr = _decode_process_output(proc.stderr)
            except subprocess.TimeoutExpired as exc:
                exit_code = 124
                stdout = _decode_process_output(exc.stdout)
                stderr = _decode_process_output(exc.stderr)
                error = f"timeout after {timeout_s}s"
            elapsed_ms = int((time.monotonic() - t0) * 1000)
            ended = utc_now()
            after = gpu_snapshot()
            stdout_path.write_text(stdout, encoding="utf-8", errors="replace")
            stderr_path.write_text(stderr, encoding="utf-8", errors="replace")

            write_json(
                result_path,
                _result_record(
                    engine="llamacpp",
                    engine_version=_llamacpp_version(binary),
                    model=str(model),
                    quantization=quantization,
                    row=row,
                    profile_name=profile_name,
                    prompt=prompt,
                    output_text=stdout.strip(),
                    command=command_text(argv),
                    started=started,
                    ended=ended,
                    elapsed_ms=elapsed_ms,
                    exit_code=exit_code,
                    error=error,
                    gpu_before=before,
                    gpu_after=after,
                    raw_stdout=stdout_path,
                    raw_stderr=stderr_path,
                    extra={
                        "candidate_engine": candidate_engine,
                        "repeat_penalty": repeat_penalty,
                        "image_min_tokens": image_min_tokens,
                        "image_max_tokens": image_max_tokens,
                        "media_placement": media_placement,
                        "deepseek_ocr_mode": deepseek_ocr_mode,
                        "deepseek_ocr_force_prompt_eos": deepseek_ocr_force_prompt_eos,
                        "deepseek_ocr_no_repeat_ngram": deepseek_ocr_no_repeat_ngram,
                        "deepseek_ocr_ngram_size": deepseek_ocr_ngram_size,
                        "deepseek_ocr_ngram_window": deepseek_ocr_ngram_window,
                        "deepseek_ocr_ngram_whitelist": deepseek_ocr_ngram_whitelist,
                        "deepseek_ocr_prefill_aware_swa": deepseek_ocr_prefill_aware_swa,
                        "deepseek_ocr_decode_window": deepseek_ocr_decode_window,
                        "deepseek_ocr_no_image_end": deepseek_ocr_no_image_end,
                        "deepseek_ocr_min_new_tokens": deepseek_ocr_min_new_tokens,
                        "debug_artifact_path": str(debug_artifact_path) if debug_artifact_path else None,
                        "debug_output_embeddings": debug_output_embeddings,
                        "preprocessing": preprocessing,
                    },
                ),
            )
            written.append(result_path)
    return written


def run_llamacpp_server(
    *,
    manifest_path: Path,
    results_dir: Path,
    profile_names: list[str],
    binary: Path,
    model: Path,
    mmproj: Path,
    limit: int | None,
    case_id: str | None,
    force: bool,
    ctx_size: int,
    max_tokens: int,
    timeout_s: int,
    candidate_engine: str,
    quantization: str | None,
    repeat_penalty: float,
    image_min_tokens: int | None,
    image_max_tokens: int | None,
    media_placement: str,
    deepseek_ocr_mode: str,
    server_url: str,
    served_model: str,
    server_log: Path,
) -> list[Path]:
    server_proc = None
    quantization = quantization or _quantization_from_model(model)
    if not _server_ready(server_url):
        server_proc = _start_llamacpp_server(
            binary=binary,
            model=model,
            mmproj=mmproj,
            ctx_size=ctx_size,
            server_url=server_url,
            served_model=served_model,
            server_log=server_log,
            image_min_tokens=image_min_tokens,
            image_max_tokens=image_max_tokens,
            deepseek_ocr_mode=deepseek_ocr_mode,
        )

    if not _server_ready(server_url):
        raise SystemExit(f"llama-server is not reachable at {server_url}.")

    media_marker = _server_media_marker(server_url)
    rows = _filter_rows(read_jsonl(manifest_path), limit=limit, case_id=case_id)
    written: list[Path] = []
    session = requests.Session()
    session.trust_env = False
    try:
        for row in rows:
            image_path = resolve_path(row["prepared_rel"])
            preprocessing = inspect_image_preprocessing(image_path)
            for profile_name in profile_names:
                profile = PROMPT_PROFILES[profile_name]
                prompt = _llamacpp_server_prompt(profile.prompt, media_marker, media_placement)
                out_dir = ensure_dir(results_dir / "candidate" / candidate_engine / row["case_id"])
                result_path = out_dir / f"{profile_name}.json"
                if result_path.exists() and not force:
                    written.append(result_path)
                    continue

                stdout_path = out_dir / f"{profile_name}.stdout.txt"
                stderr_path = out_dir / f"{profile_name}.stderr.txt"
                before = gpu_snapshot()
                started = utc_now()
                t0 = time.monotonic()
                output_text = ""
                error = None
                status_code = None
                try:
                    payload = _llamacpp_server_payload(
                        prompt=profile.prompt,
                        image_path=image_path,
                        max_tokens=max_tokens,
                        repeat_penalty=repeat_penalty,
                        media_marker=media_marker,
                        media_placement=media_placement,
                    )
                    response = session.post(
                        f"{server_url.rstrip('/')}/completion",
                        headers={"Content-Type": "application/json"},
                        data=json.dumps(payload),
                        timeout=timeout_s,
                    )
                    status_code = response.status_code
                    response.raise_for_status()
                    output_text = _completion_content(response.json())
                except Exception as exc:  # noqa: BLE001 - persisted in result JSON
                    error = f"{type(exc).__name__}: {exc}"
                elapsed_ms = int((time.monotonic() - t0) * 1000)
                ended = utc_now()
                after = gpu_snapshot()
                stdout_path.write_text(output_text, encoding="utf-8", errors="replace")
                stderr_path.write_text(error or "", encoding="utf-8", errors="replace")

                write_json(
                    result_path,
                    _result_record(
                        engine="llamacpp-server",
                        engine_version=_llamacpp_version(binary),
                        model=str(model),
                        quantization=quantization,
                        row=row,
                        profile_name=profile_name,
                        prompt=prompt,
                        output_text=output_text,
                        command=f"POST {server_url.rstrip()}/completion",
                        started=started,
                        ended=ended,
                        elapsed_ms=elapsed_ms,
                        exit_code=0 if error is None else 1,
                        error=error,
                        gpu_before=before,
                        gpu_after=after,
                        raw_stdout=stdout_path,
                        raw_stderr=stderr_path,
                        extra={
                            "candidate_engine": candidate_engine,
                            "http_status": status_code,
                            "repeat_penalty": repeat_penalty,
                            "image_min_tokens": image_min_tokens,
                            "image_max_tokens": image_max_tokens,
                            "media_placement": media_placement,
                            "deepseek_ocr_mode": deepseek_ocr_mode,
                            "preprocessing": preprocessing,
                        },
                    ),
                )
                written.append(result_path)
    finally:
        if server_proc is not None:
            server_proc.terminate()
            try:
                server_proc.wait(timeout=30)
            except subprocess.TimeoutExpired:
                server_proc.kill()
                server_proc.wait(timeout=30)
    return written


def run_sglang(
    *,
    manifest_path: Path,
    results_dir: Path,
    profile_names: list[str],
    server_url: str,
    served_model: str,
    image_mode: str,
    limit: int | None,
    case_id: str | None,
    force: bool,
    max_tokens: int,
    timeout_s: int,
    start_server: bool,
    sglang_python: Path,
    model_dir: Path,
    server_log: Path,
    attention_backend: str,
    media_placement: str,
    debug_artifacts: bool,
    debug_top_logprobs: int,
    debug_native_artifacts: bool,
    debug_return_hidden_states: bool,
    enable_return_hidden_states: bool,
) -> list[Path]:
    server_proc = None
    if start_server and not _server_ready(server_url):
        server_proc = _start_sglang_server(
            sglang_python=sglang_python,
            model_dir=model_dir,
            served_model=served_model,
            server_log=server_log,
            server_url=server_url,
            attention_backend=attention_backend,
            enable_return_hidden_states=enable_return_hidden_states or debug_return_hidden_states,
        )

    if not _server_ready(server_url):
        raise SystemExit(
            f"SGLang server is not reachable at {server_url}. "
            "Start it first or pass --start-server."
        )

    rows = _filter_rows(read_jsonl(manifest_path), limit=limit, case_id=case_id)
    written: list[Path] = []
    session = requests.Session()
    session.trust_env = False
    try:
        for row in rows:
            image_path = resolve_path(row["prepared_rel"])
            preprocessing = inspect_image_preprocessing(image_path)
            for profile_name in profile_names:
                profile = PROMPT_PROFILES[profile_name]
                prompt = _sglang_prompt(profile.prompt, media_placement)
                out_dir = ensure_dir(results_dir / "reference" / "sglang" / row["case_id"])
                result_path = out_dir / f"{profile_name}.json"
                if result_path.exists() and not force:
                    written.append(result_path)
                    continue

                stdout_path = out_dir / f"{profile_name}.stdout.txt"
                stderr_path = out_dir / f"{profile_name}.stderr.txt"
                debug_artifact_path = _debug_artifact_path(
                    results_dir=results_dir,
                    side="reference",
                    engine="sglang",
                    case_id=row["case_id"],
                    profile_name=profile_name,
                    suffix="sglang",
                ) if debug_artifacts else None
                native_debug_artifact_path = _debug_artifact_path(
                    results_dir=results_dir,
                    side="reference",
                    engine="sglang-native",
                    case_id=row["case_id"],
                    profile_name=profile_name,
                    suffix="sglang-native",
                ) if debug_native_artifacts else None
                before = gpu_snapshot()
                started = utc_now()
                t0 = time.monotonic()
                error = None
                output_text = ""
                status_code = None
                try:
                    payload = _sglang_payload(
                        model=served_model,
                        prompt=prompt,
                        image_path=image_path,
                        image_mode=image_mode,
                        max_tokens=max_tokens,
                    )
                    with session.post(
                        f"{server_url.rstrip('/')}/v1/chat/completions",
                        headers={"Content-Type": "application/json"},
                        data=json.dumps(payload),
                        stream=True,
                        timeout=timeout_s,
                    ) as response:
                        status_code = response.status_code
                        response.raise_for_status()
                        output_text = _collect_sse(response)
                    if debug_artifact_path is not None:
                        _write_sglang_debug_artifact(
                            session=session,
                            server_url=server_url,
                            served_model=served_model,
                            prompt=prompt,
                            image_path=image_path,
                            image_mode=image_mode,
                            max_tokens=max_tokens,
                            timeout_s=timeout_s,
                            top_logprobs=debug_top_logprobs,
                            artifact_path=debug_artifact_path,
                        )
                    if native_debug_artifact_path is not None:
                        _write_sglang_native_debug_artifact(
                            session=session,
                            server_url=server_url,
                            prompt=prompt,
                            image_path=image_path,
                            image_mode=image_mode,
                            max_tokens=max_tokens,
                            timeout_s=timeout_s,
                            top_logprobs=debug_top_logprobs,
                            return_hidden_states=debug_return_hidden_states,
                            artifact_path=native_debug_artifact_path,
                        )
                except Exception as exc:  # noqa: BLE001 - persisted in result JSON
                    error = f"{type(exc).__name__}: {exc}"
                elapsed_ms = int((time.monotonic() - t0) * 1000)
                ended = utc_now()
                after = gpu_snapshot()
                stdout_path.write_text(output_text, encoding="utf-8", errors="replace")
                stderr_path.write_text(error or "", encoding="utf-8", errors="replace")

                write_json(
                    result_path,
                    _result_record(
                        engine="sglang",
                        engine_version=_sglang_version(),
                        model=served_model,
                        quantization="BF16",
                        row=row,
                        profile_name=profile_name,
                        prompt=prompt,
                        output_text=output_text,
                        command=f"POST {server_url.rstrip()}/v1/chat/completions",
                        started=started,
                        ended=ended,
                        elapsed_ms=elapsed_ms,
                        exit_code=0 if error is None else 1,
                        error=error,
                        gpu_before=before,
                        gpu_after=after,
                        raw_stdout=stdout_path,
                        raw_stderr=stderr_path,
                        extra={
                            "http_status": status_code,
                            "image_mode": image_mode,
                            "media_placement": media_placement,
                            "custom_params": _sglang_ngram_params(),
                            "debug_artifact_path": str(debug_artifact_path) if debug_artifact_path else None,
                            "native_debug_artifact_path": str(native_debug_artifact_path) if native_debug_artifact_path else None,
                            "preprocessing": preprocessing,
                        },
                    ),
                )
                written.append(result_path)
    finally:
        if server_proc is not None:
            server_proc.terminate()
            try:
                server_proc.wait(timeout=30)
            except subprocess.TimeoutExpired:
                server_proc.kill()
                server_proc.wait(timeout=30)
    return written


def _filter_rows(rows: list[dict[str, Any]], *, limit: int | None, case_id: str | None) -> list[dict[str, Any]]:
    if case_id:
        requested = {item.strip() for item in case_id.split(",") if item.strip()}
        rows = [row for row in rows if row["case_id"] in requested]
    if limit is not None:
        rows = rows[:limit]
    return rows


def _result_record(
    *,
    engine: str,
    engine_version: str,
    model: str,
    quantization: str,
    row: dict[str, Any],
    profile_name: str,
    prompt: str,
    output_text: str,
    command: str,
    started: str,
    ended: str,
    elapsed_ms: int,
    exit_code: int,
    error: str | None,
    gpu_before: dict[str, Any],
    gpu_after: dict[str, Any],
    raw_stdout: Path,
    raw_stderr: Path,
    extra: dict[str, Any] | None = None,
) -> dict[str, Any]:
    before_used = first_gpu_used_mb(gpu_before)
    after_used = first_gpu_used_mb(gpu_after)
    record = {
        "engine": engine,
        "engine_version": engine_version,
        "model": model,
        "quantization": quantization,
        "case_id": row["case_id"],
        "source_path": row["source_rel"],
        "prepared_path": row["prepared_rel"],
        "page_index": row.get("page_index"),
        "prompt_profile": profile_name,
        "prompt": prompt,
        "output_text": output_text,
        "raw_stdout_path": str(raw_stdout),
        "raw_stderr_path": str(raw_stderr),
        "command": command,
        "started_at": started,
        "ended_at": ended,
        "elapsed_ms": elapsed_ms,
        "gpu_memory_before_mb": before_used,
        "gpu_memory_after_mb": after_used,
        "gpu_before": gpu_before,
        "gpu_after": gpu_after,
        "exit_code": exit_code,
        "error": error,
    }
    if extra:
        record.update(extra)
    return record


def _decode_process_output(value: bytes | str | None) -> str:
    if value is None:
        return ""
    if isinstance(value, str):
        return value
    return value.decode("utf-8", errors="replace")


def _encode_image(image_path: Path) -> dict[str, Any]:
    mime = "image/png"
    data = base64.b64encode(image_path.read_bytes()).decode("ascii")
    return {"type": "image_url", "image_url": {"url": f"data:{mime};base64,{data}"}}


def _sglang_payload(
    *, model: str, prompt: str, image_path: Path, image_mode: str, max_tokens: int
) -> dict[str, Any]:
    ngram_params = _sglang_ngram_params()
    payload = {
        "model": model,
        "messages": [
            {"role": "user", "content": [{"type": "text", "text": prompt}, _encode_image(image_path)]}
        ],
        "temperature": 0,
        "max_tokens": max_tokens,
        "skip_special_tokens": False,
        "stream": True,
        "images_config": {"image_mode": image_mode},
    }
    processor = _sglang_ngram_processor()
    if processor:
        payload["custom_logit_processor"] = processor
        payload["custom_params"] = ngram_params
    return payload


def _debug_artifact_path(
    *,
    results_dir: Path,
    side: str,
    engine: str,
    case_id: str,
    profile_name: str,
    suffix: str,
) -> Path:
    return results_dir / "artifacts" / side / engine / case_id / f"{profile_name}.{suffix}.json"


def _write_sglang_debug_artifact(
    *,
    session: requests.Session,
    server_url: str,
    served_model: str,
    prompt: str,
    image_path: Path,
    image_mode: str,
    max_tokens: int,
    timeout_s: int,
    top_logprobs: int,
    artifact_path: Path,
) -> None:
    ensure_dir(artifact_path.parent)
    payload = _sglang_debug_payload(
        model=served_model,
        prompt=prompt,
        image_path=image_path,
        image_mode=image_mode,
        max_tokens=max_tokens,
        top_logprobs=top_logprobs,
    )
    started = utc_now()
    t0 = time.monotonic()
    status_code = None
    response_json: dict[str, Any] | None = None
    error = None
    try:
        response = session.post(
            f"{server_url.rstrip('/')}/v1/chat/completions",
            headers={"Content-Type": "application/json"},
            data=json.dumps(payload),
            timeout=timeout_s,
        )
        status_code = response.status_code
        response.raise_for_status()
        response_json = response.json()
    except Exception as exc:  # noqa: BLE001 - persisted in artifact JSON
        error = f"{type(exc).__name__}: {exc}"

    processor = _sglang_ngram_processor()
    custom_params = _sglang_ngram_params()
    write_json(
        artifact_path,
        {
            "schema_version": 1,
            "engine": "sglang",
            "endpoint": "/v1/chat/completions",
            "served_model": served_model,
            "started_at": started,
            "ended_at": utc_now(),
            "elapsed_ms": int((time.monotonic() - t0) * 1000),
            "http_status": status_code,
            "error": error,
            "prompt": prompt,
            "image": _image_digest(image_path),
            "image_mode": image_mode,
            "request": _sanitize_sglang_debug_payload(payload),
            "custom_logit_processor": processor,
            "custom_params": custom_params,
            "response": response_json,
            "notes": [
                "SGLang OpenAI chat logprobs expose token text/top logprobs but not internal image embedding tensors.",
                "Use this artifact to compare API-visible output-token text against llama.cpp artifacts.",
            ],
        },
    )


def _write_sglang_native_debug_artifact(
    *,
    session: requests.Session,
    server_url: str,
    prompt: str,
    image_path: Path,
    image_mode: str,
    max_tokens: int,
    timeout_s: int,
    top_logprobs: int,
    return_hidden_states: bool,
    artifact_path: Path,
) -> None:
    ensure_dir(artifact_path.parent)
    payload = _sglang_native_debug_payload(
        prompt=prompt,
        image_path=image_path,
        image_mode=image_mode,
        max_tokens=max_tokens,
        top_logprobs=top_logprobs,
        return_hidden_states=return_hidden_states,
    )
    started = utc_now()
    t0 = time.monotonic()
    status_code = None
    response_json: dict[str, Any] | None = None
    error = None
    try:
        response = session.post(
            f"{server_url.rstrip('/')}/generate",
            headers={"Content-Type": "application/json"},
            data=json.dumps(payload),
            timeout=timeout_s,
        )
        status_code = response.status_code
        response.raise_for_status()
        response_json = response.json()
    except Exception as exc:  # noqa: BLE001 - persisted in artifact JSON
        error = f"{type(exc).__name__}: {exc}"

    write_json(
        artifact_path,
        {
            "schema_version": 1,
            "engine": "sglang-native",
            "endpoint": "/generate",
            "started_at": started,
            "ended_at": utc_now(),
            "elapsed_ms": int((time.monotonic() - t0) * 1000),
            "http_status": status_code,
            "error": error,
            "prompt": _sglang_native_processor_prompt(prompt),
            "image": _image_digest(image_path),
            "image_mode": image_mode,
            "request": _sanitize_sglang_native_debug_payload(payload),
            "custom_logit_processor": _sglang_ngram_processor(),
            "custom_params": _sglang_ngram_params(),
            "return_hidden_states": return_hidden_states,
            "response": _summarize_native_response(response_json),
            "notes": [
                "Native /generate artifacts can expose input logprobs with logprob_start_len=0.",
                "Hidden states require the SGLang server to be started with --enable-return-hidden-states.",
            ],
        },
    )


def _sglang_debug_payload(
    *,
    model: str,
    prompt: str,
    image_path: Path,
    image_mode: str,
    max_tokens: int,
    top_logprobs: int,
) -> dict[str, Any]:
    ngram_params = _sglang_ngram_params()
    payload: dict[str, Any] = {
        "model": model,
        "messages": [
            {"role": "user", "content": [{"type": "text", "text": prompt}, _encode_image(image_path)]}
        ],
        "temperature": 0,
        "max_tokens": max_tokens,
        "skip_special_tokens": False,
        "stream": False,
        "logprobs": True,
        "top_logprobs": top_logprobs,
        "return_logprob": True,
        "images_config": {"image_mode": image_mode},
        "custom_params": ngram_params,
    }
    processor = _sglang_ngram_processor()
    if processor:
        payload["custom_logit_processor"] = processor
    return payload


def _sglang_native_debug_payload(
    *,
    prompt: str,
    image_path: Path,
    image_mode: str,
    max_tokens: int,
    top_logprobs: int,
    return_hidden_states: bool,
) -> dict[str, Any]:
    ngram_params = _sglang_ngram_params()
    payload: dict[str, Any] = {
        "text": _sglang_native_processor_prompt(prompt),
        "image_data": _image_data_url(image_path),
        "sampling_params": {
            "temperature": 0,
            "max_new_tokens": max_tokens,
            "skip_special_tokens": False,
        },
        "return_logprob": True,
        "logprob_start_len": 0,
        "top_logprobs_num": top_logprobs,
        "return_text_in_logprobs": True,
        "stream": False,
        "images_config": {"image_mode": image_mode},
        "custom_params": ngram_params,
    }
    processor = _sglang_ngram_processor()
    if processor:
        payload["custom_logit_processor"] = processor
    if return_hidden_states:
        payload["return_hidden_states"] = True
    return payload


def _sglang_native_processor_prompt(prompt: str) -> str:
    return prompt if prompt.startswith("<image>") else f"<image>{prompt}"


def _sanitize_sglang_native_debug_payload(payload: dict[str, Any]) -> dict[str, Any]:
    sanitized = dict(payload)
    if "image_data" in sanitized:
        sanitized["image_data"] = "<data-url omitted>"
    return sanitized


def _summarize_native_response(response_json: dict[str, Any] | None) -> dict[str, Any] | None:
    if response_json is None:
        return None
    summary = dict(response_json)
    meta = summary.get("meta_info")
    if isinstance(meta, dict):
        for key in ("hidden_states", "input_hidden_states", "output_hidden_states"):
            if key in meta:
                meta[key] = _summarize_nested_numeric(meta[key])
        summary["meta_info"] = meta
    for key in ("hidden_states", "input_hidden_states", "output_hidden_states"):
        if key in summary:
            summary[key] = _summarize_nested_numeric(summary[key])
    return summary


def _summarize_nested_numeric(value: Any) -> dict[str, Any]:
    shape = _nested_shape(value)
    count = 0
    total = 0.0
    abs_total = 0.0
    min_value: float | None = None
    max_value: float | None = None

    stack = [value]
    while stack:
        item = stack.pop()
        if isinstance(item, list):
            stack.extend(item)
        elif isinstance(item, (int, float)):
            numeric = float(item)
            count += 1
            total += numeric
            abs_total += abs(numeric)
            min_value = numeric if min_value is None else min(min_value, numeric)
            max_value = numeric if max_value is None else max(max_value, numeric)

    return {
        "shape": shape,
        "count": count,
        "sum": total,
        "abs_sum": abs_total,
        "min": min_value,
        "max": max_value,
        "mean": (total / count) if count else None,
        "omitted": True,
    }


def _nested_shape(value: Any) -> list[int]:
    shape = []
    current = value
    while isinstance(current, list):
        shape.append(len(current))
        current = current[0] if current else None
    return shape


def _sanitize_sglang_debug_payload(payload: dict[str, Any]) -> dict[str, Any]:
    sanitized = dict(payload)
    messages = sanitized.get("messages")
    if isinstance(messages, list):
        sanitized["messages"] = [
            {
                **message,
                "content": [
                    item
                    if not (isinstance(item, dict) and item.get("type") == "image_url")
                    else {"type": "image_url", "image_url": {"url": "<data-url omitted>"}}
                    for item in message.get("content", [])
                ],
            }
            for message in messages
            if isinstance(message, dict)
        ]
    return sanitized


def _image_digest(image_path: Path) -> dict[str, Any]:
    data = image_path.read_bytes()
    return {
        "path": str(image_path),
        "bytes": len(data),
        "sha256": hashlib.sha256(data).hexdigest(),
    }


def _image_data_url(image_path: Path) -> str:
    mime = "image/png"
    data = base64.b64encode(image_path.read_bytes()).decode("ascii")
    return f"data:{mime};base64,{data}"


def _collect_sse(response: requests.Response) -> str:
    chunks: list[str] = []
    for line in response.iter_lines(decode_unicode=True):
        if not line or not line.startswith("data:"):
            continue
        data = line[len("data:") :].strip()
        if data == "[DONE]":
            break
        event = json.loads(data)
        delta = event["choices"][0].get("delta", {}).get("content", "")
        if delta:
            chunks.append(delta)
    return "".join(chunks)


def _server_ready(server_url: str) -> bool:
    try:
        response = requests.get(f"{server_url.rstrip('/')}/health", timeout=5)
        return response.status_code == 200
    except requests.RequestException:
        return False


def _start_llamacpp_server(
    *,
    binary: Path,
    model: Path,
    mmproj: Path,
    ctx_size: int,
    server_url: str,
    served_model: str,
    server_log: Path,
    image_min_tokens: int | None,
    image_max_tokens: int | None,
    deepseek_ocr_mode: str,
) -> subprocess.Popen[str]:
    ensure_dir(server_log.parent)
    port = server_url.rstrip("/").rsplit(":", 1)[-1]
    argv = [
        str(binary),
        "-m",
        str(model),
        "--mmproj",
        str(mmproj),
        "--chat-template",
        "deepseek-ocr",
        "-c",
        str(ctx_size),
        "-ngl",
        "all",
        "--alias",
        served_model,
        "--host",
        "127.0.0.1",
        "--port",
        port,
        "--log-verbosity",
        "2",
    ]
    if image_min_tokens is not None:
        argv.extend(["--image-min-tokens", str(image_min_tokens)])
    if image_max_tokens is not None:
        argv.extend(["--image-max-tokens", str(image_max_tokens)])
    log_file = server_log.open("w", encoding="utf-8")
    env = env_with_no_proxy()
    _apply_deepseek_ocr_mode(env, deepseek_ocr_mode)
    proc = subprocess.Popen(argv, stdout=log_file, stderr=subprocess.STDOUT, text=True, env=env)
    deadline = time.monotonic() + 180
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            raise SystemExit(f"llama-server exited early. Check {server_log}")
        if _server_ready(server_url):
            return proc
        time.sleep(2)
    proc.terminate()
    raise SystemExit(f"Timed out waiting for llama-server. Check {server_log}")


def _start_sglang_server(
    *,
    sglang_python: Path,
    model_dir: Path,
    served_model: str,
    server_log: Path,
    server_url: str,
    attention_backend: str,
    enable_return_hidden_states: bool,
) -> subprocess.Popen[str]:
    ensure_dir(server_log.parent)
    port = server_url.rstrip("/").rsplit(":", 1)[-1]
    argv = [
        str(sglang_python),
        "-m",
        "sglang.launch_server",
        "--model",
        str(model_dir),
        "--served-model-name",
        served_model,
        "--attention-backend",
        attention_backend,
        "--page-size",
        "1",
        "--mem-fraction-static",
        "0.8",
        "--context-length",
        "32768",
        "--enable-custom-logit-processor",
        "--disable-overlap-schedule",
        "--skip-server-warmup",
        "--trust-remote-code",
        "--host",
        "0.0.0.0",
        "--port",
        port,
    ]
    if enable_return_hidden_states:
        argv.append("--enable-return-hidden-states")
    log_file = server_log.open("w", encoding="utf-8")
    env = env_with_no_proxy()
    env["PATH"] = f"{sglang_python.parent}:{env.get('PATH', '')}"
    proc = subprocess.Popen(argv, stdout=log_file, stderr=subprocess.STDOUT, text=True, env=env)
    deadline = time.monotonic() + 300
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            raise SystemExit(f"SGLang server exited early. Check {server_log}")
        if _server_ready(server_url):
            return proc
        time.sleep(3)
    proc.terminate()
    raise SystemExit(f"Timed out waiting for SGLang server. Check {server_log}")


def _llamacpp_version(binary: Path) -> str:
    try:
        proc = subprocess.run([str(binary), "--version"], capture_output=True, text=True, timeout=10)
        return (proc.stdout or proc.stderr).strip().splitlines()[0]
    except Exception:  # noqa: BLE001
        return "unknown"


def _llamacpp_server_payload(
    *,
    prompt: str,
    image_path: Path,
    max_tokens: int,
    repeat_penalty: float,
    media_marker: str,
    media_placement: str,
) -> dict[str, Any]:
    return {
        "prompt": {
            "prompt_string": _llamacpp_server_prompt(prompt, media_marker, media_placement),
            "multimodal_data": [_image_base64(image_path)],
        },
        "temperature": 0,
        "top_k": 1,
        "n_predict": max_tokens,
        "repeat_penalty": repeat_penalty,
    }


def _completion_content(data: dict[str, Any]) -> str:
    content = data.get("content")
    if isinstance(content, str):
        return content
    return ""


def _image_base64(image_path: Path) -> str:
    return base64.b64encode(image_path.read_bytes()).decode("ascii")


def _server_media_marker(server_url: str) -> str:
    try:
        response = requests.get(f"{server_url.rstrip('/')}/props", timeout=10)
        response.raise_for_status()
        marker = response.json().get("media_marker")
        if isinstance(marker, str) and marker:
            return marker
    except requests.RequestException:
        pass
    return "<__media__>"


def _llamacpp_cli_prompt(prompt: str, media_placement: str) -> str:
    marker = "<__media__>"
    if media_placement == "auto":
        return prompt
    if media_placement == "prefix-tight":
        return f"{marker}{prompt}"
    if media_placement == "prefix-newline":
        return f"{marker}\n{prompt}"
    if media_placement == "suffix-newline":
        return f"{prompt}\n{marker}"
    raise SystemExit(f"Unknown llama.cpp media placement: {media_placement}")


def _llamacpp_server_prompt(prompt: str, media_marker: str, media_placement: str) -> str:
    if media_placement == "auto":
        return f"{media_marker}\n{prompt}"
    if media_placement == "prefix-tight":
        return f"{media_marker}{prompt}"
    if media_placement == "prefix-newline":
        return f"{media_marker}\n{prompt}"
    if media_placement == "suffix-newline":
        return f"{prompt}\n{media_marker}"
    raise SystemExit(f"Unknown llama-server media placement: {media_placement}")


def _sglang_prompt(prompt: str, media_placement: str) -> str:
    if media_placement == "separate":
        return prompt
    if media_placement == "prefix-tight":
        return f"<image>{prompt}"
    if media_placement == "prefix-newline":
        return f"<image>\n{prompt}"
    if media_placement == "suffix-newline":
        return f"{prompt}\n<image>"
    raise SystemExit(f"Unknown SGLang media placement: {media_placement}")


def _apply_deepseek_ocr_mode(env: dict[str, str], deepseek_ocr_mode: str) -> None:
    if deepseek_ocr_mode == "native":
        env.pop("LLAMA_DEEPSEEK_OCR_GUNDAM", None)
    elif deepseek_ocr_mode == "gundam":
        env["LLAMA_DEEPSEEK_OCR_GUNDAM"] = "1"
    else:
        raise SystemExit(f"Unknown DeepSeek-OCR preprocessing mode: {deepseek_ocr_mode}")


def _apply_deepseek_ocr_no_repeat_ngram(
    env: dict[str, str],
    *,
    enabled: bool,
    ngram_size: int,
    ngram_window: int,
    ngram_whitelist: list[int],
) -> None:
    if not enabled:
        env.pop("LLAMA_DEEPSEEK_OCR_NO_REPEAT_NGRAM", None)
        env.pop("LLAMA_DEEPSEEK_OCR_NGRAM_SIZE", None)
        env.pop("LLAMA_DEEPSEEK_OCR_NGRAM_WINDOW", None)
        env.pop("LLAMA_DEEPSEEK_OCR_NGRAM_WHITELIST", None)
        return
    env["LLAMA_DEEPSEEK_OCR_NO_REPEAT_NGRAM"] = "1"
    env["LLAMA_DEEPSEEK_OCR_NGRAM_SIZE"] = str(ngram_size)
    env["LLAMA_DEEPSEEK_OCR_NGRAM_WINDOW"] = str(ngram_window)
    env["LLAMA_DEEPSEEK_OCR_NGRAM_WHITELIST"] = ",".join(str(token_id) for token_id in ngram_whitelist)


def _apply_deepseek_ocr_prefill_aware_swa(
    env: dict[str, str],
    *,
    enabled: bool,
    decode_window: int,
) -> None:
    if not enabled:
        env.pop("LLAMA_DEEPSEEK_OCR_PREFILL_AWARE_SWA", None)
        env.pop("LLAMA_DEEPSEEK_OCR_DECODE_WINDOW", None)
        return
    env["LLAMA_DEEPSEEK_OCR_PREFILL_AWARE_SWA"] = "1"
    env["LLAMA_DEEPSEEK_OCR_DECODE_WINDOW"] = str(decode_window)


def _apply_deepseek_ocr_no_image_end(env: dict[str, str], *, enabled: bool) -> None:
    if not enabled:
        env.pop("LLAMA_DEEPSEEK_OCR_NO_IMAGE_END", None)
        return
    env["LLAMA_DEEPSEEK_OCR_NO_IMAGE_END"] = "1"


def _apply_deepseek_ocr_min_new_tokens(env: dict[str, str], *, n_tokens: int) -> None:
    if n_tokens <= 0:
        env.pop("LLAMA_DEEPSEEK_OCR_MIN_NEW_TOKENS", None)
        return
    env["LLAMA_DEEPSEEK_OCR_MIN_NEW_TOKENS"] = str(n_tokens)


def _quantization_from_model(model: Path) -> str:
    stem = model.stem
    if stem.startswith("Unlimited-OCR-"):
        return stem.removeprefix("Unlimited-OCR-")
    return stem


def _sglang_version() -> str:
    try:
        import sglang

        return str(getattr(sglang, "__version__", "unknown"))
    except Exception:  # noqa: BLE001
        return "unknown"


def _sglang_ngram_processor() -> str | None:
    try:
        from sglang.srt.sampling.custom_logit_processor import DeepseekOCRNoRepeatNGramLogitProcessor

        return DeepseekOCRNoRepeatNGramLogitProcessor.to_str()
    except Exception:  # noqa: BLE001
        return None


def _sglang_ngram_params() -> dict[str, Any]:
    try:
        from sglang.srt.configs.deepseek_ocr import get_default_ngram_custom_params

        return dict(get_default_ngram_custom_params())
    except Exception:  # noqa: BLE001
        return {
            "ngram_size": 30,
            "window_size": 90,
            "whitelist_token_ids": [128821, 128822],
        }
