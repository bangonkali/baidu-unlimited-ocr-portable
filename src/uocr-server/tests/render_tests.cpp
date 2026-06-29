#include <cassert>
#include <chrono>
#include <filesystem>
#include <iostream>
#include <string>

#include "uocr/render/mupdf_page_renderer.hpp"

#ifndef UOCR_SAMPLE_PDF
#define UOCR_SAMPLE_PDF ""
#endif

namespace {

std::filesystem::path temp_cache_root() {
  const auto stamp = std::chrono::steady_clock::now().time_since_epoch().count();
  return std::filesystem::temp_directory_path() / ("uocr-render-test-" + std::to_string(stamp));
}

void test_sample_pdf_render() {
  const std::filesystem::path sample_pdf = UOCR_SAMPLE_PDF;
  assert(std::filesystem::exists(sample_pdf));

  const auto cache_root = temp_cache_root();
  uocr::MupdfPageRenderer renderer;
  const auto pages = renderer.render_document(sample_pdf, cache_root, 200);
  assert(!pages.empty());
  assert(pages.front().page_no == 1);
  assert(std::filesystem::exists(pages.front().image_path));
  assert(pages.front().width_px > 0);
  assert(pages.front().height_px > 0);
  assert(pages.front().dpi == 200);

  std::error_code error;
  std::filesystem::remove_all(cache_root, error);
}

}  // namespace

int main() {
  test_sample_pdf_render();
  std::cout << "render tests passed\n";
  return 0;
}
