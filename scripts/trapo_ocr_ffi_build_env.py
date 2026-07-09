from __future__ import annotations

import os

PORTABLE_LLAMA_BACKENDS = {
    "TRAPO_LLAMA_ENABLE_CUDA": "0",
    "TRAPO_LLAMA_ENABLE_VULKAN": "0",
    "TRAPO_LLAMA_ENABLE_OPENCL": "0",
}
TRUTHY_ENV_VALUES = {"1", "ON", "TRUE", "YES"}


def portable_build_env(platform: str) -> dict[str, str]:
    env = os.environ.copy()
    for name, value in llama_backend_defaults(platform).items():
        env.setdefault(name, value)
    if "cuda" in platform.lower() and "CUDA_ARCHITECTURES" in env:
        env.setdefault("TRAPO_CUDA_ARCHITECTURES", env["CUDA_ARCHITECTURES"])
    return env


def llama_backend_defaults(platform: str) -> dict[str, str]:
    _ = platform
    return dict(PORTABLE_LLAMA_BACKENDS)
