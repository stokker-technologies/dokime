/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "capture.hpp"
#include "confirmation.hpp"

using dokime::Suite;

/* === Macros === */
/* === Constants === */
/* === Types === */
/* === Classes === */
/* === Templates === */
/* === Public Functions === */

namespace starter_project {
Suite CaptureSuite::definition() {
  auto suite = Suite::from<CaptureSuite>();
  suite.template_value(ConfirmationTestTemplate::definition());
  return suite;
}
} // namespace starter_project

/* === Private Functions === */
/* === Entry Point === */
