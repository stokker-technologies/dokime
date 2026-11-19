/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "plugin.hpp"
#include "plugin/operations/echo.hpp"
#include "plugin/suites/capture/capture.hpp"

/* === Macros === */
/* === Constants === */
/* === Types === */
/* === Classes === */
/* === Templates === */
/* === Public Functions === */

namespace starter_project {
ExamplePlugin::ExamplePlugin() : Plugin(Plugin::from<ExamplePlugin>("0.1.0")) {
  suite(CaptureSuite::definition());
  operation(dokime::operations::operation<std::string, std::string>("example.echo", dokime::operations::Visibility::Public, echo));
}
} // namespace starter_project

/* === Private Functions === */
/* === Entry Point === */
