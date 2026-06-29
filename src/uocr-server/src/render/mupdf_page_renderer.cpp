#include "uocr/render/mupdf_page_renderer.hpp"

#include <algorithm>
#include <cctype>
#include <cstdio>
#include <stdexcept>
#include <string>

#include "mupdf/fitz.h"
#include "uocr/render/png_dimensions.hpp"

namespace uocr {
namespace {

bool is_pdf(const std::filesystem::path& path) {
  auto ext = path.extension().string();
  std::transform(ext.begin(), ext.end(), ext.begin(), [](unsigned char ch) {
    return static_cast<char>(std::tolower(ch));
  });
  return ext == ".pdf";
}

std::string path_to_utf8(const std::filesystem::path& path) {
#ifdef _WIN32
  const auto value = path.u8string();
  return {reinterpret_cast<const char*>(value.data()), value.size()};
#else
  return path.string();
#endif
}

int page_no_from_path(const std::filesystem::path& path) {
  const auto stem = path.stem().string();
  if (stem.rfind("page-", 0) != 0) {
    return 0;
  }
  return std::stoi(stem.substr(5));
}

std::filesystem::path page_path(const std::filesystem::path& cache_root, int page_index) {
  char name[32];
  std::snprintf(name, sizeof(name), "page-%04d.png", page_index + 1);
  return cache_root / name;
}

std::vector<RenderedPage> collect_pages(const std::filesystem::path& cache_root, int dpi) {
  std::vector<RenderedPage> pages;
  std::error_code error;
  for (const auto& entry : std::filesystem::directory_iterator(cache_root, error)) {
    if (error || !entry.is_regular_file() || entry.path().extension() != ".png") {
      continue;
    }
    const auto page_no = page_no_from_path(entry.path());
    if (page_no <= 0) {
      continue;
    }
    const auto size = read_png_dimensions(entry.path());
    pages.push_back({.page_no = page_no,
                     .image_path = entry.path(),
                     .width_px = size.width_px,
                     .height_px = size.height_px,
                     .dpi = dpi});
  }
  std::sort(pages.begin(), pages.end(), [](const RenderedPage& left, const RenderedPage& right) {
    return left.page_no < right.page_no;
  });
  return pages;
}

void remove_cached_pages(const std::filesystem::path& cache_root) {
  std::error_code error;
  for (const auto& entry : std::filesystem::directory_iterator(cache_root, error)) {
    if (!error && entry.is_regular_file() && entry.path().extension() == ".png") {
      std::filesystem::remove(entry.path(), error);
    }
  }
}

std::string caught_message(fz_context* context, const std::string& fallback) {
  const char* message = fz_caught_message(context);
  return message == nullptr || message[0] == '\0' ? fallback : message;
}

void register_document_handlers(fz_context* context) {
  fz_try(context) {
    fz_register_document_handlers(context);
  }
  fz_catch(context) {
    throw std::runtime_error("MuPDF failed to register document handlers: " + caught_message(context, "unknown error"));
  }
}

fz_document* open_document(fz_context* context, const std::string& source_path) {
  fz_document* document = nullptr;
  fz_try(context) {
    document = fz_open_document(context, source_path.c_str());
  }
  fz_catch(context) {
    throw std::runtime_error("MuPDF failed to open PDF: " + caught_message(context, "unknown error"));
  }
  return document;
}

int count_pages(fz_context* context, fz_document* document) {
  int count = 0;
  fz_try(context) {
    count = fz_count_pages(context, document);
  }
  fz_catch(context) {
    throw std::runtime_error("MuPDF failed to count pages: " + caught_message(context, "unknown error"));
  }
  return count;
}

void render_page(fz_context* context, fz_document* document, int page_index, float scale,
                 const std::string& output_path) {
  fz_page* page = nullptr;
  fz_pixmap* pixmap = nullptr;
  fz_try(context) {
    const fz_matrix ctm = fz_scale(scale, scale);
    page = fz_load_page(context, document, page_index);
    pixmap = fz_new_pixmap_from_page(context, page, ctm, fz_device_rgb(context), 0);
    fz_save_pixmap_as_png(context, pixmap, output_path.c_str());
  }
  fz_always(context) {
    fz_drop_pixmap(context, pixmap);
    fz_drop_page(context, page);
  }
  fz_catch(context) {
    throw std::runtime_error("MuPDF failed to render page " + std::to_string(page_index + 1) + ": " +
                             caught_message(context, "unknown error"));
  }
}

void render_pdf(const std::filesystem::path& source_path, const std::filesystem::path& cache_root, int dpi) {
  fz_context* context = fz_new_context(nullptr, nullptr, FZ_STORE_UNLIMITED);
  if (context == nullptr) {
    throw std::runtime_error("MuPDF failed to create a rendering context");
  }

  fz_document* document = nullptr;
  try {
    register_document_handlers(context);
    document = open_document(context, path_to_utf8(source_path));
    const int pages = count_pages(context, document);
    if (pages <= 0) {
      throw std::runtime_error("MuPDF reported no pages in " + source_path.string());
    }
    const float scale = static_cast<float>(dpi) / 72.0F;
    for (int index = 0; index < pages; ++index) {
      render_page(context, document, index, scale, path_to_utf8(page_path(cache_root, index)));
    }
  } catch (...) {
    fz_drop_document(context, document);
    fz_drop_context(context);
    throw;
  }

  fz_drop_document(context, document);
  fz_drop_context(context);
}

}  // namespace

std::vector<RenderedPage> MupdfPageRenderer::render_document(const std::filesystem::path& source_path,
                                                             const std::filesystem::path& cache_root, int dpi) {
  if (!is_pdf(source_path)) {
    throw std::runtime_error("MuPDF renderer only accepts PDF files");
  }
  if (!std::filesystem::exists(source_path)) {
    throw std::runtime_error("PDF does not exist: " + source_path.string());
  }

  std::filesystem::create_directories(cache_root);
  auto pages = collect_pages(cache_root, dpi);
  if (!pages.empty()) {
    return pages;
  }

  remove_cached_pages(cache_root);
  render_pdf(source_path, cache_root, dpi);
  pages = collect_pages(cache_root, dpi);
  if (pages.empty()) {
    throw std::runtime_error("MuPDF did not render any pages for " + source_path.string());
  }
  return pages;
}

}  // namespace uocr
