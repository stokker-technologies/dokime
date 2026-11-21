/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "dokime/configuration.hpp"
#include <cstdlib>
#include <fstream>
#include <iterator>
#include <memory>
#include <regex>
#include <stdexcept>

using std::invalid_argument;
using std::map;
using std::optional;
using std::regex;
using std::regex_match;
using std::size_t;
using std::smatch;
using std::string;
using std::to_string;
using std::filesystem::path;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime::detail::configuration {
namespace {
string trim(string value) {
  const auto start = value.find_first_not_of(" \t");
  if (start == string::npos)
    return {};
  return value.substr(start, value.find_last_not_of(" \t") - start + 1);
}
void validate_utf8(const string &content, const path &file) {
  size_t number = 1;
  for (size_t index = 0; index < content.size();) {
    const auto first = static_cast<unsigned char>(content[index++]);
    if (first == '\n')
      ++number;
    if (first < 128)
      continue;
    unsigned count = first >= 0xc2 && first <= 0xdf ? 1 : first >= 0xe0 && first <= 0xef ? 2 : first >= 0xf0 && first <= 0xf4 ? 3 : 0;
    bool valid = count != 0 && index + count <= content.size();
    if (valid) {
      const auto second = static_cast<unsigned char>(content[index]);
      valid = !(first == 0xe0 && second < 0xa0) && !(first == 0xed && second >= 0xa0) && !(first == 0xf0 && second < 0x90) &&
              !(first == 0xf4 && second >= 0x90);
      for (unsigned offset = 0; offset < count; ++offset)
        valid = valid && (static_cast<unsigned char>(content[index + offset]) & 0xc0) == 0x80;
    }
    if (!valid)
      throw invalid_argument("Read " + file.string() + ":" + to_string(number) + ": Save the File as UTF-8");
    index += count;
  }
}
} // namespace
map<string, string> parse(const string &content, const path &file) {
  validate_utf8(content, file);
  map<string, string> values;
  size_t start = content.starts_with("\xef\xbb\xbf") ? 3 : 0;
  size_t number = 0;
  while (start <= content.size()) {
    ++number;
    const auto end = content.find('\n', start);
    auto line = content.substr(start, end == string::npos ? end : end - start);
    start = end == string::npos ? content.size() + 1 : end + 1;
    if (end != string::npos && line.ends_with('\r'))
      line.pop_back();
    line = trim(line);
    const auto malformed = [&]() {
      return invalid_argument("Read " + file.string() + ":" + to_string(number) +
                              ": Use One Unique KEY=VALUE Assignment with Balanced Quotes and Supported Escapes");
    };
    if (line.find('\0') != string::npos || line.find('\r') != string::npos)
      throw malformed();
    if (line.empty() || line.starts_with('#'))
      continue;
    smatch match;
    if (!regex_match(line, match, regex("([A-Za-z_][A-Za-z0-9_]*)[ \\t]*=[ \\t]*(.*)")) || values.contains(match[1].str()))
      throw malformed();
    const auto key = match[1].str();
    const auto raw = match[2].str();
    string value;
    if (raw.empty() || (raw[0] != '\'' && raw[0] != '"')) {
      for (size_t index = 0; index < raw.size(); ++index) {
        if (raw[index] == '#' && (index == 0 || raw[index - 1] == ' ' || raw[index - 1] == '\t'))
          break;
        value += raw[index];
      }
      values.emplace(key, trim(value));
      continue;
    }
    const auto quote = raw[0];
    size_t index = 1;
    while (index < raw.size() && raw[index] != quote) {
      auto character = raw[index++];
      if (character == '\\' && quote == '"') {
        if (index == raw.size())
          throw malformed();
        character = raw[index++];
        switch (character) {
        case '\\':
        case '"':
          break;
        case 'n':
          character = '\n';
          break;
        case 'r':
          character = '\r';
          break;
        case 't':
          character = '\t';
          break;
        default:
          throw malformed();
        }
      }
      value += character;
    }
    if (index == raw.size())
      throw malformed();
    const auto trailing = trim(raw.substr(index + 1));
    if (!trailing.empty() && !trailing.starts_with('#'))
      throw malformed();
    values.emplace(key, value);
  }
  return values;
}
map<string, string> load() {
  const auto file = std::filesystem::current_path() / ".env";
  if (!std::filesystem::exists(file))
    return {};
  std::ifstream input(file, std::ios::binary);
  if (!input)
    throw invalid_argument("Read " + file.string() + ":1: Make the Configuration File Readable");
  const string content(std::istreambuf_iterator<char>{input}, {});
  if (input.bad())
    throw invalid_argument("Read " + file.string() + ":1: Make the Configuration File Readable");
  return parse(content, file);
}
string resolve(const string &key, const optional<string> &explicit_value, const map<string, string> &values) {
  string value;
  const auto configured = environment(key);
  if (explicit_value)
    value = *explicit_value;
  else if (configured)
    value = *configured;
  else if (const auto found = values.find(key); found != values.end())
    value = found->second;
  if (value.empty())
    throw invalid_argument("Configure Connection: Set " + key + " to a Nonempty Address");
  return value;
}
optional<string> environment(const string &key) {
#ifdef _WIN32
  char *buffer = nullptr;
  size_t size = 0;
  if (_dupenv_s(&buffer, &size, key.c_str()) != 0)
    throw invalid_argument("Read Environment: Make " + key + " Readable");
  const std::unique_ptr<char, decltype(&std::free)> value(buffer, &std::free);
  if (value)
    return string(value.get());
  return {};
#else
  const auto value = std::getenv(key.c_str());
  return value ? optional<string>(value) : std::nullopt;
#endif
}
} // namespace dokime::detail::configuration

/* === Private Functions === */

/* === Entry Point === */
