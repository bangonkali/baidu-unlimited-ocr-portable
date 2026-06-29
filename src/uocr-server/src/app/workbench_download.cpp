#include "workbench_download.hpp"

#include <stdexcept>
#include <utility>

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <urlmon.h>
#include <windows.h>
#endif

namespace uocr::server {
namespace {

#ifdef _WIN32
class DownloadStatus final : public IBindStatusCallback {
 public:
  explicit DownloadStatus(DownloadProgressCallback progress) : progress_(std::move(progress)) {}

  HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** object) override {
    if (object == nullptr) {
      return E_POINTER;
    }
    if (iid == IID_IUnknown || iid == IID_IBindStatusCallback) {
      *object = static_cast<IBindStatusCallback*>(this);
      AddRef();
      return S_OK;
    }
    *object = nullptr;
    return E_NOINTERFACE;
  }

  ULONG STDMETHODCALLTYPE AddRef() override {
    return InterlockedIncrement(&ref_count_);
  }

  ULONG STDMETHODCALLTYPE Release() override {
    const auto value = InterlockedDecrement(&ref_count_);
    if (value == 0) {
      delete this;
    }
    return value;
  }

  HRESULT STDMETHODCALLTYPE OnProgress(ULONG progress,
                                       ULONG progress_max,
                                       ULONG,
                                       LPCWSTR status_text) override {
    if (!progress_) {
      return S_OK;
    }
    DownloadProgress update;
    update.downloaded_bytes = progress;
    update.total_bytes = progress_max;
    if (status_text != nullptr) {
      const int bytes = WideCharToMultiByte(CP_UTF8, 0, status_text, -1, nullptr, 0, nullptr, nullptr);
      update.current_file.resize(static_cast<std::size_t>(bytes > 0 ? bytes - 1 : 0));
      if (bytes > 1) {
        WideCharToMultiByte(CP_UTF8, 0, status_text, -1, update.current_file.data(), bytes, nullptr, nullptr);
      }
    }
    progress_(update);
    return S_OK;
  }

  HRESULT STDMETHODCALLTYPE OnStartBinding(DWORD, IBinding*) override { return S_OK; }
  HRESULT STDMETHODCALLTYPE GetPriority(LONG*) override { return E_NOTIMPL; }
  HRESULT STDMETHODCALLTYPE OnLowResource(DWORD) override { return S_OK; }
  HRESULT STDMETHODCALLTYPE OnStopBinding(HRESULT, LPCWSTR) override { return S_OK; }
  HRESULT STDMETHODCALLTYPE GetBindInfo(DWORD*, BINDINFO*) override { return E_NOTIMPL; }
  HRESULT STDMETHODCALLTYPE OnDataAvailable(DWORD, DWORD, FORMATETC*, STGMEDIUM*) override { return S_OK; }
  HRESULT STDMETHODCALLTYPE OnObjectAvailable(REFIID, IUnknown*) override { return S_OK; }

 private:
  volatile LONG ref_count_ = 1;
  DownloadProgressCallback progress_;
};
#endif

}  // namespace

void download_to_file(const std::string& url,
                      const std::filesystem::path& destination,
                      const DownloadProgressCallback& progress) {
  std::filesystem::create_directories(destination.parent_path());
  const auto temp = destination.string() + ".download";
#ifdef _WIN32
  auto* callback = new DownloadStatus(progress);
  const auto result = URLDownloadToFileA(nullptr, url.c_str(), temp.c_str(), 0, callback);
  callback->Release();
  if (result != S_OK) {
    throw std::runtime_error("download failed for " + url);
  }
#else
  (void)url;
  (void)progress;
  throw std::runtime_error("model download is implemented for Windows portable builds first");
#endif
  std::error_code error;
  std::filesystem::rename(temp, destination, error);
  if (error) {
    std::filesystem::remove(destination, error);
    std::filesystem::rename(temp, destination, error);
  }
  if (error) {
    throw std::runtime_error("could not finalize model download: " + destination.string());
  }
}

}  // namespace uocr::server
