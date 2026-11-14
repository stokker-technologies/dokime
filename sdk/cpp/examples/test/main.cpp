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

using dokime::plugin::Context;
using dokime::plugin::Plugin;
using dokime::plugin::Template;

using dokime::plugin::metadata;
using std::exception;
using std::invalid_argument;
using std::make_shared;
using std::move;
using std::string;

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
    Template value;
    *value.definition.mutable_metadata() = metadata("sample.check", "Check");
    value.initialize = [](Context &ctx) { ctx.set(make_shared<double>(0)); };
    value.steps.push_back({metadata("measure", "Measure"), [](Context &ctx) { *ctx.get<double>() = 42; }});
    value.criteria.push_back({metadata("valid", "Valid"), [](Context &ctx) { return *ctx.get<double>() == 42; }});
    Plugin plugin("sample", "0.1.0");
    plugin.suite({metadata("sample", "Sample"), {move(value)}});

    // Empty Address Uses the Endpoint Passed by the Host in HARMOS_ADDRESS.
    if (argc == 3 && string(argv[1]) == "--address")
      plugin.serve(argv[2]);
    else if (argc == 1)
      plugin.serve();
    else
      throw invalid_argument("Usage: dokime-plugin [--address ADDRESS]");
  } catch (const exception &error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
