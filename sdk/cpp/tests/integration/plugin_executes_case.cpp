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
#include "dokime/dokime.hpp"
#include "dokime/plugin/v1/analyses.pb.h"
#include "dokime/plugin/v1/criteria.pb.h"
#include "dokime/plugin/v1/executions.pb.h"
#include "dokime/plugin/v1/steps.pb.h"
#include "dokime/plugin/v1/test_cases.pb.h"
#include <catch2/catch_session.hpp>

using dokime::Analyses;
using dokime::Origin;
using dokime::Suites;
using dokime::TERMINATION_COMPLETED;
using dokime::plugin::v1::AnalysesComputeRequest;
using dokime::plugin::v1::AnalysesComputeResponse;
using dokime::plugin::v1::CriteriaEvaluateRequest;
using dokime::plugin::v1::CriteriaEvaluateResponse;
using dokime::plugin::v1::ExecutionsCleanupRequest;
using dokime::plugin::v1::ExecutionsCleanupResponse;
using dokime::plugin::v1::ExecutionsInitializeRequest;
using dokime::plugin::v1::ExecutionsInitializeResponse;
using dokime::plugin::v1::StepsExecuteRequest;
using dokime::plugin::v1::StepsExecuteResponse;
using dokime::plugin::v1::TestCasesMaterializeRequest;
using dokime::plugin::v1::TestCasesMaterializeResponse;
using harmos::TransportError;
using harmos::VISIBILITY_PROTECTED;

using std::string;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

namespace {
string address;
string placement;
} // namespace

