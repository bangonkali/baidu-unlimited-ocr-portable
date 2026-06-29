#pragma once

#include <filesystem>
#include <vector>

#include "uocr/render/page_renderer.hpp"

namespace uocr {

class MupdfPageRenderer final : public PageRenderer {
 public:
  std::vector<RenderedPage> render_document(const std::filesystem::path& source_path,
                                            const std::filesystem::path& cache_root, int dpi) override;
};

}  // namespace uocr
