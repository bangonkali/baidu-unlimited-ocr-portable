#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import platform
import subprocess
import shutil
import sys
import tarfile
import tempfile
import urllib.error
import urllib.request
import zipfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from package_runtime import REPO_ROOT, load_platforms, sha256_file


USER_AGENT = "baidu-unlimited-ocr-portable-runtime-installer"


@dataclass(frozen=True)
class DetectedPlatform:
    platform_id: str | None
    os_name: str
    arch: str
    accelerator_ok: bool
    accelerator_detail: str
    supported: bool
    reason: str


def die(message: str) -> None:
    raise SystemExit(f"error: {message}")


def eprint(message: str) -> None:
    print(message, file=sys.stderr)


def normalize_arch(raw: str) -> str:
    value = raw.lower()
    if value in {"amd64", "x64"}:
        return "x86_64"
    if value in {"aarch64", "arm64"}:
        return "arm64"
    return value


def normalize_os() -> str:
    system = platform.system().lower()
    if system == "darwin":
        return "macos"
    if system == "windows":
        return "windows"
    if system == "linux":
        return "linux"
    return system


def parse_compute_capability(value: str) -> tuple[int, int] | None:
    parts = value.strip().split(".", 1)
    if len(parts) != 2:
        return None
    try:
        return int(parts[0]), int(parts[1])
    except ValueError:
        return None


def compute_capability_at_least(actual: str, minimum: str) -> bool:
    actual_parts = parse_compute_capability(actual)
    minimum_parts = parse_compute_capability(minimum)
    if actual_parts is None or minimum_parts is None:
        return False
    return actual_parts >= minimum_parts


def query_nvidia_compute_caps(command_path: str) -> tuple[list[tuple[str, str]], str]:
    try:
        output = subprocess.check_output(
            [
                command_path,
                "--query-gpu=name,compute_cap",
                "--format=csv,noheader",
            ],
            text=True,
            stderr=subprocess.STDOUT,
            timeout=15,
        )
    except Exception as exc:
        return [], f"compute capability query unavailable: {exc}"

    gpus: list[tuple[str, str]] = []
    for line in output.splitlines():
        if not line.strip():
            continue
        if "," in line:
            name, cap = [part.strip() for part in line.split(",", 1)]
        else:
            name, cap = line.strip(), ""
        if cap:
            gpus.append((name, cap))
    return gpus, ""


def probe_accelerator(command: str | None, target: dict[str, Any] | None = None) -> tuple[bool, str]:
    if not command:
        return True, "no accelerator probe required"
    resolved = shutil.which(command)
    if not resolved:
        return False, f"{command} not found on PATH"
    target = target or {}
    minimum_compute_capability = target.get("minimum_compute_capability")
    if command == "nvidia-smi" and minimum_compute_capability:
        gpus, query_error = query_nvidia_compute_caps(resolved)
        if gpus:
            parseable = [
                f"{name} ({cap})"
                for name, cap in gpus
                if parse_compute_capability(cap) is not None
            ]
            supported = [
                f"{name} (compute capability {cap})"
                for name, cap in gpus
                if compute_capability_at_least(cap, minimum_compute_capability)
            ]
            if supported:
                return True, f"{resolved}; supported GPU: {supported[0]}"
            if not parseable:
                found = ", ".join(f"{name} ({cap})" for name, cap in gpus)
                return True, f"{resolved}; compute capability query returned no parseable values: {found}"
            found = ", ".join(f"{name} ({cap})" for name, cap in gpus)
            return False, (
                f"CUDA target requires compute capability >= {minimum_compute_capability}; "
                f"detected {found}"
            )
        return True, f"{resolved}; {query_error}"
    return True, resolved


def detect_platform(repo_root: Path, requested_platform: str | None = None) -> DetectedPlatform:
    platforms = load_platforms(repo_root)
    targets = platforms["targets"]
    os_name = normalize_os()
    arch = normalize_arch(platform.machine())

    if requested_platform:
        target = targets.get(requested_platform)
        if not target:
            return DetectedPlatform(
                platform_id=requested_platform,
                os_name=os_name,
                arch=arch,
                accelerator_ok=False,
                accelerator_detail="",
                supported=False,
                reason=f"unknown platform label: {requested_platform}",
            )
        if target["os"] != os_name or target["arch"] != arch:
            return DetectedPlatform(
                platform_id=requested_platform,
                os_name=os_name,
                arch=arch,
                accelerator_ok=False,
                accelerator_detail="",
                supported=False,
                reason=f"requested {requested_platform}, but detected {os_name}/{arch}",
            )
        accelerator_ok, accelerator_detail = probe_accelerator(target.get("accelerator_probe"), target)
        return DetectedPlatform(
            platform_id=requested_platform,
            os_name=os_name,
            arch=arch,
            accelerator_ok=accelerator_ok,
            accelerator_detail=accelerator_detail,
            supported=accelerator_ok,
            reason="supported" if accelerator_ok else accelerator_detail,
        )

    candidates = [
        (platform_id, target)
        for platform_id, target in targets.items()
        if target["os"] == os_name and target["arch"] == arch
    ]
    if not candidates:
        return DetectedPlatform(
            platform_id=None,
            os_name=os_name,
            arch=arch,
            accelerator_ok=False,
            accelerator_detail="",
            supported=False,
            reason=f"no supported runtime label for {os_name}/{arch}",
        )

    platform_id, target = candidates[0]
    accelerator_ok, accelerator_detail = probe_accelerator(target.get("accelerator_probe"), target)
    return DetectedPlatform(
        platform_id=platform_id,
        os_name=os_name,
        arch=arch,
        accelerator_ok=accelerator_ok,
        accelerator_detail=accelerator_detail,
        supported=accelerator_ok,
        reason="supported" if accelerator_ok else accelerator_detail,
    )


