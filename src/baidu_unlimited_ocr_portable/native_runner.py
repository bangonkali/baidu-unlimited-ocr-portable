from __future__ import annotations

import codecs
import atexit
import base64
import ctypes
import json
import os
import queue
import re
import shlex
import socket
import subprocess
import tempfile
import threading
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Iterator

import requests

from .config import (
    CANDIDATE_PROFILES,
    DEFAULT_MODEL,
    DEFAULT_MMPROJ,
    REPO_ROOT,
    CandidateProfile,
    default_ffi_library,
    default_llama_binary,
    default_llama_server_binary,
    resolve_repo_path,
)


ANSI_RE = re.compile(r"\x1b\[[0-9;?]*[ -/]*[@-~]")
COUNTING_RUN_NUMBER_RE = re.compile(r"(?<![A-Za-z0-9_.-])\d{2,6}(?![A-Za-z0-9_.-])")
HISTOGRAM_RUN_ENTRY_RE = re.compile(r"(?<![A-Za-z0-9_.-])(\d{1,6})\s*:\s*1(?![A-Za-z0-9_.-])")
REPEATED_NUMBER_RUN_RE = re.compile(r"(?<![A-Za-z0-9_.-])(\d{1,6})\.?(?![A-Za-z0-9_.-])")
COUNTING_RUN_SEPARATOR_RE = re.compile(r"^[\s,;:/|()\[\]{}<>-]*$")
ORPHAN_DET_CLOSE_PREFIX_RE = re.compile(
    r"^\s*,?\s*\[\s*\d+(?:\s*,\s*\d+){3}\s*\]\s*<\|/det\|>\s*"
)
REPEATED_NUMBER_PREFIX_RE = re.compile(r"^\s*(?:[,;:\s]*\d{1,6}\.\s*){6,}")
REPEATED_COMMA_PREFIX_RE = re.compile(r"^\s*(?:,\s*){6,}")
LEADING_LOW_VALUE_DET_PREFIX_RE = re.compile(
    r"^\s*,?\s*(?:and\s+)?(?:the\s+)?"
    r"(?:image|picture|page)\s+(?:is\s+)?"
    r"(?:too\s+blurry|contains\s+no\s+text|does\s+not\s+contain\s+text|has\s+no\s+text)"
    r"[^<]{0,180}(?=<\|det\|>)",
    re.IGNORECASE | re.DOTALL,
)
LEADING_CONJUNCTION_DET_PREFIX_RE = re.compile(r"^\s*,\s*(?:and\s+)?(?:the\s+)?(?=<\|det\|>)", re.IGNORECASE)
LEADING_SHORT_NUMBER_DET_PREFIX_RE = re.compile(r"^\s*\d{1,2}\s+(?=<\|det\|>)")
LOG_PREFIXES = (
    "build:",
    "clip_",
    "common_",
    "ggml_",
    "llama_",
    "load_",
    "main:",
    "mtmd_",
    "sampling:",
    "system_info:",
)
RUNTIME_BACKENDS = {
    "ffi": "Persistent ctypes model session",
    "server": "Persistent llama-server session",
    "executable": "Executable per request",
}
DEFAULT_RUNTIME_BACKEND = os.environ.get("UOCR_RUNTIME_BACKEND", "ffi")


@dataclass(frozen=True)
class RuntimePaths:
    binary: Path
    ffi_library: Path
    server_binary: Path
    model: Path
    mmproj: Path

    @classmethod
    def from_env(cls) -> "RuntimePaths":
        return cls(
            binary=resolve_repo_path(os.environ.get("UOCR_LLAMA_BIN"), default_llama_binary()),
            ffi_library=resolve_repo_path(os.environ.get("UOCR_FFI_LIB"), default_ffi_library()),
            server_binary=resolve_repo_path(os.environ.get("UOCR_LLAMA_SERVER_BIN"), default_llama_server_binary()),
            model=resolve_repo_path(os.environ.get("UOCR_MODEL"), DEFAULT_MODEL),
            mmproj=resolve_repo_path(os.environ.get("UOCR_MMPROJ"), DEFAULT_MMPROJ),
        )

    def missing(self, backend: str = "executable") -> list[str]:
        missing: list[str] = []
        normalized = normalize_runtime_backend(backend)
        if normalized == "ffi":
            runner_path = self.ffi_library
            runner_label = "ffi library"
        elif normalized == "server":
            runner_path = self.server_binary
            runner_label = "server binary"
        else:
            runner_path = self.binary
            runner_label = "binary"
        for label, path in ((runner_label, runner_path), ("model", self.model), ("mmproj", self.mmproj)):
            if not path.exists():
                missing.append(f"{label}: {path}")
        return missing


@dataclass(frozen=True)
class NativeEvent:
    kind: str
    text: str = ""
    metadata: dict | None = None


