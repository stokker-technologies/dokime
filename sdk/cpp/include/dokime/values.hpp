/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#pragma once

/* === Imports === */

// Standard Library Imports
#include <chrono>
#include <cstdint>
#include <map>
#include <optional>
#include <string>
#include <variant>
#include <vector>

// Third Party Imports

// Local Imports
#include "dokime/messages.hpp"

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace dokime {
enum class ResultStatus { Unspecified, Pending, Passed, Failed, Blocked, Cancelled };
struct Outcome {
  ResultStatus status = ResultStatus::Pending;
  std::string attribution;
  std::string reason;
  std::optional<Termination> termination;
  std::optional<std::string> importer;
};
struct Progress {
  std::string phase;
  std::uint64_t completed = 0;
  std::uint64_t total = 0;
  std::string label;
  bool sealed = false;
};
struct Note {
  std::string identifier;
  std::string text;
  std::optional<std::string> author;
  std::chrono::sys_time<std::chrono::nanoseconds> created_at;
  std::string cycle;
  std::optional<std::string> execution;
};
struct Artifact {
  std::string identifier;
  std::string name;
  std::string mime;
  std::uint64_t size_bytes = 0;
  std::vector<std::uint8_t> sha256;
  std::optional<std::string> cycle;
  std::optional<std::string> execution;
};
struct Scalar {
  struct Exact {
    std::string numerator;
    std::string denominator;
    bool negative = false;
  };
  std::variant<std::monostate, bool, std::int64_t, std::uint64_t, double, std::string, std::vector<std::uint8_t>, Exact,
               std::chrono::sys_time<std::chrono::nanoseconds>, std::vector<Scalar>>
      value;
};
struct PromptQuestion {
  std::string identifier;
  std::string text;
  std::vector<std::string> choices;
  std::string execution;
};
struct PromptAnswer {
  std::string choice;
};
template <class T> struct RecordRow {
  std::string identifier;
  std::uint64_t sequence = 0;
  std::chrono::sys_time<std::chrono::nanoseconds> timestamp;
  std::chrono::sys_time<std::chrono::nanoseconds> received_at;
  std::string record_type;
  std::uint32_t schema_version = 0;
  std::string source;
  T value;
};
template <class T> struct SampleRow {
  std::string identifier;
  std::uint64_t sequence = 0;
  std::chrono::sys_time<std::chrono::nanoseconds> timestamp;
  std::chrono::sys_time<std::chrono::nanoseconds> received_at;
  std::string stream;
  std::uint32_t schema_version = 0;
  std::string source;
  T value;
  std::map<std::string, Scalar> values;
};
} // namespace dokime

/* === Templates === */

/* === Public Declarations === */
