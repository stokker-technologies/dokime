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
#include <string>

// Third Party Imports
#include <catch2/catch_session.hpp>
#include <catch2/catch_test_macros.hpp>
#include <google/protobuf/empty.pb.h>
#include <google/protobuf/wrappers.pb.h>
#include <grpc/event_engine/event_engine.h>

// Local Imports
#include "../support/plugin_host.hpp"
#include "dokime/plugin.hpp"
#include "dokime/plugin/v1/executions.pb.h"
#include "dokime/plugin/v1/test_cases.pb.h"

using dokime::Suites;
using dokime::TERMINATION_FAILED;
using dokime::plugin::v1::ExecutionsCleanupRequest;
using dokime::plugin::v1::ExecutionsCleanupResponse;
using dokime::plugin::v1::ExecutionsInitializeRequest;
using dokime::plugin::v1::ExecutionsInitializeResponse;
using dokime::plugin::v1::TestCasesMaterializeRequest;
using dokime::plugin::v1::TestCasesMaterializeResponse;
using google::protobuf::BoolValue;
using google::protobuf::Empty;
using google::protobuf::StringValue;
using harmos::OperationError;
using harmos::SettingsDeclarations;
using harmos::VISIBILITY_PROTECTED;
using harmos::plugin::SettingsListResponse;
using std::string;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

namespace {
string address;
}

/* === Entry Point === */

TEST_CASE("Failed Initialization Retains Template State for Cleanup and Both Contexts Share the Author Plugin") {
  // Read the Plugin Description.
  PluginHost host(address);
  const auto description = host.describe();
  for (const auto &identifier : {description.id() + ".initialize", description.id() + ".finalize"}) {
    bool found = false;
    for (const auto &operation : description.declarations().operations())
      if (operation.id() == identifier) {
        REQUIRE(operation.visibility() == VISIBILITY_PROTECTED);
        found = true;
      }
    REQUIRE(found);
  }
  SettingsDeclarations settings;
  for (const auto &section : description.declarations().sections())
    if (section.type() == SettingsDeclarations::descriptor()->full_name())
      REQUIRE(settings.ParseFromString(section.payload()));
  REQUIRE(settings.values_size() == 2);
  string saved_address = "127.0.0.1:30059";
  unsigned reads = 0;
  host.child = [&](const harmos::plugin::v1::ChildRequest &) {
    ++reads;
    SettingsListResponse response;
    auto &values = *response.mutable_value()->mutable_values();
    values["api_address"].set_text(saved_address);
    values["dll_timeout"].mutable_duration()->set_seconds(5);
    return response.SerializeAsString();
  };
  host.invoke<Empty>(description.id() + ".initialize", Empty{});
  REQUIRE(reads == 1);
  REQUIRE(host.invoke<StringValue>("lifecycle.snapshot", StringValue{}).value() == saved_address);
  saved_address = "127.0.0.1:30060";
  REQUIRE(host.invoke<StringValue>("lifecycle.snapshot", StringValue{}).value() == "127.0.0.1:30059");
  REQUIRE(reads == 1);
  Suites suites;
  for (const auto &section : description.declarations().sections())
    if (section.type() == Suites::descriptor()->full_name())
      REQUIRE(suites.ParseFromString(section.payload()));
  const auto &binding = suites.suites(0).templates(0);

  // Materialize a Case and Freeze Its Origin.
  TestCasesMaterializeRequest materialize;
  materialize.set_template_(binding.metadata().identifier());
  const auto test_case = host.invoke<TestCasesMaterializeResponse>(binding.materialize(), materialize).test_case();
  ExecutionsInitializeRequest initialize;
  initialize.set_execution("execution");
  *initialize.mutable_test_case() = test_case;
  auto *origin = initialize.mutable_origin()->mutable_plugin_run();
  origin->set_plugin(description.id());
  origin->set_template_(binding.metadata().identifier());
  origin->set_digest(description.declarations().digest());
  *origin->mutable_started_at() = description.started_at();
  *origin->mutable_test_case() = test_case;

  // Fail Initialization After Opening the Template Resource.
  REQUIRE_THROWS_AS(host.invoke<ExecutionsInitializeResponse>(binding.initialize(), initialize, "execution"), OperationError);

  // Clean Up the Same Instance With Its Termination.
  ExecutionsCleanupRequest cleanup;
  cleanup.set_execution("execution");
  cleanup.set_termination(TERMINATION_FAILED);
  *cleanup.mutable_test_case() = test_case;
  *cleanup.mutable_origin() = initialize.origin();
  REQUIRE_NOTHROW(host.invoke<ExecutionsCleanupResponse>(binding.cleanup(), cleanup, "execution"));

  // Read Shared Plugin State Through an Ordinary Operation.
  const auto cleaned = host.invoke<BoolValue>("lifecycle.cleaned", BoolValue{});
  REQUIRE(cleaned.value());
  host.invoke<Empty>(description.id() + ".finalize", Empty{});
  const auto finalized = host.invoke<BoolValue>("lifecycle.finalized", BoolValue{});
  REQUIRE(finalized.value());
  host.invoke<Empty>(description.id() + ".initialize", Empty{});
  REQUIRE(reads == 2);
  REQUIRE(host.invoke<StringValue>("lifecycle.snapshot", StringValue{}).value() == saved_address);
  host.invoke<Empty>(description.id() + ".finalize", Empty{});
  host.stop();
}

int main(int argc, char **argv) {
  if (argc != 3)
    return 2;
  address = argv[1];
  const auto result = Catch::Session{}.run(1, argv);
  grpc_event_engine::experimental::ShutdownDefaultEventEngine();
  return result;
}
