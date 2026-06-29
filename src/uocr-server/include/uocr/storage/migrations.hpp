#pragma once

#include <string_view>
#include <vector>

namespace uocr {

struct Migration {
  int id;
  std::string_view name;
  std::string_view sql;
};

const std::vector<Migration>& duckdb_migrations();

}  // namespace uocr

