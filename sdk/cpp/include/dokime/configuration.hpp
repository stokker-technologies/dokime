/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#pragma once

/* === Imports === */

#include <chrono>
#include <filesystem>
#include <map>
#include <optional>
#include <string>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace dokime {
struct ConnectionSettings {
  std::optional<std::string> remote;
  std::chrono::duration<double> connect_timeout{30};
  std::optional<std::string> certificate_pem;
  std::optional<std::string> access_key;
  std::optional<std::string> expected_backend_id;
};
namespace detail::configuration {
std::map<std::string, std::string> parse(const std::string &content, const std::filesystem::path &path);
std::map<std::string, std::string> load();
std::optional<std::string> environment(const std::string &key);
std::string resolve(const std::string &key, const std::optional<std::string> &explicit_value,
                    const std::map<std::string, std::string> &values);
} // namespace detail::configuration
} // namespace dokime

/* === Templates === */

/* === Public Declarations === */
