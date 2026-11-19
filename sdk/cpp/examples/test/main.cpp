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

using dokime::Plugin;
using dokime::Suite;
using dokime::templates::Context;
using dokime::templates::Criterion;
using dokime::templates::Record;
using dokime::templates::Step;
using dokime::templates::Template;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace example {
class ConfirmationTestTemplate : public dokime::templates::TestTemplate {
public:
  inline static const Record<bool> confirmation{"confirmation", "Confirmation"};

  void confirm(Context &context) {
    const auto answer = context.prompt("Continue?", {"Yes", "No"});
    context.record(confirmation, answer == "Yes");
  }
  bool confirmed(Context &context) {
    return context.records<&ConfirmationTestTemplate::confirm>(confirmation).last();
  }
  static Template definition() {
    auto value = Template::from<ConfirmationTestTemplate>();
    value.record(confirmation);
    value.steps.push_back(Step::method<&ConfirmationTestTemplate::confirm>("Confirm"));
    value.criteria.push_back(Criterion::method<&ConfirmationTestTemplate::confirmed>("Confirmed"));
    return value;
  }
};
class CaptureSuite : public Suite {
public:
  static Suite definition() {
    auto value = Suite::from<CaptureSuite>();
    value.template_value(ConfirmationTestTemplate::definition());
    return value;
  }
};
class ExamplePlugin : public Plugin {
public:
  ExamplePlugin() : Plugin(Plugin::from<ExamplePlugin>("0.1.0")) {
    suite(CaptureSuite::definition());
  }
};
} // namespace example

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

int main() {
  example::ExamplePlugin plugin;
  plugin.serve();
}
