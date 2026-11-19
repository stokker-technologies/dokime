/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
#include "dokime/client.hpp"
#include "dokime/templates.hpp"

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

// Compiler Story: Calls Are Type-Checked Without Starting a Host.
void generated_surface(dokime::detail::Transport &client) {
  const auto plugins = client.plugins.list();
  (void)plugins;
}

/* === Private Functions === */

/* === Entry Point === */
