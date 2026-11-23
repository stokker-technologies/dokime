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
#include <chrono>
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
using std::shared_ptr;
using std::string;
using std::chrono::duration;
using std::chrono::seconds;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

class ConnectionSettings : public dokime::operations::Settings {
public:
  using Settings::Settings;
  const string api_address = setting<string>("api_address", "Remote API Address", "127.0.0.1:30059");
  const duration<double> dll_timeout = setting<duration<double>>("dll_timeout", "DLL Host Timeout", seconds(5), seconds(0), seconds(30));
};
class Transport : public dokime::operations::Resource {
public:
  bool closed = false;
  void close() override {
    closed = true;
  }
};
class RemoteApi : public dokime::operations::Resource {
public:
  string address;
  unsigned closes = 0;
  shared_ptr<Transport> transport;
  void open(dokime::operations::Context &context) override {
    transport = context.resource<Transport>();
    address = context.settings<ConnectionSettings>().api_address;
  }
  void close() override {
    if (transport->closed)
      throw runtime_error("Close Remote API: Keep the Transport Open Until Its Dependents Close");
    ++closes;
  }
};
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
    value.identifier = "lifecycle.lifecycle";
    value.template_value(Template::from<FailedTestTemplate>());
    return value;
  }
};
class LifecyclePlugin : public Plugin {
public:
  LifecyclePlugin() : Plugin(Plugin::from<LifecyclePlugin>("1.0.0")) {
    suite(LifecycleSuite::definition());
    resource<RemoteApi>();
    resource<Transport>();
    settings<ConnectionSettings>();
    operation(dokime::operations::operation<string, string>("lifecycle.snapshot", Visibility::Public,
                                                            [](dokime::operations::Context &context, string) {
                                                              return context.settings<ConnectionSettings>().api_address;
                                                            }));
    operation(
        dokime::operations::operation<bool, bool>("lifecycle.cleaned", Visibility::Public, [](dokime::operations::Context &context, bool) {
          const auto &plugin = context.plugin<LifecyclePlugin>();
          return plugin.cleanups == 1 && plugin.initializations == 1 && plugin.finalizations == 0;
        }));
    operation(dokime::operations::operation<bool, bool>(
        "lifecycle.finalized", Visibility::Public, [](dokime::operations::Context &context, bool) {
          const auto &plugin = context.plugin<LifecyclePlugin>();
          return plugin.cleanups == 1 && plugin.initializations == 1 && plugin.finalizations == 1 && plugin.api->closes == 1 &&
                 plugin.api->transport->closed;
        }));
  }
  void initialize(dokime::operations::Context &context) override {
    if (&context.plugin<LifecyclePlugin>() != this)
      throw runtime_error("Initialize Fixture: Retain the Author Plugin");
    if (api && (api->closes != 1 || !api->transport->closed))
      throw runtime_error("Reactivate Fixture: Close the Previous Activation's Resources");
    const auto previous = api;
    api = context.resource<RemoteApi>();
    if (api == previous || api != context.resource<RemoteApi>())
      throw runtime_error("Initialize Fixture: Open One Resource Instance per Activation");
    ++initializations;
  }
  void finalize(dokime::operations::Context &context) override {
    if (&context.plugin<LifecyclePlugin>() != this)
      throw runtime_error("Finalize Fixture: Retain the Author Plugin");
    ++finalizations;
  }
  shared_ptr<RemoteApi> api;
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