class RunawayGenerationGuard:
    def __init__(self) -> None:
        self.min_run = max(8, int(os.environ.get("UOCR_RUNAWAY_NUMBER_MIN_RUN", "48")))
        self.min_repeat_run = max(12, int(os.environ.get("UOCR_RUNAWAY_REPEAT_MIN_RUN", "64")))
        self.tail_chars = max(1000, int(os.environ.get("UOCR_RUNAWAY_TAIL_CHARS", "12000")))
        self.text = ""
        self.triggered = False
        self.reason = ""
        self.tail = ""

    def append(self, text: str) -> bool:
        if self.triggered:
            return True
        self.text += text
        return self._check_counting_run() or self._check_histogram_run() or self._check_repeated_number_run()

    def _check_counting_run(self) -> bool:
        base = max(0, len(self.text) - self.tail_chars)
        tail = self.text[base:]
        matches = list(COUNTING_RUN_NUMBER_RE.finditer(tail))
        return self._check_numeric_matches(
            tail=tail,
            base=base,
            matches=matches,
            min_run=self.min_run,
            mode="increment",
            reason="Stopped runaway numeric counting output from the native OCR model. "
            "This page likely triggered a hallucinated sequence instead of stable OCR.",
        )

    def _check_histogram_run(self) -> bool:
        base = max(0, len(self.text) - self.tail_chars)
        tail = self.text[base:]
        matches = list(HISTOGRAM_RUN_ENTRY_RE.finditer(tail))
        return self._check_numeric_matches(
            tail=tail,
            base=base,
            matches=matches,
            min_run=self.min_run,
            mode="increment",
            reason="Stopped runaway numeric histogram output from the native OCR model. "
            "This page likely triggered a hallucinated token-frequency sequence instead of stable OCR.",
        )

    def _check_repeated_number_run(self) -> bool:
        base = max(0, len(self.text) - self.tail_chars)
        tail = self.text[base:]
        matches = list(REPEATED_NUMBER_RUN_RE.finditer(tail))
        return self._check_numeric_matches(
            tail=tail,
            base=base,
            matches=matches,
            min_run=self.min_repeat_run,
            mode="repeat",
            reason="Stopped runaway repeated-number output from the native OCR model. "
            "This page likely triggered a hallucinated repeated numeric sequence instead of stable OCR.",
        )

    def _check_numeric_matches(
        self,
        *,
        tail: str,
        base: int,
        matches: list[re.Match[str]],
        min_run: int,
        mode: str,
        reason: str,
    ) -> bool:
        if len(matches) < min_run:
            return False

        run_start = 0
        run_length = 1
        previous = int(matches[0].group(1) if matches[0].lastindex else matches[0].group())
        for index in range(1, len(matches)):
            current = int(matches[index].group(1) if matches[index].lastindex else matches[index].group())
            gap = tail[matches[index - 1].end() : matches[index].start()]
            if mode == "increment":
                run_continues = current == previous + 1
            elif mode == "repeat":
                run_continues = current == previous
            else:
                run_continues = False
            separator_only = len(gap) <= 12 and bool(COUNTING_RUN_SEPARATOR_RE.fullmatch(gap))
            if run_continues and separator_only:
                run_length += 1
            else:
                run_start = index
                run_length = 1
            previous = current

            if run_length >= min_run:
                start = matches[run_start].start()
                self.triggered = True
                self.tail = tail[start : matches[index].end()][-2000:]
                self.reason = reason
                return True
        return False


def detect_recoverable_output_issue(text: str) -> str | None:
    guard = RunawayGenerationGuard()
    if guard.append(text):
        return guard.reason
    return None


def strip_output_artifacts(text: str) -> str:
    changed = True
    while changed:
        changed = False
        for pattern in (
            ORPHAN_DET_CLOSE_PREFIX_RE,
            REPEATED_NUMBER_PREFIX_RE,
            REPEATED_COMMA_PREFIX_RE,
            LEADING_LOW_VALUE_DET_PREFIX_RE,
            LEADING_CONJUNCTION_DET_PREFIX_RE,
            LEADING_SHORT_NUMBER_DET_PREFIX_RE,
        ):
            updated = pattern.sub("", text, count=1)
            if updated != text:
                text = updated
                changed = True
    return text.strip()


def profile_by_key(key: str) -> CandidateProfile:
    return CANDIDATE_PROFILES.get(key, CANDIDATE_PROFILES["best-zero-empty-q4"])


def normalize_runtime_backend(value: str | None) -> str:
    backend = (value or DEFAULT_RUNTIME_BACKEND or "ffi").strip().lower()
    aliases = {
        "persistent": "ffi",
        "ctypes": "ffi",
        "library": "ffi",
        "shared-library": "ffi",
        "llama-server": "server",
        "http": "server",
        "exe": "executable",
        "cli": "executable",
        "process": "executable",
    }
    backend = aliases.get(backend, backend)
    if backend not in RUNTIME_BACKENDS:
        raise ValueError(f"Unknown runtime backend: {value}")
    return backend


def format_prompt(prompt: str, media_placement: str) -> str:
    prompt = prompt or "document parsing."
    marker = "<__media__>"
    if media_placement == "auto":
        return prompt
    if media_placement == "prefix-tight":
        return f"{marker}{prompt}"
    if media_placement == "prefix-newline":
        return f"{marker}\n{prompt}"
    if media_placement == "suffix-newline":
        return f"{prompt}\n{marker}"
    raise ValueError(f"Unknown media placement: {media_placement}")


def build_command(
    *,
    paths: RuntimePaths,
    image_path: Path,
    prompt: str,
    profile: CandidateProfile,
    max_tokens: int | None,
) -> list[str]:
    argv = [
        str(paths.binary),
        "-m",
        str(paths.model),
        "--mmproj",
        str(paths.mmproj),
        "--image",
        str(image_path),
        "-p",
        format_prompt(prompt, profile.media_placement),
        "--chat-template",
        "deepseek-ocr",
        "--temp",
        "0",
        "--top-k",
        "1",
        "-n",
        str(max_tokens or profile.default_max_tokens),
        "-c",
        str(profile.ctx_size),
        "-ngl",
        "all",
        "--log-verbosity",
        "2",
    ]
    if profile.force_prompt_eos:
        argv.extend(["--override-kv", "tokenizer.ggml.add_eos_token=bool:true"])
    return argv