def request_json(url: str) -> dict[str, Any]:
    headers = {
        "Accept": "application/vnd.github+json",
        "User-Agent": USER_AGENT,
        "X-GitHub-Api-Version": "2022-11-28",
    }
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(req, timeout=60) as response:
        return json.loads(response.read().decode("utf-8"))


def download_url(url: str, output_path: Path) -> None:
    headers = {"User-Agent": USER_AGENT}
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if token and "github.com" in url:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(req, timeout=300) as response, output_path.open("wb") as fh:
        shutil.copyfileobj(response, fh)


def github_release(runtime_repo: str, runtime_version: str) -> dict[str, Any]:
    repo = runtime_repo.strip("/")
    if "/" not in repo:
        die(f"--runtime-repo must be OWNER/REPO, got: {runtime_repo}")
    if runtime_version == "latest":
        url = f"https://api.github.com/repos/{repo}/releases/latest"
    else:
        url = f"https://api.github.com/repos/{repo}/releases/tags/{runtime_version}"
    try:
        return request_json(url)
    except urllib.error.HTTPError as exc:
        die(f"could not find GitHub release {runtime_version!r} in {repo}: HTTP {exc.code}")
    except urllib.error.URLError as exc:
        die(f"could not reach GitHub Releases for {repo}: {exc.reason}")


def assets_by_name(release: dict[str, Any]) -> dict[str, dict[str, Any]]:
    return {asset["name"]: asset for asset in release.get("assets", [])}


def parse_sha256_text(text: str) -> str:
    for line in text.splitlines():
        line = line.strip()
        if not line:
            continue
        return line.split()[0]
    die("empty sha256 asset")


def release_platform_entry(
    *,
    repo_root: Path,
    release: dict[str, Any],
    assets: dict[str, dict[str, Any]],
    aggregate_manifest_name: str,
    platform_id: str,
    runtime_repo: str,
) -> dict[str, Any]:
    manifest_asset = assets.get(aggregate_manifest_name)
    if manifest_asset:
        with tempfile.NamedTemporaryFile(prefix="uocr-release-manifest-", delete=False) as tmp:
            tmp_path = Path(tmp.name)
        try:
            download_url(manifest_asset["browser_download_url"], tmp_path)
            manifest = json.loads(tmp_path.read_text(encoding="utf-8"))
        finally:
            tmp_path.unlink(missing_ok=True)
        entry = manifest.get("platforms", {}).get(platform_id)
        if entry:
            return entry

    tag = release.get("tag_name") or release.get("name") or "unknown"
    platforms = load_platforms(repo_root)
    target = platforms["targets"][platform_id]
    archive_name = f"{platforms['asset_prefix']}-{platform_id}-{tag}.{target['archive_ext']}"
    sha_name = f"{archive_name}.sha256"
    archive_asset = assets.get(archive_name)
    sha_asset = assets.get(sha_name)
    if not archive_asset:
        die(f"release has no runtime asset for {platform_id}; expected {archive_name}")
    if not sha_asset:
        die(f"release has no checksum asset for {platform_id}; expected {sha_name}")

    with tempfile.NamedTemporaryFile(prefix="uocr-runtime-sha-", delete=False) as tmp:
        tmp_path = Path(tmp.name)
    try:
        download_url(sha_asset["browser_download_url"], tmp_path)
        expected_hash = parse_sha256_text(tmp_path.read_text(encoding="utf-8"))
    finally:
        tmp_path.unlink(missing_ok=True)

    return {
        "platform": platform_id,
        "version": tag,
        "archive_name": archive_name,
        "archive_sha256": expected_hash,
        "archive_size": archive_asset.get("size"),
        "layout": {
            "root": f"uocr-runtime-{platform_id}-{tag}",
            "bin_dir": "bin",
            "primary_binary": f"bin/{target['primary_binary']}",
        },
        "target": {
            "os": target["os"],
            "arch": target["arch"],
            "backend": target["backend"],
            "cuda_major": target.get("cuda_major"),
            "cuda_architectures": target.get("cuda_architectures"),
            "minimum_compute_capability": target.get("minimum_compute_capability"),
            "known_gpu_support": target.get("known_gpu_support", []),
            "support_status": target["support_status"],
        },
        "asset_runtime_repo": runtime_repo,
    }


