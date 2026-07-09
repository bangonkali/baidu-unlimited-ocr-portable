from __future__ import annotations


def validate_packaged_ingest_engines(engines: list) -> None:
    """Assert packaged engines are wired; GGUF models may be deferred to download."""
    ppocr = _require_listed_engine(engines, "pp-ocrv6")
    if ppocr.get("availability") == "native_runner_missing":
        raise SystemExit("packaged engine is missing native runner: pp-ocrv6")
    if ppocr.get("available") is not True or ppocr.get("availability") != "ready":
        raise SystemExit(
            "packaged engine is unavailable: pp-ocrv6 "
            f"availability={ppocr.get('availability')} "
            f"runner_status={ppocr.get('runner_status')}"
        )

    paddle = _require_listed_engine(engines, "paddleocr-vl-1.6-gguf")
    if paddle.get("availability") == "native_runner_missing":
        raise SystemExit("packaged engine is missing native runner: paddleocr-vl-1.6-gguf")
    if paddle.get("requires_model") is not True:
        raise SystemExit("paddleocr-vl-1.6-gguf must require a downloadable model")
    if "paddleocr-vl-1-6-gguf" not in (paddle.get("download_model_ids") or []):
        raise SystemExit(
            "paddleocr-vl-1.6-gguf download_model_ids must include paddleocr-vl-1-6-gguf"
        )
    if paddle.get("runner_status") not in {"wired", "ready"}:
        raise SystemExit(
            "packaged engine runner is not wired: paddleocr-vl-1.6-gguf "
            f"runner_status={paddle.get('runner_status')}"
        )
    if paddle.get("availability") not in {"missing_model", "ready"}:
        raise SystemExit(
            "packaged engine has unexpected availability: paddleocr-vl-1.6-gguf "
            f"availability={paddle.get('availability')} "
            f"runner_status={paddle.get('runner_status')}"
        )


def _require_listed_engine(engines: list, engine_id: str) -> dict:
    engine = next((item for item in engines if item.get("engine_id") == engine_id), None)
    if engine is None:
        raise SystemExit(f"packaged engine was not listed by /api/ingest/engines: {engine_id}")
    return engine
