/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "../support/check.hpp"
#include "../support/plugin_host.hpp"
#include "dokime/plugin.hpp"
#include "dokime/plugin/v1/criteria.pb.h"
#include "dokime/plugin/v1/executions.pb.h"
#include "dokime/plugin/v1/steps.pb.h"
#include "dokime/plugin/v1/test_cases.pb.h"
#include <catch2/catch_session.hpp>
#include <grpc/event_engine/event_engine.h>

using dokime::Origin;
using dokime::RecordContracts;
using dokime::Suites;
using dokime::plugin::v1::CriteriaEvaluateRequest;
using dokime::plugin::v1::CriteriaEvaluateResponse;
using dokime::plugin::v1::ExecutionsCleanupRequest;
using dokime::plugin::v1::ExecutionsCleanupResponse;
using dokime::plugin::v1::StepsExecuteRequest;
using dokime::plugin::v1::StepsExecuteResponse;
using dokime::plugin::v1::TestCasesMaterializeRequest;
using dokime::plugin::v1::TestCasesMaterializeResponse;
using google::protobuf::BoolValue;
using grpc_event_engine::experimental::ShutdownDefaultEventEngine;
using harmos::TransportError;
using std::invalid_argument;
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
string placement;
} // namespace

/* === Entry Point === */

