#!/usr/bin/env python3
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
import zipfile
from pathlib import Path
from urllib.parse import quote, urlparse

from onnxruntime_staging import stage_onnxruntime_files
from package_runtime import REPO_ROOT, load_platforms, sha256_file
from runtime_engine_guard_runtime import is_forbidden_runtime_path

USER_AGENT = "trapo-workbench-packager"
PDFIUM_REPO = "bblanchon/pdfium-binaries"
DEFAULT_PDFIUM_RELEASE = "chromium/7920"
PDFIUM_VERSION = "151.0.7920.0"
PDFIUM_DOWNLOAD_HOSTS = {"github.com"}

PLATFORMS = {
    "windows-x64": dict(
        archive_ext="zip",
        server="trapo-server.exe",
        duckdb="duckdb.dll",
        pdfium_asset="pdfium-win-x64.tgz",
        pdfium_lib="pdfium.dll",
        pdfium_dir=("thirdparty", "pdfium", "bin"),
    ),
    "windows-arm64": dict(
        archive_ext="zip",
        server="trapo-server.exe",
        duckdb="duckdb.dll",
        pdfium_asset="pdfium-win-arm64.tgz",
        pdfium_lib="pdfium.dll",
        pdfium_dir=("thirdparty", "pdfium", "bin"),
    ),
    "macos-arm64": dict(
        archive_ext="zip",
        server="trapo-server",
        duckdb="libduckdb.dylib",
        pdfium_asset="pdfium-mac-arm64.tgz",
        pdfium_lib="libpdfium.dylib",
        pdfium_dir=("thirdparty", "pdfium", "lib"),
    ),
    "linux-x64": dict(
        archive_ext="tar.gz",
        server="trapo-server",
        duckdb="libduckdb.so",
        pdfium_asset="pdfium-linux-x64.tgz",
        pdfium_lib="libpdfium.so",
        pdfium_dir=("thirdparty", "pdfium", "lib"),
    ),
    "linux-arm64": dict(
        archive_ext="tar.gz",
        server="trapo-server",
        duckdb="libduckdb.so",
        pdfium_asset="pdfium-linux-arm64.tgz",
        pdfium_lib="libpdfium.so",
        pdfium_dir=("thirdparty", "pdfium", "lib"),
    ),
}
ONNXRUNTIME_RECEIPTS: dict[str, dict[str, object]] = {}

RUNTIME_PLATFORM_ALIASES = {
    "linux-x64": "linux-x86_64-cuda13",
    "linux-arm64": "linux-arm64-cpu",
    "macos-arm64": "macos-arm64-metal",
    "windows-x64": "windows-x86_64-cuda13",
    "windows-arm64": "windows-arm64-cpu",
}
KNOWN_RUNTIME_PLATFORMS = sorted(load_platforms(REPO_ROOT)["targets"].keys())


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


def safe_version(value: str) -> str:
    return value.strip().replace("/", "-").replace("\\", "-") or "dev"


def validate_pdfium_url(url: str) -> str:
    parsed = urlparse(url)
    host = (parsed.hostname or "").lower()
    if parsed.scheme != "https" or host not in PDFIUM_DOWNLOAD_HOSTS:
        die(f"refusing non-PDFium GitHub download URL: {url}")
    return url


def run(command: list[str], *, cwd: Path = REPO_ROOT, env: dict[str, str] | None = None) -> None:
    print("+ " + " ".join(command), flush=True)
    subprocess.run(command, cwd=cwd, env=env, check=True)


def prepare_onnxruntime_deps(platform_id: str) -> dict[str, object]:
    if platform_id in ONNXRUNTIME_RECEIPTS:
        return ONNXRUNTIME_RECEIPTS[platform_id]
    command = [
        "cargo",
        "run",
        "--quiet",
        "-p",
        "trapo-xtask",
        "--",
        "native-deps",
        "prepare",
        "--platform",
        platform_id,
        "--repo-root",
        str(REPO_ROOT),
    ]
    print("+ " + " ".join(command), flush=True)
    output = subprocess.check_output(command, cwd=REPO_ROOT, text=True)
    lines = [line for line in output.splitlines() if line.strip()]
    if not lines:
        die("native dependency helper did not return an ONNX Runtime receipt")
    receipt = json.loads(lines[-1])
    ONNXRUNTIME_RECEIPTS[platform_id] = receipt
    return receipt


