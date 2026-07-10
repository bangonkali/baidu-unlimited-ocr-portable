from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import nvidia_redist_staging  # noqa: E402
import windows_runtime_staging  # noqa: E402


class NvidiaRedistStagingTests(unittest.TestCase):
    def test_windows_cuda_and_cudnn_files_are_staged_with_notices(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            cuda_root = root / "cuda"
            cuda_bin = cuda_root / "bin"
            cuda_bin.mkdir(parents=True)
            (cuda_bin / "cudart64_13.dll").write_bytes(b"cuda")
            (cuda_root / "EULA.txt").write_text("CUDA EULA", encoding="utf-8")

            cudnn_root = root / "cudnn"
            cudnn_bin = cudnn_root / "nvidia" / "cudnn" / "bin"
            cudnn_bin.mkdir(parents=True)
            (cudnn_bin / "cudnn64_9.dll").write_bytes(b"cudnn")
            license_dir = cudnn_root / "package.dist-info" / "licenses"
            license_dir.mkdir(parents=True)
            (license_dir / "License.txt").write_text("cuDNN license", encoding="utf-8")

            manifest = {
                "schema_version": 1,
                "notice_names": {
                    "cuda": "NVIDIA-CUDA-EULA.txt",
                    "cudnn": "NVIDIA-cuDNN-LICENSE.txt",
                },
                "targets": {
                    "windows-x86_64-cuda13": {
                        "cuda_search_dirs": ["bin"],
                        "cuda_patterns": ["cudart64_13.dll"],
                        "cuda_notice_candidates": ["EULA.txt"],
                        "cudnn": {
                            "version": "9.test",
                            "archive": "cudnn.whl",
                            "url": "https://files.pythonhosted.org/cudnn.whl",
                            "sha256": "0" * 64,
                            "library_pattern": "**/nvidia/cudnn/bin/cudnn*.dll",
                            "license_pattern": "**/*.dist-info/licenses/License.txt",
                        },
                        "required_libraries": [
                            "cudart64_13.dll",
                            "cudnn64_9.dll",
                        ],
                    }
                },
            }
            manifest_path = root / "nvidia-redist.json"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            output = root / "output"

            with (
                mock.patch.dict(
                    nvidia_redist_staging.os.environ,
                    {"CUDA_PATH": str(cuda_root)},
                    clear=True,
                ),
                mock.patch.object(
                    nvidia_redist_staging,
                    "prepare_cudnn",
                    return_value=cudnn_root,
                ),
            ):
                nvidia_redist_staging.stage_nvidia_redist(
                    output,
                    "windows-x86_64-cuda13",
                    manifest_path,
                )

            self.assertEqual((output / "cudart64_13.dll").read_bytes(), b"cuda")
            self.assertEqual((output / "cudnn64_9.dll").read_bytes(), b"cudnn")
            self.assertTrue((output / "NVIDIA-CUDA-EULA.txt").is_file())
            self.assertTrue((output / "NVIDIA-cuDNN-LICENSE.txt").is_file())

    def test_manifest_dependencies_match_cuda_runtime_targets(self) -> None:
        manifest = nvidia_redist_staging.load_manifest()
        platforms = json.loads(
            (REPO_ROOT / "runtime" / "platforms.json").read_text(encoding="utf-8")
        )
        for platform_id, target in manifest["targets"].items():
            runtime_target = platforms["targets"][platform_id]
            self.assertTrue(
                set(target["required_libraries"]).issubset(
                    runtime_target["bundled_dependency_libraries"]
                )
            )
            self.assertTrue(
                set(manifest["notice_names"].values()).issubset(
                    runtime_target["bundled_notice_files"]
                )
            )

    def test_non_cuda_platform_needs_no_nvidia_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            staged = nvidia_redist_staging.stage_nvidia_redist(
                Path(tmp),
                "windows-x86_64-cpu",
            )
        self.assertEqual(staged, [])

    def test_windows_msvc_runtime_is_staged_from_visual_studio_redist(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            crt = root / "x64" / "Microsoft.VC145.CRT"
            crt.mkdir(parents=True)
            for name in windows_runtime_staging.REQUIRED_MSVC_RUNTIME_DLLS:
                (crt / name).write_bytes(name.encode())
            output = root / "output"

            with mock.patch.object(
                windows_runtime_staging,
                "redist_roots",
                return_value=[root],
            ):
                staged = windows_runtime_staging.stage_windows_runtime(
                    output,
                    "windows-x86_64-cuda13",
                )

            self.assertEqual(len(staged), 3)
            for name in windows_runtime_staging.REQUIRED_MSVC_RUNTIME_DLLS:
                self.assertTrue((output / name).is_file())


if __name__ == "__main__":
    unittest.main()
