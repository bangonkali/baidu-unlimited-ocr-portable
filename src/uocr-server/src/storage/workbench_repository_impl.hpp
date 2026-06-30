#pragma once

#include "uocr/storage/workbench_repository.hpp"

#include <duckdb.h>

#include <mutex>
#include <stdexcept>
#include <string>
#include <string_view>

namespace uocr::storage {

struct QueryResult {
  duckdb_result value{};

  QueryResult() = default;
  QueryResult(const QueryResult&) = delete;
  QueryResult& operator=(const QueryResult&) = delete;
  QueryResult(QueryResult&& other) noexcept : value(other.value) { other.value = duckdb_result{}; }
  QueryResult& operator=(QueryResult&& other) noexcept {
    if (this != &other) {
      duckdb_destroy_result(&value);
      value = other.value;
      other.value = duckdb_result{};
    }
    return *this;
  }
  ~QueryResult() { duckdb_destroy_result(&value); }

  idx_t rows() const { return duckdb_row_count(const_cast<duckdb_result*>(&value)); }
  bool is_null(idx_t column, idx_t row) const {
    return duckdb_value_is_null(const_cast<duckdb_result*>(&value), column, row);
  }
  std::string text(idx_t column, idx_t row) const;
  int int32(idx_t column, idx_t row) const {
    return duckdb_value_int32(const_cast<duckdb_result*>(&value), column, row);
  }
  std::uint64_t uint64(idx_t column, idx_t row) const {
    return duckdb_value_uint64(const_cast<duckdb_result*>(&value), column, row);
  }
  double number(idx_t column, idx_t row) const {
    return duckdb_value_double(const_cast<duckdb_result*>(&value), column, row);
  }
};

struct Statement {
  duckdb_prepared_statement value{};

  Statement(duckdb_connection connection, std::string_view sql);
  Statement(const Statement&) = delete;
  Statement& operator=(const Statement&) = delete;
  Statement(Statement&& other) noexcept : value(other.value) { other.value = nullptr; }
  Statement& operator=(Statement&& other) noexcept {
    if (this != &other) {
      duckdb_destroy_prepare(&value);
      value = other.value;
      other.value = nullptr;
    }
    return *this;
  }
  ~Statement() { duckdb_destroy_prepare(&value); }

  void bind_text(idx_t index, std::string_view text);
  void bind_int32(idx_t index, int value);
  void bind_uint64(idx_t index, std::uint64_t value);
  void bind_double(idx_t index, double value);
  void bind_null(idx_t index);
  void execute();
  QueryResult query();
};

struct WorkbenchRepository::Impl {
  explicit Impl(std::filesystem::path path);
  ~Impl();

  duckdb_database database = nullptr;
  duckdb_connection connection = nullptr;
  std::filesystem::path path;
  mutable std::mutex mutex;

  void execute(std::string_view sql) const;
  QueryResult query(std::string_view sql) const;
  Statement statement(std::string_view sql) const;
  void apply_migrations() const;
};

std::string escape_json_string(std::string_view text);
std::string quote_sql_string(std::string_view text);
std::string make_event_id(std::string_view seed);
std::string terminal_timestamp_sql(std::string_view status);

}  // namespace uocr::storage
