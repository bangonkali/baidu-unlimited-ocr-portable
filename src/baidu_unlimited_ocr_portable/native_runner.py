from __future__ import annotations

import codecs
import atexit
import base64
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
    default_llama_binary,
    default_llama_server_binary,
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
RUNTIME_BACKENDS = {
    "ffi": "Persistent model session",
    "executable": "Executable per request",
}
DEFAULT_RUNTIME_BACKEND = os.environ.get("UOCR_RUNTIME_BACKEND", "ffi")


@dataclass(frozen=True)
class RuntimePaths:
    binary: Path
    server_binary: Path
    model: Path
    mmproj: Path

    @classmethod
    def from_env(cls) -> "RuntimePaths":
        return cls(
            binary=resolve_repo_path(os.environ.get("UOCR_LLAMA_BIN"), default_llama_binary()),
            server_binary=resolve_repo_path(os.environ.get("UOCR_LLAMA_SERVER_BIN"), default_llama_server_binary()),
            model=resolve_repo_path(os.environ.get("UOCR_MODEL"), DEFAULT_MODEL),
            mmproj=resolve_repo_path(os.environ.get("UOCR_MMPROJ"), DEFAULT_MMPROJ),
        )

    def missing(self, backend: str = "executable") -> list[str]:
        missing: list[str] = []
        runner_path = self.server_binary if normalize_runtime_backend(backend) == "ffi" else self.binary
        runner_label = "server binary" if normalize_runtime_backend(backend) == "ffi" else "binary"
        for label, path in ((runner_label, runner_path), ("model", self.model), ("mmproj", self.mmproj)):
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


def normalize_runtime_backend(value: str | None) -> str:
    backend = (value or DEFAULT_RUNTIME_BACKEND or "ffi").strip().lower()
    aliases = {
        "persistent": "ffi",
        "server": "ffi",
        "llama-server": "ffi",
        "http": "ffi",
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
    missing = paths.missing("ffi")
    if missing:
        yield NativeEvent("error", metadata={"error": "Missing runtime files", "missing": missing, "backend": "ffi"})
        return
    if not image_path.exists():
        yield NativeEvent("error", metadata={"error": f"Image not found: {image_path}", "backend": "ffi"})
        return

    started = time.monotonic()
    stdout_parts: list[str] = []
    try:
        session = _ensure_server(paths, profile)
    except Exception as exc:  # noqa: BLE001 - surfaced to UI
        yield NativeEvent("error", metadata={"error": f"Failed to start persistent runtime: {exc}", "backend": "ffi"})
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
                        "backend": "ffi",
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
                "backend": "ffi",
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
            "backend": "ffi",
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
    return "".join(cleaned_lines).strip()


def command_text(argv: list[str]) -> str:
    if os.name == "nt":
        return subprocess.list2cmdline(argv)
    return shlex.join(argv)
