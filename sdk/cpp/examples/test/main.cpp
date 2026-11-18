/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include <dokime/plugin.hpp>
#include <grpc/event_engine/event_engine.h>
#include <iostream>

using dokime::plugin::Context;
using dokime::plugin::Criterion;
using dokime::plugin::Plugin;
using dokime::plugin::Record;
using dokime::plugin::Step;
using dokime::plugin::Suite;
using dokime::plugin::Template;
using grpc_event_engine::experimental::ShutdownDefaultEventEngine;
using std::cerr;

using std::exception;
using std::invalid_argument;
using std::string;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace example {
class Capture {
public:
  inline static const Record<bool> confirmation{"confirmation", "Confirmation"};

  void confirm(Context &context) {
    const auto answer = context.prompt("Continue?", {"Yes", "No"});
    context.record(confirmation, answer == "Yes");
  }
  bool confirmed(Context &context) {
    return context.records<&Capture::confirm>(confirmation).last();
  }
  static void declare(Template &value) {
    value.record(confirmation);
    value.steps.push_back(Step::method<&Capture::confirm>("Confirm"));
    value.criteria.push_back(Criterion::method<&Capture::confirmed>("Confirmed"));
  }
};
} // namespace example

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

int main(int argc, char **argv) {
  int result = 0;
  try {
    // Start the Sidecar.
    Plugin plugin("example", "0.1.0", {Suite::from<example::Capture>("Capture")});
    if (argc == 3 && string(argv[1]) == "--address")
      plugin.serve(argv[2]);
    else if (argc == 1)
      plugin.serve();
    else
      throw invalid_argument("Serve Plugin: Expected [--address ADDRESS]");
  } catch (const exception &error) {
    cerr << error.what() << '\n';
    result = 1;
  }
  // Drain EventEngine Work Before Windows Begins DLL Teardown.
  ShutdownDefaultEventEngine();
  return result;
}
