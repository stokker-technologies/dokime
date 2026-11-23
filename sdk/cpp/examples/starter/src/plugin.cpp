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
#include "plugin/suites/capture/confirmation.hpp"

using dokime::Plugin;
using dokime::operations::Visibility;
using std::string;

/* === Macros === */
/* === Constants === */
/* === Types === */
/* === Classes === */
/* === Templates === */
/* === Public Functions === */

namespace starter_project {
ExamplePlugin::ExamplePlugin() : Plugin(Plugin::from<ExamplePlugin>("0.1.0")) {
  record(ConfirmationTestTemplate::confirmation);
  suite(CaptureSuite::definition());
  settings<ConnectionSettings>();
  operation(dokime::operations::operation<string, string>("example.echo", Visibility::Public, echo));
}
} // namespace starter_project

/* === Private Functions === */
/* === Entry Point === */