def build_env(profile: CandidateProfile) -> dict[str, str]:
    env = os.environ.copy()
    env["NO_PROXY"] = env.get("NO_PROXY", "*")
    env["no_proxy"] = env.get("no_proxy", env["NO_PROXY"])

    if profile.deepseek_ocr_mode == "gundam":
        env["LLAMA_DEEPSEEK_OCR_GUNDAM"] = "1"
    else:
        env.pop("LLAMA_DEEPSEEK_OCR_GUNDAM", None)

    if profile.no_repeat_ngram:
        env["LLAMA_DEEPSEEK_OCR_NO_REPEAT_NGRAM"] = "1"
        env["LLAMA_DEEPSEEK_OCR_NGRAM_SIZE"] = str(profile.ngram_size)
        env["LLAMA_DEEPSEEK_OCR_NGRAM_WINDOW"] = str(profile.ngram_window)
        env["LLAMA_DEEPSEEK_OCR_NGRAM_WHITELIST"] = ",".join(
            str(token_id) for token_id in profile.ngram_whitelist
        )
    else:
        for key in (
            "LLAMA_DEEPSEEK_OCR_NO_REPEAT_NGRAM",
            "LLAMA_DEEPSEEK_OCR_NGRAM_SIZE",
            "LLAMA_DEEPSEEK_OCR_NGRAM_WINDOW",
            "LLAMA_DEEPSEEK_OCR_NGRAM_WHITELIST",
        ):
            env.pop(key, None)

    if profile.prefill_aware_swa:
        env["LLAMA_DEEPSEEK_OCR_PREFILL_AWARE_SWA"] = "1"
        env["LLAMA_DEEPSEEK_OCR_DECODE_WINDOW"] = str(profile.decode_window)
        env.pop("LLAMA_DEEPSEEK_OCR_LEGACY_KV_PRUNE", None)
    else:
        env.pop("LLAMA_DEEPSEEK_OCR_PREFILL_AWARE_SWA", None)
        env.pop("LLAMA_DEEPSEEK_OCR_LEGACY_KV_PRUNE", None)
        env.pop("LLAMA_DEEPSEEK_OCR_DECODE_WINDOW", None)

    if profile.no_image_end:
        env["LLAMA_DEEPSEEK_OCR_NO_IMAGE_END"] = "1"
    else:
        env.pop("LLAMA_DEEPSEEK_OCR_NO_IMAGE_END", None)

    env.pop("LLAMA_DEEPSEEK_OCR_MIN_NEW_TOKENS", None)
    return env


def _stream_executable_ocr(
    *,
    paths: RuntimePaths,
    image_path: Path,
    prompt: str,
    profile: CandidateProfile,
    max_tokens: int | None = None,
) -> Iterator[NativeEvent]:
    missing = paths.missing("executable")
    if missing:
        yield NativeEvent("error", metadata={"error": "Missing runtime files", "missing": missing})
        return
    if not image_path.exists():
        yield NativeEvent("error", metadata={"error": f"Image not found: {image_path}"})
        return

    argv = build_command(
        paths=paths,
        image_path=image_path,
        prompt=prompt,
        profile=profile,
        max_tokens=max_tokens,
    )
    env = build_env(profile)
    started = time.monotonic()
    proc: subprocess.Popen[bytes] | None = None
    events: queue.Queue[tuple[str, bytes | None]] = queue.Queue()

    try:
        proc = subprocess.Popen(
            argv,
            cwd=str(REPO_ROOT),
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except OSError as exc:
        yield NativeEvent("error", metadata={"error": f"Failed to start native runner: {exc}"})
        return

    assert proc.stdout is not None
    assert proc.stderr is not None

    stdout_decoder = codecs.getincrementaldecoder("utf-8")(errors="replace")
    stderr_decoder = codecs.getincrementaldecoder("utf-8")(errors="replace")
    stderr_parts: list[str] = []
    stdout_parts: list[str] = []
    closed = set()

    def reader(name: str, chunk_size: int) -> None:
        pipe = proc.stdout if name == "stdout" else proc.stderr
        assert pipe is not None
        try:
            while True:
                data = pipe.read(chunk_size)
                if not data:
                    break
                events.put((name, data))
        finally:
            events.put((name, None))

    threading.Thread(target=reader, args=("stdout", 1), daemon=True).start()
    threading.Thread(target=reader, args=("stderr", 1024), daemon=True).start()

    try:
        while len(closed) < 2:
            try:
                name, data = events.get(timeout=0.1)
            except queue.Empty:
                if proc.poll() is not None and len(closed) == 2:
                    break
                continue
            if data is None:
                closed.add(name)
                continue
            if name == "stdout":
                text = stdout_decoder.decode(data)
                if text:
                    stdout_parts.append(text)
                    yield NativeEvent("token", text=text)
            else:
                text = stderr_decoder.decode(data)
                if text:
                    stderr_parts.append(text)

        tail_stdout = stdout_decoder.decode(b"", final=True)
        tail_stderr = stderr_decoder.decode(b"", final=True)
        if tail_stdout:
            stdout_parts.append(tail_stdout)
            yield NativeEvent("token", text=tail_stdout)
        if tail_stderr:
            stderr_parts.append(tail_stderr)

        exit_code = proc.wait()
        elapsed_ms = int((time.monotonic() - started) * 1000)
        command = command_text(argv)
        yield NativeEvent(
            "done" if exit_code == 0 else "error",
            text="".join(stdout_parts),
            metadata={
                "exit_code": exit_code,
                "elapsed_ms": elapsed_ms,
                "command": command,
                "profile": profile.engine_name,
                "binary": str(paths.binary),
                "model": str(paths.model),
                "mmproj": str(paths.mmproj),
                "stderr_tail": "".join(stderr_parts)[-4000:],
            },
        )
    finally:
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=2)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=2)


