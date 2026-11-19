/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
#include <stdexcept>

// Third Party Imports

// Local Imports
#include "dokime/plugin.hpp"

using dokime::Plugin;
using dokime::Suite;
using dokime::operations::Visibility;
using dokime::templates::Template;
using dokime::templates::TestTemplate;
using std::runtime_error;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

class FailedTestTemplate : public TestTemplate {
public:
  void initialize(dokime::templates::Context &) {
    opened_ = true;
    throw runtime_error("Initialize Fixture: Exercise Cleanup After an Initialization Failure");
  }
  void cleanup(dokime::templates::Context &context);

private:
  bool opened_ = false;
};
class LifecycleSuite : public Suite {
public:
  static Suite definition() {
    auto value = Suite::from<LifecycleSuite>();
    value.template_value(Template::from<FailedTestTemplate>());
    return value;
  }
};
class LifecyclePlugin : public Plugin {
public:
  LifecyclePlugin() : Plugin(Plugin::from<LifecyclePlugin>("1.0.0")) {
    suite(LifecycleSuite::definition());
    operation(
        dokime::operations::operation<bool, bool>("lifecycle.cleaned", Visibility::Public, [](dokime::operations::Context &context, bool) {
          const auto &plugin = context.plugin<LifecyclePlugin>();
          return plugin.cleanups == 1 && plugin.initializations == 1 && plugin.finalizations == 0;
        }));
    operation(dokime::operations::operation<bool, bool>(
        "lifecycle.finalized", Visibility::Public, [](dokime::operations::Context &context, bool) {
          const auto &plugin = context.plugin<LifecyclePlugin>();
          return plugin.cleanups == 1 && plugin.initializations == 1 && plugin.finalizations == 1;
        }));
  }
  void initialize(dokime::operations::Context &context) override {
    if (&context.plugin<LifecyclePlugin>() != this)
      throw runtime_error("Initialize Fixture: Retain the Author Plugin");
    ++initializations;
  }
  void finalize(dokime::operations::Context &context) override {
    if (&context.plugin<LifecyclePlugin>() != this)
      throw runtime_error("Finalize Fixture: Retain the Author Plugin");
    ++finalizations;
  }
  unsigned cleanups = 0;
  unsigned initializations = 0;
  unsigned finalizations = 0;
};

/* === Templates === */

/* === Public Functions === */

void FailedTestTemplate::cleanup(dokime::templates::Context &context) {
  if (!opened_ || context.termination() != dokime::TERMINATION_FAILED)
    throw runtime_error("Clean Up Fixture: Preserve the Failed Initialization State and Termination");
  ++context.plugin<LifecyclePlugin>().cleanups;
}

/* === Private Functions === */

/* === Entry Point === */

int main() {
  LifecyclePlugin plugin;
  plugin.serve();
}
