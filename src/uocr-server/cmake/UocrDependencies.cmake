function(uocr_link_windows_system_libraries target)
  if(WIN32)
    target_link_libraries(${target} PRIVATE
      user32
      gdi32
      winspool
      comdlg32
      advapi32
      shell32
      ole32
      oleaut32
      uuid
      odbc32
      odbccp32
    )
  endif()
endfunction()

function(uocr_link_mupdf target)
  if(NOT UOCR_EMBED_MUPDF)
    return()
  endif()
  if(NOT WIN32)
    message(FATAL_ERROR "Embedded MuPDF is currently configured for the Windows portable build only.")
  endif()
  target_include_directories(${target} SYSTEM PRIVATE "${UOCR_MUPDF_ROOT}/include")
  set(UOCR_MUPDF_LIB_DIR "${UOCR_MUPDF_ROOT}/platform/win32/x64/$<CONFIG>")
  target_link_libraries(${target} PRIVATE
    "${UOCR_MUPDF_LIB_DIR}/libmupdf.lib"
    "${UOCR_MUPDF_LIB_DIR}/libresources.lib"
    "${UOCR_MUPDF_LIB_DIR}/libthirdparty.lib"
    "${UOCR_MUPDF_LIB_DIR}/libpkcs7.lib"
    "${UOCR_MUPDF_LIB_DIR}/libtesseract.lib"
    "${UOCR_MUPDF_LIB_DIR}/libleptonica.lib"
    "${UOCR_MUPDF_LIB_DIR}/libharfbuzz.lib"
    "${UOCR_MUPDF_LIB_DIR}/libextract.lib"
    "${UOCR_MUPDF_LIB_DIR}/libmubarcode.lib"
    "${UOCR_MUPDF_LIB_DIR}/libzxing.lib"
    "${UOCR_MUPDF_LIB_DIR}/libmuthreads.lib"
  )
  uocr_link_windows_system_libraries(${target})
endfunction()

function(uocr_link_duckdb target)
  if(NOT WIN32)
    message(FATAL_ERROR "Bundled DuckDB is currently configured for the Windows portable build only.")
  endif()
  target_include_directories(${target} PUBLIC "${UOCR_DUCKDB_ROOT}/generated/include")
  target_link_libraries(${target} PUBLIC "${UOCR_DUCKDB_ROOT}/lib/duckdb.lib")
endfunction()

function(uocr_copy_duckdb_runtime target)
  add_custom_command(TARGET ${target} POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
      "${UOCR_DUCKDB_ROOT}/bin/duckdb.dll"
      "$<TARGET_FILE_DIR:${target}>/duckdb.dll"
    VERBATIM
  )
endfunction()

function(uocr_configure_openssl)
  find_package(OpenSSL 3.6.3 EXACT REQUIRED)
  add_library(uocr_openssl_crypto INTERFACE)
  target_link_libraries(uocr_openssl_crypto INTERFACE OpenSSL::Crypto)
  set(_uocr_openssl_version "${OPENSSL_VERSION}")
  if(NOT _uocr_openssl_version AND DEFINED OpenSSL_VERSION)
    set(_uocr_openssl_version "${OpenSSL_VERSION}")
  endif()
  if(DEFINED OPENSSL_VERSION)
    set(UOCR_OPENSSL_VERSION "${OPENSSL_VERSION}" PARENT_SCOPE)
  elseif(DEFINED OpenSSL_VERSION)
    set(UOCR_OPENSSL_VERSION "${OpenSSL_VERSION}" PARENT_SCOPE)
  endif()
  message(STATUS "Using vcpkg OpenSSL ${_uocr_openssl_version} for SHA verification")
endfunction()

function(uocr_configure_drogon)
  find_package(Drogon 1.9.13 EXACT CONFIG QUIET)
  set(Drogon_FOUND "${Drogon_FOUND}" PARENT_SCOPE)
  if(Drogon_FOUND)
    message(STATUS "Using vcpkg Drogon 1.9.13")
  endif()
endfunction()
