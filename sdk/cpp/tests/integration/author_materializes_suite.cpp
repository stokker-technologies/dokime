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
#include "dokime/plugin/v1/criteria.pb.h"
#include "dokime/plugin/v1/executions.pb.h"
#include "dokime/plugin/v1/steps.pb.h"
#include "dokime/plugin/v1/test_cases.pb.h"
#include <catch2/catch_test_macros.hpp>
#include <map>

using dokime::Prompt;
using dokime::PromptDefinitions;
using dokime::Suites;
using dokime::plugin::Context;
using dokime::plugin::Plugin;
using dokime::plugin::Template;
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
using google::protobuf::Descriptor;
using harmos::VISIBILITY_PROTECTED;

using dokime::plugin::metadata;
using std::invalid_argument;
using std::map;
using std::string;

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
  Template value;
  *value.definition.mutable_metadata() = metadata("sample.measure", "Measure");
  value.initialize = [](Context &) {
  };
  value.steps.push_back({metadata("capture", "Capture"),
                         [](Context &) {
                         },
                         2});
  value.criteria.push_back({metadata("valid", "Valid"), [](Context &) {
                              return true;
                            }});
  Plugin plugin("sample", "1.0.0");
  plugin.suite({metadata("sample", "Sample"), {value}});
  Prompt prompt;
  *prompt.mutable_metadata() = metadata("sample.confirm", "Confirm");
  plugin.prompt(prompt);

  // Inspect the Frozen Declarations and Its Protected Lifecycle Contracts.
  auto declarations = plugin.declarations();
  Suites suites;
  for (const auto &section : declarations.sections())
    if (section.type() == Suites::descriptor()->full_name())
      check(suites.ParseFromString(section.payload()), "Suite Did Not Decode");
  check(suites.suites_size() == 1 && suites.suites(0).templates(0).materialize() == "sample.measure.materialize",
        "Suite Was Not Registered");
  // The Catalog Declares the Procedure Before Any Case Is Materialized.
  const auto &declared = suites.suites(0).templates(0);
  check(declared.steps_size() == 1 && declared.steps(0).metadata().identifier() == "capture" &&
            declared.steps(0).metadata().label() == "Capture" && declared.steps(0).operation() == "sample.measure.step.0",
        "Template Did Not Declare Its Steps");
  // Decode Published Prompt Definitions with Their Shared Declaration Type.
  PromptDefinitions prompts;
  for (const auto &section : declarations.sections())
    if (section.type() == PromptDefinitions::descriptor()->full_name())
      check(prompts.ParseFromString(section.payload()), "Prompt Definitions Did Not Decode");
  check(prompts.prompts_size() == 1 && prompts.prompts(0).metadata().identifier() == prompt.metadata().identifier(),
        "Prompt Definition Was Not Published");

  bool retry = false;
  map<string, std::pair<const Descriptor *, const Descriptor *>> expected{
      {value.definition.metadata().identifier() + ".materialize",
       {TestCasesMaterializeRequest::descriptor(), TestCasesMaterializeResponse::descriptor()}},
      {value.definition.metadata().identifier() + ".initialize",
       {ExecutionsInitializeRequest::descriptor(), ExecutionsInitializeResponse::descriptor()}},
      {value.definition.metadata().identifier() + ".cleanup",
       {ExecutionsCleanupRequest::descriptor(), ExecutionsCleanupResponse::descriptor()}},
      {"sample.measure.step.0", {StepsExecuteRequest::descriptor(), StepsExecuteResponse::descriptor()}},
      {"sample.measure.criterion.0", {CriteriaEvaluateRequest::descriptor(), CriteriaEvaluateResponse::descriptor()}},
  };
  for (const auto &operation : declarations.operations()) {
    check(operation.visibility() == VISIBILITY_PROTECTED, "An Authoring Operation Became Public");
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
    plugin.suite({metadata("sample", "Sample"), {value}});
  } catch (const invalid_argument &) {
    duplicate = true;
  }
  check(duplicate, "Duplicate Suite Was Accepted");
}