class _UocrFfiEvent(ctypes.Structure):
    _fields_ = [
        ("struct_size", ctypes.c_uint32),
        ("type", ctypes.c_uint32),
        ("text_utf8", ctypes.c_void_p),
        ("text_len", ctypes.c_uint64),
        ("json_utf8", ctypes.c_void_p),
        ("json_len", ctypes.c_uint64),
        ("code", ctypes.c_int32),
        ("reserved_u32", ctypes.c_uint32),
        ("index", ctypes.c_uint64),
        ("reserved_ptr0", ctypes.c_void_p),
        ("reserved_ptr1", ctypes.c_void_p),
        ("reserved_ptr2", ctypes.c_void_p),
        ("reserved_ptr3", ctypes.c_void_p),
    ]


_UocrFfiEventCallback = ctypes.CFUNCTYPE(ctypes.c_int32, ctypes.POINTER(_UocrFfiEvent), ctypes.c_void_p)


class _UocrFfiParams(ctypes.Structure):
    _fields_ = [
        ("struct_size", ctypes.c_uint32),
        ("flags", ctypes.c_uint32),
        ("model_path", ctypes.c_char_p),
        ("mmproj_path", ctypes.c_char_p),
        ("chat_template", ctypes.c_char_p),
        ("ctx_size", ctypes.c_int32),
        ("n_batch", ctypes.c_int32),
        ("n_gpu_layers", ctypes.c_int32),
        ("log_verbosity", ctypes.c_int32),
        ("force_prompt_eos", ctypes.c_int32),
        ("no_image_end", ctypes.c_int32),
        ("gundam_mode", ctypes.c_int32),
        ("no_repeat_ngram", ctypes.c_int32),
        ("ngram_size", ctypes.c_int32),
        ("ngram_window", ctypes.c_int32),
        ("ngram_whitelist", ctypes.c_char_p),
        ("prefill_aware_swa", ctypes.c_int32),
        ("legacy_kv_prune", ctypes.c_int32),
        ("decode_window", ctypes.c_int32),
        ("min_new_tokens", ctypes.c_int32),
        ("reserved_ptr0", ctypes.c_void_p),
        ("reserved_ptr1", ctypes.c_void_p),
        ("reserved_ptr2", ctypes.c_void_p),
        ("reserved_ptr3", ctypes.c_void_p),
    ]


class _UocrFfiRequest(ctypes.Structure):
    _fields_ = [
        ("struct_size", ctypes.c_uint32),
        ("flags", ctypes.c_uint32),
        ("image_path", ctypes.c_char_p),
        ("prompt", ctypes.c_char_p),
        ("max_tokens", ctypes.c_int32),
        ("reserved_i32", ctypes.c_int32),
        ("event_callback", _UocrFfiEventCallback),
        ("user_data", ctypes.c_void_p),
        ("reserved_ptr0", ctypes.c_void_p),
        ("reserved_ptr1", ctypes.c_void_p),
        ("reserved_ptr2", ctypes.c_void_p),
        ("reserved_ptr3", ctypes.c_void_p),
    ]


UOCR_FFI_STATUS_OK = 0
UOCR_FFI_STATUS_CANCELLED = 6
UOCR_FFI_EVENT_TOKEN = 1
UOCR_FFI_EVENT_DONE = 4
UOCR_FFI_ABI_VERSION = 1


@dataclass(frozen=True)
class _FfiKey:
    ffi_library: str
    model: str
    mmproj: str
    profile_key: str
    force_prompt_eos: bool
    no_image_end: bool
    deepseek_ocr_mode: str
    no_repeat_ngram: bool
    ngram_size: int
    ngram_window: int
    ngram_whitelist: tuple[int, ...]
    prefill_aware_swa: bool
    decode_window: int
    ctx_size: int
    n_batch: int
    n_gpu_layers: int


@dataclass
class _FfiSession:
    key: _FfiKey
    lib: ctypes.CDLL
    handle: int
    library_path: Path


_ffi_lock = threading.RLock()
_ffi_session: _FfiSession | None = None
_dll_directory_handles: list[object] = []
_dll_directory_paths: set[str] = set()
_WINDOWS_RUNTIME_DEPENDENCY_DLLS = (
    "cublas64_13.dll",
    "cublasLt64_13.dll",
    "cudart64_13.dll",
    "libcrypto-3-x64.dll",
    "libssl-3-x64.dll",
    "nvrtc64_130_0.dll",
)


def _utf8(value: str | os.PathLike[str]) -> bytes:
    return str(value).encode("utf-8")


def _env_int(name: str, default: int) -> int:
    raw = os.environ.get(name)
    if not raw:
        return default
    try:
        return int(raw)
    except ValueError:
        return default


