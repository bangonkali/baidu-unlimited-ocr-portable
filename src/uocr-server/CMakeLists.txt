cmake_minimum_required(VERSION 3.24)

project(uocr_server LANGUAGES CXX)

include(CTest)

option(UOCR_BUILD_SERVER "Build the Drogon API server when Drogon is available" ON)
option(UOCR_BUILD_TESTS "Build uocr-server unit tests" ON)
option(UOCR_BUILD_CLIENT "Build and copy the React client beside uocr-server" ON)
set(UOCR_VERSION "" CACHE STRING "uocr-server release version, usually the git tag")
set(UOCR_CLIENT_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../uocr-client" CACHE PATH "React client directory")

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

add_library(uocr_core STATIC
  src/core/ocr_parser.cpp
  src/core/profiles.cpp
  src/core/runaway_guard.cpp
  src/fs/file_scanner.cpp
  src/ocr/unlimited_ocr_ffi_engine.cpp
  src/storage/migrations.cpp
)

target_include_directories(uocr_core PUBLIC include)

if(CMAKE_DL_LIBS)
  target_link_libraries(uocr_core PUBLIC ${CMAKE_DL_LIBS})
endif()

if(MSVC)
  target_compile_options(uocr_core PRIVATE /W4 /permissive-)
else()
  target_compile_options(uocr_core PRIVATE -Wall -Wextra -Wpedantic)
endif()

if(UOCR_BUILD_TESTS)
  add_executable(uocr-core-tests tests/core_tests.cpp)
  target_link_libraries(uocr-core-tests PRIVATE uocr_core)
  add_test(NAME uocr-core-tests COMMAND uocr-core-tests)
endif()

if(UOCR_BUILD_SERVER)
  find_package(Drogon CONFIG QUIET)
  if(Drogon_FOUND)
    if(NOT UOCR_VERSION)
      execute_process(
        COMMAND git describe --tags --dirty --always
        WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/../.."
        OUTPUT_VARIABLE UOCR_VERSION
        OUTPUT_STRIP_TRAILING_WHITESPACE
        ERROR_QUIET
      )
    endif()
    if(NOT UOCR_VERSION)
      set(UOCR_VERSION "0.0.0-dev")
    endif()
    execute_process(
      COMMAND git rev-parse --short=12 HEAD
      WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/../.."
      OUTPUT_VARIABLE UOCR_GIT_SHA
      OUTPUT_STRIP_TRAILING_WHITESPACE
      ERROR_QUIET
    )
    execute_process(
      COMMAND git describe --tags --exact-match HEAD
      WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/../.."
      OUTPUT_VARIABLE UOCR_GIT_TAG
      OUTPUT_STRIP_TRAILING_WHITESPACE
      ERROR_QUIET
    )
    if(NOT UOCR_GIT_SHA)
      set(UOCR_GIT_SHA "unknown")
    endif()
    if(NOT UOCR_GIT_TAG)
      set(UOCR_GIT_TAG "${UOCR_VERSION}")
    endif()

    file(TO_CMAKE_PATH "${CMAKE_CURRENT_SOURCE_DIR}" UOCR_SERVER_SOURCE_DIR)
    add_executable(uocr-server
      src/app/main.cpp
      src/app/routes.cpp
    )
    target_link_libraries(uocr-server PRIVATE uocr_core Drogon::Drogon)
    target_compile_definitions(uocr-server PRIVATE
      UOCR_APP_VERSION="${UOCR_VERSION}"
      UOCR_GIT_SHA="${UOCR_GIT_SHA}"
      UOCR_GIT_TAG="${UOCR_GIT_TAG}"
      UOCR_SERVER_SOURCE_DIR="${UOCR_SERVER_SOURCE_DIR}"
    )

    add_custom_command(TARGET uocr-server POST_BUILD
      COMMAND ${CMAKE_COMMAND} -E make_directory "$<TARGET_FILE_DIR:uocr-server>/openapi"
      COMMAND ${CMAKE_COMMAND} -E make_directory "$<TARGET_FILE_DIR:uocr-server>/logs"
      COMMAND ${CMAKE_COMMAND} -E make_directory "$<TARGET_FILE_DIR:uocr-server>/cache"
      COMMAND ${CMAKE_COMMAND} -E make_directory "$<TARGET_FILE_DIR:uocr-server>/data"
      COMMAND ${CMAKE_COMMAND} -E make_directory "$<TARGET_FILE_DIR:uocr-server>/config"
      COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "${CMAKE_CURRENT_SOURCE_DIR}/openapi/uocr.openapi.json"
        "$<TARGET_FILE_DIR:uocr-server>/openapi/uocr.openapi.json"
      VERBATIM
    )

    if(UOCR_BUILD_CLIENT)
      find_program(BUN_EXECUTABLE bun)
      if(NOT BUN_EXECUTABLE)
        message(FATAL_ERROR "bun was not found; install Bun or configure with -DUOCR_BUILD_CLIENT=OFF")
      endif()
      add_custom_target(uocr-client-dist
        COMMAND ${BUN_EXECUTABLE} run build
        WORKING_DIRECTORY "${UOCR_CLIENT_DIR}"
        COMMENT "Building uocr-client React app"
        VERBATIM
      )
      add_dependencies(uocr-server uocr-client-dist)
      add_custom_command(TARGET uocr-server POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_directory
          "${UOCR_CLIENT_DIR}/dist"
          "$<TARGET_FILE_DIR:uocr-server>/web"
        VERBATIM
      )
    endif()

    if(MSVC)
      target_compile_options(uocr-server PRIVATE /W4 /permissive-)
    else()
      target_compile_options(uocr-server PRIVATE -Wall -Wextra -Wpedantic)
    endif()
  else()
    message(WARNING "Drogon was not found; uocr-server executable is skipped. uocr_core and tests still build.")
  endif()
endif()
