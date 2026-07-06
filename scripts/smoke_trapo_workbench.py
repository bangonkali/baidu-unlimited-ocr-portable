from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
import urllib.request
import zipfile
from pathlib import Path


def main() -> int:
    args = parse_args()
    archive = args.archive.resolve()
    if not archive.is_file():
        raise SystemExit(f"release archive was not produced: {archive}")

    extract_root = Path(tempfile.mkdtemp(prefix="trapo-smoke-"))
    try:
        extract_archive(archive, extract_root)
        app_root = find_app_root(extract_root)
        names = platform_names()
        runtime_bin = app_root / "thirdparty" / "uocr-runtime" / args.runtime_platform / "bin"
        assert_file(app_root / names["duckdb"])
        assert_file(app_root / names["pdfium"])
        assert_file(runtime_bin / names["ffi"])
        assert_file(runtime_bin / names["llama"])

        env = runtime_env(app_root)
        server = app_root / names["server"]
        run_checked([server, "--version"], app_root, env)
        run_checked(
            [server, "--check-ocr-runtime", choose_runtime(app_root, names["ffi"])],
            app_root,
            env,
        )
        run_checked(
            [server, "--check-embedding-runtime", choose_runtime(app_root, names["llama"])],
            app_root,
            env,
        )
        smoke_server(server, app_root, env, args.port)
    finally:
        shutil.rmtree(
            extract_root,
            ignore_errors=True,
        )  # skylos: ignore[SKY-D215] temp smoke extraction root is created by this process.
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--runtime-platform", required=True)
    parser.add_argument("--port", type=int, required=True)
    return parser.parse_args()


def extract_archive(archive: Path, destination: Path) -> None:
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as zip_file:
            extract_zip(zip_file, destination)
        return
    if archive.name.endswith(".tar.gz"):
        with tarfile.open(archive, "r:gz") as tar_file:
            extract_tar(tar_file, destination)
        return
    raise SystemExit(f"unsupported archive: {archive}")


def extract_zip(zip_file: zipfile.ZipFile, destination: Path) -> None:
    for member in zip_file.infolist():
        target = checked_archive_target(destination, member.filename)
        if member.is_dir():
            target.mkdir(parents=True, exist_ok=True)
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        with (
            zip_file.open(member) as source,
            target.open("wb") as output,  # skylos: ignore[SKY-D325] bounded smoke extraction root.
        ):
            shutil.copyfileobj(source, output)


def extract_tar(tar_file: tarfile.TarFile, destination: Path) -> None:
    for member in tar_file.getmembers():
        target = checked_archive_target(destination, member.name)
        if member.isdir():
            target.mkdir(parents=True, exist_ok=True)
            continue
        if not member.isfile():
            continue
        source = tar_file.extractfile(member)
        if source is None:
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        with (
            source,
            target.open("wb") as output,  # skylos: ignore[SKY-D325] bounded smoke extraction root.
        ):
            shutil.copyfileobj(source, output)
        target.chmod(member.mode & 0o777)


def checked_archive_target(destination: Path, member_name: str) -> Path:
    target = (destination / member_name).resolve()
    if not target.is_relative_to(destination.resolve()):
        raise SystemExit(f"archive member escapes extraction root: {member_name}")
    return target


def find_app_root(extract_root: Path) -> Path:
    server_name = platform_names()["server"]
    matches = sorted(extract_root.rglob(server_name))
    if not matches:
        raise SystemExit(f"{server_name} not found in archive")
    server = matches[0]
    if not server.is_file():
        raise SystemExit(f"{server} is not a file")
    server.chmod(server.stat().st_mode | 0o755)
    return server.parent


