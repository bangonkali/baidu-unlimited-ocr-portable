#pragma once

#include <filesystem>
#include <string>
#include <vector>

namespace uocr {

struct RenderedPage {
  int page_no = 1;
  std::filesystem::path image_path;
  int width_px = 0;
  int height_px = 0;
  int dpi = 200;
};

class PageRenderer {
 public:
  virtual ~PageRenderer() = default;
  virtual std::vector<RenderedPage> render_document(const std::filesystem::path& source_path,
                                                    const std::filesystem::path& cache_root, int dpi) = 0;
};

}  // namespace uocr

