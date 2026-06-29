#include <cassert>
#include <filesystem>
#include <fstream>
#include <sstream>

#include "uocr/core/ocr_parser.hpp"
#include "uocr/core/profiles.hpp"
#include "uocr/core/runaway_guard.hpp"
#include "uocr/fs/file_scanner.hpp"
#include "uocr/render/png_dimensions.hpp"
#include "uocr/storage/migrations.hpp"

namespace {

void test_parser() {
  const std::string raw =
      "hello <|ref|>Invoice total<|/ref|><|det|>[[10, 20, 100, 220]]<|/det|> "
      "<|det|>Logo [300, 40, 500, 180]<|/det|>";
  const auto parsed = uocr::parse_ocr_markers(raw, {.file_hash = "abc", .page_no = 2});
  assert(parsed.boxes.size() == 2);
  assert(parsed.text_region_spans.size() == 2);
  assert(parsed.cleaned_text.find("Invoice total") != std::string::npos);
  assert(parsed.cleaned_text.find("<|det|>") == std::string::npos);

  const auto overlays = uocr::to_overlay_boxes(parsed, 2);
  assert(overlays.size() == 2);
  assert(overlays[0].page_no == 2);
  assert(overlays[0].width_percent > 8.9 && overlays[0].width_percent < 9.1);
}

void test_runaway_guard() {
  std::ostringstream output;
  for (int value = 1; value <= 60; ++value) {
    output << value << ", ";
  }
  assert(uocr::detect_recoverable_output_issue(output.str()).has_value());
  assert(!uocr::detect_recoverable_output_issue("ordinary OCR text 10, 12, 18").has_value());
}

void test_scanner() {
  const auto root = std::filesystem::temp_directory_path() / "uocr_scanner_test";
  std::filesystem::remove_all(root);
  std::filesystem::create_directories(root / "nested");
  std::ofstream(root / "nested" / "page.png").put('x');
  std::ofstream(root / "sample.pdf").put('x');
  std::ofstream(root / "notes.txt").put('x');

  const auto files = uocr::discover_supported_files(root);
  assert(files.size() == 2);
  assert(files[0].relative_path.generic_string() == "nested/page.png");
  assert(files[1].relative_path.generic_string() == "sample.pdf");
  std::filesystem::remove_all(root);
}

void test_png_dimensions() {
  const auto path = std::filesystem::temp_directory_path() / "uocr_png_dimensions_test.png";
  const unsigned char bytes[] = {
      0x89, 'P', 'N', 'G', '\r', '\n', 0x1a, '\n', 0, 0, 0, 13, 'I', 'H', 'D', 'R',
      0,    0,   1,   0x2c, 0,    0,    0,    0xc8, 8, 2, 0, 0,
  };
  std::ofstream output(path, std::ios::binary);
  output.write(reinterpret_cast<const char*>(bytes), sizeof(bytes));
  output.close();
  const auto size = uocr::read_png_dimensions(path);
  assert(size.width_px == 300);
  assert(size.height_px == 200);
  std::filesystem::remove(path);
}

void test_profiles_and_migrations() {
  assert(uocr::default_ocr_profile().key == "best-zero-empty-q4");
  assert(uocr::find_ocr_profile("experimental-exact-prefill-q4") != nullptr);
  assert(!uocr::duckdb_migrations().empty());
}

}  // namespace

int main() {
  test_parser();
  test_runaway_guard();
  test_scanner();
  test_png_dimensions();
  test_profiles_and_migrations();
  return 0;
}
