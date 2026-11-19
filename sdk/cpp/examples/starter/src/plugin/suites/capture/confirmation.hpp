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

#include <dokime/templates.hpp>

/* === Macros === */
/* === Constants === */
/* === Types === */
/* === Classes === */

namespace starter_project {
class ConfirmationTestTemplate : public dokime::templates::TestTemplate {
public:
  inline static const dokime::templates::Record<bool> confirmation{"confirmation", "Confirmation"};
  void confirm(dokime::templates::Context &context);
  bool confirmed(dokime::templates::Context &context);
  static dokime::templates::Template definition();
};
} // namespace starter_project

/* === Templates === */
/* === Public Declarations === */
