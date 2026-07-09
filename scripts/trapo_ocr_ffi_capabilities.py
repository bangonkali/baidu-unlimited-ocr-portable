from __future__ import annotations

import ctypes
import json
from typing import Any

TRAPO_OCR_GEN_BACKEND_CUDA = 3


class TrapoOcrResult(ctypes.Structure):
    _fields_ = [
        ("struct_size", ctypes.c_size_t),
        ("json", ctypes.c_void_p),
        ("json_length", ctypes.c_size_t),
    ]


def trapo_ocr_runtime_capabilities(lib: ctypes.CDLL) -> dict[str, Any]:
    result_ptr = ctypes.POINTER(TrapoOcrResult)()
    lib.trapo_ocr_get_runtime_capabilities.argtypes = [
        ctypes.POINTER(ctypes.POINTER(TrapoOcrResult))
    ]
    lib.trapo_ocr_get_runtime_capabilities.restype = ctypes.c_int
    lib.trapo_ocr_free_result.argtypes = [ctypes.POINTER(TrapoOcrResult)]
    lib.trapo_ocr_free_result.restype = None
    lib.trapo_ocr_last_error.argtypes = []
    lib.trapo_ocr_last_error.restype = ctypes.c_char_p
    status = int(lib.trapo_ocr_get_runtime_capabilities(ctypes.byref(result_ptr)))
    if status != 0:
        error = (lib.trapo_ocr_last_error() or b"").decode("utf-8", errors="replace")
        raise SystemExit(f"trapo_ocr_get_runtime_capabilities failed: {error}")
    if not result_ptr:
        raise SystemExit("trapo_ocr_get_runtime_capabilities returned no result")
    try:
        result = result_ptr.contents
        text = ctypes.string_at(result.json, result.json_length).decode("utf-8", errors="replace")
        return json.loads(text)
    finally:
        lib.trapo_ocr_free_result(result_ptr)


def assert_generative_backend_compiled(
    capabilities: dict[str, Any], backend: str
) -> dict[str, Any]:
    backend_ids = {"cuda": TRAPO_OCR_GEN_BACKEND_CUDA}
    backend_id = backend_ids.get(backend.lower())
    if backend_id is None:
        raise SystemExit(f"unsupported required generative backend: {backend}")
    accelerators = capabilities.get("generativeAccelerators", [])
    match = next((item for item in accelerators if item.get("backend") == backend_id), None)
    if match is None:
        raise SystemExit(f"trapo-ocr-ffi did not report generative backend: {backend}")
    reason = str(match.get("unavailableReason", "")).lower()
    if "not compiled" in reason:
        raise SystemExit(f"trapo-ocr-ffi generative backend is not compiled: {backend}")
    return match
