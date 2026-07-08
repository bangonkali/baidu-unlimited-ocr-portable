cmake_minimum_required(VERSION 3.18)

project(agus_ocr_core LANGUAGES C CXX)

set(AGUS_OCR_CORE_SOURCES
  src/agus_ocr.cpp
  src/model/ocr_model_bundle.cpp
)

if((ANDROID AND DEFINED ORT_ROOT) OR WIN32)
  list(APPEND AGUS_OCR_CORE_SOURCES
    third_party/clipper/ocr_clipper.cpp
    src/gemma/gemma_common.cpp
    src/gemma/gemma_image_processor.cpp
    src/gemma/gemma_markdown_engine.cpp
    src/gemma/gemma_markdown_result.cpp
    src/gemma/gemma_markdown_sampling.cpp
    src/gemma/gemma_onnx_runner.cpp
    src/gemma/gemma_tokenizer.cpp
    src/gemma/gemma_tokenizer_common.cpp
    src/gemma/gemma_tokenizer_json_scanner.cpp
    src/gemma/gemma_tokenizer_loader.cpp
    src/vl/layout_analyzer.cpp
    src/vl/llama_vlm_support.cpp
    src/vl/llama_vlm_runner.cpp
    src/vl/paddle_ocr_vl_engine.cpp
    src/vl/paddle_ocr_vl_result.cpp
  )
endif()

if(ANDROID AND DEFINED ORT_ROOT)
  list(APPEND AGUS_OCR_CORE_SOURCES
    ../../android/src/main/cpp/agus_ocr_jni.cpp
  )
endif()

add_library(agus_ocr_core SHARED ${AGUS_OCR_CORE_SOURCES})

target_include_directories(agus_ocr_core
  PUBLIC
    ${CMAKE_CURRENT_LIST_DIR}/include
  PRIVATE
    ${CMAKE_CURRENT_LIST_DIR}/src
)

target_compile_features(agus_ocr_core PUBLIC cxx_std_17)

if(MSVC)
  target_compile_options(agus_ocr_core PRIVATE /W4)
else()
  target_compile_options(agus_ocr_core PRIVATE -Wall -Wextra -Wpedantic)
endif()

