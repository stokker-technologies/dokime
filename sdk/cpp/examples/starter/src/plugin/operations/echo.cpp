/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "echo.hpp"

using dokime::operations::Context;
using std::string;

/* === Macros === */
/* === Constants === */
/* === Types === */
/* === Classes === */
/* === Templates === */
/* === Public Functions === */

namespace starter_project {
string echo(Context &, string request) {
  return request;
}
} // namespace starter_project

/* === Private Functions === */
/* === Entry Point === */
