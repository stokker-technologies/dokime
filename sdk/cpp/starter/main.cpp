/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include <dokime/dokime.hpp>
#include <iostream>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

int main(int argc, char **argv) {
  try {
    // Compose the Suite from Ordinary Values.
    dokime::Template value;
    *value.definition.mutable_metadata() = dokime::metadata("sample.check", "Check");
    value.steps.push_back({dokime::metadata("measure", "Measure"), [](dokime::Context &ctx) { ctx.set(std::make_shared<double>(42)); }});
    value.criteria.push_back({dokime::metadata("valid", "Valid"), [](dokime::Context &ctx) { return *ctx.get<double>() == 42; }});
    dokime::Plugin plugin("sample", "0.1.0");
    plugin.suite({dokime::metadata("sample", "Sample"), {std::move(value)}});

    // Empty Address Uses the Endpoint Passed by the Host in HARMOS_ADDRESS.
    if (argc == 3 && std::string(argv[1]) == "--address")
      plugin.serve(argv[2]);
    else if (argc == 1)
      plugin.serve();
    else
      throw std::invalid_argument("Usage: dokime-plugin [--address ADDRESS]");
  } catch (const std::exception &error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
