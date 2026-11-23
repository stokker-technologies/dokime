/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#pragma once

/* === Imports === */

#include <dokime/plugin.hpp>

/* === Macros === */
/* === Constants === */
/* === Types === */
/* === Classes === */

namespace starter_project {
class ConnectionSettings : public dokime::operations::Settings {
public:
  using Settings::Settings;
  const std::string api_address = setting<std::string>("api_address", "Remote API Address", "127.0.0.1:30059");
  const std::chrono::duration<double> dll_timeout = setting<std::chrono::duration<double>>(
      "dll_timeout", "DLL Host Timeout", std::chrono::seconds(5), std::chrono::seconds(0), std::chrono::seconds(30));
};
class ExamplePlugin : public dokime::Plugin {
public:
  ExamplePlugin();
};
} // namespace starter_project

/* === Templates === */
/* === Public Declarations === */
