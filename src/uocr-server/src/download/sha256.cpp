#include "uocr/download/sha256.hpp"

#include <openssl/evp.h>

#include <array>
#include <fstream>
#include <iomanip>
#include <memory>
#include <sstream>
#include <stdexcept>

namespace uocr::download {
namespace {

using EvpContext = std::unique_ptr<EVP_MD_CTX, decltype(&EVP_MD_CTX_free)>;

}  // namespace

std::string sha256_file(const std::filesystem::path& path) {
  std::ifstream input(path, std::ios::binary);
  if (!input) {
    throw std::runtime_error("could not open file for SHA256: " + path.string());
  }

  EvpContext context(EVP_MD_CTX_new(), EVP_MD_CTX_free);
  if (!context || EVP_DigestInit_ex(context.get(), EVP_sha256(), nullptr) != 1) {
    throw std::runtime_error("could not initialize SHA256");
  }

  std::array<char, 1024 * 1024> buffer{};
  while (input.good()) {
    input.read(buffer.data(), static_cast<std::streamsize>(buffer.size()));
    const auto read = input.gcount();
    if (read > 0 && EVP_DigestUpdate(context.get(), buffer.data(), static_cast<std::size_t>(read)) != 1) {
      throw std::runtime_error("could not update SHA256");
    }
  }

  std::array<unsigned char, EVP_MAX_MD_SIZE> digest{};
  unsigned int digest_length = 0;
  if (EVP_DigestFinal_ex(context.get(), digest.data(), &digest_length) != 1) {
    throw std::runtime_error("could not finalize SHA256");
  }

  std::ostringstream out;
  out << std::hex << std::setfill('0');
  for (unsigned int index = 0; index < digest_length; ++index) {
    out << std::setw(2) << static_cast<int>(digest[index]);
  }
  return out.str();
}

}  // namespace uocr::download