def _windows_dependency_dirs(path: Path) -> list[Path]:
    dirs = [path.parent]
    for raw in os.environ.get("PATH", "").split(os.pathsep):
        if not raw:
            continue
        candidate = Path(raw.strip('"'))
        dirs.append(candidate)
        if candidate.name.lower() in {"bin", "cmd"} and candidate.parent.name.lower() == "git":
            dirs.append(candidate.parent / "mingw64" / "bin")

    unique: list[Path] = []
    seen: set[str] = set()
    for directory in dirs:
        try:
            resolved = directory.resolve(strict=False)
        except OSError:
            continue
        key = str(resolved).lower()
        if key in seen or not resolved.is_dir():
            continue
        if resolved == path.parent or any((resolved / name).exists() for name in _WINDOWS_RUNTIME_DEPENDENCY_DLLS):
            unique.append(resolved)
            seen.add(key)
    return unique


def _add_windows_dll_directory(directory: Path) -> None:
    key = str(directory.resolve(strict=False)).lower()
    if key in _dll_directory_paths:
        return
    _dll_directory_handles.append(os.add_dll_directory(str(directory)))
    _dll_directory_paths.add(key)


def _load_ffi_library(path: Path) -> ctypes.CDLL:
    if os.name == "nt" and hasattr(os, "add_dll_directory"):
        for directory in _windows_dependency_dirs(path):
            _add_windows_dll_directory(directory)
    elif os.name != "nt":
        pattern = "lib*.dylib" if os.uname().sysname == "Darwin" else "lib*.so*"
        for sibling in sorted(path.parent.glob(pattern)):
            if sibling.resolve(strict=False) == path.resolve(strict=False):
                continue
            try:
                ctypes.CDLL(str(sibling), mode=ctypes.RTLD_GLOBAL)
            except OSError:
                pass
    lib = ctypes.CDLL(str(path))
    lib.uocr_ffi_abi_version.argtypes = []
    lib.uocr_ffi_abi_version.restype = ctypes.c_uint32
    lib.uocr_ffi_build_info.argtypes = []
    lib.uocr_ffi_build_info.restype = ctypes.c_char_p
    lib.uocr_ffi_media_marker.argtypes = []
    lib.uocr_ffi_media_marker.restype = ctypes.c_char_p
    lib.uocr_ffi_create.argtypes = [ctypes.POINTER(_UocrFfiParams)]
    lib.uocr_ffi_create.restype = ctypes.c_void_p
    lib.uocr_ffi_destroy.argtypes = [ctypes.c_void_p]
    lib.uocr_ffi_destroy.restype = None
    lib.uocr_ffi_run_image.argtypes = [ctypes.c_void_p, ctypes.POINTER(_UocrFfiRequest)]
    lib.uocr_ffi_run_image.restype = ctypes.c_int32
    lib.uocr_ffi_last_error.argtypes = [ctypes.c_void_p]
    lib.uocr_ffi_last_error.restype = ctypes.c_char_p
    lib.uocr_ffi_last_status.argtypes = [ctypes.c_void_p]
    lib.uocr_ffi_last_status.restype = ctypes.c_int32
    lib.uocr_ffi_run_count.argtypes = [ctypes.c_void_p]
    lib.uocr_ffi_run_count.restype = ctypes.c_uint64
    abi_version = int(lib.uocr_ffi_abi_version())
    if abi_version != UOCR_FFI_ABI_VERSION:
        raise RuntimeError(f"unsupported uocr ffi ABI {abi_version}; expected {UOCR_FFI_ABI_VERSION}")
    return lib


def _ffi_last_error(lib: ctypes.CDLL, handle: int | None = None) -> str:
    raw = lib.uocr_ffi_last_error(ctypes.c_void_p(handle) if handle else None)
    return raw.decode("utf-8", errors="replace") if raw else ""


def _ffi_key(paths: RuntimePaths, profile: CandidateProfile) -> _FfiKey:
    return _FfiKey(
        ffi_library=str(paths.ffi_library),
        model=str(paths.model),
        mmproj=str(paths.mmproj),
        profile_key=profile.key,
        force_prompt_eos=profile.force_prompt_eos,
        no_image_end=profile.no_image_end,
        deepseek_ocr_mode=profile.deepseek_ocr_mode,
        no_repeat_ngram=profile.no_repeat_ngram,
        ngram_size=profile.ngram_size,
        ngram_window=profile.ngram_window,
        ngram_whitelist=profile.ngram_whitelist,
        prefill_aware_swa=profile.prefill_aware_swa,
        decode_window=profile.decode_window,
        ctx_size=profile.ctx_size,
        n_batch=_env_int("UOCR_FFI_N_BATCH", 2048),
        n_gpu_layers=_env_int("UOCR_FFI_N_GPU_LAYERS", -2),
    )


def _destroy_ffi_locked() -> None:
    global _ffi_session
    session = _ffi_session
    _ffi_session = None
    if session and session.handle:
        session.lib.uocr_ffi_destroy(ctypes.c_void_p(session.handle))


def _destroy_ffi() -> None:
    with _ffi_lock:
        _destroy_ffi_locked()


atexit.register(_destroy_ffi)


