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

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

int main(int argc, char **argv) {
  if (argc != 3)
    return 2;

  // Connect to the Managed or Remote Starter.
  PluginHost host(argv[1]);
  auto description = host.describe();
  check(description.id() == "sample", "Wrong Plugin Connected");

  // Preview a Materialized Case Without Creating Execution State.
  dokime::v1::MaterializeRequest materialize;
  materialize.set_template_("sample.check");
  (*materialize.mutable_parameters())["limit"].set_unsigned_integer(18446744073709551615ULL);
  auto test_case = host.invoke<dokime::v1::TestCase>("sample.check.materialize", materialize);
  check(test_case.steps_size() == 1 && test_case.criteria_size() == 1, "Case Lost Its Operations");
  check(test_case.parameters().at("limit").unsigned_integer() == 18446744073709551615ULL, "Parameter Precision Was Lost");

  // Freeze Its Origin and Execute the Step.
  dokime::v1::Origin origin;
  *origin.mutable_plugin_run()->mutable_test_case() = test_case;
  origin.mutable_plugin_run()->set_plugin("sample");
  origin.mutable_plugin_run()->set_template_("sample.check");
  origin.mutable_plugin_run()->set_digest(description.catalog().digest());
  *origin.mutable_plugin_run()->mutable_started_at() = description.started_at();
  dokime::v1::StepRequest step;
  step.set_execution("execution");
  step.set_step("measure");
  *step.mutable_test_case() = test_case;
  *step.mutable_origin() = origin;
  host.invoke<dokime::v1::StepResponse>(test_case.steps(0).operation(), step, "execution");

  // Evaluate Using the Resource Created by the Step.
  dokime::v1::CriterionRequest criterion;
  criterion.set_execution("execution");
  criterion.set_criterion("valid");
  *criterion.mutable_test_case() = test_case;
  *criterion.mutable_origin() = origin;
  auto verdict = host.invoke<dokime::v1::CriterionResponse>(test_case.criteria(0).operation(), criterion, "execution");
  check(verdict.passed(), "Criterion Lost the Execution Resource");

  // Refuse an Attempt to Select Another Execution Scope.
  bool refused = false;
  try {
    host.invoke<dokime::v1::StepResponse>(test_case.steps(0).operation(), step, "other");
  } catch (const harmos::TransportError &) {
    refused = true;
  }
  check(refused, "Mismatched Scope Was Accepted");

  // Clean Up and Observe That the Resource Was Released.
  dokime::v1::CleanupRequest cleanup;
  cleanup.set_execution("execution");
  cleanup.set_termination(dokime::v1::TERMINATION_COMPLETED);
  *cleanup.mutable_test_case() = test_case;
  *cleanup.mutable_origin() = origin;
  host.invoke<dokime::v1::CleanupResponse>("sample.check.cleanup", cleanup, "execution");
  bool released = false;
  try {
    host.invoke<dokime::v1::CriterionResponse>(test_case.criteria(0).operation(), criterion, "execution");
  } catch (const harmos::TransportError &) {
    released = true;
  }
  check(released, "Cleanup Retained the Execution Resource");

  // Stop and Check the Placement's Lifecycle.
  host.stop();
  if (std::string(argv[2]) == "remote")
    check(host.describe().id() == "sample", "Remote Stop Exited the Plugin");
}
