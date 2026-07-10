cmake_minimum_required(VERSION 3.18)

project(trapo_ocr_native LANGUAGES C CXX)

set(TRAPO_OCR_CORE_SOURCES
  src/trapo_ocr.cpp
  src/model/ocr_model_bundle.cpp
)

if((ANDROID AND DEFINED ORT_ROOT) OR WIN32)
  list(APPEND TRAPO_OCR_CORE_SOURCES
    third_party/clipper/ocr_clipper.cpp
    src/ort/ort_cuda_ep.cpp
    src/document_markdown/document_markdown_common.cpp
    src/document_markdown/document_markdown_image_processor.cpp
    src/document_markdown/document_markdown_engine.cpp
    src/document_markdown/document_markdown_result.cpp
    src/document_markdown/document_markdown_sampling.cpp
    src/document_markdown/document_markdown_onnx_runner.cpp
    src/document_markdown/document_markdown_tokenizer.cpp
    src/document_markdown/document_markdown_tokenizer_common.cpp
    src/document_markdown/document_markdown_tokenizer_json_scanner.cpp
    src/document_markdown/document_markdown_tokenizer_loader.cpp
    src/vl/layout_analyzer.cpp
    src/vl/llama_vlm_support.cpp
    src/vl/llama_vlm_runner.cpp
    src/vl/paddle_ocr_vl_engine.cpp
    src/vl/paddle_ocr_vl_result.cpp
  )
endif()

if(ANDROID AND DEFINED ORT_ROOT)
  list(APPEND TRAPO_OCR_CORE_SOURCES
    ../../android/src/main/cpp/trapo_ocr_jni.cpp
  )
endif()

add_library(trapo_ocr_native SHARED ${TRAPO_OCR_CORE_SOURCES})

set_target_properties(trapo_ocr_native PROPERTIES
  OUTPUT_NAME "trapo-ocr-ffi")

target_include_directories(trapo_ocr_native
  PUBLIC
    ${CMAKE_CURRENT_LIST_DIR}/include
  PRIVATE
    ${CMAKE_CURRENT_LIST_DIR}/src
)

target_compile_features(trapo_ocr_native PUBLIC cxx_std_17)

if(MSVC)
  target_compile_options(trapo_ocr_native PRIVATE /W4)
else()
  target_compile_options(trapo_ocr_native PRIVATE -Wall -Wextra -Wpedantic)
endif()

