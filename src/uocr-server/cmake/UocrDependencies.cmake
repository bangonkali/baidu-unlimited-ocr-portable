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

function(uocr_assert_vcpkg_path label path_value)
  if(NOT UOCR_REQUIRE_VCPKG OR NOT DEFINED VCPKG_INSTALLED_DIR)
    return()
  endif()
  if(NOT path_value OR NOT IS_ABSOLUTE "${path_value}")
    return()
  endif()

  file(TO_CMAKE_PATH "${path_value}" _uocr_path)
  file(TO_CMAKE_PATH "${VCPKG_INSTALLED_DIR}" _uocr_vcpkg_root)
  string(FIND "${_uocr_path}" "${_uocr_vcpkg_root}" _uocr_vcpkg_offset)
  if(NOT _uocr_vcpkg_offset EQUAL 0)
    message(FATAL_ERROR
      "${label} resolved outside the vcpkg installed tree.\n"
      "  resolved: ${path_value}\n"
      "  vcpkg:    ${VCPKG_INSTALLED_DIR}\n"
      "Use the repository vcpkg manifest unless a dependency is documented as an explicit exception."
    )
  endif()
endfunction()

function(uocr_assert_vcpkg_target target)
  if(NOT TARGET ${target})
    message(FATAL_ERROR "Expected vcpkg target ${target} was not defined.")
  endif()

  foreach(_uocr_prop IN ITEMS IMPORTED_LOCATION_RELEASE IMPORTED_IMPLIB_RELEASE IMPORTED_LOCATION IMPORTED_IMPLIB)
    get_target_property(_uocr_value ${target} ${_uocr_prop})
    if(_uocr_value AND NOT _uocr_value STREQUAL "_uocr_value-NOTFOUND")
      uocr_assert_vcpkg_path("${target} ${_uocr_prop}" "${_uocr_value}")
    endif()
  endforeach()
endfunction()

function(uocr_link_mupdf target)
  if(NOT UOCR_EMBED_MUPDF)
    return()
  endif()

  find_package(unofficial-libmupdf CONFIG QUIET)
  find_package(libmupdf CONFIG QUIET)
  find_package(mupdf CONFIG QUIET)
  foreach(_candidate IN ITEMS unofficial::libmupdf::libmupdf mupdf::mupdf libmupdf::mupdf mupdf libmupdf)
    if(TARGET ${_candidate})
      uocr_assert_vcpkg_target(${_candidate})
      target_link_libraries(${target} PRIVATE ${_candidate})
      return()
    endif()
  endforeach()
  message(FATAL_ERROR "vcpkg libmupdf was found without a recognized CMake target.")
endfunction()

function(uocr_link_duckdb target)
  if(WIN32)
    target_include_directories(${target} PUBLIC "${UOCR_DUCKDB_ROOT}/generated/include")
    target_link_libraries(${target} PUBLIC "${UOCR_DUCKDB_ROOT}/lib/duckdb.lib")
    message(STATUS "Using bundled Windows DuckDB SDK because the vcpkg duckdb port fails on MSVC 19.51 with C1083 generated-file errors")
    return()
  endif()

  find_package(DuckDB CONFIG REQUIRED)
  foreach(_candidate IN ITEMS DuckDB::DuckDB duckdb::duckdb duckdb duckdb_static)
    if(TARGET ${_candidate})
      uocr_assert_vcpkg_target(${_candidate})
      target_link_libraries(${target} PUBLIC ${_candidate})
      return()
    endif()
  endforeach()
  message(FATAL_ERROR "vcpkg duckdb was found without a recognized CMake target.")
endfunction()

function(uocr_copy_duckdb_runtime target)
  if(WIN32)
    add_custom_command(TARGET ${target} POST_BUILD
      COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "${UOCR_DUCKDB_ROOT}/bin/duckdb.dll"
        "$<TARGET_FILE_DIR:${target}>/duckdb.dll"
      VERBATIM
    )
  endif()
endfunction()

function(uocr_configure_openssl)
  find_package(OpenSSL 3.6.3 EXACT REQUIRED COMPONENTS SSL Crypto)
  uocr_assert_vcpkg_target(OpenSSL::SSL)
  uocr_assert_vcpkg_target(OpenSSL::Crypto)
  uocr_assert_vcpkg_path("OpenSSL include directory" "${OPENSSL_INCLUDE_DIR}")

  add_library(uocr_openssl_crypto INTERFACE)
  target_link_libraries(uocr_openssl_crypto INTERFACE OpenSSL::Crypto)
  add_library(uocr_openssl_ssl INTERFACE)
  target_link_libraries(uocr_openssl_ssl INTERFACE OpenSSL::SSL)

  set(_uocr_openssl_version "${OPENSSL_VERSION}")
  if(NOT _uocr_openssl_version AND DEFINED OpenSSL_VERSION)
    set(_uocr_openssl_version "${OpenSSL_VERSION}")
  endif()
  if(DEFINED OPENSSL_VERSION)
    set(UOCR_OPENSSL_VERSION "${OPENSSL_VERSION}" PARENT_SCOPE)
  elseif(DEFINED OpenSSL_VERSION)
    set(UOCR_OPENSSL_VERSION "${OpenSSL_VERSION}" PARENT_SCOPE)
  endif()
  message(STATUS "Using vcpkg OpenSSL ${_uocr_openssl_version} for TLS and SHA verification")
endfunction()

function(uocr_configure_drogon)
  set(TRANTOR_USE_TLS "openssl" CACHE STRING "Trantor TLS backend" FORCE)
  find_package(Trantor 1.5.28 EXACT CONFIG REQUIRED)
  find_package(Drogon 1.9.13 EXACT CONFIG REQUIRED)
  uocr_assert_vcpkg_target(Trantor::Trantor)
  uocr_assert_vcpkg_target(Drogon::Drogon)

  if(DEFINED Trantor_DIR)
    set(_uocr_trantor_config "${Trantor_DIR}/TrantorConfig.cmake")
    if(EXISTS "${_uocr_trantor_config}")
      file(READ "${_uocr_trantor_config}" _uocr_trantor_config_text)
      if(NOT _uocr_trantor_config_text MATCHES "find_dependency\\(OpenSSL")
        message(FATAL_ERROR "Trantor vcpkg config does not require OpenSSL; Drogon TLS is not enabled.")
      endif()
    endif()
  endif()

  set(Drogon_FOUND "${Drogon_FOUND}" PARENT_SCOPE)
  message(STATUS "Using vcpkg Drogon 1.9.13 with Trantor/OpenSSL TLS")
endfunction()
