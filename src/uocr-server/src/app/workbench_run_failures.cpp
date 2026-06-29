#include "workbench_state.hpp"

#include <vector>

namespace uocr::server {

void WorkbenchService::Impl::fail_run(const std::string& run_id, const std::string& message) {
  Json::Value run_event;
  std::vector<Json::Value> document_events;
  {
    std::scoped_lock lock(mutex);
    auto& run = runs[run_id];
    run.status = "failed";
    run.error = message;
    for (const auto& hash : run.file_hashes) {
      auto& document = documents[hash];
      if (document.status == "queued" || document.status == "running" || document.status == "rendering") {
        document.status = "failed";
        document.error = message;
      }
      document_events.push_back(document_summary(document));
    }
    run_event = run_record(run);
  }
  publish_event("run.changed", run_event);
  for (const auto& document_event : document_events) {
    publish_event("document.changed", document_event);
  }
  publish_status_changed();
}

}  // namespace uocr::server
