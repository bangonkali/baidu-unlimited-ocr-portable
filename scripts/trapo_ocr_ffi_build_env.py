from __future__ import annotations

import os

PORTABLE_LLAMA_BACKENDS = {
    "TRAPO_LLAMA_ENABLE_CUDA": "0",
    "TRAPO_LLAMA_ENABLE_VULKAN": "0",
    "TRAPO_LLAMA_ENABLE_OPENCL": "0",
}
# Matches runtime/platforms.json and build-runtime.yml GPU-less CUDA 13 fallback.
PORTABLE_CUDA_ARCHITECTURES = "75-virtual;80-virtual;86-real;89-real;90-virtual;120a-real;121a-real"
TRUTHY_ENV_VALUES = {"1", "ON", "TRUE", "YES"}


def is_cuda13_platform(platform: str) -> bool:
    return "cuda13" in platform.lower()


def portable_build_env(platform: str) -> dict[str, str]:
    env = os.environ.copy()
    for name, value in llama_backend_defaults(platform).items():
        env.setdefault(name, value)
    if is_cuda13_platform(platform):
        if "CUDA_ARCHITECTURES" in env:
            env.setdefault("TRAPO_CUDA_ARCHITECTURES", env["CUDA_ARCHITECTURES"])
        env.setdefault("TRAPO_CUDA_ARCHITECTURES", PORTABLE_CUDA_ARCHITECTURES)
    return env


def llama_backend_defaults(platform: str) -> dict[str, str]:
    defaults = dict(PORTABLE_LLAMA_BACKENDS)
    if is_cuda13_platform(platform):
        defaults["TRAPO_LLAMA_ENABLE_CUDA"] = "1"
    return defaults
