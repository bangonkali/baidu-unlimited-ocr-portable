#include <drogon/drogon.h>
#include <drogon/version.h>
#include <openssl/opensslv.h>
#include <trantor/utils/Utilities.h>

#include <cassert>
#include <string>

int main() {
  assert(std::string(DROGON_VERSION) == "1.9.13");
  assert(std::string(OPENSSL_VERSION_TEXT).find("OpenSSL 3.6.3") != std::string::npos);
  assert(trantor::utils::tlsBackend() == "OpenSSL");
  assert(drogon::app().supportSSL());
  return 0;
}
