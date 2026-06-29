#include "workbench_state.hpp"

#include <algorithm>
#include <cctype>
#include <stdexcept>
#include <utility>

#include "uocr/app/app_logger.hpp"
#include "uocr/render/mupdf_page_renderer.hpp"
#include "uocr/render/png_dimensions.hpp"

namespace uocr::server {
namespace {

bool has_extension(const std::filesystem::path& path, std::string_view expected) {
  auto ext = path.extension().string();
  std::transform(ext.begin(), ext.end(), ext.begin(), [](unsigned char ch) {
    return static_cast<char>(std::tolower(ch));
  });
  return ext == expected;
}

bool is_image_file(const std::filesystem::path& path) {
  auto ext = path.extension().string();
  std::transform(ext.begin(), ext.end(), ext.begin(), [](unsigned char ch) {
    return static_cast<char>(std::tolower(ch));
  });
  return ext == ".bmp" || ext == ".jpeg" || ext == ".jpg" || ext == ".png" || ext == ".tif" ||
         ext == ".tiff" || ext == ".webp";
}

void log_info(const std::shared_ptr<AppLogger>& logger, std::string_view component, const std::string& message) {
  if (logger) {
    logger->info(component, message);
  }
}

}  // namespace

std::vector<WorkbenchService::Impl::PageState> WorkbenchService::Impl::prepare_pages(
    const DiscoveredFile& file, const std::string& file_hash) const {
  if (is_image_file(file.absolute_path)) {
    PageState page;
    page.image_path = file.absolute_path;
    try {
      const auto size = read_png_dimensions(file.absolute_path);
      page.width_px = size.width_px;
      page.height_px = size.height_px;
    } catch (const std::exception&) {
      page.width_px = 0;
      page.height_px = 0;
    }
    return {page};
  }

  if (!has_extension(file.absolute_path, ".pdf")) {
    throw std::runtime_error("unsupported input type");
  }

  log_info(logger, "pdf", "rendering " + file.relative_path.generic_string() + " at 200 DPI with MuPDF");
  const auto cache_root = app_root / "cache" / "rendered-pages" / file_hash;
  MupdfPageRenderer renderer;
  const auto rendered_pages = renderer.render_document(file.absolute_path, cache_root, 200);
  std::vector<PageState> pages;
  pages.reserve(rendered_pages.size());
  for (const auto& rendered : rendered_pages) {
    PageState page;
    page.page_no = rendered.page_no;
    page.image_path = rendered.image_path;
    page.width_px = rendered.width_px;
    page.height_px = rendered.height_px;
    page.dpi = rendered.dpi;
    pages.push_back(std::move(page));
  }
  return pages;
}

}  // namespace uocr::server