def git_output(args: list[str]) -> str:
    try:
        return subprocess.check_output(["git", *args], cwd=REPO_ROOT, text=True).strip()
    except Exception:
        return ""


def csv(value: str) -> list[str]:
    return [item.strip() for item in value.split(",") if item.strip()]


def supported_runtime_platforms() -> list[str]:
    return KNOWN_RUNTIME_PLATFORMS


def canonical_runtime_platform(platform_id: str) -> str:
    runtime_platforms = supported_runtime_platforms()
    if platform_id in runtime_platforms:
        return platform_id
    canonical = RUNTIME_PLATFORM_ALIASES.get(platform_id)
    if canonical and canonical in runtime_platforms:
        print(
            f"warning: runtime platform '{platform_id}' is an alias for '{canonical}'",
            file=sys.stderr,
        )
        return canonical
    die(
        "unsupported runtime platform: "
        f"{platform_id}. Known runtime platforms: {', '.join(runtime_platforms)}"
    )


def github_headers() -> dict[str, str]:
    headers = {"User-Agent": USER_AGENT}
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"
    return headers


def download(url: str, destination: Path) -> None:
    url = validate_pdfium_url(url)
    destination.parent.mkdir(parents=True, exist_ok=True)
    request = urllib.request.Request(url, headers=github_headers())
    with (
        urllib.request.urlopen(
            request, timeout=300
        ) as response,  # skylos: ignore[SKY-D216] PDFium GitHub host allowlist.
        destination.open("wb") as fh,  # skylos: ignore[SKY-D324] fixed temp asset path.
    ):
        shutil.copyfileobj(response, fh)


def pdfium_url(release: str, asset: str) -> str:
    if release == "latest":
        return f"https://github.com/{PDFIUM_REPO}/releases/latest/download/{asset}"
    return f"https://github.com/{PDFIUM_REPO}/releases/download/{quote(release, safe='')}/{asset}"


def safe_extract_tar(archive: Path, destination: Path) -> None:
    root = destination.resolve()
    with tarfile.open(archive, "r:*") as tar:
        for member in tar.getmembers():
            target = (destination / member.name).resolve()
            if root not in (target, *target.parents):
                die(f"refusing to extract archive member outside destination: {member.name}")
        tar.extractall(
            destination, filter="data"
        )  # skylos: ignore[SKY-D326] members bounded; data filter.


def install_pdfium(platform_id: str, stage_root: Path, release: str) -> dict[str, str]:
    config = PLATFORMS[platform_id]
    asset = config["pdfium_asset"]
    with tempfile.TemporaryDirectory(prefix="trapo-pdfium-") as tmp:
        tmp_root = Path(tmp)
        archive = tmp_root / asset
        print(f"Downloading PDFium {asset} from {release}", flush=True)
        download(pdfium_url(release, asset), archive)
        extracted = tmp_root / "extract"
        safe_extract_tar(archive, extracted)
        library = next(extracted.rglob(config["pdfium_lib"]), None)
        if library is None:
            die(f"PDFium archive did not contain {config['pdfium_lib']}")
        destination = stage_root.joinpath(*config["pdfium_dir"])
        destination.mkdir(parents=True, exist_ok=True)
        shutil.copy2(library, destination / library.name)
        for notice in extracted.rglob("LICENSE*"):
            if notice.is_file():
                shutil.copy2(notice, destination / notice.name)
        return {
            "release": release,
            "version": PDFIUM_VERSION if release == DEFAULT_PDFIUM_RELEASE else release,
            "asset": asset,
            "library": str(Path(*config["pdfium_dir"]) / library.name),
        }


def runtime_ffi_name(platform_id: str) -> str:
    if platform_id.startswith("windows-"):
        return "uocr-ffi.dll"
    if platform_id.startswith("macos-"):
        return "libuocr-ffi.dylib"
    return "libuocr-ffi.so"


def native_runner_names(platform_id: str) -> list[str]:
    suffix = ".exe" if platform_id.startswith("windows-") else ""
    return [
        f"trapo-tesseract-rs-runner{suffix}",
    ]


