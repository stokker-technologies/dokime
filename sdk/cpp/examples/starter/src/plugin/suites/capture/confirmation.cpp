/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "confirmation.hpp"

using dokime::templates::Context;
using dokime::templates::Criterion;
using dokime::templates::Step;
using dokime::templates::Template;

/* === Macros === */
/* === Constants === */
/* === Types === */
/* === Classes === */
/* === Templates === */
/* === Public Functions === */

namespace starter_project {
void ConfirmationTestTemplate::confirm(Context &context) {
  const auto answer = context.prompt("Continue?", {"Yes", "No"});
  context.record(confirmation, answer == "Yes");
}
bool ConfirmationTestTemplate::confirmed(Context &context) {
  return context.records<&ConfirmationTestTemplate::confirm>(confirmation).last();
}
Template ConfirmationTestTemplate::definition() {
  auto value = Template::from<ConfirmationTestTemplate>();
  value.step(Step::method<&ConfirmationTestTemplate::confirm>("Confirm"));
  value.criterion(Criterion::method<&ConfirmationTestTemplate::confirmed>("Confirmed"));
  return value;
}
} // namespace starter_project

/* === Private Functions === */
/* === Entry Point === */
