from __future__ import annotations

import codecs
import os
import queue
import re
import shlex
import subprocess
import threading
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Iterator

from .config import (
    CANDIDATE_PROFILES,
    DEFAULT_MODEL,
    DEFAULT_MMPROJ,
    REPO_ROOT,
    CandidateProfile,
    default_llama_binary,
    resolve_repo_path,
)


ANSI_RE = re.compile(r"\x1b\[[0-9;?]*[ -/]*[@-~]")
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


@dataclass(frozen=True)
class RuntimePaths:
    binary: Path
    model: Path
    mmproj: Path

    @classmethod
    def from_env(cls) -> "RuntimePaths":
        return cls(
            binary=resolve_repo_path(os.environ.get("UOCR_LLAMA_BIN"), default_llama_binary()),
            model=resolve_repo_path(os.environ.get("UOCR_MODEL"), DEFAULT_MODEL),
            mmproj=resolve_repo_path(os.environ.get("UOCR_MMPROJ"), DEFAULT_MMPROJ),
        )

    def missing(self) -> list[str]:
        missing: list[str] = []
        for label, path in (("binary", self.binary), ("model", self.model), ("mmproj", self.mmproj)):
            if not path.exists():
                missing.append(f"{label}: {path}")
        return missing


@dataclass(frozen=True)
class NativeEvent:
    kind: str
    text: str = ""
    metadata: dict | None = None


def profile_by_key(key: str) -> CandidateProfile:
    return CANDIDATE_PROFILES.get(key, CANDIDATE_PROFILES["best-zero-empty-q4"])


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
    else:
        env.pop("LLAMA_DEEPSEEK_OCR_PREFILL_AWARE_SWA", None)
        env.pop("LLAMA_DEEPSEEK_OCR_DECODE_WINDOW", None)

    if profile.no_image_end:
        env["LLAMA_DEEPSEEK_OCR_NO_IMAGE_END"] = "1"
    else:
        env.pop("LLAMA_DEEPSEEK_OCR_NO_IMAGE_END", None)

    env.pop("LLAMA_DEEPSEEK_OCR_MIN_NEW_TOKENS", None)
    return env


def stream_ocr(
    *,
    paths: RuntimePaths,
    image_path: Path,
    prompt: str,
    profile: CandidateProfile,
    max_tokens: int | None = None,
) -> Iterator[NativeEvent]:
    missing = paths.missing()
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
    return "".join(cleaned_lines).strip()


def command_text(argv: list[str]) -> str:
    if os.name == "nt":
        return subprocess.list2cmdline(argv)
    return shlex.join(argv)