def safe_extract_tar(archive: Path, destination: Path) -> None:
    destination_resolved = destination.resolve()
    with tarfile.open(archive, "r:*") as tar:
        for member in tar.getmembers():
            target = (destination / member.name).resolve()
            try:
                target.relative_to(destination_resolved)
            except ValueError:
                die(f"refusing to extract archive member outside destination: {member.name}")
        tar.extractall(destination)


def safe_extract_zip(archive: Path, destination: Path) -> None:
    destination_resolved = destination.resolve()
    with zipfile.ZipFile(archive) as zipf:
        for member in zipf.namelist():
            target = (destination / member).resolve()
            try:
                target.relative_to(destination_resolved)
            except ValueError:
                die(f"refusing to extract archive member outside destination: {member}")
        zipf.extractall(destination)


def extract_archive(archive: Path, destination: Path) -> Path:
    destination.mkdir(parents=True, exist_ok=True)
    before = {item.name for item in destination.iterdir()} if destination.exists() else set()
    if archive.name.endswith(".zip"):
        safe_extract_zip(archive, destination)
    else:
        safe_extract_tar(archive, destination)
    after = [item for item in destination.iterdir() if item.name not in before]
    if len(after) == 1 and after[0].is_dir():
        return after[0]
    if after:
        return destination / sorted(item.name for item in after)[0]
    return destination


def installed_paths(root: Path, entry: dict[str, Any]) -> dict[str, str]:
    primary = root / entry["layout"]["primary_binary"]
    bin_dir = primary.parent
    paths = {
        "UOCR_RUNTIME_LABEL": entry["platform"],
        "UOCR_RUNTIME_SOURCE": "download",
        "UOCR_RUNTIME_VERSION": entry.get("version", ""),
        "UOCR_RUNTIME_ROOT": str(root),
        "UOCR_LLAMA_BIN": str(primary),
    }
    mtmd = bin_dir / ("llama-mtmd-cli.exe" if os.name == "nt" else "llama-mtmd-cli")
    server = bin_dir / ("llama-server.exe" if os.name == "nt" else "llama-server")
    ffi_layout = entry.get("layout", {}).get("ffi_library")
    if ffi_layout:
        ffi_lib = root / ffi_layout
        if ffi_lib.exists():
            paths["UOCR_FFI_LIB"] = str(ffi_lib)
    if "UOCR_FFI_LIB" not in paths:
        ffi_names = ["uocr-ffi.dll", "libuocr-ffi.dll"] if os.name == "nt" else (
            ["libuocr-ffi.dylib"] if sys.platform == "darwin" else ["libuocr-ffi.so"]
        )
        for ffi_name in ffi_names:
            ffi_lib = bin_dir / ffi_name
            if ffi_lib.exists():
                paths["UOCR_FFI_LIB"] = str(ffi_lib)
                break
    if mtmd.exists():
        paths["UOCR_LLAMA_MTMD_BIN"] = str(mtmd)
    if server.exists():
        paths["UOCR_LLAMA_SERVER_BIN"] = str(server)
    return paths


def quote_sh(value: str) -> str:
    import shlex

    return shlex.quote(value)


def quote_ps(value: str) -> str:
    return "'" + value.replace("'", "''") + "'"


def emit_env(values: dict[str, str], style: str) -> None:
    if style == "json":
        print(json.dumps(values, indent=2, sort_keys=True))
        return
    if style == "sh":
        for key, value in values.items():
            print(f"export {key}={quote_sh(value)}")
        return
    if style == "powershell":
        for key, value in values.items():
            print(f"$env:{key} = {quote_ps(value)}")
        return
    die(f"unknown env output style: {style}")