def platform_names() -> dict[str, Path | str]:
    if sys.platform == "win32":
        return {
            "server": "trapo-server.exe",
            "duckdb": Path("duckdb.dll"),
            "pdfium": Path("thirdparty/pdfium/bin/pdfium.dll"),
            "ffi": "uocr-ffi.dll",
            "llama": "llama.dll",
            "library_var": "PATH",
        }
    if sys.platform == "darwin":
        return {
            "server": "trapo-server",
            "duckdb": Path("libduckdb.dylib"),
            "pdfium": Path("thirdparty/pdfium/lib/libpdfium.dylib"),
            "ffi": "libuocr-ffi.dylib",
            "llama": "libllama.dylib",
            "library_var": "DYLD_LIBRARY_PATH",
        }
    return {
        "server": "trapo-server",
        "duckdb": Path("libduckdb.so"),
        "pdfium": Path("thirdparty/pdfium/lib/libpdfium.so"),
        "ffi": "libuocr-ffi.so",
        "llama": "libllama.so",
        "library_var": "LD_LIBRARY_PATH",
    }


def assert_file(path: Path) -> None:
    if not path.is_file():
        raise SystemExit(f"required packaged file was not found: {path}")


def runtime_env(app_root: Path) -> dict[str, str]:
    names = platform_names()
    env = os.environ.copy()
    runtime_bins = sorted((app_root / "thirdparty" / "uocr-runtime").glob("*/bin"))
    search_paths = [str(app_root), str((app_root / names["pdfium"]).parent)]
    search_paths.extend(str(path) for path in runtime_bins if path.is_dir())
    var = str(names["library_var"])
    separator = ";" if sys.platform == "win32" else ":"
    env[var] = separator.join([*search_paths, env.get(var, "")])
    return env


def choose_runtime(app_root: Path, library_name: str | Path) -> Path:
    matches = sorted((app_root / "thirdparty" / "uocr-runtime").glob(f"*/bin/{library_name}"))
    if not matches:
        raise SystemExit(f"{library_name} was not found in packaged runtimes")
    return next((path for path in matches if path.parts[-3].endswith("-cpu")), matches[0])


def run_checked(command: list[Path | str], cwd: Path, env: dict[str, str]) -> None:
    subprocess.run([str(part) for part in command], cwd=cwd, env=env, check=True)


def smoke_server(server: Path, app_root: Path, env: dict[str, str], port: int) -> None:
    stdout_path = app_root / "logs" / "smoke-stdout.log"
    stderr_path = app_root / "logs" / "smoke-stderr.log"
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    with (
        stdout_path.open(
            "w", encoding="utf-8"
        ) as stdout,  # skylos: ignore[SKY-D325] temp smoke log dir.
        stderr_path.open(
            "w", encoding="utf-8"
        ) as stderr,  # skylos: ignore[SKY-D325] temp smoke log dir.
    ):
        process = subprocess.Popen(
            [str(server), "--port", str(port), "--no-browser"],
            cwd=app_root,
            env=env,
            stdout=stdout,
            stderr=stderr,
        )
        try:
            wait_for_health(process, port)
            request_json(port, "/api/health")
            status = request_json(port, "/api/status")
            extensions = status["duckdb_extensions"]
            if not extensions["fts_loaded"]:
                raise SystemExit("DuckDB fts extension was not loaded")
            if not extensions["vss_loaded"]:
                raise SystemExit("DuckDB vss extension was not loaded")
            request_text(port, "/")
            request_json(port, "/api/openapi.json")
            assert_file(app_root / "data" / "trapo.duckdb")
            assert_file(app_root / "logs" / "trapo-server.log")
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()


def wait_for_health(process: subprocess.Popen[bytes], port: int) -> None:
    for _ in range(60):
        if process.poll() is not None:
            raise SystemExit(f"trapo-server exited early with code {process.returncode}")
        try:
            payload = request_json(port, "/api/health")
            if payload.get("ok") is True:
                return
        except Exception:
            time.sleep(0.5)
    raise SystemExit("health check failed")


def request_json(port: int, path: str) -> dict:
    import json

    return json.loads(request_text(port, path))


def request_text(port: int, path: str) -> str:
    with (
        urllib.request.urlopen(f"http://127.0.0.1:{port}{path}", timeout=2) as response
    ):  # skylos: ignore[SKY-D216] smoke requests are restricted to the local packaged server.
        return response.read().decode("utf-8")


if __name__ == "__main__":
    raise SystemExit(main())
