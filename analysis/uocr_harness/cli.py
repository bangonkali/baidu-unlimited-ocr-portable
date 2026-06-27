from __future__ import annotations

import argparse
from pathlib import Path

from .artifacts import compare_debug_artifacts, compare_generation_artifacts
from .compare import compare_results
from .engines import run_llamacpp, run_llamacpp_server, run_sglang
from .manifest import prepare_dataset
from .preprocess import inspect_manifest_preprocessing
from .profiles import parse_profile_names
from .runtime_parity import compare_runtime_parity, inspect_sglang_processor
from .util import (
    DEFAULT_DATASET,
    DEFAULT_MANIFEST,
    DEFAULT_RESULTS_DIR,
    DEFAULT_SUMMARIES_DIR,
    REPO_ROOT,
    llama_executable,
    thirdparty_file,
)


def main() -> None:
    parser = argparse.ArgumentParser(description="Unlimited-OCR portable validation harness")
    sub = parser.add_subparsers(dest="command", required=True)

    prepare = sub.add_parser("prepare", help="Normalize dataset files and build manifest")
    _add_common_paths(prepare)
    prepare.add_argument("--pdf-dpi", type=int, default=300)
    prepare.add_argument("--force", action="store_true")

    inspect = sub.add_parser("inspect-preprocessing", help="Write SGLang/llama.cpp image-token diagnostics")
    _add_common_paths(inspect)
    inspect.add_argument(
        "--output",
        type=Path,
        default=DEFAULT_RESULTS_DIR / "inspection/preprocessing.jsonl",
    )

    inspect_processor = sub.add_parser(
        "inspect-sglang-processor",
        help="Write SGLang processor tokenizer/template/media-token diagnostics",
    )
    _add_common_paths(inspect_processor)
    _add_run_filters(inspect_processor)
    inspect_processor.add_argument("--model-dir", type=Path, default=REPO_ROOT / "unlimited-ocr")
    inspect_processor.add_argument(
        "--image-mode",
        choices=("tiny", "small", "base", "large", "gundam"),
        default="gundam",
    )
    inspect_processor.add_argument(
        "--media-placement",
        choices=("separate", "prefix-tight", "prefix-newline", "suffix-newline"),
        default="separate",
    )
    inspect_processor.add_argument(
        "--output",
        type=Path,
        default=DEFAULT_RESULTS_DIR / "inspection/sglang_processor.jsonl",
    )

    llama = sub.add_parser("run-llamacpp", help="Run llama.cpp candidate over manifest cases")
    _add_common_paths(llama)
    _add_run_filters(llama)
    llama.add_argument("--binary", type=Path, default=llama_executable("llama-mtmd-cli"))
    llama.add_argument("--model", type=Path, default=thirdparty_file("uocr-gguf", "Unlimited-OCR-Q4_K_M.gguf"))
    llama.add_argument("--mmproj", type=Path, default=thirdparty_file("uocr-gguf", "mmproj-Unlimited-OCR-F16.gguf"))
    llama.add_argument("--ctx-size", type=int, default=32768)
    llama.add_argument("--max-tokens", type=int, default=8192)
    llama.add_argument("--timeout-s", type=int, default=1800)
    llama.add_argument("--candidate-engine", default="llamacpp-q4_k_m")
    llama.add_argument("--quantization", default=None)
    llama.add_argument("--repeat-penalty", type=float, default=1.0)
    llama.add_argument("--image-min-tokens", type=int, default=None)
    llama.add_argument("--image-max-tokens", type=int, default=None)
    llama.add_argument(
        "--media-placement",
        choices=("auto", "prefix-tight", "prefix-newline", "suffix-newline"),
        default="auto",
    )
    llama.add_argument("--deepseek-ocr-mode", choices=("native", "gundam"), default="native")
    llama.add_argument("--deepseek-ocr-force-prompt-eos", action="store_true")
    llama.add_argument("--deepseek-ocr-no-repeat-ngram", action="store_true")
    llama.add_argument("--deepseek-ocr-ngram-size", type=int, default=30)
    llama.add_argument("--deepseek-ocr-ngram-window", type=int, default=90)
    llama.add_argument("--deepseek-ocr-ngram-whitelist", default="128821,128822")
    llama.add_argument("--deepseek-ocr-prefill-aware-swa", action="store_true")
    llama.add_argument(
        "--deepseek-ocr-legacy-kv-prune",
        action="store_true",
        help="Diagnostic only: enable the old CLI KV-pruning SWA experiment on top of core R-SWA",
    )
    llama.add_argument("--deepseek-ocr-decode-window", type=int, default=128)
    llama.add_argument("--deepseek-ocr-no-image-end", action="store_true")
    llama.add_argument("--deepseek-ocr-min-new-tokens", type=int, default=0)
    llama.add_argument("--debug-artifacts", action="store_true", help="Write native llama.cpp parity artifacts")
    llama.add_argument("--debug-top-k", type=int, default=8, help="Top raw logits to store per generation step")
    llama.add_argument(
        "--debug-output-embeddings",
        action="store_true",
        help="Include llama.cpp output embedding summaries in native parity artifacts",
    )

    llama_server = sub.add_parser("run-llamacpp-server", help="Run llama-server candidate over manifest cases")
    _add_common_paths(llama_server)
    _add_run_filters(llama_server)
    llama_server.add_argument("--binary", type=Path, default=llama_executable("llama-server"))
    llama_server.add_argument("--model", type=Path, default=thirdparty_file("uocr-gguf", "Unlimited-OCR-Q4_K_M.gguf"))
    llama_server.add_argument("--mmproj", type=Path, default=thirdparty_file("uocr-gguf", "mmproj-Unlimited-OCR-F16.gguf"))
    llama_server.add_argument("--ctx-size", type=int, default=32768)
    llama_server.add_argument("--max-tokens", type=int, default=8192)
    llama_server.add_argument("--timeout-s", type=int, default=1800)
    llama_server.add_argument("--candidate-engine", default="llamacpp-server-q4_k_m")
    llama_server.add_argument("--quantization", default=None)
    llama_server.add_argument("--repeat-penalty", type=float, default=1.0)
    llama_server.add_argument("--image-min-tokens", type=int, default=None)
    llama_server.add_argument("--image-max-tokens", type=int, default=None)
    llama_server.add_argument(
        "--media-placement",
        choices=("auto", "prefix-tight", "prefix-newline", "suffix-newline"),
        default="prefix-newline",
    )
    llama_server.add_argument("--deepseek-ocr-mode", choices=("native", "gundam"), default="native")
    llama_server.add_argument("--server-url", default="http://127.0.0.1:18080")
    llama_server.add_argument("--served-model", default="Unlimited-OCR")
    llama_server.add_argument("--server-log", type=Path, default=DEFAULT_RESULTS_DIR / "logs/llamacpp_server.log")

    sglang = sub.add_parser("run-sglang", help="Run SGLang BF16 reference over manifest cases")
    _add_common_paths(sglang)
    _add_run_filters(sglang)
    sglang.add_argument("--server-url", default="http://127.0.0.1:10000")
    sglang.add_argument("--served-model", default="Unlimited-OCR")
    sglang.add_argument("--image-mode", choices=("gundam", "base"), default="gundam")
    sglang.add_argument("--max-tokens", type=int, default=8192)
    sglang.add_argument("--timeout-s", type=int, default=1800)
    sglang.add_argument("--start-server", action="store_true")
    sglang.add_argument("--sglang-python", type=Path, default=REPO_ROOT / ".venv/bin/python")
    sglang.add_argument("--model-dir", type=Path, default=REPO_ROOT / "unlimited-ocr")
    sglang.add_argument("--server-log", type=Path, default=DEFAULT_RESULTS_DIR / "logs/sglang_server.log")
    sglang.add_argument("--attention-backend", default="flashinfer")
    sglang.add_argument(
        "--media-placement",
        choices=("separate", "prefix-tight", "prefix-newline", "suffix-newline"),
        default="separate",
    )
    sglang.add_argument("--debug-artifacts", action="store_true", help="Write SGLang chat logprob artifacts")
    sglang.add_argument("--debug-top-logprobs", type=int, default=8)
    sglang.add_argument(
        "--debug-native-artifacts",
        action="store_true",
        help="Write SGLang /generate artifacts with input logprobs",
    )
    sglang.add_argument(
        "--debug-return-hidden-states",
        action="store_true",
        help="Request hidden states in native SGLang debug artifacts",
    )
    sglang.add_argument(
        "--enable-return-hidden-states",
        action="store_true",
        help="Start SGLang with --enable-return-hidden-states",
    )

    compare = sub.add_parser("compare", help="Compare reference and candidate outputs")
    _add_common_paths(compare)
    compare.add_argument("--profiles", default=None)
    compare.add_argument("--limit", type=int, default=None, help="Compare only first N manifest cases")
    compare.add_argument("--case-id", default=None, help="Compare one or more comma-separated case ids")
    compare.add_argument("--summary", type=Path, default=DEFAULT_SUMMARIES_DIR / "SUMMARY.md")
    compare.add_argument("--reference-engine", default="sglang")
    compare.add_argument("--candidate-engine", default="llamacpp-q4_k_m")

    compare_artifacts = sub.add_parser("compare-artifacts", help="Compare SGLang and llama.cpp debug artifacts")
    _add_common_paths(compare_artifacts)
    compare_artifacts.add_argument("--profiles", default=None)
    compare_artifacts.add_argument("--limit", type=int, default=None)
    compare_artifacts.add_argument("--case-id", default=None)
    compare_artifacts.add_argument("--summary", type=Path, default=DEFAULT_SUMMARIES_DIR / "SUMMARY-parity-artifacts.md")
    compare_artifacts.add_argument("--reference-engine", default="sglang")
    compare_artifacts.add_argument("--candidate-engine", default="llamacpp-q4_k_m")

    compare_generation = sub.add_parser(
        "compare-generation-artifacts",
        help="Compare SGLang and llama.cpp generated token IDs/top-k step by step",
    )
    _add_common_paths(compare_generation)
    compare_generation.add_argument("--profiles", default=None)
    compare_generation.add_argument("--limit", type=int, default=None)
    compare_generation.add_argument("--case-id", default=None)
    compare_generation.add_argument("--summary", type=Path, default=DEFAULT_SUMMARIES_DIR / "SUMMARY-generation-artifacts.md")
    compare_generation.add_argument("--reference-engine", default="sglang-native")
    compare_generation.add_argument("--candidate-engine", default="llamacpp-q4_k_m")

    compare_runtime = sub.add_parser(
        "compare-runtime-parity",
        help="Compare SGLang processor artifacts against native llama.cpp parity artifacts",
    )
    _add_common_paths(compare_runtime)
    compare_runtime.add_argument("--profiles", default=None)
    compare_runtime.add_argument("--limit", type=int, default=None)
    compare_runtime.add_argument("--case-id", default=None)
    compare_runtime.add_argument("--summary", type=Path, default=DEFAULT_SUMMARIES_DIR / "SUMMARY-runtime-parity.md")
    compare_runtime.add_argument("--reference-engine", default="sglang-processor")
    compare_runtime.add_argument("--candidate-engine", default="llamacpp-q4_k_m")

    args = parser.parse_args()

    if args.command == "prepare":
        rows = prepare_dataset(
            dataset_dir=args.dataset,
            results_dir=args.results,
            manifest_path=args.manifest,
            pdf_dpi=args.pdf_dpi,
            force=args.force,
        )
        print(f"Prepared {len(rows)} cases -> {args.manifest}")
    elif args.command == "inspect-preprocessing":
        rows = inspect_manifest_preprocessing(manifest_path=args.manifest, output_path=args.output)
        print(f"Wrote {len(rows)} preprocessing inspection rows -> {args.output}")
    elif args.command == "inspect-sglang-processor":
        rows = inspect_sglang_processor(
            manifest_path=args.manifest,
            results_dir=args.results,
            profile_names=parse_profile_names(args.profiles),
            model_dir=args.model_dir,
            image_mode=args.image_mode,
            media_placement=args.media_placement,
            output_path=args.output,
            limit=args.limit,
            case_id=args.case_id,
            force=args.force,
        )
        print(f"Wrote {len(rows)} SGLang processor inspection rows -> {args.output}")
    elif args.command == "run-llamacpp":
        paths = run_llamacpp(
            manifest_path=args.manifest,
            results_dir=args.results,
            profile_names=parse_profile_names(args.profiles),
            binary=args.binary,
            model=args.model,
            mmproj=args.mmproj,
            limit=args.limit,
            case_id=args.case_id,
            force=args.force,
            ctx_size=args.ctx_size,
            max_tokens=args.max_tokens,
            timeout_s=args.timeout_s,
            candidate_engine=args.candidate_engine,
            quantization=args.quantization,
            repeat_penalty=args.repeat_penalty,
            image_min_tokens=args.image_min_tokens,
            image_max_tokens=args.image_max_tokens,
            media_placement=args.media_placement,
            deepseek_ocr_mode=args.deepseek_ocr_mode,
            deepseek_ocr_force_prompt_eos=args.deepseek_ocr_force_prompt_eos,
            deepseek_ocr_no_repeat_ngram=args.deepseek_ocr_no_repeat_ngram,
            deepseek_ocr_ngram_size=args.deepseek_ocr_ngram_size,
            deepseek_ocr_ngram_window=args.deepseek_ocr_ngram_window,
            deepseek_ocr_ngram_whitelist=_parse_token_ids(args.deepseek_ocr_ngram_whitelist),
            deepseek_ocr_prefill_aware_swa=args.deepseek_ocr_prefill_aware_swa,
            deepseek_ocr_legacy_kv_prune=args.deepseek_ocr_legacy_kv_prune,
            deepseek_ocr_decode_window=args.deepseek_ocr_decode_window,
            deepseek_ocr_no_image_end=args.deepseek_ocr_no_image_end,
            deepseek_ocr_min_new_tokens=args.deepseek_ocr_min_new_tokens,
            debug_artifacts=args.debug_artifacts,
            debug_top_k=args.debug_top_k,
            debug_output_embeddings=args.debug_output_embeddings,
        )
        print(f"Wrote {len(paths)} llama.cpp result files")
    elif args.command == "run-llamacpp-server":
        paths = run_llamacpp_server(
            manifest_path=args.manifest,
            results_dir=args.results,
            profile_names=parse_profile_names(args.profiles),
            binary=args.binary,
            model=args.model,
            mmproj=args.mmproj,
            limit=args.limit,
            case_id=args.case_id,
            force=args.force,
            ctx_size=args.ctx_size,
            max_tokens=args.max_tokens,
            timeout_s=args.timeout_s,
            candidate_engine=args.candidate_engine,
            quantization=args.quantization,
            repeat_penalty=args.repeat_penalty,
            image_min_tokens=args.image_min_tokens,
            image_max_tokens=args.image_max_tokens,
            media_placement=args.media_placement,
            deepseek_ocr_mode=args.deepseek_ocr_mode,
            server_url=args.server_url,
            served_model=args.served_model,
            server_log=args.server_log,
        )
        print(f"Wrote {len(paths)} llama-server result files")
    elif args.command == "run-sglang":
        paths = run_sglang(
            manifest_path=args.manifest,
            results_dir=args.results,
            profile_names=parse_profile_names(args.profiles),
            server_url=args.server_url,
            served_model=args.served_model,
            image_mode=args.image_mode,
            limit=args.limit,
            case_id=args.case_id,
            force=args.force,
            max_tokens=args.max_tokens,
            timeout_s=args.timeout_s,
            start_server=args.start_server,
            sglang_python=args.sglang_python,
            model_dir=args.model_dir,
            server_log=args.server_log,
            attention_backend=args.attention_backend,
            media_placement=args.media_placement,
            debug_artifacts=args.debug_artifacts,
            debug_top_logprobs=args.debug_top_logprobs,
            debug_native_artifacts=args.debug_native_artifacts,
            debug_return_hidden_states=args.debug_return_hidden_states,
            enable_return_hidden_states=args.enable_return_hidden_states,
        )
        print(f"Wrote {len(paths)} SGLang result files")
    elif args.command == "compare":
        metrics = compare_results(
            manifest_path=args.manifest,
            results_dir=args.results,
            profile_names=parse_profile_names(args.profiles),
            reference_engine=args.reference_engine,
            candidate_engine=args.candidate_engine,
            summary_path=args.summary,
            limit=args.limit,
            case_id=args.case_id,
        )
        print(f"Wrote {len(metrics)} comparison rows -> {args.summary}")
    elif args.command == "compare-artifacts":
        rows = compare_debug_artifacts(
            manifest_path=args.manifest,
            results_dir=args.results,
            profile_names=parse_profile_names(args.profiles),
            reference_engine=args.reference_engine,
            candidate_engine=args.candidate_engine,
            summary_path=args.summary,
            limit=args.limit,
            case_id=args.case_id,
        )
        print(f"Wrote {len(rows)} artifact comparison rows -> {args.summary}")
    elif args.command == "compare-generation-artifacts":
        rows = compare_generation_artifacts(
            manifest_path=args.manifest,
            results_dir=args.results,
            profile_names=parse_profile_names(args.profiles),
            reference_engine=args.reference_engine,
            candidate_engine=args.candidate_engine,
            summary_path=args.summary,
            limit=args.limit,
            case_id=args.case_id,
        )
        print(f"Wrote {len(rows)} generation artifact comparison rows -> {args.summary}")
    elif args.command == "compare-runtime-parity":
        rows = compare_runtime_parity(
            manifest_path=args.manifest,
            results_dir=args.results,
            profile_names=parse_profile_names(args.profiles),
            reference_engine=args.reference_engine,
            candidate_engine=args.candidate_engine,
            summary_path=args.summary,
            limit=args.limit,
            case_id=args.case_id,
        )
        print(f"Wrote {len(rows)} runtime parity rows -> {args.summary}")


def _add_common_paths(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--dataset", type=Path, default=DEFAULT_DATASET)
    parser.add_argument("--results", type=Path, default=DEFAULT_RESULTS_DIR)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)


def _add_run_filters(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--profiles", default=None, help="Comma-separated profile names; default is all")
    parser.add_argument("--limit", type=int, default=None, help="Run only first N manifest cases")
    parser.add_argument("--case-id", default=None, help="Run only one case id")
    parser.add_argument("--force", action="store_true", help="Overwrite existing result files")


def _parse_token_ids(value: str) -> list[int]:
    if not value:
        return []
    return [int(item.strip()) for item in value.split(",") if item.strip()]


if __name__ == "__main__":
    main()
