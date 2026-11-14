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
#include "dokime/plugin.hpp"
#include <catch2/catch_test_macros.hpp>
#include <map>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

TEST_CASE("Author Materializes a Suite") {
  // Compose a Suite and Hand Its Values to the Plugin.
  dokime::Template value;
  *value.definition.mutable_metadata() = dokime::metadata("sample.measure", "Measure");
  value.initialize = [](dokime::Context &) {};
  value.steps.push_back({dokime::metadata("capture", "Capture"), [](dokime::Context &) {}, 2});
  value.criteria.push_back({dokime::metadata("valid", "Valid"), [](dokime::Context &) { return true; }});
  dokime::Plugin plugin("sample", "1.0.0");
  plugin.suite({dokime::metadata("sample", "Sample"), {value}});
  dokime::Prompt prompt;
  *prompt.mutable_metadata() = dokime::metadata("sample.confirm", "Confirm");
  plugin.prompt(prompt);

  // Inspect the Frozen Catalog and Its Protected Lifecycle Contracts.
  auto catalog = plugin.catalog();
  dokime::v1::Suites suites;
  for (const auto &section : catalog.sections())
    if (section.type() == "dokime.v1.Suites")
      check(suites.ParseFromString(section.payload()), "Suite Did Not Decode");
  check(suites.suites_size() == 1 && suites.suites(0).templates(0).materialize() == "sample.measure.materialize",
        "Suite Was Not Registered");
  // Decode Published Prompt Definitions with Their Shared Declaration Type.
  dokime::v1::PromptDefinitions prompts;
  for (const auto &section : catalog.sections())
    if (section.type() == dokime::v1::PromptDefinitions::descriptor()->full_name())
      check(prompts.ParseFromString(section.payload()), "Prompt Definitions Did Not Decode");
  check(prompts.prompts_size() == 1 && prompts.prompts(0).metadata().identifier() == prompt.metadata().identifier(),
        "Prompt Definition Was Not Published");

  bool retry = false;
  std::map<std::string, std::pair<const google::protobuf::Descriptor *, const google::protobuf::Descriptor *>> expected{
      {value.definition.metadata().identifier() + ".materialize",
       {dokime::plugin::v1::TestCasesMaterializeRequest::descriptor(), dokime::plugin::v1::TestCasesMaterializeResponse::descriptor()}},
      {value.definition.metadata().identifier() + ".initialize",
       {dokime::plugin::v1::ExecutionsInitializeRequest::descriptor(), dokime::plugin::v1::ExecutionsInitializeResponse::descriptor()}},
      {value.definition.metadata().identifier() + ".cleanup",
       {dokime::plugin::v1::ExecutionsCleanupRequest::descriptor(), dokime::plugin::v1::ExecutionsCleanupResponse::descriptor()}},
      {"sample.measure.step.0",
       {dokime::plugin::v1::StepsExecuteRequest::descriptor(), dokime::plugin::v1::StepsExecuteResponse::descriptor()}},
      {"sample.measure.criterion.0",
       {dokime::plugin::v1::CriteriaEvaluateRequest::descriptor(), dokime::plugin::v1::CriteriaEvaluateResponse::descriptor()}},
  };
  for (const auto &operation : catalog.operations()) {
    check(operation.visibility() == harmos::wire::VISIBILITY_PROTECTED, "An Authoring Operation Became Public");
    auto binding = expected.find(operation.id());
    check(binding != expected.end(), "Unexpected Lifecycle Binding");
    check(operation.request() == binding->second.first->full_name() && operation.response() == binding->second.second->full_name(),
          "Lifecycle Binding Lost Its Plugin-Face Payloads");
    expected.erase(binding);
    if (operation.id() == "sample.measure.step.0")
      retry = operation.policy().retries().total_attempts() == value.steps.front().retries + 1;
  }
  check(retry, "Step Retry Policy Was Lost");
  check(expected.empty(), "Lifecycle Binding Was Not Registered");

  // Correct a Duplicate Identifier Before Serving.
  bool duplicate = false;
  try {
    plugin.suite({dokime::metadata("sample", "Sample"), {value}});
  } catch (const std::invalid_argument &) {
    duplicate = true;
  }
  check(duplicate, "Duplicate Suite Was Accepted");
}