def _ensure_ffi(paths: RuntimePaths, profile: CandidateProfile) -> _FfiSession:
    global _ffi_session
    key = _ffi_key(paths, profile)
    with _ffi_lock:
        if _ffi_session and _ffi_session.key == key and _ffi_session.handle:
            return _ffi_session
        if _ffi_session:
            _destroy_ffi_locked()

        lib = _load_ffi_library(paths.ffi_library)
        params = _UocrFfiParams(
            struct_size=ctypes.sizeof(_UocrFfiParams),
            flags=0,
            model_path=_utf8(paths.model),
            mmproj_path=_utf8(paths.mmproj),
            chat_template=b"deepseek-ocr",
            ctx_size=key.ctx_size,
            n_batch=key.n_batch,
            n_gpu_layers=key.n_gpu_layers,
            log_verbosity=2,
            force_prompt_eos=int(profile.force_prompt_eos),
            no_image_end=int(profile.no_image_end),
            gundam_mode=int(profile.deepseek_ocr_mode == "gundam"),
            no_repeat_ngram=int(profile.no_repeat_ngram),
            ngram_size=profile.ngram_size,
            ngram_window=profile.ngram_window,
            ngram_whitelist=",".join(str(token_id) for token_id in profile.ngram_whitelist).encode("utf-8"),
            prefill_aware_swa=int(profile.prefill_aware_swa),
            legacy_kv_prune=0,
            decode_window=profile.decode_window,
            min_new_tokens=0,
            reserved_ptr0=None,
            reserved_ptr1=None,
            reserved_ptr2=None,
            reserved_ptr3=None,
        )
        handle = lib.uocr_ffi_create(ctypes.byref(params))
        if not handle:
            raise RuntimeError(_ffi_last_error(lib) or "failed to create native ffi session")
        _ffi_session = _FfiSession(key=key, lib=lib, handle=int(handle), library_path=paths.ffi_library)
        return _ffi_session


def _stream_ffi_ocr(
    *,
    paths: RuntimePaths,
    image_path: Path,
    prompt: str,
    profile: CandidateProfile,
    max_tokens: int | None = None,
) -> Iterator[NativeEvent]:
    missing = paths.missing("ffi")
    if missing:
        yield NativeEvent("error", metadata={"error": "Missing runtime files", "missing": missing, "backend": "ffi"})
        return
    if not image_path.exists():
        yield NativeEvent("error", metadata={"error": f"Image not found: {image_path}", "backend": "ffi"})
        return

    started = time.monotonic()
    try:
        session = _ensure_ffi(paths, profile)
    except Exception as exc:  # noqa: BLE001 - surfaced to UI
        yield NativeEvent("error", metadata={"error": f"Failed to load native ffi runtime: {exc}", "backend": "ffi"})
        return

    events: queue.Queue[NativeEvent | None] = queue.Queue()
    stdout_parts: list[str] = []
    guard = RunawayGenerationGuard()

    def worker() -> None:
        callback_ref: _UocrFfiEventCallback | None = None
        try:
            def on_event(event_ptr: ctypes.POINTER(_UocrFfiEvent), _user_data: ctypes.c_void_p) -> int:
                event = event_ptr.contents
                if event.type == UOCR_FFI_EVENT_TOKEN and event.text_utf8 and event.text_len:
                    text = ctypes.string_at(event.text_utf8, event.text_len).decode("utf-8", errors="replace")
                    stdout_parts.append(text)
                    if guard.append(text):
                        return 1
                    events.put(NativeEvent("token", text=text))
                return 0

            callback_ref = _UocrFfiEventCallback(on_event)
            request = _UocrFfiRequest(
                struct_size=ctypes.sizeof(_UocrFfiRequest),
                flags=0,
                image_path=_utf8(image_path),
                prompt=_utf8(format_prompt(prompt, profile.media_placement)),
                max_tokens=max_tokens or profile.default_max_tokens,
                reserved_i32=0,
                event_callback=callback_ref,
                user_data=None,
                reserved_ptr0=None,
                reserved_ptr1=None,
                reserved_ptr2=None,
                reserved_ptr3=None,
            )
            status = int(session.lib.uocr_ffi_run_image(ctypes.c_void_p(session.handle), ctypes.byref(request)))
            elapsed_ms = int((time.monotonic() - started) * 1000)
            if status == UOCR_FFI_STATUS_OK:
                events.put(
                    NativeEvent(
                        "done",
                        text="".join(stdout_parts),
                        metadata={
                            "exit_code": 0,
                            "elapsed_ms": elapsed_ms,
                            "command": f"ctypes {session.library_path} uocr_ffi_run_image",
                            "profile": profile.engine_name,
                            "backend": "ffi",
                            "ffi_library": str(session.library_path),
                            "ffi_run_count": int(session.lib.uocr_ffi_run_count(ctypes.c_void_p(session.handle))),
                            "model": str(paths.model),
                            "mmproj": str(paths.mmproj),
                            "stderr_tail": "",
                        },
                    )
                )
            else:
                error = _ffi_last_error(session.lib, session.handle)
                if guard.triggered:
                    error = guard.reason
                events.put(
                    NativeEvent(
                        "error",
                        text="" if guard.triggered else "".join(stdout_parts),
                        metadata={
                            "error": error or f"native ffi runtime returned status {status}",
                            "exit_code": status,
                            "elapsed_ms": elapsed_ms,
                            "backend": "ffi",
                            "ffi_library": str(session.library_path),
                            "runaway_generation": guard.triggered,
                            "runaway_tail": guard.tail if guard.triggered else "",
                            "stderr_tail": error[-4000:],
                        },
                    )
                )
        except Exception as exc:  # noqa: BLE001 - surfaced to UI
            events.put(
                NativeEvent(
                    "error",
                    text="".join(stdout_parts),
                    metadata={"error": f"Native ffi runtime failed: {exc}", "backend": "ffi"},
                )
            )
        finally:
            callback_ref = None
            events.put(None)

    threading.Thread(target=worker, daemon=True).start()

    while True:
        event = events.get()
        if event is None:
            break
        yield event