def ensure_runtime(
    platform_id: str, args: argparse.Namespace, *, optional: bool = False
) -> Path | None:
    runtime_dir = REPO_ROOT / "thirdparty" / "uocr-runtime" / platform_id
    ffi = runtime_dir / "bin" / runtime_ffi_name(platform_id)
    if not ffi.exists() and not args.no_runtime_download:
        command = [
            sys.executable,
            str(REPO_ROOT / "scripts" / "install_runtime.py"),
            "install",
            "--repo-root",
            str(REPO_ROOT),
            "--install-dir",
            str(REPO_ROOT / "thirdparty" / "uocr-runtime"),
            "--runtime-repo",
            args.runtime_repo,
            "--runtime-version",
            args.runtime_version,
            "--platform",
            platform_id,
            "--skip-accelerator-probe",
        ]
        try:
            run(command)
        except subprocess.CalledProcessError:
            if optional:
                print(
                    f"warning: optional runtime {platform_id} was not available",
                    file=sys.stderr,
                )
                return None
            raise
    if not ffi.exists():
        if optional:
            print(
                f"warning: optional runtime FFI library is missing: {ffi}",
                file=sys.stderr,
            )
            return None
        die(f"runtime FFI library is missing: {ffi}")
    return runtime_dir


def build_outputs(args: argparse.Namespace) -> None:
    if args.no_build:
        return
    env = os.environ.copy()
    env["DUCKDB_DOWNLOAD_LIB"] = "1"
    env["TRAPO_GIT_TAG"] = args.version
    env["TRAPO_GIT_SHA"] = git_output(["rev-parse", "--short=12", "HEAD"]) or "unknown"
    run(
        ["bun", "install", "--frozen-lockfile", "--ignore-scripts"],
        cwd=REPO_ROOT / "src" / "trapo-client",
    )
    run(["bun", "run", "build"], cwd=REPO_ROOT / "src" / "trapo-client")
    run(
        ["cargo", "build", "-p", "trapo-server", "-p", "trapo-native-runners", "--release"], env=env
    )


def stage_native_runners(runtime_dir: Path, platform_id: str, *, required: bool) -> None:
    bin_dir = runtime_dir / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    for name in native_runner_names(platform_id):
        source = REPO_ROOT / "target" / "release" / name
        if not source.exists():
            message = f"native runner binary is missing: {source}"
            if required:
                die(message)
            print(f"warning: {message}", file=sys.stderr)
            continue
        destination = bin_dir / name
        shutil.copy2(source, destination)
        if not platform_id.startswith("windows-"):
            destination.chmod(
                destination.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH
            )


def ensure_trapo_ocr_ffi(runtime_dir: Path, platform_id: str, *, required: bool) -> None:
    shared_bin = runtime_dir / "bin"
    if required:
        run(
            [
                sys.executable,
                str(REPO_ROOT / "scripts" / "build_trapo_ocr_ffi.py"),
                "--platform",
                platform_id,
                "--build-dir",
                str(REPO_ROOT / "target" / "trapo-ocr-ffi" / platform_id),
                "--output-dir",
                str(shared_bin),
                "--required",
            ]
        )
        return
    if any((shared_bin / name).is_file() for name in ppocrv6_ffi_names(platform_id)):
        return
    die(f"PP-OCRv6 native FFI is missing and --no-build prevents staging it: {shared_bin}")


def ensure_onnxruntime_files(runtime_dir: Path, platform_id: str) -> None:
    shared_bin = runtime_dir / "bin"
    shared_bin.mkdir(parents=True, exist_ok=True)
    stage_onnxruntime_files(shared_bin, platform_id, prepare_onnxruntime_deps)


def ensure_ppocrv6_engine(runtime_dir: Path, platform_id: str) -> None:
    engine_dir = runtime_dir / "ppocrv6"
    remove_stale_ppocrv6_python_assets(engine_dir)
    if not (engine_dir / "models" / "manifest.json").is_file():
        run(
            [
                sys.executable,
                str(REPO_ROOT / "scripts" / "install_ppocrv6_runtime.py"),
                "--output-dir",
                str(engine_dir),
            ]
        )
    shared_bin = runtime_dir / "bin"
    if not any(
        (shared_bin / name).is_file() or (engine_dir / "bin" / name).is_file()
        for name in ppocrv6_ffi_names(platform_id)
    ):
        expected = ", ".join(ppocrv6_ffi_names(platform_id))
        raise SystemExit(f"PP-OCRv6 native FFI is missing under {shared_bin}: {expected}")


