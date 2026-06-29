#include "hf_transfer.hpp"

#include <curl/curl.h>

#include <algorithm>
#include <chrono>
#include <cctype>
#include <fstream>
#include <sstream>
#include <utility>

#include "uocr/download/download_progress.hpp"
#include "uocr/download/sha256.hpp"

namespace uocr::download::detail {
namespace {

struct CurlGlobal {
  CurlGlobal() { curl_global_init(CURL_GLOBAL_DEFAULT); }
  ~CurlGlobal() { curl_global_cleanup(); }
};

struct HeaderState {
  int status = 0;
  std::uint64_t content_length = 0;
  std::uint64_t linked_size = 0;
  std::string linked_etag;
  std::string location;
};

struct CurlHandle {
  CurlHandle() : handle(curl_easy_init()) {
    if (handle == nullptr) {
      throw HfDownloadException("could not initialize libcurl", true);
    }
  }
  ~CurlHandle() { curl_easy_cleanup(handle); }
  CURL* handle;
};

std::string trim(std::string value) {
  auto is_space = [](unsigned char ch) { return std::isspace(ch) != 0; };
  value.erase(value.begin(), std::find_if_not(value.begin(), value.end(), is_space));
  value.erase(std::find_if_not(value.rbegin(), value.rend(), is_space).base(), value.end());
  return value;
}

std::string lower(std::string value) {
  std::transform(value.begin(), value.end(), value.begin(), [](unsigned char ch) {
    return static_cast<char>(std::tolower(ch));
  });
  return value;
}

std::uint64_t parse_size(std::string_view value) {
  try {
    return static_cast<std::uint64_t>(std::stoull(std::string(value)));
  } catch (const std::exception&) {
    return 0;
  }
}

bool is_sha256(std::string_view value) {
  return value.size() == 64 && std::all_of(value.begin(), value.end(), [](unsigned char ch) {
           return std::isxdigit(ch) != 0;
         });
}

std::string clean_etag(std::string value) {
  value = trim(std::move(value));
  if (value.size() >= 2 && value.front() == '"' && value.back() == '"') {
    value = value.substr(1, value.size() - 2);
  }
  return is_sha256(value) ? lower(std::move(value)) : std::string();
}

std::string resolve_location(std::string location) {
  location = trim(std::move(location));
  if (location.starts_with("/")) {
    return "https://huggingface.co" + location;
  }
  return location;
}

size_t header_callback(char* buffer, size_t size, size_t items, void* user_data) {
  const auto bytes = size * items;
  auto* state = static_cast<HeaderState*>(user_data);
  std::string line(buffer, bytes);
  line = trim(std::move(line));
  if (line.empty()) {
    return bytes;
  }
  if (line.starts_with("HTTP/")) {
    state->location.clear();
    std::istringstream input(line);
    std::string http_version;
    input >> http_version >> state->status;
    return bytes;
  }
  const auto separator = line.find(':');
  if (separator == std::string::npos) {
    return bytes;
  }
  const auto name = lower(trim(line.substr(0, separator)));
  const auto value = trim(line.substr(separator + 1));
  if (name == "content-length") {
    state->content_length = parse_size(value);
  } else if (name == "x-linked-size") {
    state->linked_size = parse_size(value);
  } else if (name == "x-linked-etag" || name == "etag") {
    state->linked_etag = value;
  } else if (name == "location") {
    state->location = value;
  }
  return bytes;
}

curl_slist* build_headers(const std::string& token) {
  curl_slist* headers = nullptr;
  if (!token.empty()) {
    const auto auth = "Authorization: Bearer " + token;
    headers = curl_slist_append(headers, auth.c_str());
  }
  return headers;
}

std::string resolve_url(const HfDownloadOptions& options, const HfFileSpec& spec) {
  return "https://huggingface.co/" + options.repo_id + "/resolve/" + options.revision + "/" + spec.file_name;
}

void set_common_options(CURL* curl, const std::string& url, const std::string& user_agent) {
  curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
  curl_easy_setopt(curl, CURLOPT_USERAGENT, user_agent.c_str());
  curl_easy_setopt(curl, CURLOPT_NOSIGNAL, 1L);
  curl_easy_setopt(curl, CURLOPT_CONNECTTIMEOUT, 30L);
  curl_easy_setopt(curl, CURLOPT_LOW_SPEED_LIMIT, 1024L);
  curl_easy_setopt(curl, CURLOPT_LOW_SPEED_TIME, 60L);
}

}  // namespace

void initialize_curl_once() {
  static CurlGlobal curl_global;
  (void)curl_global;
}

PreparedFile prepare_file(const HfFileSpec& spec, const HfDownloadOptions& options) {
  CurlHandle curl;
  HeaderState headers;
  const auto url = resolve_url(options, spec);
  auto* request_headers = build_headers(options.token);
  set_common_options(curl.handle, url, options.user_agent);
  curl_easy_setopt(curl.handle, CURLOPT_NOBODY, 1L);
  curl_easy_setopt(curl.handle, CURLOPT_FOLLOWLOCATION, 0L);
  curl_easy_setopt(curl.handle, CURLOPT_HEADERFUNCTION, header_callback);
  curl_easy_setopt(curl.handle, CURLOPT_HEADERDATA, &headers);
  curl_easy_setopt(curl.handle, CURLOPT_HTTPHEADER, request_headers);
  const auto result = curl_easy_perform(curl.handle);
  curl_slist_free_all(request_headers);
  if (result != CURLE_OK) {
    throw HfDownloadException("metadata request failed: " + std::string(curl_easy_strerror(result)), true);
  }
  long response_code = headers.status;
  curl_easy_getinfo(curl.handle, CURLINFO_RESPONSE_CODE, &response_code);
  if (response_code == 401 || response_code == 403 || response_code == 404) {
    throw HfDownloadException("Hugging Face returned HTTP " + std::to_string(response_code), false,
                              static_cast<int>(response_code));
  }
  if (response_code >= 500) {
    throw HfDownloadException("Hugging Face returned HTTP " + std::to_string(response_code), true,
                              static_cast<int>(response_code));
  }
  if (response_code >= 300 && response_code < 400 && headers.location.empty()) {
    throw HfDownloadException("Hugging Face did not provide a download redirect", true, static_cast<int>(response_code));
  }
  PreparedFile prepared;
  prepared.spec = spec;
  prepared.url = headers.location.empty() ? url : resolve_location(headers.location);
  prepared.send_auth = headers.location.empty() && !options.token.empty();
  prepared.size = headers.linked_size != 0 ? headers.linked_size : headers.content_length;
  prepared.sha256 = clean_etag(headers.linked_etag);
  return prepared;
}

}  // namespace uocr::download::detail
