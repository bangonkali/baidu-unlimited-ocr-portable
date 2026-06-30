#include "workbench_state.hpp"

namespace uocr::server {

void WorkbenchService::Impl::refresh_document_aggregate(DocumentState& document) const {
  document.raw_text.clear();
  document.cleaned_text.clear();
  document.boxes.clear();
  document.spans.clear();
  for (const auto& page : document.pages) {
    if (!document.raw_text.empty()) {
      document.raw_text += "\n\n";
      document.cleaned_text += "\n\n";
    }
    document.raw_text += page.raw_text;
    document.cleaned_text += page.cleaned_text;
    document.boxes.insert(document.boxes.end(), page.boxes.begin(), page.boxes.end());
    document.spans.insert(document.spans.end(), page.spans.begin(), page.spans.end());
  }
}

void WorkbenchService::Impl::apply_region_content(PageState& page) const {
  for (auto& box : page.boxes) {
    for (const auto& span : page.spans) {
      if (span.region_id == box.region_id && span.start <= span.end && span.end <= page.cleaned_text.size()) {
        box.content_markdown = page.cleaned_text.substr(span.start, span.end - span.start);
        break;
      }
    }
    if (box.content_markdown.empty()) {
      box.content_markdown = box.label;
    }
  }
}

std::string WorkbenchService::Impl::document_status_for(const DocumentState& document) const {
  bool completed = false;
  bool failed = false;
  for (const auto& page : document.pages) {
    completed = completed || page.status == "completed";
    failed = failed || page.status == "failed";
  }
  if (completed && failed) {
    return "completed_with_errors";
  }
  return failed ? "failed" : "completed";
}

}  // namespace uocr::server