def ensure_paddleocr_vl_engine(runtime_dir: Path) -> None:
    engine_dir = runtime_dir / "paddleocr_vl_1_6"
    required = [
        engine_dir / "manifest.json",
        engine_dir / "layout_detection" / "inference.onnx",
        engine_dir / "layout_detection" / "inference.yml",
    ]
    if all(path.is_file() for path in required):
        return
    run(
        [
            sys.executable,
            str(REPO_ROOT / "scripts" / "install_paddleocr_vl_runtime.py"),
            "--output-dir",
            str(engine_dir),
        ]
    )


def ppocrv6_ffi_names(platform_id: str) -> list[str]:
    if platform_id.startswith("windows-"):
        return ["trapo-ocr-ffi.dll"]
    if platform_id.startswith("macos-"):
        return ["libtrapo-ocr-ffi.dylib"]
    return ["libtrapo-ocr-ffi.so"]


def remove_stale_ppocrv6_python_assets(engine_dir: Path) -> None:
    if not engine_dir.exists():
        return
    candidates = [
        *(path for path in engine_dir.rglob("build") if path.is_dir()),
        *(path for path in engine_dir.rglob(".paddlex") if path.is_dir()),
        *(path for path in engine_dir.rglob("ppocrv6") if path.is_dir()),
        *(path for path in engine_dir.rglob("__pycache__") if path.is_dir()),
        *(path for path in engine_dir.rglob("trapo_ppocrv6_engine*")),
        *engine_dir.rglob("trapo_ppocrv6_engine.py"),
        *engine_dir.rglob(".venv"),
        *engine_dir.rglob("*.py"),
        *engine_dir.rglob("*.pyc"),
        *engine_dir.rglob("*.pyo"),
        *engine_dir.rglob("*.pyd"),
        *engine_dir.rglob("*.spec"),
    ]
    for path in sorted(set(candidates), key=lambda item: len(item.parts), reverse=True):
        remove_generated_runtime_path(path, engine_dir)


def remove_generated_runtime_path(path: Path, root: Path) -> None:
    if not path.exists():
        return
    resolved_root = root.resolve()
    resolved_path = path.resolve()
    if resolved_root not in (resolved_path, *resolved_path.parents):
        die(f"refusing to remove PP-OCRv6 asset outside runtime root: {path}")
    if resolved_path.is_dir():
        shutil.rmtree(
            resolved_path
        )  # skylos: ignore[SKY-D215] bounded stale generated runtime directory.
    else:
        resolved_path.unlink()
    print(f"Removed stale PP-OCRv6 Python runtime asset: {resolved_path}", flush=True)


def ensure_tesseract_engine(runtime_dir: Path, platform_id: str) -> None:
    engine_dir = runtime_dir / "tesseract"
    binary = "tesseract.exe" if platform_id.startswith("windows-") else "tesseract"
    if (engine_dir / "bin" / binary).is_file() and (
        engine_dir / "tessdata" / "eng.traineddata"
    ).is_file():
        return
    run(
        [
            sys.executable,
            str(REPO_ROOT / "scripts" / "install_tesseract_runtime.py"),
            "--output-dir",
            str(engine_dir),
        ]
    )


def copy_tree(source: Path, destination: Path) -> None:
    if destination.exists():
        shutil.rmtree(
            destination
        )  # skylos: ignore[SKY-D215] destination is a deterministic package stage child.
    shutil.copytree(source, destination)


def fix_macos_server_rpath(stage_root: Path, platform_id: str) -> None:
    binary = stage_root / "trapo-server"
    if platform_id.startswith("macos-") and "@executable_path" not in subprocess.check_output(
        ["otool", "-l", str(binary)], text=True
    ):
        run(
            ["install_name_tool", "-add_rpath", "@executable_path", str(binary)],
            cwd=stage_root,
        )


