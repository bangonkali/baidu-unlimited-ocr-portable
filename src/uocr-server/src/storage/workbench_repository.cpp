#include "workbench_repository_impl.hpp"

#include "uocr/storage/migrations.hpp"

#include <chrono>
#include <filesystem>
#include <iomanip>
#include <sstream>

namespace uocr::storage {
namespace {

std::string duckdb_error(duckdb_result* result) {
  const auto* message = duckdb_result_error(result);
  return message != nullptr ? std::string(message) : "DuckDB query failed";
}

void throw_if_error(duckdb_state state, duckdb_result* result) {
  if (state == DuckDBError) {
    throw std::runtime_error(duckdb_error(result));
  }
}

}  // namespace

std::string QueryResult::text(idx_t column, idx_t row) const {
  if (is_null(column, row)) {
    return "";
  }
  auto* raw = duckdb_value_varchar(const_cast<duckdb_result*>(&value), column, row);
  std::string text = raw != nullptr ? raw : "";
  duckdb_free(raw);
  return text;
}

Statement::Statement(duckdb_connection connection, std::string_view sql) {
  const std::string query(sql);
  if (duckdb_prepare(connection, query.c_str(), &value) == DuckDBError) {
    const auto* message = duckdb_prepare_error(value);
    const std::string error = message != nullptr ? message : "DuckDB prepare failed";
    duckdb_destroy_prepare(&value);
    throw std::runtime_error(error);
  }
}

void Statement::bind_text(idx_t index, std::string_view text) {
  const std::string value_text(text);
  if (duckdb_bind_varchar(value, index, value_text.c_str()) == DuckDBError) {
    throw std::runtime_error("DuckDB text bind failed");
  }
}

void Statement::bind_int32(idx_t index, int bound_value) {
  if (duckdb_bind_int32(value, index, bound_value) == DuckDBError) {
    throw std::runtime_error("DuckDB int bind failed");
  }
}

void Statement::bind_uint64(idx_t index, std::uint64_t bound_value) {
  if (duckdb_bind_uint64(value, index, bound_value) == DuckDBError) {
    throw std::runtime_error("DuckDB uint bind failed");
  }
}

void Statement::bind_double(idx_t index, double bound_value) {
  if (duckdb_bind_double(value, index, bound_value) == DuckDBError) {
    throw std::runtime_error("DuckDB double bind failed");
  }
}

void Statement::bind_null(idx_t index) {
  if (duckdb_bind_null(value, index) == DuckDBError) {
    throw std::runtime_error("DuckDB null bind failed");
  }
}

void Statement::execute() {
  duckdb_result result{};
  throw_if_error(duckdb_execute_prepared(value, &result), &result);
  duckdb_destroy_result(&result);
}

QueryResult Statement::query() {
  QueryResult result;
  throw_if_error(duckdb_execute_prepared(value, &result.value), &result.value);
  return result;
}

WorkbenchRepository::Impl::Impl(std::filesystem::path database_path) : path(std::move(database_path)) {
  std::filesystem::create_directories(path.parent_path());
  const auto path_text = path.string();
  if (duckdb_open(path_text.c_str(), &database) == DuckDBError) {
    throw std::runtime_error("Failed to open DuckDB database: " + path_text);
  }
  if (duckdb_connect(database, &connection) == DuckDBError) {
    duckdb_close(&database);
    throw std::runtime_error("Failed to connect DuckDB database: " + path_text);
  }
  apply_migrations();
}

WorkbenchRepository::Impl::~Impl() {
  if (connection != nullptr) {
    duckdb_disconnect(&connection);
  }
  if (database != nullptr) {
    duckdb_close(&database);
  }
}

void WorkbenchRepository::Impl::execute(std::string_view sql) const {
  const std::string query(sql);
  duckdb_result result{};
  throw_if_error(duckdb_query(connection, query.c_str(), &result), &result);
  duckdb_destroy_result(&result);
}

QueryResult WorkbenchRepository::Impl::query(std::string_view sql) const {
  const std::string query(sql);
  QueryResult result;
  throw_if_error(duckdb_query(connection, query.c_str(), &result.value), &result.value);
  return result;
}

Statement WorkbenchRepository::Impl::statement(std::string_view sql) const {
  return Statement(connection, sql);
}

void WorkbenchRepository::Impl::apply_migrations() const {
  std::scoped_lock lock(mutex);
  execute("CREATE TABLE IF NOT EXISTS schema_migrations (id INTEGER PRIMARY KEY, name TEXT NOT NULL, "
          "applied_at TIMESTAMP NOT NULL DEFAULT current_timestamp)");
  for (const auto& migration : duckdb_migrations()) {
    auto check = statement("SELECT id FROM schema_migrations WHERE id = ?");
    check.bind_int32(1, migration.id);
    if (check.query().rows() > 0) {
      continue;
    }
    execute("BEGIN TRANSACTION");
    try {
      execute(migration.sql);
      auto insert = statement("INSERT INTO schema_migrations(id, name) VALUES (?, ?)");
      insert.bind_int32(1, migration.id);
      insert.bind_text(2, migration.name);
      insert.execute();
      execute("COMMIT");
    } catch (...) {
      execute("ROLLBACK");
      throw;
    }
  }
}

std::string escape_json_string(std::string_view text) {
  std::ostringstream output;
  output << '"';
  for (const unsigned char ch : text) {
    switch (ch) {
      case '\\':
        output << "\\\\";
        break;
      case '"':
        output << "\\\"";
        break;
      case '\n':
        output << "\\n";
        break;
      case '\r':
        output << "\\r";
        break;
      case '\t':
        output << "\\t";
        break;
      default:
        if (ch < 0x20) {
          output << "\\u" << std::hex << std::setw(4) << std::setfill('0') << static_cast<int>(ch);
        } else {
          output << static_cast<char>(ch);
        }
    }
  }
  output << '"';
  return output.str();
}

std::string quote_sql_string(std::string_view text) {
  std::string output;
  output.reserve(text.size() + 2);
  output.push_back('\'');
  for (const char ch : text) {
    if (ch == '\'') {
      output.push_back('\'');
    }
    output.push_back(ch);
  }
  output.push_back('\'');
  return output;
}

std::string make_event_id(std::string_view seed) {
  const auto now = std::chrono::system_clock::now().time_since_epoch();
  std::uint64_t hash = 14695981039346656037ULL;
  for (const unsigned char ch : seed) {
    hash ^= ch;
    hash *= 1099511628211ULL;
  }
  hash ^= static_cast<std::uint64_t>(std::chrono::duration_cast<std::chrono::microseconds>(now).count());
  std::ostringstream out;
  out << "evt_" << std::hex << std::setw(16) << std::setfill('0') << hash;
  return out.str();
}

std::string terminal_timestamp_sql(std::string_view status) {
  return status == "completed" || status == "completed_with_errors" || status == "failed" ||
                 status == "cancelled"
             ? "current_timestamp"
             : "finished_at";
}

WorkbenchRepository::WorkbenchRepository(std::filesystem::path database_path)
    : impl_(std::make_unique<Impl>(std::move(database_path))) {}

WorkbenchRepository::~WorkbenchRepository() = default;
WorkbenchRepository::WorkbenchRepository(WorkbenchRepository&&) noexcept = default;
WorkbenchRepository& WorkbenchRepository::operator=(WorkbenchRepository&&) noexcept = default;

const std::filesystem::path& WorkbenchRepository::database_path() const {
  return impl_->path;
}

}  // namespace uocr::storage
