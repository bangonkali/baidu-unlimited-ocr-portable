#ifdef _WIN32
#include <windows.h>
#endif

#include <algorithm>

int main() {
#ifdef _WIN32
#ifdef min
#error "windows.h min macro leaked into the uocr C++ build; NOMINMAX must be defined"
#endif
#ifdef max
#error "windows.h max macro leaked into the uocr C++ build; NOMINMAX must be defined"
#endif
#endif

  const auto smaller = std::min(3, 7);
  const auto larger = std::max(3, 7);
  return smaller == 3 && larger == 7 ? 0 : 1;
}