if((ANDROID AND DEFINED ORT_ROOT) OR WIN32)
  target_compile_definitions(agus_ocr_core PRIVATE AGUS_OCR_ENABLE_NATIVE_PIPELINE)

  target_include_directories(agus_ocr_core PRIVATE
    ${CMAKE_CURRENT_LIST_DIR}/third_party/clipper
  )

  set(AGUS_LLAMA_CPP_ROOT
    "${CMAKE_CURRENT_LIST_DIR}/../../.deps/llama.cpp"
    CACHE PATH "Pinned llama.cpp checkout created by tool/fetch_llama_cpp.dart")
  if(EXISTS "${AGUS_LLAMA_CPP_ROOT}/CMakeLists.txt" AND
      EXISTS "${AGUS_LLAMA_CPP_ROOT}/tools/mtmd/mtmd.cpp")
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
    set(AGUS_LLAMA_CUDA_AVAILABLE OFF)
    set(AGUS_LLAMA_VULKAN_AVAILABLE OFF)
    set(AGUS_LLAMA_OPENCL_AVAILABLE OFF)
    if(WIN32)
      set(AGUS_CUDA_ROOT "" CACHE PATH
        "Preferred CUDA Toolkit root for llama.cpp CUDA/OpenCL backends")
      set(AGUS_CUDA_DLL_MAJOR "")
      if(NOT AGUS_CUDA_ROOT)
        file(GLOB AGUS_CUDA_ROOT_CANDIDATES
          "$ENV{ProgramFiles}/NVIDIA GPU Computing Toolkit/CUDA/v*")
        list(FILTER AGUS_CUDA_ROOT_CANDIDATES INCLUDE REGEX ".*/v[0-9]+(\\.[0-9]+)*$")
        list(SORT AGUS_CUDA_ROOT_CANDIDATES COMPARE NATURAL ORDER DESCENDING)
        foreach(AGUS_CUDA_CANDIDATE IN LISTS AGUS_CUDA_ROOT_CANDIDATES)
          if(EXISTS "${AGUS_CUDA_CANDIDATE}/bin/nvcc.exe")
            set(AGUS_CUDA_ROOT "${AGUS_CUDA_CANDIDATE}" CACHE PATH
              "Preferred CUDA Toolkit root for llama.cpp CUDA/OpenCL backends"
              FORCE)
            break()
          endif()
        endforeach()
      endif()
      if(AGUS_CUDA_ROOT)
        if(AGUS_CUDA_ROOT MATCHES "/v([0-9]+)(\\.[0-9]+)*$")
          set(AGUS_CUDA_DLL_MAJOR "${CMAKE_MATCH_1}")
        endif()
        set(CUDAToolkit_ROOT "${AGUS_CUDA_ROOT}" CACHE PATH
          "CUDA Toolkit root selected for llama.cpp" FORCE)
        set(CMAKE_CUDA_COMPILER "${AGUS_CUDA_ROOT}/bin/nvcc.exe"
          CACHE FILEPATH "CUDA compiler selected for llama.cpp" FORCE)
        if(AGUS_CUDA_ROOT MATCHES "/v([0-9]+)\\.([0-9]+)$")
          set(AGUS_CUDA_VERSION_ENV
            "CUDA_PATH_V${CMAKE_MATCH_1}_${CMAKE_MATCH_2}")
          set(ENV{${AGUS_CUDA_VERSION_ENV}} "${AGUS_CUDA_ROOT}")
        endif()
        set(ENV{CUDA_PATH} "${AGUS_CUDA_ROOT}")
        if(CMAKE_GENERATOR MATCHES "Visual Studio")
          set(CMAKE_VS_GLOBALS
            "CudaToolkitDir=${AGUS_CUDA_ROOT}/"
            "CudaToolkitCustomDir=${AGUS_CUDA_ROOT}/"
            ${CMAKE_VS_GLOBALS})
        endif()
      endif()
      find_package(CUDAToolkit QUIET)
      set(AGUS_LLAMA_CUDA_HAS_VS_TOOLSET ON)
      if(CMAKE_GENERATOR MATCHES "Visual Studio")
        set(AGUS_LLAMA_CUDA_HAS_VS_TOOLSET OFF)
        foreach(AGUS_CUDA_TOOLSET_ROOT
            "${CMAKE_GENERATOR_INSTANCE}/MSBuild/Microsoft/VC"
            "$ENV{ProgramFiles}/Microsoft Visual Studio")
          file(GLOB_RECURSE AGUS_CUDA_TOOLSET_PROPS
            "${AGUS_CUDA_TOOLSET_ROOT}/CUDA*.props")
          if(AGUS_CUDA_TOOLSET_PROPS)
            set(AGUS_LLAMA_CUDA_HAS_VS_TOOLSET ON)
          endif()
        endforeach()
      endif()
      if(CUDAToolkit_FOUND AND AGUS_LLAMA_CUDA_HAS_VS_TOOLSET)
        set(AGUS_LLAMA_CUDA_AVAILABLE ON)
      elseif(CUDAToolkit_FOUND)
        message(STATUS
          "CUDA Toolkit was found, but the Visual Studio CUDA toolset was not. "
          "Skipping llama.cpp CUDA backend for this build.")
      endif()
      set(AGUS_VULKAN_SDK_ROOT "$ENV{VULKAN_SDK}" CACHE PATH
        "Preferred Vulkan SDK root for llama.cpp Vulkan backend")
      if(NOT AGUS_VULKAN_SDK_ROOT)
        file(GLOB AGUS_VULKAN_ROOT_CANDIDATES "C:/VulkanSDK/*")
        list(SORT AGUS_VULKAN_ROOT_CANDIDATES COMPARE NATURAL ORDER DESCENDING)
        foreach(AGUS_VULKAN_CANDIDATE IN LISTS AGUS_VULKAN_ROOT_CANDIDATES)
          if(EXISTS "${AGUS_VULKAN_CANDIDATE}/Include/vulkan/vulkan.h" AND
              EXISTS "${AGUS_VULKAN_CANDIDATE}/Lib/vulkan-1.lib")
            set(AGUS_VULKAN_SDK_ROOT "${AGUS_VULKAN_CANDIDATE}" CACHE PATH
              "Preferred Vulkan SDK root for llama.cpp Vulkan backend" FORCE)
            break()
          endif()
        endforeach()
      endif()
      if(AGUS_VULKAN_SDK_ROOT)
        set(ENV{VULKAN_SDK} "${AGUS_VULKAN_SDK_ROOT}")
        set(Vulkan_INCLUDE_DIR "${AGUS_VULKAN_SDK_ROOT}/Include"
          CACHE PATH "Vulkan SDK include directory" FORCE)
        set(Vulkan_LIBRARY "${AGUS_VULKAN_SDK_ROOT}/Lib/vulkan-1.lib"
          CACHE FILEPATH "Vulkan SDK import library" FORCE)
      endif()
      find_package(Vulkan QUIET)
      if(Vulkan_FOUND)
        set(AGUS_LLAMA_VULKAN_AVAILABLE ON)
      endif()
      set(AGUS_OPENCL_INCLUDE_DIR "" CACHE PATH
        "Preferred OpenCL include directory for llama.cpp OpenCL backend")
      set(AGUS_OPENCL_LIBRARY "" CACHE FILEPATH
        "Preferred OpenCL import library for llama.cpp OpenCL backend")
      if(NOT AGUS_OPENCL_INCLUDE_DIR AND AGUS_CUDA_ROOT AND
          EXISTS "${AGUS_CUDA_ROOT}/include/CL/cl.h")
        set(AGUS_OPENCL_INCLUDE_DIR "${AGUS_CUDA_ROOT}/include" CACHE PATH
          "Preferred OpenCL include directory for llama.cpp OpenCL backend"
          FORCE)
      endif()
      if(NOT AGUS_OPENCL_LIBRARY AND AGUS_CUDA_ROOT AND
          EXISTS "${AGUS_CUDA_ROOT}/lib/x64/OpenCL.lib")
        set(AGUS_OPENCL_LIBRARY "${AGUS_CUDA_ROOT}/lib/x64/OpenCL.lib"
          CACHE FILEPATH
          "Preferred OpenCL import library for llama.cpp OpenCL backend"
          FORCE)
      endif()
      if(AGUS_OPENCL_INCLUDE_DIR AND AGUS_OPENCL_LIBRARY)
        set(OpenCL_INCLUDE_DIR "${AGUS_OPENCL_INCLUDE_DIR}" CACHE PATH
          "OpenCL include directory selected for llama.cpp" FORCE)
        set(OpenCL_LIBRARY "${AGUS_OPENCL_LIBRARY}" CACHE FILEPATH
          "OpenCL import library selected for llama.cpp" FORCE)
      endif()
      find_package(OpenCL QUIET)
      if(OpenCL_FOUND)
        set(AGUS_LLAMA_OPENCL_AVAILABLE ON)
      endif()
    endif()
    if(DEFINED ENV{AGUS_LLAMA_ENABLE_CUDA})
      set(AGUS_LLAMA_ENABLE_CUDA "$ENV{AGUS_LLAMA_ENABLE_CUDA}"
        CACHE BOOL "Enable llama.cpp CUDA backend" FORCE)
    else()
      set(AGUS_LLAMA_ENABLE_CUDA ${AGUS_LLAMA_CUDA_AVAILABLE}
        CACHE BOOL "Enable llama.cpp CUDA backend" FORCE)
    endif()
    if(DEFINED ENV{AGUS_LLAMA_ENABLE_VULKAN})
      set(AGUS_LLAMA_ENABLE_VULKAN "$ENV{AGUS_LLAMA_ENABLE_VULKAN}"
        CACHE BOOL "Enable llama.cpp Vulkan backend" FORCE)
    else()
      set(AGUS_LLAMA_ENABLE_VULKAN ${AGUS_LLAMA_VULKAN_AVAILABLE}
        CACHE BOOL "Enable llama.cpp Vulkan backend" FORCE)
    endif()
    if(DEFINED ENV{AGUS_LLAMA_ENABLE_OPENCL})
      set(AGUS_LLAMA_ENABLE_OPENCL "$ENV{AGUS_LLAMA_ENABLE_OPENCL}"
        CACHE BOOL "Enable llama.cpp OpenCL backend" FORCE)
    else()
      set(AGUS_LLAMA_ENABLE_OPENCL ${AGUS_LLAMA_OPENCL_AVAILABLE}
        CACHE BOOL "Enable llama.cpp OpenCL backend" FORCE)
    endif()
    message(STATUS
      "PaddleOCR-VL llama.cpp backends: CUDA=${AGUS_LLAMA_ENABLE_CUDA} "
      "Vulkan=${AGUS_LLAMA_ENABLE_VULKAN} OpenCL=${AGUS_LLAMA_ENABLE_OPENCL}")
    if(AGUS_LLAMA_ENABLE_CUDA)
      set(AGUS_CUDA_ARCHITECTURES "" CACHE STRING
        "CUDA architectures for llama.cpp; defaults to native")
      if(DEFINED ENV{AGUS_CUDA_ARCHITECTURES})
        set(AGUS_CUDA_ARCHITECTURES "$ENV{AGUS_CUDA_ARCHITECTURES}"
          CACHE STRING "CUDA architectures for llama.cpp" FORCE)
      endif()
      if(NOT AGUS_CUDA_ARCHITECTURES)
        set(AGUS_CUDA_ARCHITECTURES "native" CACHE STRING
          "CUDA architectures for llama.cpp" FORCE)
      endif()
      set(CMAKE_CUDA_ARCHITECTURES "${AGUS_CUDA_ARCHITECTURES}"
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
    set(GGML_CUDA ${AGUS_LLAMA_ENABLE_CUDA} CACHE BOOL "" FORCE)
    set(GGML_HIP OFF CACHE BOOL "" FORCE)
    set(GGML_VULKAN ${AGUS_LLAMA_ENABLE_VULKAN} CACHE BOOL "" FORCE)
    set(GGML_METAL OFF CACHE BOOL "" FORCE)
    set(GGML_OPENCL ${AGUS_LLAMA_ENABLE_OPENCL} CACHE BOOL "" FORCE)
    set(GGML_OPENVINO OFF CACHE BOOL "" FORCE)
    set(GGML_RPC OFF CACHE BOOL "" FORCE)
    set(GGML_SYCL OFF CACHE BOOL "" FORCE)
    set(GGML_WEBGPU OFF CACHE BOOL "" FORCE)
    set(GGML_CPU_KLEIDIAI OFF CACHE BOOL "" FORCE)
    set(GGML_BUILD_TESTS OFF CACHE BOOL "" FORCE)
    set(GGML_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
    set(GGML_ALL_WARNINGS OFF CACHE BOOL "" FORCE)
    set(GGML_ALL_WARNINGS_3RD_PARTY OFF CACHE BOOL "" FORCE)

    set(AGUS_SAVED_CMAKE_INSTALL_BINDIR "${CMAKE_INSTALL_BINDIR}")
    set(AGUS_SAVED_CMAKE_INSTALL_LIBDIR "${CMAKE_INSTALL_LIBDIR}")
    set(AGUS_SAVED_CMAKE_INSTALL_INCLUDEDIR "${CMAKE_INSTALL_INCLUDEDIR}")
    set(AGUS_LLAMA_INSTALL_ROOT
      "${CMAKE_CURRENT_BINARY_DIR}/llama.cpp/install")
    set(CMAKE_INSTALL_BINDIR
      "${AGUS_LLAMA_INSTALL_ROOT}/bin" CACHE PATH "" FORCE)
    set(CMAKE_INSTALL_LIBDIR
      "${AGUS_LLAMA_INSTALL_ROOT}/lib" CACHE PATH "" FORCE)
    set(CMAKE_INSTALL_INCLUDEDIR
      "${AGUS_LLAMA_INSTALL_ROOT}/include" CACHE PATH "" FORCE)
    add_subdirectory(
      "${AGUS_LLAMA_CPP_ROOT}"
      "${CMAKE_CURRENT_BINARY_DIR}/llama.cpp")
    set(CMAKE_INSTALL_BINDIR
      "${AGUS_SAVED_CMAKE_INSTALL_BINDIR}" CACHE PATH "" FORCE)
    set(CMAKE_INSTALL_LIBDIR
      "${AGUS_SAVED_CMAKE_INSTALL_LIBDIR}" CACHE PATH "" FORCE)
    set(CMAKE_INSTALL_INCLUDEDIR
      "${AGUS_SAVED_CMAKE_INSTALL_INCLUDEDIR}" CACHE PATH "" FORCE)

    find_package(Threads REQUIRED)
    set(AGUS_LLAMA_MTMD_DIR "${AGUS_LLAMA_CPP_ROOT}/tools/mtmd")
    add_library(agus_llama_mtmd STATIC
      "${AGUS_LLAMA_MTMD_DIR}/mtmd.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/mtmd-audio.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/mtmd-image.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/mtmd-helper.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/clip.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/cogvlm.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/conformer.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/deepseekocr.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/deepseekocr2.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/dotsocr.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/exaone4_5.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/gemma4a.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/gemma4ua.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/gemma4uv.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/gemma4v.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/glm4v.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/granite-speech.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/granite4-vision.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/hunyuanvl.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/internvl.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/kimik25.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/kimivl.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/llama4.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/llava.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/mimovl.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/minicpmv.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/mobilenetv5.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/nemotron-v2-vl.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/paddleocr.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/pixtral.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/qwen2vl.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/qwen3a.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/qwen3vl.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/siglip.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/step3vl.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/whisper-enc.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/yasa2.cpp"
      "${AGUS_LLAMA_MTMD_DIR}/models/youtuvl.cpp"
    )
    target_compile_features(agus_llama_mtmd PRIVATE cxx_std_17)
    set_target_properties(agus_llama_mtmd PROPERTIES
      POSITION_INDEPENDENT_CODE ON)
    target_include_directories(agus_llama_mtmd
      PUBLIC
        "${AGUS_LLAMA_MTMD_DIR}"
      PRIVATE
        "${AGUS_LLAMA_CPP_ROOT}"
        "${AGUS_LLAMA_CPP_ROOT}/vendor")
    target_link_libraries(agus_llama_mtmd PUBLIC llama ggml)
    target_link_libraries(agus_llama_mtmd PRIVATE Threads::Threads)

    target_compile_definitions(agus_ocr_core PRIVATE AGUS_OCR_ENABLE_LLAMA_CPP)
    if(AGUS_LLAMA_ENABLE_CUDA)
      target_compile_definitions(agus_ocr_core PRIVATE
        AGUS_OCR_LLAMA_BACKEND_CUDA)
    endif()
    if(AGUS_LLAMA_ENABLE_VULKAN)
      target_compile_definitions(agus_ocr_core PRIVATE
        AGUS_OCR_LLAMA_BACKEND_VULKAN)
    endif()
    if(AGUS_LLAMA_ENABLE_OPENCL)
      target_compile_definitions(agus_ocr_core PRIVATE
        AGUS_OCR_LLAMA_BACKEND_OPENCL)
    endif()
    target_include_directories(agus_ocr_core PRIVATE
      "${AGUS_LLAMA_MTMD_DIR}"
      "${AGUS_LLAMA_CPP_ROOT}/include"
      "${AGUS_LLAMA_CPP_ROOT}/ggml/include")
    target_link_libraries(agus_ocr_core PRIVATE
      agus_llama_mtmd
      llama
      ggml
      Threads::Threads)
  else()
    message(STATUS
      "PaddleOCR-VL llama.cpp runtime is disabled. Run `dart run tool/fetch_llama_cpp.dart` to enable it.")
  endif()
endif()

if(ANDROID AND DEFINED ORT_ROOT)
  target_include_directories(agus_ocr_core PRIVATE
    ${ORT_ROOT}/headers
  )

  add_library(onnxruntime SHARED IMPORTED)
  set_target_properties(onnxruntime PROPERTIES
    IMPORTED_LOCATION "${ORT_ROOT}/jni/${ANDROID_ABI}/libonnxruntime.so"
  )

  find_package(OpenCV REQUIRED CONFIG)
  if(TARGET OpenCV::opencv_java4)
    set(AGUS_OPENCV_TARGET OpenCV::opencv_java4)
  elseif(TARGET opencv_java4)
    set(AGUS_OPENCV_TARGET opencv_java4)
  else()
    message(FATAL_ERROR "OpenCV prefab target opencv_java4 was not found")
  endif()

  target_link_libraries(agus_ocr_core PRIVATE
    onnxruntime
    ${AGUS_OPENCV_TARGET}
    android
    log
  )
endif()

if(WIN32)
  if(NOT DEFINED AGUS_ORT_INCLUDE_DIR OR NOT DEFINED AGUS_ORT_LIB)
    message(FATAL_ERROR "Windows OCR build requires AGUS_ORT_INCLUDE_DIR and AGUS_ORT_LIB")
  endif()
  set(AGUS_DIRECTML_PROVIDER_INCLUDE_DIR "")
  foreach(AGUS_DIRECTML_CANDIDATE
      "${AGUS_ORT_INCLUDE_DIR}"
      "${AGUS_ORT_INCLUDE_DIR}/onnxruntime/core/providers/dml"
      "${AGUS_DIRECTML_INCLUDE_DIR}")
    if(EXISTS "${AGUS_DIRECTML_CANDIDATE}/dml_provider_factory.h")
      set(AGUS_DIRECTML_PROVIDER_INCLUDE_DIR "${AGUS_DIRECTML_CANDIDATE}")
      break()
    endif()
  endforeach()
  if(AGUS_DIRECTML_PROVIDER_INCLUDE_DIR)
    message(STATUS
      "DirectML provider factory: ${AGUS_DIRECTML_PROVIDER_INCLUDE_DIR}")
  else()
    message(STATUS
      "DirectML provider factory was not found; building OCR core without DirectML")
  endif()

  target_compile_definitions(agus_ocr_core PRIVATE
    AGUS_OCR_USE_OPENCV_MOBILE
    NOMINMAX
    _CRT_SECURE_NO_WARNINGS
  )
  if(AGUS_DIRECTML_PROVIDER_INCLUDE_DIR)
    target_compile_definitions(agus_ocr_core PRIVATE AGUS_OCR_ENABLE_DIRECTML)
  endif()

  target_include_directories(agus_ocr_core PRIVATE
    ${AGUS_ORT_INCLUDE_DIR}
  )
  if(AGUS_DIRECTML_PROVIDER_INCLUDE_DIR)
    target_include_directories(agus_ocr_core PRIVATE
      ${AGUS_DIRECTML_PROVIDER_INCLUDE_DIR}
      ${AGUS_DIRECTML_INCLUDE_DIR}
    )
  endif()

  add_library(onnxruntime SHARED IMPORTED)
  set_target_properties(onnxruntime PROPERTIES
    IMPORTED_IMPLIB "${AGUS_ORT_LIB}"
  )

  set(OpenCV_STATIC ON)
  find_package(OpenCV REQUIRED CONFIG)

  target_link_libraries(agus_ocr_core PRIVATE
    onnxruntime
    ${OpenCV_LIBS}
  )
  if(AGUS_DIRECTML_PROVIDER_INCLUDE_DIR)
    target_link_libraries(agus_ocr_core PRIVATE
      d3d12
      dxgi
    )
  endif()

  if(MSVC)
    set(AGUS_DELAYLOAD_DLLS)
    if(AGUS_LLAMA_ENABLE_CUDA AND AGUS_CUDA_DLL_MAJOR)
      list(APPEND AGUS_DELAYLOAD_DLLS
        "cublas64_${AGUS_CUDA_DLL_MAJOR}.dll")
    endif()
    if(AGUS_LLAMA_ENABLE_VULKAN)
      list(APPEND AGUS_DELAYLOAD_DLLS "vulkan-1.dll")
    endif()
    if(AGUS_LLAMA_ENABLE_OPENCL)
      list(APPEND AGUS_DELAYLOAD_DLLS "OpenCL.dll")
    endif()
    if(AGUS_DELAYLOAD_DLLS)
      foreach(AGUS_DELAYLOAD_DLL IN LISTS AGUS_DELAYLOAD_DLLS)
        target_link_options(agus_ocr_core PRIVATE
          "/DELAYLOAD:${AGUS_DELAYLOAD_DLL}")
      endforeach()
      target_link_libraries(agus_ocr_core PRIVATE delayimp)
      message(STATUS
        "PaddleOCR-VL delay-loaded GPU runtime DLLs: ${AGUS_DELAYLOAD_DLLS}")
    endif()
  endif()
endif()