def install_runtime(args: argparse.Namespace) -> None:
    repo_root = args.repo_root.resolve()
    platforms = load_platforms(repo_root)
    aggregate_manifest = platforms["aggregate_manifest"]
    runtime_repo = args.runtime_repo or platforms["default_release_repo"]

    detected = detect_platform(repo_root, args.platform)
    if not detected.supported or not detected.platform_id:
        die(f"unsupported runtime platform: {detected.reason}")

    release = github_release(runtime_repo, args.runtime_version)
    assets = assets_by_name(release)
    entry = release_platform_entry(
        repo_root=repo_root,
        release=release,
        assets=assets,
        aggregate_manifest_name=aggregate_manifest,
        platform_id=detected.platform_id,
        runtime_repo=runtime_repo,
    )
    archive_name = entry["archive_name"]
    archive_asset = assets.get(archive_name)
    if not archive_asset:
        die(f"release asset not found: {archive_name}")

    install_dir = args.install_dir.resolve() / detected.platform_id
    installed_manifest = install_dir / "manifest.json"
    if installed_manifest.exists() and not args.force:
        try:
            manifest = json.loads(installed_manifest.read_text(encoding="utf-8"))
            if manifest.get("archive_sha256") == entry.get("archive_sha256"):
                env = installed_paths(install_dir, manifest)
                ffi_env = env.get("UOCR_FFI_LIB")
                if Path(env["UOCR_LLAMA_BIN"]).exists() and ffi_env and Path(ffi_env).exists():
                    eprint(f"Using cached runtime: {install_dir}")
                    if args.print_env:
                        emit_env(env, args.print_env)
                    else:
                        print(f"Installed runtime: {install_dir}")
                    return
        except Exception:
            pass

    download_dir = args.install_dir.resolve() / "_downloads"
    download_dir.mkdir(parents=True, exist_ok=True)
    archive_path = download_dir / archive_name
    eprint(f"Downloading {archive_name} from {runtime_repo}...")
    download_url(archive_asset["browser_download_url"], archive_path)

    actual_hash = sha256_file(archive_path)
    expected_hash = entry.get("archive_sha256")
    if expected_hash and actual_hash.lower() != str(expected_hash).lower():
        archive_path.unlink(missing_ok=True)
        die(f"checksum mismatch for {archive_name}: expected {expected_hash}, got {actual_hash}")

    if install_dir.exists():
        shutil.rmtree(install_dir)
    with tempfile.TemporaryDirectory(prefix="uocr-runtime-extract-") as tmp:
        extracted_root = extract_archive(archive_path, Path(tmp))
        shutil.move(str(extracted_root), str(install_dir))

    entry["archive_sha256"] = actual_hash
    entry["installed_from_release"] = release.get("tag_name") or args.runtime_version
    entry["installed_from_repo"] = runtime_repo
    installed_manifest.write_text(json.dumps(entry, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    env = installed_paths(install_dir, entry)
    primary = Path(env["UOCR_LLAMA_BIN"])
    if not primary.exists():
        die(f"installed runtime is missing primary binary: {primary}")
    if os.name != "nt":
        for exe_key in ("UOCR_LLAMA_BIN", "UOCR_LLAMA_MTMD_BIN", "UOCR_LLAMA_SERVER_BIN"):
            exe = env.get(exe_key)
            if exe and Path(exe).exists():
                mode = Path(exe).stat().st_mode
                Path(exe).chmod(mode | 0o755)

    eprint(f"Installed runtime: {install_dir}")
    if args.print_env:
        emit_env(env, args.print_env)
    else:
        print(f"Installed runtime: {install_dir}")


def detect_command(args: argparse.Namespace) -> None:
    repo_root = args.repo_root.resolve()
    detected = detect_platform(repo_root, args.platform)
    payload = {
        "platform": detected.platform_id,
        "os": detected.os_name,
        "arch": detected.arch,
        "accelerator_ok": detected.accelerator_ok,
        "accelerator_detail": detected.accelerator_detail,
        "supported": detected.supported,
        "reason": detected.reason,
    }
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        status = "supported" if detected.supported else "unsupported"
        print(f"{status}: {detected.platform_id or detected.os_name + '-' + detected.arch} ({detected.reason})")


def main() -> None:
    parser = argparse.ArgumentParser(description="Install Unlimited-OCR prebuilt native runtimes.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    install_parser = subparsers.add_parser("install", help="Download and install a GitHub Release runtime.")
    install_parser.add_argument("--repo-root", type=Path, default=REPO_ROOT)
    install_parser.add_argument("--install-dir", type=Path, default=REPO_ROOT / "thirdparty" / "uocr-runtime")
    install_parser.add_argument("--runtime-repo", default="")
    install_parser.add_argument("--runtime-version", default="latest")
    install_parser.add_argument("--platform", default="")
    install_parser.add_argument("--force", action="store_true")
    install_parser.add_argument("--print-env", choices=["sh", "powershell", "json"], default="")
    install_parser.set_defaults(func=install_runtime)

    detect_parser = subparsers.add_parser("detect", help="Detect the exact supported runtime platform.")
    detect_parser.add_argument("--repo-root", type=Path, default=REPO_ROOT)
    detect_parser.add_argument("--platform", default="")
    detect_parser.add_argument("--json", action="store_true")
    detect_parser.set_defaults(func=detect_command)

    args = parser.parse_args()
    args.platform = args.platform or None
    args.func(args)


if __name__ == "__main__":
    main()
