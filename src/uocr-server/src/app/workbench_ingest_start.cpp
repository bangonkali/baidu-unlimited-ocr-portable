#include "workbench_state.hpp"

#include <thread>
#include <utility>

namespace uocr::server {

void WorkbenchService::Impl::start_run(std::string const& run_id,
                                       std::vector<DiscoveredFile> files,
                                       std::string profile_id,
                                       std::string model_id) {
  std::thread([shared = shared_from_this(),
               run_id,
               files = std::move(files),
               profile_id = std::move(profile_id),
               model_id = std::move(model_id)]() {
    shared->process_run(run_id, files, profile_id, model_id);
  }).detach();
}

}  // namespace uocr::server
