#include "uocr/core/runaway_guard.hpp"

#include <regex>
#include <vector>

namespace uocr {
namespace {

struct NumberMatch {
  int value;
  std::size_t start;
  std::size_t end;
};

std::vector<NumberMatch> find_numbers(const std::string& text, const std::regex& pattern) {
  std::vector<NumberMatch> matches;
  for (std::sregex_iterator it(text.begin(), text.end(), pattern), end; it != end; ++it) {
    const auto& match = *it;
    const auto prefix = match[1].str().size();
    matches.push_back({std::stoi(match[2].str()), static_cast<std::size_t>(match.position() + prefix),
                       static_cast<std::size_t>(match.position() + match.length())});
  }
  return matches;
}

bool separator_only(const std::string& gap) {
  static const std::regex separator_pattern(R"(^[\s,;:/|()\[\]{}<>-]*$)");
  return gap.size() <= 12 && std::regex_match(gap, separator_pattern);
}

bool has_numeric_run(const std::string& text, const std::regex& pattern, int min_run, bool repeat) {
  const auto matches = find_numbers(text, pattern);
  if (static_cast<int>(matches.size()) < min_run) {
    return false;
  }

  int run_length = 1;
  for (std::size_t index = 1; index < matches.size(); ++index) {
    const auto previous = matches[index - 1];
    const auto current = matches[index];
    const std::string gap = text.substr(previous.end, current.start - previous.end);
    const bool value_continues = repeat ? current.value == previous.value : current.value == previous.value + 1;
    if (value_continues && separator_only(gap)) {
      ++run_length;
    } else {
      run_length = 1;
    }
    if (run_length >= min_run) {
      return true;
    }
  }
  return false;
}

}  // namespace

std::optional<std::string> detect_recoverable_output_issue(std::string_view text) {
  const std::string tail(text.substr(text.size() > 12000 ? text.size() - 12000 : 0));
  static const std::regex number_pattern(R"((^|[^A-Za-z0-9_.-])(\d{2,6})(?![A-Za-z0-9_.-]))");
  static const std::regex repeated_number_pattern(R"((^|[^A-Za-z0-9_.-])(\d{1,6})\.?(?![A-Za-z0-9_.-]))");

  if (has_numeric_run(tail, number_pattern, 48, false)) {
    return "Stopped runaway numeric counting output from the native OCR model.";
  }
  if (has_numeric_run(tail, repeated_number_pattern, 64, true)) {
    return "Stopped runaway repeated-number output from the native OCR model.";
  }
  return std::nullopt;
}

}  // namespace uocr

