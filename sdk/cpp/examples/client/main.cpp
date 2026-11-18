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
#include <iostream>

// Third Party Imports
#include <grpc/event_engine/event_engine.h>

// Local Imports
#include <dokime/client.hpp>

using dokime::Client;
using dokime::Metadata;
using dokime::PluginPlacement;
using dokime::PluginState;
using dokime::PluginValue;
using dokime::client::ExecutionBinding;
using dokime::client::PluginsWatchOptions;
using dokime::client::PromptsListOptions;
using dokime::client::SelectionPlan;
using dokime::client::upload;
using grpc_event_engine::experimental::ShutdownDefaultEventEngine;
using std::cerr;
using std::cin;
using std::cout;
using std::exception;
using std::flush;
using std::getline;
using std::invalid_argument;
using std::runtime_error;
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
  int result = 0;
  try {
    // Connect and Upload the Compiled Plugin Bundle.
    if (argc != 3)
      throw invalid_argument("Run Capture: Supply HOST:PORT and a Compiled .dokime Bundle");
    auto client = Client::connect(argv[1]);
    const auto content = upload(client, argv[2]);
    PluginPlacement placement;
    *placement.mutable_managed()->mutable_content() = content;
    placement.mutable_managed()->set_sha256(content.sha256());
    PluginValue value;
    value.set_enabled(true);
    const auto created = client.plugins.create("example", placement, value);
    try {
      // Wait for the Host to Start and Admit the Sidecar.
      auto entry = created.entry();
      PluginsWatchOptions watching;
      watching.name = entry.name();
      auto changes = client.plugins.watch(watching);
      while (entry.state() != PluginState::PLUGIN_STATE_READY) {
        if (entry.has_failure())
          throw runtime_error("Start Plugin: Resolve the Reported Failure: " + entry.failure().message());
        if (!changes.next())
          throw runtime_error("Start Plugin: Reconnect to the Host After Its Watch Closed");
        entry = client.plugins.get(entry.name()).entry();
      }
      changes.cancel();

      // Discover and Materialize the Shared Capture Suite.
      const auto catalog = client.suites.list(entry.name());
      if (catalog.values_size() == 0)
        throw runtime_error("Run Capture: Supply the Bundle Built From the Test Example");
      const auto suite = client.suites.get(catalog.values(0).metadata().identifier(), entry.name(), 1).value();
      if (suite.templates_size() == 0)
        throw runtime_error("Run Capture: Declare a Template in the Selected Suite");
      SelectionPlan plan;
      auto *selection = plan.add_selections();
      selection->set_suite_id(suite.metadata().identifier());
      selection->set_template_id(suite.templates(0).metadata().identifier());
      const auto cases = client.cases.create(plan, false);
      if (cases.cases_size() != 1)
        throw runtime_error("Run Capture: Select a Template That Produces One Case");
      Metadata metadata;
      metadata.set_label("Capture");
      const auto cycle = client.cycles.create({}, metadata, true, "").cycle();
      // Cycle and Execution Identities Belong to Separate Resources.
      const auto execution = cycle.cycle_id();
      ExecutionBinding binding;
      binding.set_execution_id(execution);
      binding.set_case_id(cases.cases(0).case_id());
      client.executions.create(cycle.cycle_id(), {binding});

      // Answer the Operator Through the Client.
      auto progress = client.executions.progress.watch(execution);
      while (const auto frame = progress.next()) {
        if (frame->phase() != "prompt")
          continue;
        PromptsListOptions pending;
        pending.execution_id = execution;
        const auto prompts = client.prompts.list("", true, pending);
        for (const auto &item : prompts.prompts()) {
          const auto question = client.prompts.get(item.prompt().prompt_id()).prompt();
          cout << question.prompt().definition().message() << '\n';
          for (const auto &action : question.prompt().definition().actions())
            cout << action.identifier() << ": " << action.label() << '\n';
          cout << "Choose an Action: " << flush;
          string answer;
          if (!getline(cin, answer))
            throw runtime_error("Answer Prompt: Supply One of the Displayed Action Identifiers");
          client.prompts.answers.create(item.prompt().prompt_id(), answer, {}, true);
        }
      }
      cout << client.executions.get(execution).execution().DebugString();
    } catch (...) {
      // Retire the Plugin Entry While Preserving Execution Evidence.
      const auto current = client.plugins.get(created.entry().name()).entry();
      client.plugins.delete_(current.name(), current.revision());
      throw;
    }
    const auto current = client.plugins.get(created.entry().name()).entry();
    client.plugins.delete_(current.name(), current.revision());
  } catch (const exception &error) {
    cerr << error.what() << '\n';
    result = 1;
  }
  // Drain EventEngine Work Before Windows Begins DLL Teardown.
  ShutdownDefaultEventEngine();
  return result;
}