@dataclass(frozen=True)
class _ServerKey:
    server_binary: str
    model: str
    mmproj: str
    profile_key: str
    force_prompt_eos: bool
    no_image_end: bool
    deepseek_ocr_mode: str
    no_repeat_ngram: bool
    ngram_size: int
    ngram_window: int
    prefill_aware_swa: bool
    decode_window: int
    ctx_size: int


@dataclass
class _ServerSession:
    key: _ServerKey
    proc: subprocess.Popen[str]
    url: str
    log_path: Path
    media_marker: str
    command: list[str]


_server_lock = threading.RLock()
_server_session: _ServerSession | None = None


def _free_local_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def _server_ready(url: str) -> bool:
    try:
        response = requests.get(f"{url.rstrip('/')}/health", timeout=2)
        return response.status_code == 200
    except requests.RequestException:
        return False


def _server_media_marker(url: str) -> str:
    try:
        response = requests.get(f"{url.rstrip('/')}/props", timeout=10)
        response.raise_for_status()
        marker = response.json().get("media_marker")
        if isinstance(marker, str) and marker:
            return marker
    except requests.RequestException:
        pass
    return "<__media__>"


def _server_key(paths: RuntimePaths, profile: CandidateProfile) -> _ServerKey:
    return _ServerKey(
        server_binary=str(paths.server_binary),
        model=str(paths.model),
        mmproj=str(paths.mmproj),
        profile_key=profile.key,
        force_prompt_eos=profile.force_prompt_eos,
        no_image_end=profile.no_image_end,
        deepseek_ocr_mode=profile.deepseek_ocr_mode,
        no_repeat_ngram=profile.no_repeat_ngram,
        ngram_size=profile.ngram_size,
        ngram_window=profile.ngram_window,
        prefill_aware_swa=profile.prefill_aware_swa,
        decode_window=profile.decode_window,
        ctx_size=profile.ctx_size,
    )


def _stop_server_locked() -> None:
    global _server_session
    session = _server_session
    _server_session = None
    if not session:
        return
    if session.proc.poll() is None:
        session.proc.terminate()
        try:
            session.proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            session.proc.kill()
            session.proc.wait(timeout=5)


def _stop_server() -> None:
    with _server_lock:
        _stop_server_locked()


atexit.register(_stop_server)


def _ensure_server(paths: RuntimePaths, profile: CandidateProfile) -> _ServerSession:
    global _server_session
    key = _server_key(paths, profile)
    with _server_lock:
        if _server_session and _server_session.key == key and _server_session.proc.poll() is None:
            return _server_session
        if _server_session:
            _stop_server_locked()

        port = _free_local_port()
        url = f"http://127.0.0.1:{port}"
        log_path = Path(os.environ["UOCR_SERVER_LOG"]) if os.environ.get("UOCR_SERVER_LOG") else (
            Path(tempfile.gettempdir()) / f"uocr_llama_server_{port}.log"
        )
        log_path.parent.mkdir(parents=True, exist_ok=True)
        argv = [
            str(paths.server_binary),
            "-m",
            str(paths.model),
            "--mmproj",
            str(paths.mmproj),
            "--chat-template",
            "deepseek-ocr",
            "-c",
            str(profile.ctx_size),
            "-ngl",
            "all",
            "--alias",
            "Unlimited-OCR",
            "--host",
            "127.0.0.1",
            "--port",
            str(port),
            "--log-verbosity",
            "2",
        ]
        if profile.force_prompt_eos:
            argv.extend(["--override-kv", "tokenizer.ggml.add_eos_token=bool:true"])

        env = build_env(profile)
        log_file = log_path.open("w", encoding="utf-8")
        try:
            proc = subprocess.Popen(
                argv,
                cwd=str(REPO_ROOT),
                env=env,
                stdout=log_file,
                stderr=subprocess.STDOUT,
                text=True,
            )
        except OSError:
            log_file.close()
            raise

        deadline = time.monotonic() + int(os.environ.get("UOCR_SERVER_START_TIMEOUT", "180"))
        while time.monotonic() < deadline:
            if proc.poll() is not None:
                log_file.close()
                raise RuntimeError(f"llama-server exited during startup. Check {log_path}")
            if _server_ready(url):
                session = _ServerSession(
                    key=key,
                    proc=proc,
                    url=url,
                    log_path=log_path,
                    media_marker=_server_media_marker(url),
                    command=argv,
                )
                _server_session = session
                return session
            time.sleep(0.5)

        proc.terminate()
        log_file.close()
        raise TimeoutError(f"Timed out waiting for llama-server. Check {log_path}")


def _server_prompt(prompt: str, media_marker: str, media_placement: str) -> str:
    prompt = prompt or "document parsing."
    if media_placement == "auto":
        return f"{media_marker}\n{prompt}"
    if media_placement == "prefix-tight":
        return f"{media_marker}{prompt}"
    if media_placement == "prefix-newline":
        return f"{media_marker}\n{prompt}"
    if media_placement == "suffix-newline":
        return f"{prompt}\n{media_marker}"
    raise ValueError(f"Unknown media placement: {media_placement}")