TEST_CASE("Capture Prompts Records and Reads Plain Values Over the Carrier") {
  // Connect to the Shared Example.
  PluginHost host(address);
  const auto description = host.describe();
  check(description.id() == "example", "Wrong Plugin Connected");
  Suites suites;
  RecordContracts contracts;
  for (const auto &section : description.declarations().sections()) {
    if (section.type() == Suites::descriptor()->full_name())
      check(suites.ParseFromString(section.payload()), "Suite Did Not Decode");
    if (section.type() == RecordContracts::descriptor()->full_name())
      check(contracts.ParseFromString(section.payload()), "Records Did Not Decode");
  }
  check(contracts.records(0).metadata().identifier() == "capture.confirmation.confirmation", "Record Was Not Qualified");
  check(contracts.records(0).message() == BoolValue::descriptor()->full_name(), "Plain Record Contract Was Lost");
  const auto &binding = suites.suites(0).templates(0);
  check(binding.metadata().identifier() == "capture.confirmation", "Explicit Template Identifier Changed");

  // Materialize Without an Execution Scope.
  TestCasesMaterializeRequest materialize;
  materialize.set_template_(binding.metadata().identifier());
  (*materialize.mutable_parameters())["limit"].set_unsigned_integer(UINT64_MAX);
  const auto test_case = host.invoke<TestCasesMaterializeResponse>(binding.materialize(), materialize).test_case();
  check(test_case.parameters().at("limit").unsigned_integer() == UINT64_MAX, "Parameter Precision Was Lost");
  check(test_case.steps(0).metadata().identifier() == "capture.confirmation.confirm", "Step Identifier Changed");
  check(test_case.criteria(0).metadata().identifier() == "capture.confirmation.confirmed", "Criterion Identifier Changed");

  // Answer Host Calls Deterministically and Preserve Record Bytes.
  string answer = "0";
  string saved;
  bool has_record = false;
  bool accepted = true;
  host.child = [&](const string &operation, const string &bytes) -> string {
    if (operation == "dokime.host.v1.Prompts/Create") {
      const auto request = harmos::plugin::decode<dokime::host::v1::PromptsCreateRequest>(bytes);
      check(request.definition().actions_size() == 2, "Prompt Lost Its Actions");
      check(request.prompt() == "capture.confirmation.question" &&
                request.definition().metadata().identifier() == "capture.confirmation.question",
            "Short Prompt Was Not Qualified by Its Template");
      dokime::host::v1::PromptsCreateResponse response;
      response.set_accepted(accepted);
      response.set_action(answer);
      return response.SerializeAsString();
    }
    if (operation == "harmos.plugin.v1.Time/Get") {
      harmos::host::v1::TimeGetResponse response;
      response.mutable_now()->set_seconds(123);
      return response.SerializeAsString();
    }
    if (operation == "dokime.host.v1.evidence.Records/Append") {
      const auto request = harmos::plugin::decode<dokime::host::v1::evidence::RecordsAppendRequest>(bytes);
      check(request.kind() == "capture.confirmation.confirmation" && request.execution() == "execution",
            "Record Declaration Was Not Resolved");
      saved = request.payload();
      has_record = true;
      return dokime::host::v1::evidence::RecordsAppendResponse{}.SerializeAsString();
    }
    if (operation == "dokime.host.v1.evidence.Records/List") {
      const auto request = harmos::plugin::decode<dokime::host::v1::evidence::RecordsListRequest>(bytes);
      check(request.record_type() == "capture.confirmation.confirmation" && request.step() == "capture.confirmation.confirm",
            "Query Declaration Was Not Resolved");
      dokime::host::v1::evidence::RecordsListResponse response;
      if (has_record)
        response.mutable_last()->set_payload(saved);
      return response.SerializeAsString();
    }
    throw invalid_argument("Answer Child Call: Implement " + operation);
  };

  // Freeze the Case and Run Both Operator Choices.
  Origin origin;
  *origin.mutable_plugin_run()->mutable_test_case() = test_case;
  origin.mutable_plugin_run()->set_plugin("example");
  origin.mutable_plugin_run()->set_template_("capture.confirmation");
  origin.mutable_plugin_run()->set_digest(description.declarations().digest());
  *origin.mutable_plugin_run()->mutable_started_at() = description.started_at();
  StepsExecuteRequest step;
  step.set_execution("execution");
  step.set_step("capture.confirmation.confirm");
  *step.mutable_test_case() = test_case;
  *step.mutable_origin() = origin;
  CriteriaEvaluateRequest criterion;
  criterion.set_execution("execution");
  criterion.set_criterion("capture.confirmation.confirmed");
  *criterion.mutable_test_case() = test_case;
  *criterion.mutable_origin() = origin;
  for (const bool yes : {true, false}) {
    answer = yes ? "0" : "1";
    host.invoke<StepsExecuteResponse>(test_case.steps(0).operation(), step, "execution");
    check(harmos::plugin::decode<BoolValue>(saved).value() == yes, "Plain Bool Did Not Round Trip");
    const auto verdict = host.invoke<CriteriaEvaluateResponse>(test_case.criteria(0).operation(), criterion, "execution");
    check(verdict.passed() == yes, "Criterion Did Not Unwrap the Bool");
  }

  // Refuse Empty Queries, Invalid Answers and a Widened Execution Scope.
  has_record = false;
  REQUIRE_THROWS_AS(host.invoke<CriteriaEvaluateResponse>(test_case.criteria(0).operation(), criterion, "execution"), TransportError);
  accepted = false;
  REQUIRE_THROWS_AS(host.invoke<StepsExecuteResponse>(test_case.steps(0).operation(), step, "execution"), TransportError);
  accepted = true;
  answer = "unknown";
  REQUIRE_THROWS_AS(host.invoke<StepsExecuteResponse>(test_case.steps(0).operation(), step, "execution"), TransportError);
  REQUIRE_THROWS_AS(host.invoke<StepsExecuteResponse>(test_case.steps(0).operation(), step, "other"), TransportError);

  // Clean Up and Stop the Selected Placement.
  ExecutionsCleanupRequest cleanup;
  cleanup.set_execution("execution");
  *cleanup.mutable_test_case() = test_case;
  *cleanup.mutable_origin() = origin;
  host.invoke<ExecutionsCleanupResponse>(binding.cleanup(), cleanup, "execution");
  host.stop();
  if (placement == "remote")
    check(host.describe().id() == "example", "Remote Stop Exited the Plugin");
}

int main(int argc, char **argv) {
  if (argc != 3)
    return 2;
  address = argv[1];
  placement = argv[2];
  const auto result = Catch::Session{}.run(1, argv);
  ShutdownDefaultEventEngine();
  return result;
}
