#include "folder_dialog.hpp"

#include <array>
#include <cstdio>
#include <future>
#include <string>
#include <thread>
#include <utility>

#ifdef _WIN32
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <shobjidl.h>
#include <windows.h>
#endif

#ifdef __APPLE__
#include <sys/wait.h>
#endif

namespace uocr::server {
namespace {

Json::Value dialog_payload(bool cancelled, const std::string& path, const std::string& error = "") {
  Json::Value payload;
  payload["cancelled"] = cancelled;
  payload["selected_path"] = path;
  payload["manual_path_supported"] = true;
  if (!error.empty()) {
    payload["error"] = error;
  }
  return payload;
}

#ifdef _WIN32
Json::Value show_windows_folder_dialog() {
  const auto init = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
  if (FAILED(init)) {
    return dialog_payload(true, "", "could not initialize Windows folder picker");
  }

  IFileOpenDialog* dialog = nullptr;
  auto hr = CoCreateInstance(CLSID_FileOpenDialog, nullptr, CLSCTX_INPROC_SERVER,
                             IID_PPV_ARGS(&dialog));
  if (FAILED(hr) || dialog == nullptr) {
    CoUninitialize();
    return dialog_payload(true, "", "could not create Windows folder picker");
  }

  DWORD options = 0;
  if (SUCCEEDED(dialog->GetOptions(&options))) {
    dialog->SetOptions(options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST);
  }
  hr = dialog->Show(nullptr);
  if (hr == HRESULT_FROM_WIN32(ERROR_CANCELLED)) {
    dialog->Release();
    CoUninitialize();
    return dialog_payload(true, "");
  }
  if (FAILED(hr)) {
    dialog->Release();
    CoUninitialize();
    return dialog_payload(true, "", "folder picker failed");
  }

  IShellItem* item = nullptr;
  hr = dialog->GetResult(&item);
  if (FAILED(hr) || item == nullptr) {
    dialog->Release();
    CoUninitialize();
    return dialog_payload(true, "", "folder picker returned no folder");
  }

  PWSTR path = nullptr;
  hr = item->GetDisplayName(SIGDN_FILESYSPATH, &path);
  std::string selected;
  if (SUCCEEDED(hr) && path != nullptr) {
    const int bytes = WideCharToMultiByte(CP_UTF8, 0, path, -1, nullptr, 0, nullptr, nullptr);
    selected.resize(static_cast<std::size_t>(bytes > 0 ? bytes - 1 : 0));
    if (bytes > 1) {
      WideCharToMultiByte(CP_UTF8, 0, path, -1, selected.data(), bytes, nullptr, nullptr);
    }
    CoTaskMemFree(path);
  }
  item->Release();
  dialog->Release();
  CoUninitialize();
  return selected.empty() ? dialog_payload(true, "", "folder picker returned an empty path")
                          : dialog_payload(false, selected);
}
#endif

#ifdef __APPLE__
std::string trim_trailing_line_endings(std::string value) {
  while (!value.empty() && (value.back() == '\n' || value.back() == '\r')) {
    value.pop_back();
  }
  return value;
}

bool user_cancelled_apple_dialog(const std::string& output) {
  return output.find("User canceled") != std::string::npos ||
         output.find("(-128)") != std::string::npos;
}

Json::Value show_macos_folder_dialog() {
  constexpr const char* command =
      "/usr/bin/osascript "
      "-e 'set selectedFolder to choose folder with prompt \"Choose a folder to scan with Unlimited OCR\"' "
      "-e 'POSIX path of selectedFolder' 2>&1";
  FILE* pipe = popen(command, "r");
  if (pipe == nullptr) {
    return dialog_payload(true, "", "could not start macOS folder picker");
  }

  std::string output;
  std::array<char, 256> buffer{};
  while (fgets(buffer.data(), static_cast<int>(buffer.size()), pipe) != nullptr) {
    output += buffer.data();
  }

  const int status = pclose(pipe);
  output = trim_trailing_line_endings(output);
  if (status == -1) {
    return dialog_payload(true, "", "macOS folder picker failed");
  }
  if (WIFEXITED(status) && WEXITSTATUS(status) == 0) {
    return output.empty() ? dialog_payload(true, "", "folder picker returned an empty path")
                          : dialog_payload(false, output);
  }
  if (user_cancelled_apple_dialog(output)) {
    return dialog_payload(true, "");
  }
  return dialog_payload(true, "", output.empty() ? "macOS folder picker failed" : output);
}
#endif

}  // namespace

Json::Value open_folder_dialog() {
#ifdef _WIN32
  std::packaged_task<Json::Value()> task(show_windows_folder_dialog);
  auto result = task.get_future();
  std::thread(std::move(task)).detach();
  return result.get();
#elif defined(__APPLE__)
  return show_macos_folder_dialog();
#else
  return dialog_payload(true, "", "native folder picker is only implemented on Windows and macOS");
#endif
}

}  // namespace uocr::server