def _server_payload(
    *,
    image_path: Path,
    prompt: str,
    profile: CandidateProfile,
    media_marker: str,
    max_tokens: int | None,
) -> dict:
    return {
        "prompt": {
            "prompt_string": _server_prompt(prompt, media_marker, profile.media_placement),
            "multimodal_data": [base64.b64encode(image_path.read_bytes()).decode("ascii")],
        },
        "temperature": 0,
        "top_k": 1,
        "n_predict": max_tokens or profile.default_max_tokens,
        "stream": True,
    }


def _stream_delta(payload: dict) -> str:
    content = payload.get("content")
    if isinstance(content, str):
        return content
    choices = payload.get("choices")
    if not isinstance(choices, list) or not choices:
        return ""
    first = choices[0]
    if not isinstance(first, dict):
        return ""
    text = first.get("text")
    if isinstance(text, str):
        return text
    delta = first.get("delta")
    if isinstance(delta, dict):
        delta_content = delta.get("content")
        if isinstance(delta_content, str):
            return delta_content
    return ""


def _stream_server_ocr(
    *,
    paths: RuntimePaths,
    image_path: Path,
    prompt: str,
    profile: CandidateProfile,
    max_tokens: int | None = None,
) -> Iterator[NativeEvent]:
    missing = paths.missing("server")
    if missing:
        yield NativeEvent("error", metadata={"error": "Missing runtime files", "missing": missing, "backend": "server"})
        return
    if not image_path.exists():
        yield NativeEvent("error", metadata={"error": f"Image not found: {image_path}", "backend": "server"})
        return

    started = time.monotonic()
    stdout_parts: list[str] = []
    try:
        session = _ensure_server(paths, profile)
    except Exception as exc:  # noqa: BLE001 - surfaced to UI
        yield NativeEvent("error", metadata={"error": f"Failed to start persistent runtime: {exc}", "backend": "server"})
        return

    payload = _server_payload(
        image_path=image_path,
        prompt=prompt,
        profile=profile,
        media_marker=session.media_marker,
        max_tokens=max_tokens,
    )

    try:
        with requests.post(
            f"{session.url.rstrip('/')}/completion",
            headers={"Content-Type": "application/json"},
            data=json.dumps(payload),
            stream=True,
            timeout=(10, None),
        ) as response:
            if response.status_code >= 400:
                yield NativeEvent(
                    "error",
                    metadata={
                        "error": f"Persistent runtime request failed: HTTP {response.status_code}",
                        "stderr_tail": response.text[-4000:],
                        "backend": "server",
                        "server_log": str(session.log_path),
                    },
                )
                return
            for raw_line in response.iter_lines(decode_unicode=True):
                if not raw_line or not raw_line.startswith("data: "):
                    continue
                data = raw_line[6:].strip()
                if data == "[DONE]":
                    break
                try:
                    event_payload = json.loads(data)
                except json.JSONDecodeError:
                    continue
                delta = _stream_delta(event_payload)
                if delta:
                    stdout_parts.append(delta)
                    yield NativeEvent("token", text=delta)
    except requests.RequestException as exc:
        yield NativeEvent(
            "error",
            text="".join(stdout_parts),
            metadata={
                "error": f"Persistent runtime request failed: {exc}",
                "backend": "server",
                "server_url": session.url,
                "server_log": str(session.log_path),
            },
        )
        return

    elapsed_ms = int((time.monotonic() - started) * 1000)
    yield NativeEvent(
        "done",
        text="".join(stdout_parts),
        metadata={
            "exit_code": 0,
            "elapsed_ms": elapsed_ms,
            "command": f"POST {session.url.rstrip('/')}/completion",
            "server_command": command_text(session.command),
            "profile": profile.engine_name,
            "backend": "server",
            "server_pid": session.proc.pid,
            "server_url": session.url,
            "server_log": str(session.log_path),
            "binary": str(paths.server_binary),
            "model": str(paths.model),
            "mmproj": str(paths.mmproj),
            "stderr_tail": "",
        },
    )


def stream_ocr(
    *,
    paths: RuntimePaths,
    image_path: Path,
    prompt: str,
    profile: CandidateProfile,
    max_tokens: int | None = None,
    runtime_backend: str | None = None,
) -> Iterator[NativeEvent]:
    backend = normalize_runtime_backend(runtime_backend)
    if backend == "ffi":
        yield from _stream_ffi_ocr(
            paths=paths,
            image_path=image_path,
            prompt=prompt,
            profile=profile,
            max_tokens=max_tokens,
        )
        return
    if backend == "server":
        yield from _stream_server_ocr(
            paths=paths,
            image_path=image_path,
            prompt=prompt,
            profile=profile,
            max_tokens=max_tokens,
        )
        return
    yield from _stream_executable_ocr(
        paths=paths,
        image_path=image_path,
        prompt=prompt,
        profile=profile,
        max_tokens=max_tokens,
    )


def clean_generated_text(text: str) -> str:
    text = ANSI_RE.sub("", text).replace("\r\n", "\n").replace("\r", "\n")
    cleaned_lines: list[str] = []
    for line in text.splitlines(keepends=True):
        stripped = line.strip()
        lowered = stripped.lower()
        if not stripped:
            cleaned_lines.append(line)
            continue
        if lowered.startswith(LOG_PREFIXES):
            continue
        if "tokens/s" in lowered or "token/s" in lowered:
            continue
        cleaned_lines.append(line)
    return strip_output_artifacts("".join(cleaned_lines).strip())


def command_text(argv: list[str]) -> str:
    if os.name == "nt":
        return subprocess.list2cmdline(argv)
    return shlex.join(argv)
