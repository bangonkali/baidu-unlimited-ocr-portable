#include <cassert>
#include <chrono>
#include <cstdlib>
#include <exception>
#include <filesystem>
#include <fstream>
#include <thread>
#include <string>

#include "uocr/download/download_progress.hpp"
#include "uocr/download/hf_auth.hpp"
#include "uocr/download/sha256.hpp"

namespace {

void set_env(const char* name, const char* value) {
#ifdef _WIN32
  _putenv_s(name, value == nullptr ? "" : value);
#else
  if (value == nullptr) {
    unsetenv(name);
  } else {
    setenv(name, value, 1);
  }
#endif
}

void test_auth_precedence() {
  set_env("HF_TOKEN", "");
  set_env("HUGGING_FACE_HUB_TOKEN", "");
  assert(!uocr::download::read_hf_auth_from_environment().available());

  set_env("HUGGING_FACE_HUB_TOKEN", "fallback-token");
  auto auth = uocr::download::read_hf_auth_from_environment();
  assert(auth.available());
  assert(auth.source == "HUGGING_FACE_HUB_TOKEN");
  assert(auth.token == "fallback-token");

  set_env("HF_TOKEN", "primary-token");
  auth = uocr::download::read_hf_auth_from_environment();
  assert(auth.available());
  assert(auth.source == "HF_TOKEN");
  assert(auth.token == "primary-token");

  set_env("HF_TOKEN", "");
  set_env("HUGGING_FACE_HUB_TOKEN", "");
}

void test_progress_math() {
  assert(uocr::download::percent_complete(50, 200) == 25.0);
  assert(uocr::download::percent_complete(250, 200) == 100.0);
  assert(uocr::download::percent_complete(1, 0) == 0.0);

  const auto rate = uocr::download::transfer_rate_bytes_per_second(10 * 1024 * 1024, std::chrono::seconds(2));
  assert(rate == 5 * 1024 * 1024);
  const auto eta = uocr::download::eta_seconds(10, 30, 5);
  assert(eta.has_value());
  assert(*eta == 4.0);
  assert(!uocr::download::eta_seconds(30, 30, 5).has_value());
}

void test_sha256() {
  const auto path = std::filesystem::temp_directory_path() / "uocr-sha256-test.txt";
  std::ofstream output(path, std::ios::binary);
  output << "abc";
  output.close();
  assert(uocr::download::sha256_file(path) == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
  std::filesystem::remove(path);
}

void test_sha256_on_worker_thread() {
  const auto path = std::filesystem::temp_directory_path() / "uocr-sha256-worker-test.txt";
  std::ofstream output(path, std::ios::binary);
  output << "abc";
  output.close();

  std::string hash;
  std::exception_ptr error;
  std::thread worker([&hash, &error, &path]() {
    try {
      hash = uocr::download::sha256_file(path);
    } catch (...) {
      error = std::current_exception();
    }
  });
  worker.join();
  if (error) {
    std::rethrow_exception(error);
  }

  assert(hash == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
  std::filesystem::remove(path);
}

}  // namespace

int main() {
  test_auth_precedence();
  test_progress_math();
  test_sha256();
  test_sha256_on_worker_thread();
  return 0;
}