def make_launcher(stage_root: Path, platform_id: str) -> None:
    if platform_id.startswith("windows-"):
        (
            stage_root / "trapo-server.cmd"
        ).write_text(  # skylos: ignore[SKY-D324] fixed package launcher path.
            "@echo off\r\nsetlocal\r\nset TRAPO_HOME=%~dp0\r\n"
            'for /D %%D in ("%~dp0thirdparty\\uocr-runtime\\*") do '
            'if exist "%%~fD\\bin" set "PATH=%%~fD\\bin;%PATH%"\r\n'
            '"%~dp0trapo-server.exe" %*\r\n',
            encoding="ascii",
        )
        return
    launcher = stage_root / "trapo-server.sh"
    lib_var = "DYLD_LIBRARY_PATH" if platform_id.startswith("macos-") else "LD_LIBRARY_PATH"
    launcher.write_text(  # skylos: ignore[SKY-D324] fixed package launcher path.
        "#!/usr/bin/env bash\n"
        "set -euo pipefail\n"
        'cd "$(dirname "$0")"\n'
        'runtime_lib_path=$(find "$PWD/thirdparty/uocr-runtime" '
        "-mindepth 2 -maxdepth 2 -type d -name bin -print 2>/dev/null | "
        "paste -sd: -)\n"
        f'export {lib_var}="${{runtime_lib_path:+$runtime_lib_path:}}'
        f'$PWD/thirdparty/pdfium/lib:$PWD:${{{lib_var}:-}}"\n'
        'exec ./trapo-server "$@"\n',
        encoding="utf-8",
    )
    launcher.chmod(launcher.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def create_archive(stage_root: Path, archive_path: Path) -> None:
    archive_path.parent.mkdir(parents=True, exist_ok=True)
    if archive_path.suffix == ".zip":
        with zipfile.ZipFile(archive_path, "w", zipfile.ZIP_DEFLATED) as zipf:
            for path in stage_root.rglob("*"):
                zipf.write(path, path.relative_to(stage_root.parent))
        return
    with tarfile.open(archive_path, "w:gz") as tar:
        tar.add(stage_root, arcname=stage_root.name)


def write_readme(stage_root: Path, args: argparse.Namespace, runtimes: list[str]) -> None:
    (stage_root / "README.txt").write_text(  # skylos: ignore[SKY-D324] fixed package README path.
        f"""Trapo Workbench {args.version}

Run trapo-server to start the local Axum backend and hosted React app.
Default URL: http://127.0.0.1:8765/
Logs: logs/trapo-server.log
PDF support: bundled PDFium {PDFIUM_VERSION} through PDFium-rs.
Primary runtime: {args.runtime_platform} from {args.runtime_repo} {args.runtime_version}.
Bundled runtimes: {", ".join(runtimes)}.
Optional authenticated model downloads: set HF_TOKEN before launching.
Uninstall: delete this folder.
""",
        encoding="utf-8",
    )


def ensure_no_python_runtime_files(stage_root: Path) -> None:
    forbidden = sorted(
        str(path.relative_to(stage_root)).replace("\\", "/")
        for path in stage_root.rglob("*")
        if is_forbidden_runtime_path(str(path.relative_to(stage_root)))
    )
    if forbidden:
        die(
            "workbench package contains forbidden Python runtime files: "
            + ", ".join(forbidden[:10])
        )


def package(args: argparse.Namespace) -> None:
    if args.platform not in PLATFORMS:
        die(f"unknown platform: {args.platform}")

    args.runtime_platform = canonical_runtime_platform(args.runtime_platform)
    if args.additional_runtime_platforms:
        args.additional_runtime_platforms = ",".join(
            canonical_runtime_platform(item)
            for item in csv(args.additional_runtime_platforms)
            if item
        )

    build_outputs(args)
    config = PLATFORMS[args.platform]
    output_dir = args.output_dir.resolve()
    name = f"trapo-workbench-{args.platform}-{safe_version(args.version)}"
    stage_root = output_dir / name
    archive = output_dir / f"{name}.{config['archive_ext']}"
    sha_path = Path(f"{archive}.sha256")
    for path in (stage_root, archive, sha_path):
        if path.is_dir():
            shutil.rmtree(path)  # skylos: ignore[SKY-D215] deterministic package output.
        else:
            path.unlink(missing_ok=True)
    stage_root.mkdir(parents=True)

    release_dir = REPO_ROOT / "target" / "release"
    for file_name in (config["server"], config["duckdb"]):
        source = release_dir / file_name
        if not source.exists():
            die(f"built runtime file is missing: {source}")
        shutil.copy2(source, stage_root / file_name)
    if not args.platform.startswith("windows-"):
        (stage_root / config["server"]).chmod(0o755)
    fix_macos_server_rpath(stage_root, args.platform)

    web_dist = REPO_ROOT / "src" / "trapo-client" / "dist"
    if not (web_dist / "index.html").exists():
        die(f"React build was not found at {web_dist}")
    copy_tree(web_dist, stage_root / "web")
    openapi_dir = stage_root / "openapi"
    openapi_dir.mkdir()
    shutil.copy2(
        REPO_ROOT / "src" / "trapo-server" / "openapi" / "trapo.openapi.json",
        openapi_dir,
    )

    runtime_platforms = list(
        dict.fromkeys([args.runtime_platform, *csv(args.additional_runtime_platforms)])
    )
    copied_runtimes: list[str] = []
    runtime_stage = stage_root / "thirdparty" / "uocr-runtime"
    runtime_stage.mkdir(parents=True)
    for index, platform_id in enumerate(runtime_platforms):
        runtime_dir = ensure_runtime(platform_id, args, optional=index > 0)
        if runtime_dir is None:
            continue
        stage_native_runners(runtime_dir, platform_id, required=not args.no_build)
        ensure_trapo_ocr_ffi(runtime_dir, platform_id, required=not args.no_build)
        ensure_onnxruntime_files(runtime_dir, platform_id)
        ensure_ppocrv6_engine(runtime_dir, platform_id)
        ensure_paddleocr_vl_engine(runtime_dir)
        ensure_tesseract_engine(runtime_dir, platform_id)
        copy_tree(runtime_dir, runtime_stage / platform_id)
        copied_runtimes.append(platform_id)

    pdfium = install_pdfium(args.platform, stage_root, args.pdfium_release)
    for directory in ("models", "data", "cache", "logs", "config", "uploads"):
        (stage_root / directory).mkdir()
    make_launcher(stage_root, args.platform)
    write_readme(stage_root, args, copied_runtimes)
    manifest = {
        "schema_version": 1,
        "name": "trapo-workbench",
        "version": args.version,
        "platform": args.platform,
        "runtime_platform": args.runtime_platform,
        "runtime_platforms": copied_runtimes,
        "runtime_version": args.runtime_version,
        "pdf_renderer": "pdfium-rs",
        "pdfium": pdfium,
        "created_at": dt.datetime.now(dt.UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
    }
    (stage_root / "install-manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    ensure_no_python_runtime_files(stage_root)
    create_archive(stage_root, archive)
    sha_path.write_text(  # skylos: ignore[SKY-D324] deterministic checksum path.
        f"{sha256_file(archive)}  {archive.name}\n", encoding="ascii"
    )
    print(f"Packaged {archive}")
    print(f"Checksum {sha_path}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Package Trapo Workbench release artifacts.")
    parser.add_argument(
        "--version",
        default=git_output(["describe", "--tags", "--dirty", "--always"]) or "0.0.0-dev",
    )
    parser.add_argument("--platform", required=True, choices=sorted(PLATFORMS))
    parser.add_argument("--runtime-version", default="latest")
    parser.add_argument("--runtime-repo", default="bangonkali/baidu-unlimited-ocr-portable")
    parser.add_argument(
        "--runtime-platform",
        required=True,
        help="Canonical runtime platform label, e.g. macos-arm64-metal. "
        "Aliases such as macos-arm64/windows-x64 are accepted for compatibility.",
    )
    parser.add_argument("--additional-runtime-platforms", default="")
    parser.add_argument("--pdfium-release", default=DEFAULT_PDFIUM_RELEASE)
    parser.add_argument("--output-dir", type=Path, default=REPO_ROOT / "dist")
    parser.add_argument("--no-build", action="store_true")
    parser.add_argument("--no-runtime-download", action="store_true")
    package(parser.parse_args())


if __name__ == "__main__":
    main()
