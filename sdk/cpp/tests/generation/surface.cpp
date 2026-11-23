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
void bound_surface(dokime::Client &client, dokime::Execution &execution) {
  const auto operation = dokime::operations::operation<std::string, std::string>("example.echo", dokime::operations::Visibility::Public);
  auto echo = client.operation(operation);
  const auto response = echo("request");
  const auto records = execution.records<bool>("example.capture.confirmation.confirmation").read();
  const auto samples = execution.samples<bool>("example.samples").collect();
  (void)response;
  (void)records;
  (void)samples;
}

/* === Private Functions === */

/* === Entry Point === */
