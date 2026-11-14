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
#include "dokime/context.hpp"


/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

// Compiler Story: Calls Are Type-Checked Without Starting a Host.
void generated_surface(dokime::Client &client, dokime::plugin::Context &context) {
  auto analyses = client.analyses.list();
  auto capture = context.host().files.capture(dokime::FileSource{}, "output");
  (void)capture;
  auto artifact = context.host().content.artifacts.create(harmos::FileReference{}, "capture.bin", "application/octet-stream");
  (void)analyses;
  (void)artifact;
}


/* === Private Functions === */

/* === Entry Point === */