if((ANDROID AND DEFINED ORT_ROOT) OR WIN32)
  target_compile_definitions(trapo_ocr_native PRIVATE TRAPO_OCR_ENABLE_NATIVE_PIPELINE)

  target_include_directories(trapo_ocr_native PRIVATE
    ${CMAKE_CURRENT_LIST_DIR}/third_party/clipper
  )

  set(TRAPO_LLAMA_CPP_ROOT
    "${CMAKE_CURRENT_LIST_DIR}/../../.deps/llama.cpp"
    CACHE PATH "Pinned llama.cpp checkout created by tool/fetch_llama_cpp.dart")
  if(EXISTS "${TRAPO_LLAMA_CPP_ROOT}/CMakeLists.txt" AND
      EXISTS "${TRAPO_LLAMA_CPP_ROOT}/tools/mtmd/mtmd.cpp")
    set(BUILD_SHARED_LIBS OFF CACHE BOOL "Build llama.cpp static libraries" FORCE)
    set(CMAKE_POSITION_INDEPENDENT_CODE ON)
    set(LLAMA_BUILD_COMMON OFF CACHE BOOL "" FORCE)
    set(LLAMA_BUILD_TESTS OFF CACHE BOOL "" FORCE)
    set(LLAMA_BUILD_TOOLS OFF CACHE BOOL "" FORCE)
    set(LLAMA_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
    set(LLAMA_BUILD_SERVER OFF CACHE BOOL "" FORCE)
    set(LLAMA_BUILD_APP OFF CACHE BOOL "" FORCE)
    set(LLAMA_BUILD_UI OFF CACHE BOOL "" FORCE)
    set(LLAMA_OPENSSL OFF CACHE BOOL "" FORCE)
    set(LLAMA_ALL_WARNINGS OFF CACHE BOOL "" FORCE)
    set(LLAMA_ALL_WARNINGS_3RD_PARTY OFF CACHE BOOL "" FORCE)
    set(TRAPO_LLAMA_CUDA_AVAILABLE OFF)
    set(TRAPO_LLAMA_VULKAN_AVAILABLE OFF)
    set(TRAPO_LLAMA_OPENCL_AVAILABLE OFF)
    if(WIN32)
      set(TRAPO_CUDA_ROOT "" CACHE PATH
        "Preferred CUDA Toolkit root for llama.cpp CUDA/OpenCL backends")
      set(TRAPO_CUDA_DLL_MAJOR "")
      if(NOT TRAPO_CUDA_ROOT)
        file(GLOB TRAPO_CUDA_ROOT_CANDIDATES
          "$ENV{ProgramFiles}/NVIDIA GPU Computing Toolkit/CUDA/v*")
        list(FILTER TRAPO_CUDA_ROOT_CANDIDATES INCLUDE REGEX ".*/v[0-9]+(\\.[0-9]+)*$")
        list(SORT TRAPO_CUDA_ROOT_CANDIDATES COMPARE NATURAL ORDER DESCENDING)
        foreach(TRAPO_CUDA_CANDIDATE IN LISTS TRAPO_CUDA_ROOT_CANDIDATES)
          if(EXISTS "${TRAPO_CUDA_CANDIDATE}/bin/nvcc.exe")
            set(TRAPO_CUDA_ROOT "${TRAPO_CUDA_CANDIDATE}" CACHE PATH
              "Preferred CUDA Toolkit root for llama.cpp CUDA/OpenCL backends"
              FORCE)
            break()
          endif()
        endforeach()
      endif()
      if(TRAPO_CUDA_ROOT)
        if(TRAPO_CUDA_ROOT MATCHES "/v([0-9]+)(\\.[0-9]+)*$")
          set(TRAPO_CUDA_DLL_MAJOR "${CMAKE_MATCH_1}")
        endif()
        set(CUDAToolkit_ROOT "${TRAPO_CUDA_ROOT}" CACHE PATH
          "CUDA Toolkit root selected for llama.cpp" FORCE)
        set(CMAKE_CUDA_COMPILER "${TRAPO_CUDA_ROOT}/bin/nvcc.exe"
          CACHE FILEPATH "CUDA compiler selected for llama.cpp" FORCE)
        if(TRAPO_CUDA_ROOT MATCHES "/v([0-9]+)\\.([0-9]+)$")
          set(TRAPO_CUDA_VERSION_ENV
            "CUDA_PATH_V${CMAKE_MATCH_1}_${CMAKE_MATCH_2}")
          set(ENV{${TRAPO_CUDA_VERSION_ENV}} "${TRAPO_CUDA_ROOT}")
        endif()
        set(ENV{CUDA_PATH} "${TRAPO_CUDA_ROOT}")
        if(CMAKE_GENERATOR MATCHES "Visual Studio")
          set(CMAKE_VS_GLOBALS
            "CudaToolkitDir=${TRAPO_CUDA_ROOT}/"
            "CudaToolkitCustomDir=${TRAPO_CUDA_ROOT}/"
            ${CMAKE_VS_GLOBALS})
        endif()
      endif()
      find_package(CUDAToolkit QUIET)
      set(TRAPO_LLAMA_CUDA_HAS_VS_TOOLSET ON)
      if(CMAKE_GENERATOR MATCHES "Visual Studio")
        set(TRAPO_LLAMA_CUDA_HAS_VS_TOOLSET OFF)
        foreach(TRAPO_CUDA_TOOLSET_ROOT
            "${CMAKE_GENERATOR_INSTANCE}/MSBuild/Microsoft/VC"
            "$ENV{ProgramFiles}/Microsoft Visual Studio")
          file(GLOB_RECURSE TRAPO_CUDA_TOOLSET_PROPS
            "${TRAPO_CUDA_TOOLSET_ROOT}/CUDA*.props")
          if(TRAPO_CUDA_TOOLSET_PROPS)
            set(TRAPO_LLAMA_CUDA_HAS_VS_TOOLSET ON)
          endif()
        endforeach()
      endif()
      if(CUDAToolkit_FOUND AND TRAPO_LLAMA_CUDA_HAS_VS_TOOLSET)
        set(TRAPO_LLAMA_CUDA_AVAILABLE ON)
      elseif(CUDAToolkit_FOUND)
        message(STATUS
          "CUDA Toolkit was found, but the Visual Studio CUDA toolset was not. "
          "Skipping llama.cpp CUDA backend for this build.")
      endif()
      set(TRAPO_VULKAN_SDK_ROOT "$ENV{VULKAN_SDK}" CACHE PATH
        "Preferred Vulkan SDK root for llama.cpp Vulkan backend")
      if(NOT TRAPO_VULKAN_SDK_ROOT)
        file(GLOB TRAPO_VULKAN_ROOT_CANDIDATES "C:/VulkanSDK/*")
        list(SORT TRAPO_VULKAN_ROOT_CANDIDATES COMPARE NATURAL ORDER DESCENDING)
        foreach(TRAPO_VULKAN_CANDIDATE IN LISTS TRAPO_VULKAN_ROOT_CANDIDATES)
          if(EXISTS "${TRAPO_VULKAN_CANDIDATE}/Include/vulkan/vulkan.h" AND
              EXISTS "${TRAPO_VULKAN_CANDIDATE}/Lib/vulkan-1.lib")
            set(TRAPO_VULKAN_SDK_ROOT "${TRAPO_VULKAN_CANDIDATE}" CACHE PATH
              "Preferred Vulkan SDK root for llama.cpp Vulkan backend" FORCE)
            break()
          endif()
        endforeach()
      endif()
      if(TRAPO_VULKAN_SDK_ROOT)
        set(ENV{VULKAN_SDK} "${TRAPO_VULKAN_SDK_ROOT}")
        set(Vulkan_INCLUDE_DIR "${TRAPO_VULKAN_SDK_ROOT}/Include"
          CACHE PATH "Vulkan SDK include directory" FORCE)
        set(Vulkan_LIBRARY "${TRAPO_VULKAN_SDK_ROOT}/Lib/vulkan-1.lib"
          CACHE FILEPATH "Vulkan SDK import library" FORCE)
      endif()
      find_package(Vulkan QUIET)
      if(Vulkan_FOUND)
        set(TRAPO_LLAMA_VULKAN_AVAILABLE ON)
      endif()
      set(TRAPO_OPENCL_INCLUDE_DIR "" CACHE PATH
        "Preferred OpenCL include directory for llama.cpp OpenCL backend")
      set(TRAPO_OPENCL_LIBRARY "" CACHE FILEPATH
        "Preferred OpenCL import library for llama.cpp OpenCL backend")
      if(NOT TRAPO_OPENCL_INCLUDE_DIR AND TRAPO_CUDA_ROOT AND
          EXISTS "${TRAPO_CUDA_ROOT}/include/CL/cl.h")
        set(TRAPO_OPENCL_INCLUDE_DIR "${TRAPO_CUDA_ROOT}/include" CACHE PATH
          "Preferred OpenCL include directory for llama.cpp OpenCL backend"
          FORCE)
      endif()
      if(NOT TRAPO_OPENCL_LIBRARY AND TRAPO_CUDA_ROOT AND
          EXISTS "${TRAPO_CUDA_ROOT}/lib/x64/OpenCL.lib")
        set(TRAPO_OPENCL_LIBRARY "${TRAPO_CUDA_ROOT}/lib/x64/OpenCL.lib"
          CACHE FILEPATH
          "Preferred OpenCL import library for llama.cpp OpenCL backend"
          FORCE)
      endif()
      if(TRAPO_OPENCL_INCLUDE_DIR AND TRAPO_OPENCL_LIBRARY)
        set(OpenCL_INCLUDE_DIR "${TRAPO_OPENCL_INCLUDE_DIR}" CACHE PATH
          "OpenCL include directory selected for llama.cpp" FORCE)
        set(OpenCL_LIBRARY "${TRAPO_OPENCL_LIBRARY}" CACHE FILEPATH
          "OpenCL import library selected for llama.cpp" FORCE)
      endif()
      find_package(OpenCL QUIET)
      if(OpenCL_FOUND)
        set(TRAPO_LLAMA_OPENCL_AVAILABLE ON)
      endif()
    endif()
    if(DEFINED ENV{TRAPO_LLAMA_ENABLE_CUDA})
      set(TRAPO_LLAMA_ENABLE_CUDA "$ENV{TRAPO_LLAMA_ENABLE_CUDA}"
        CACHE BOOL "Enable llama.cpp CUDA backend" FORCE)
    else()
      set(TRAPO_LLAMA_ENABLE_CUDA ${TRAPO_LLAMA_CUDA_AVAILABLE}
        CACHE BOOL "Enable llama.cpp CUDA backend" FORCE)
    endif()
    if(DEFINED ENV{TRAPO_LLAMA_ENABLE_VULKAN})
      set(TRAPO_LLAMA_ENABLE_VULKAN "$ENV{TRAPO_LLAMA_ENABLE_VULKAN}"
        CACHE BOOL "Enable llama.cpp Vulkan backend" FORCE)
    else()
      set(TRAPO_LLAMA_ENABLE_VULKAN ${TRAPO_LLAMA_VULKAN_AVAILABLE}
        CACHE BOOL "Enable llama.cpp Vulkan backend" FORCE)
    endif()
    if(DEFINED ENV{TRAPO_LLAMA_ENABLE_OPENCL})
      set(TRAPO_LLAMA_ENABLE_OPENCL "$ENV{TRAPO_LLAMA_ENABLE_OPENCL}"
        CACHE BOOL "Enable llama.cpp OpenCL backend" FORCE)
    else()
      set(TRAPO_LLAMA_ENABLE_OPENCL ${TRAPO_LLAMA_OPENCL_AVAILABLE}
        CACHE BOOL "Enable llama.cpp OpenCL backend" FORCE)
    endif()
    message(STATUS
      "PaddleOCR-VL llama.cpp backends: CUDA=${TRAPO_LLAMA_ENABLE_CUDA} "
      "Vulkan=${TRAPO_LLAMA_ENABLE_VULKAN} OpenCL=${TRAPO_LLAMA_ENABLE_OPENCL}")
    if(TRAPO_LLAMA_ENABLE_CUDA)
      set(TRAPO_CUDA_ARCHITECTURES "" CACHE STRING
        "CUDA architectures for llama.cpp; defaults to native")
      if(DEFINED ENV{TRAPO_CUDA_ARCHITECTURES})
        set(TRAPO_CUDA_ARCHITECTURES "$ENV{TRAPO_CUDA_ARCHITECTURES}"
          CACHE STRING "CUDA architectures for llama.cpp" FORCE)
      endif()
      if(NOT TRAPO_CUDA_ARCHITECTURES)
        set(TRAPO_CUDA_ARCHITECTURES "native" CACHE STRING
          "CUDA architectures for llama.cpp" FORCE)
      endif()
      set(CMAKE_CUDA_ARCHITECTURES "${TRAPO_CUDA_ARCHITECTURES}"
        CACHE STRING "CUDA architectures selected for llama.cpp" FORCE)
      message(STATUS
        "PaddleOCR-VL llama.cpp CUDA architectures: "
        "${CMAKE_CUDA_ARCHITECTURES}")
    endif()
    set(GGML_BACKEND_DL OFF CACHE BOOL "" FORCE)
    set(GGML_NATIVE OFF CACHE BOOL "" FORCE)
    set(GGML_OPENMP OFF CACHE BOOL "" FORCE)
    set(GGML_BLAS OFF CACHE BOOL "" FORCE)
    set(GGML_ACCELERATE OFF CACHE BOOL "" FORCE)
    set(GGML_LLAMAFILE OFF CACHE BOOL "" FORCE)
    set(GGML_CUDA ${TRAPO_LLAMA_ENABLE_CUDA} CACHE BOOL "" FORCE)
    set(GGML_HIP OFF CACHE BOOL "" FORCE)
    set(GGML_VULKAN ${TRAPO_LLAMA_ENABLE_VULKAN} CACHE BOOL "" FORCE)
    set(GGML_METAL OFF CACHE BOOL "" FORCE)
    set(GGML_OPENCL ${TRAPO_LLAMA_ENABLE_OPENCL} CACHE BOOL "" FORCE)
    set(GGML_OPENVINO OFF CACHE BOOL "" FORCE)
    set(GGML_RPC OFF CACHE BOOL "" FORCE)
    set(GGML_SYCL OFF CACHE BOOL "" FORCE)
    set(GGML_WEBGPU OFF CACHE BOOL "" FORCE)
    set(GGML_CPU_KLEIDIAI OFF CACHE BOOL "" FORCE)
    set(GGML_BUILD_TESTS OFF CACHE BOOL "" FORCE)
    set(GGML_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
    set(GGML_ALL_WARNINGS OFF CACHE BOOL "" FORCE)
    set(GGML_ALL_WARNINGS_3RD_PARTY OFF CACHE BOOL "" FORCE)

    set(TRAPO_SAVED_CMAKE_INSTALL_BINDIR "${CMAKE_INSTALL_BINDIR}")
    set(TRAPO_SAVED_CMAKE_INSTALL_LIBDIR "${CMAKE_INSTALL_LIBDIR}")
    set(TRAPO_SAVED_CMAKE_INSTALL_INCLUDEDIR "${CMAKE_INSTALL_INCLUDEDIR}")
    set(TRAPO_LLAMA_INSTALL_ROOT
      "${CMAKE_CURRENT_BINARY_DIR}/llama.cpp/install")
    set(CMAKE_INSTALL_BINDIR
      "${TRAPO_LLAMA_INSTALL_ROOT}/bin" CACHE PATH "" FORCE)
    set(CMAKE_INSTALL_LIBDIR
      "${TRAPO_LLAMA_INSTALL_ROOT}/lib" CACHE PATH "" FORCE)
    set(CMAKE_INSTALL_INCLUDEDIR
      "${TRAPO_LLAMA_INSTALL_ROOT}/include" CACHE PATH "" FORCE)
    add_subdirectory(
      "${TRAPO_LLAMA_CPP_ROOT}"
      "${CMAKE_CURRENT_BINARY_DIR}/llama.cpp")
    set(CMAKE_INSTALL_BINDIR
      "${TRAPO_SAVED_CMAKE_INSTALL_BINDIR}" CACHE PATH "" FORCE)
    set(CMAKE_INSTALL_LIBDIR
      "${TRAPO_SAVED_CMAKE_INSTALL_LIBDIR}" CACHE PATH "" FORCE)
    set(CMAKE_INSTALL_INCLUDEDIR
      "${TRAPO_SAVED_CMAKE_INSTALL_INCLUDEDIR}" CACHE PATH "" FORCE)

    find_package(Threads REQUIRED)
    set(TRAPO_LLAMA_MTMD_DIR "${TRAPO_LLAMA_CPP_ROOT}/tools/mtmd")
    add_library(trapo_llama_mtmd STATIC
      "${TRAPO_LLAMA_MTMD_DIR}/mtmd.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/mtmd-audio.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/mtmd-image.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/mtmd-helper.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/clip.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/cogvlm.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/conformer.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/deepseekocr.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/deepseekocr2.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/dotsocr.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/exaone4_5.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/gemma4a.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/gemma4ua.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/gemma4uv.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/gemma4v.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/glm4v.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/granite-speech.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/granite4-vision.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/hunyuanvl.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/internvl.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/kimik25.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/kimivl.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/llama4.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/llava.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/mimovl.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/minicpmv.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/mobilenetv5.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/nemotron-v2-vl.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/paddleocr.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/pixtral.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/qwen2vl.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/qwen3a.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/qwen3vl.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/siglip.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/step3vl.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/whisper-enc.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/yasa2.cpp"
      "${TRAPO_LLAMA_MTMD_DIR}/models/youtuvl.cpp"
    )
    target_compile_features(trapo_llama_mtmd PRIVATE cxx_std_17)
    set_target_properties(trapo_llama_mtmd PROPERTIES
      POSITION_INDEPENDENT_CODE ON)
    target_include_directories(trapo_llama_mtmd
      PUBLIC
        "${TRAPO_LLAMA_MTMD_DIR}"
      PRIVATE
        "${TRAPO_LLAMA_CPP_ROOT}"
        "${TRAPO_LLAMA_CPP_ROOT}/vendor")
    target_link_libraries(trapo_llama_mtmd PUBLIC llama ggml)
    target_link_libraries(trapo_llama_mtmd PRIVATE Threads::Threads)

    target_compile_definitions(trapo_ocr_native PRIVATE TRAPO_OCR_ENABLE_LLAMA_CPP)
    if(TRAPO_LLAMA_ENABLE_CUDA)
      target_compile_definitions(trapo_ocr_native PRIVATE
        TRAPO_OCR_LLAMA_BACKEND_CUDA)
    endif()
    if(TRAPO_LLAMA_ENABLE_VULKAN)
      target_compile_definitions(trapo_ocr_native PRIVATE
        TRAPO_OCR_LLAMA_BACKEND_VULKAN)
    endif()
    if(TRAPO_LLAMA_ENABLE_OPENCL)
      target_compile_definitions(trapo_ocr_native PRIVATE
        TRAPO_OCR_LLAMA_BACKEND_OPENCL)
    endif()
    target_include_directories(trapo_ocr_native PRIVATE
      "${TRAPO_LLAMA_MTMD_DIR}"
      "${TRAPO_LLAMA_CPP_ROOT}/include"
      "${TRAPO_LLAMA_CPP_ROOT}/ggml/include")
    target_link_libraries(trapo_ocr_native PRIVATE
      trapo_llama_mtmd
      llama
      ggml
      Threads::Threads)
  else()
    message(STATUS
      "PaddleOCR-VL llama.cpp runtime is disabled. Run `dart run tool/fetch_llama_cpp.dart` to enable it.")
  endif()
endif()

if(ANDROID AND DEFINED ORT_ROOT)
  target_include_directories(trapo_ocr_native PRIVATE
    ${ORT_ROOT}/headers
  )

  add_library(onnxruntime SHARED IMPORTED)
  set_target_properties(onnxruntime PROPERTIES
    IMPORTED_LOCATION "${ORT_ROOT}/jni/${ANDROID_ABI}/libonnxruntime.so"
  )

  find_package(OpenCV REQUIRED CONFIG)
  if(TARGET OpenCV::opencv_java4)
    set(TRAPO_OPENCV_TARGET OpenCV::opencv_java4)
  elseif(TARGET opencv_java4)
    set(TRAPO_OPENCV_TARGET opencv_java4)
  else()
    message(FATAL_ERROR "OpenCV prefab target opencv_java4 was not found")
  endif()

  target_link_libraries(trapo_ocr_native PRIVATE
    onnxruntime
    ${TRAPO_OPENCV_TARGET}
    android
    log
  )
endif()

if(WIN32)
  if(NOT DEFINED TRAPO_ORT_INCLUDE_DIR OR NOT DEFINED TRAPO_ORT_LIB)
    message(FATAL_ERROR "Windows OCR build requires TRAPO_ORT_INCLUDE_DIR and TRAPO_ORT_LIB")
  endif()
  set(TRAPO_DIRECTML_PROVIDER_INCLUDE_DIR "")
  foreach(TRAPO_DIRECTML_CANDIDATE
      "${TRAPO_ORT_INCLUDE_DIR}"
      "${TRAPO_ORT_INCLUDE_DIR}/onnxruntime/core/providers/dml"
      "${TRAPO_DIRECTML_INCLUDE_DIR}")
    if(EXISTS "${TRAPO_DIRECTML_CANDIDATE}/dml_provider_factory.h")
      set(TRAPO_DIRECTML_PROVIDER_INCLUDE_DIR "${TRAPO_DIRECTML_CANDIDATE}")
      break()
    endif()
  endforeach()
  if(TRAPO_DIRECTML_PROVIDER_INCLUDE_DIR)
    message(STATUS
      "DirectML provider factory: ${TRAPO_DIRECTML_PROVIDER_INCLUDE_DIR}")
  else()
    message(STATUS
      "DirectML provider factory was not found; building OCR core without DirectML")
  endif()

  target_compile_definitions(trapo_ocr_native PRIVATE
    TRAPO_OCR_USE_OPENCV_MOBILE
    NOMINMAX
    _CRT_SECURE_NO_WARNINGS
  )
  if(TRAPO_DIRECTML_PROVIDER_INCLUDE_DIR)
    target_compile_definitions(trapo_ocr_native PRIVATE TRAPO_OCR_ENABLE_DIRECTML)
  endif()

  target_include_directories(trapo_ocr_native PRIVATE
    ${TRAPO_ORT_INCLUDE_DIR}
  )
  if(TRAPO_DIRECTML_PROVIDER_INCLUDE_DIR)
    target_include_directories(trapo_ocr_native PRIVATE
      ${TRAPO_DIRECTML_PROVIDER_INCLUDE_DIR}
      ${TRAPO_DIRECTML_INCLUDE_DIR}
    )
  endif()

  add_library(onnxruntime SHARED IMPORTED)
  set_target_properties(onnxruntime PROPERTIES
    IMPORTED_IMPLIB "${TRAPO_ORT_LIB}"
  )

  set(OpenCV_STATIC ON)
  if(CMAKE_GENERATOR_PLATFORM STREQUAL "ARM64" AND NOT TARGET OpenMP::OpenMP_CXX)
    add_library(OpenMP::OpenMP_CXX INTERFACE IMPORTED)
  endif()
  find_package(OpenCV REQUIRED CONFIG)

  target_link_libraries(trapo_ocr_native PRIVATE
    onnxruntime
    ${OpenCV_LIBS}
  )
  if(TRAPO_DIRECTML_PROVIDER_INCLUDE_DIR)
    target_link_libraries(trapo_ocr_native PRIVATE
      d3d12
      dxgi
    )
  endif()

  if(MSVC)
    set(TRAPO_DELAYLOAD_DLLS)
    if(TRAPO_LLAMA_ENABLE_CUDA AND TRAPO_CUDA_DLL_MAJOR)
      list(APPEND TRAPO_DELAYLOAD_DLLS
        "cublas64_${TRAPO_CUDA_DLL_MAJOR}.dll")
    endif()
    if(TRAPO_LLAMA_ENABLE_VULKAN)
      list(APPEND TRAPO_DELAYLOAD_DLLS "vulkan-1.dll")
    endif()
    if(TRAPO_LLAMA_ENABLE_OPENCL)
      list(APPEND TRAPO_DELAYLOAD_DLLS "OpenCL.dll")
    endif()
    if(TRAPO_DELAYLOAD_DLLS)
      foreach(TRAPO_DELAYLOAD_DLL IN LISTS TRAPO_DELAYLOAD_DLLS)
        target_link_options(trapo_ocr_native PRIVATE
          "/DELAYLOAD:${TRAPO_DELAYLOAD_DLL}")
      endforeach()
      target_link_libraries(trapo_ocr_native PRIVATE delayimp)
      message(STATUS
        "PaddleOCR-VL delay-loaded GPU runtime DLLs: ${TRAPO_DELAYLOAD_DLLS}")
    endif()
  endif()
endif()