TEST_CASE("Plugin Executes a Case") {
  // Connect to the Managed or Remote Test Example.
  PluginHost host(address);
  auto description = host.describe();
  check(description.id() == "sample", "Wrong Plugin Connected");

  // Discover and Invoke a Protected Analysis with Raw Source-Time Coordinates.
  Analyses analyses;
  for (const auto &section : description.declarations().sections())
    if (section.type() == Analyses::descriptor()->full_name())
      check(analyses.ParseFromString(section.payload()), "Analyses Did Not Decode");
  check(analyses.analyses_size() == 1, "Analysis Was Not Published");
  const auto &analysis = analyses.analyses(0);
  bool protected_analysis = false;
  for (const auto &operation : description.declarations().operations())
    if (operation.id() == analysis.metadata().identifier())
      protected_analysis = operation.visibility() == VISIBILITY_PROTECTED &&
                           operation.request() == AnalysesComputeRequest::descriptor()->full_name() &&
                           operation.response() == AnalysesComputeResponse::descriptor()->full_name();
  check(protected_analysis, "Analysis Lost Its Protected Binding");
  AnalysesComputeRequest compute;
  auto *samples = compute.mutable_samples();
  samples->set_axis_unit(analysis.outputs(0).axis_unit());
  samples->set_unit(analysis.outputs(0).unit());
  samples->add_axis(0.125);
  samples->add_axis(0.75);
  samples->add_values(3);
  samples->add_values(-2);
  (*compute.mutable_parameters()->mutable_values())["gain"].set_number(4);
  const auto computed = host.invoke<AnalysesComputeResponse>(analysis.metadata().identifier(), compute);
  check(computed.outputs_size() == 1, "Analysis Output Was Lost");
  const auto &output = computed.outputs(0);
  check(output.axis_size() == 2 && output.axis(0) == samples->axis(0) && output.axis(1) == samples->axis(1), "Raw Analysis Axis Changed");
  check(output.values_size() == 2 && output.values(0) == 12 && output.values(1) == -8, "Analysis Parameters or Values Changed");
  check(output.key() == analysis.outputs(0).metadata().identifier() && output.unit() == samples->unit() &&
            output.axis_unit() == samples->axis_unit(),
        "Analysis Output Identity or Units Changed");

  // Preview a Materialized Case Without Creating Execution State.
  TestCasesMaterializeRequest materialize;
  materialize.set_template_("sample.check");
  (*materialize.mutable_parameters())["limit"].set_unsigned_integer(18446744073709551615ULL);
  auto *fraction = (*materialize.mutable_parameters())["ratio"].mutable_exact();
  fraction->set_negative(true);
  fraction->set_numerator("36893488147419103229");
  fraction->set_denominator("2");
  auto materialized = host.invoke<TestCasesMaterializeResponse>("sample.check.materialize", materialize);
  auto test_case = materialized.test_case();
  check(test_case.steps_size() == 1 && test_case.criteria_size() == 1, "Case Lost Its Operations");
  check(test_case.parameters().at("limit").unsigned_integer() == 18446744073709551615ULL, "Parameter Precision Was Lost");

  const auto &ratio = test_case.parameters().at("ratio");
  check(ratio.has_exact() && ratio.exact().negative() && ratio.exact().numerator() == fraction->numerator() &&
            ratio.exact().denominator() == "2",
        "Exact Parameter Was Coerced");

  // Freeze Its Origin and Execute the Step.
  Origin origin;
  *origin.mutable_plugin_run()->mutable_test_case() = test_case;
  origin.mutable_plugin_run()->set_plugin("sample");
  origin.mutable_plugin_run()->set_template_("sample.check");
  origin.mutable_plugin_run()->set_digest(description.declarations().digest());
  *origin.mutable_plugin_run()->mutable_started_at() = description.started_at();
  ExecutionsInitializeRequest initialize;
  initialize.set_execution("execution");
  *initialize.mutable_test_case() = test_case;
  *initialize.mutable_origin() = origin;
  const auto &definition = description.declarations();
  Suites suites;
  for (const auto &section : definition.sections())
    if (section.type() == Suites::descriptor()->full_name())
      check(suites.ParseFromString(section.payload()), "Suite Did Not Decode");
  const auto &binding = suites.suites(0).templates(0);
  host.invoke<ExecutionsInitializeResponse>(binding.initialize(), initialize, "execution");

  StepsExecuteRequest step;
  step.set_execution("execution");
  step.set_step("measure");
  *step.mutable_test_case() = test_case;
  *step.mutable_origin() = origin;
  host.invoke<StepsExecuteResponse>(test_case.steps(0).operation(), step, "execution");

  // Evaluate Using the Resource Created by the Step.
  CriteriaEvaluateRequest criterion;
  criterion.set_execution("execution");
  criterion.set_criterion("valid");
  *criterion.mutable_test_case() = test_case;
  *criterion.mutable_origin() = origin;
  auto verdict = host.invoke<CriteriaEvaluateResponse>(test_case.criteria(0).operation(), criterion, "execution");
  check(verdict.passed(), "Criterion Lost the Execution Resource");

  // Refuse an Attempt to Select Another Execution Scope.
  bool refused = false;
  try {
    host.invoke<StepsExecuteResponse>(test_case.steps(0).operation(), step, "other");
  } catch (const TransportError &) {
    refused = true;
  }
  check(refused, "Mismatched Scope Was Accepted");

  // Clean Up and Observe That the Resource Was Released.
  ExecutionsCleanupRequest cleanup;
  cleanup.set_execution("execution");
  cleanup.set_termination(TERMINATION_COMPLETED);
  *cleanup.mutable_test_case() = test_case;
  *cleanup.mutable_origin() = origin;
  host.invoke<ExecutionsCleanupResponse>(binding.cleanup(), cleanup, "execution");
  bool released = false;
  try {
    host.invoke<CriteriaEvaluateResponse>(test_case.criteria(0).operation(), criterion, "execution");
  } catch (const TransportError &) {
    released = true;
  }
  check(released, "Cleanup Retained the Execution Resource");

  // Stop and Check the Placement's Lifecycle.
  host.stop();
  if (placement == "remote")
    check(host.describe().id() == "sample", "Remote Stop Exited the Plugin");
}

int main(int argc, char **argv) {
  if (argc != 3)
    return 2;
  address = argv[1];
  placement = argv[2];
  Catch::Session session;
  return session.run(1, argv);
}
