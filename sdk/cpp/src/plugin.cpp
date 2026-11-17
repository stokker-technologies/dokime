/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "dokime/plugin.hpp"
#include "dokime/plugin/v1/criteria.pb.h"
#include "dokime/plugin/v1/executions.pb.h"
#include "dokime/plugin/v1/steps.pb.h"
#include "dokime/plugin/v1/test_cases.pb.h"
#include <google/protobuf/util/message_differencer.h>
#include <limits>
#include <set>

using google::protobuf::util::MessageDifferencer;
using harmos::Declarations;
using harmos::Failure;
using harmos::OperationPolicy;
using harmos::VISIBILITY_PROTECTED;
using harmos::plugin::Operation;

using std::invalid_argument;
using std::lock_guard;
using std::make_shared;
using std::move;
using std::numeric_limits;
using std::set;
using std::shared_ptr;
using std::size_t;
using std::string;
using std::to_string;
using std::uint32_t;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

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
using std::function;
using std::vector;

namespace dokime::plugin {
namespace {
void identifier(const string &value) {
  if (value.empty())
    throw invalid_argument("Identifier Is Required");
}
template <class Q>
Context execution_context(harmos::plugin::Context &transport, const Q &request, const string &template_id,
                          const shared_ptr<detail::Executions> &executions, const string &action, string item = {}) {
  const auto &origin = request.origin();
  const bool same_case = MessageDifferencer::Equals(request.test_case(), origin.plugin_run().test_case());
  const bool has_entry = !origin.plugin_run().plugin().empty();
  const bool same_template = origin.plugin_run().template_() == template_id;
  if (request.execution().empty() || transport.scope() != request.execution() || !origin.has_plugin_run() || !same_case || !same_template ||
      !has_entry)
    throw invalid_argument(action + ": Execution Scope, Template and Frozen Origin Must Agree");
  auto resources = executions->open(request.execution());
  {
    lock_guard lock(resources->mutex);
    resources->test_case = request.test_case();
  }
  return Context(transport, request.test_case(), move(item), move(resources));
}
} // namespace

Metadata metadata(string identifier, string label) {
  Metadata value;
  value.set_identifier(move(identifier));
  value.set_label(move(label));
  return value;
}
Plugin::Plugin(string identifier, string version)
    : native_(identifier, move(version)), identifier_(move(identifier)), executions_(make_shared<detail::Executions>()) {
  native_.on_stop([state = executions_] {
    state->clear();
  });
}

Plugin &Plugin::suite(Suite suite) {
  identifier(suite.metadata.identifier());
  for (const auto &existing : suites_.suites())
    if (existing.metadata().identifier() == suite.metadata.identifier())
      throw invalid_argument("Duplicate Suite Identifier");
  dokime::Suite declaration;
  *declaration.mutable_metadata() = move(suite.metadata);
  for (auto &value : suite.templates) {
    const auto id = value.definition.metadata().identifier();
    identifier(id);
    for (const auto &existing : suites_.suites())
      for (const auto &previous : existing.templates())
        if (previous.metadata().identifier() == id)
          throw invalid_argument("Duplicate Template Identifier");
    for (const auto &previous : declaration.templates())
      if (previous.metadata().identifier() == id)
        throw invalid_argument("Duplicate Template Identifier");
    const auto state = executions_;
    TestCase test_case;
    set<string> steps;
    size_t step_index = 0;
    for (auto &step : value.steps) {
      identifier(step.metadata.identifier());
      if (!step.run || !steps.insert(step.metadata.identifier()).second)
        throw invalid_argument("Step Handler Is Required and Identifiers Must Be Unique");
      if (step.retries == numeric_limits<uint32_t>::max())
        throw invalid_argument("Retry Count Is Too Large");
      const auto operation_id = id + ".step." + to_string(step_index++);
      auto *declared = test_case.add_steps();
      *declared->mutable_metadata() = step.metadata;
      declared->set_operation(operation_id);
      declared->set_repeatable(step.repeatable);
      const auto item = step.metadata.identifier();
      Operation<StepsExecuteRequest, StepsExecuteResponse> operation(
          operation_id, VISIBILITY_PROTECTED,
          [state, id, item, run = move(step.run)](harmos::plugin::Context &transport, StepsExecuteRequest request) {
            if (request.step() != item)
              throw invalid_argument("Execute Step: Identifier Does Not Match Its Operation");
            auto ctx = execution_context(transport, request, id, state, "Execute Step", item);
            run(ctx);
            return StepsExecuteResponse{};
          });
      if (step.retries) {
        OperationPolicy policy;
        policy.mutable_retries()->set_total_attempts(step.retries + 1);
        policy.mutable_retries()->mutable_initial()->set_nanos(100000000);
        operation.replay_safe();
        operation.policy(policy);
      }
      native_.operation(operation);
    }
    set<string> criteria;
    size_t criterion_index = 0;
    for (auto &criterion : value.criteria) {
      identifier(criterion.metadata.identifier());
      if (!criterion.evaluate || !criteria.insert(criterion.metadata.identifier()).second)
        throw invalid_argument("Criterion Handler Is Required and Identifiers Must Be Unique");
      const auto operation_id = id + ".criterion." + to_string(criterion_index++);
      auto *declared = test_case.add_criteria();
      *declared->mutable_metadata() = criterion.metadata;
      declared->set_operation(operation_id);
      const auto item = criterion.metadata.identifier();
      native_.operation(Operation<CriteriaEvaluateRequest, CriteriaEvaluateResponse>(
          operation_id, VISIBILITY_PROTECTED,
          [state, id, item, evaluate = move(criterion.evaluate)](harmos::plugin::Context &transport, CriteriaEvaluateRequest request) {
            if (request.criterion() != item)
              throw invalid_argument("Evaluate Criterion: Identifier Does Not Match Its Operation");
            auto ctx = execution_context(transport, request, id, state, "Evaluate Criterion", item);
            CriteriaEvaluateResponse response;
            response.set_passed(evaluate(ctx));
            return response;
          }));
    }
    for (auto &stream : value.streams)
      test_case.add_streams(move(stream));
    value.definition.set_materialize(id + ".materialize");
    native_.operation(Operation<TestCasesMaterializeRequest, TestCasesMaterializeResponse>(
        id + ".materialize", VISIBILITY_PROTECTED,
        [id, test_case, materialize = move(value.materialize)](harmos::plugin::Context &transport, TestCasesMaterializeRequest request) {
          try {
            transport.scope();
            throw invalid_argument("Materialize Requires No Execution Scope");
          } catch (const Failure &failure) {
            if (failure.value.code() != "scope_required")
              throw;
          }
          if (request.template_() != id)
            throw invalid_argument("Materialize Requires Its Template and No Execution Scope");
          auto result = test_case;
          *result.mutable_parameters() = request.parameters();
          if (materialize)
            materialize(result, request.parameters());
          TestCasesMaterializeResponse response;
          *response.mutable_test_case() = move(result);
          return response;
        }));
    if (value.initialize) {
      value.definition.set_initialize(id + ".initialize");
      native_.operation(Operation<ExecutionsInitializeRequest, ExecutionsInitializeResponse>(
          id + ".initialize", VISIBILITY_PROTECTED,
          [state, id, initialize = move(value.initialize)](harmos::plugin::Context &transport, ExecutionsInitializeRequest request) {
            auto ctx = execution_context(transport, request, id, state, "Initialize Execution");
            try {
              initialize(ctx);
            } catch (...) {
              state->close(request.execution());
              throw;
            }
            return ExecutionsInitializeResponse{};
          }));
    }
    // Always Register Cleanup So the Host Can Release Execution Resources.
    value.definition.set_cleanup(id + ".cleanup");
    native_.operation(Operation<ExecutionsCleanupRequest, ExecutionsCleanupResponse>(
        id + ".cleanup", VISIBILITY_PROTECTED,
        [state, id, cleanup = move(value.cleanup)](harmos::plugin::Context &transport, ExecutionsCleanupRequest request) {
          auto ctx = execution_context(transport, request, id, state, "Clean Up Execution");
          try {
            if (cleanup)
              cleanup(ctx);
          } catch (...) {
            state->close(request.execution());
            throw;
          }
          state->close(request.execution());
          return ExecutionsCleanupResponse{};
        }));
    // Declare the Procedure on the Template So Catalogs Show It Before Materialization.
    *value.definition.mutable_steps() = test_case.steps();
    *declaration.add_templates() = move(value.definition);
  }
  *suites_.add_suites() = move(declaration);
  return *this;
}
Plugin &Plugin::prompt(Prompt prompt) {
  identifier(prompt.metadata().identifier());
  for (const auto &existing : prompts_.prompts())
    if (existing.metadata().identifier() == prompt.metadata().identifier())
      throw invalid_argument("Duplicate Prompt Identifier");
  *prompts_.add_prompts() = move(prompt);
  return *this;
}
Plugin &Plugin::context_operations() {
  requires_operation(HostOperation::PromptsCreate);
  requires_operation(HostOperation::PluginsSettingsList);
  requires_operation(HostOperation::SecretsGet);
  requires_operation(HostOperation::EvidenceRecordsAppend);
  requires_operation(HostOperation::EvidenceRecordsList);
  requires_operation(HostOperation::EvidenceStatistics);
  requires_operation(HostOperation::ExecutionsProgressGet);
  requires_operation(HostOperation::ExecutionsProgressSet);
  requires_operation(HostOperation::EvidenceLogsAppend);
  requires_operation(HostOperation::EvidenceTraceabilityGet);
  requires_operation(HostOperation::EvidenceTraceabilitySet);
  requires_operation(HostOperation::DocumentsAppendParts);
  requires_operation(HostOperation::TimeGet);
  requires_operation(HostOperation::TimeSleep);
  return *this;
}
harmos::plugin::Plugin Plugin::compose() const {
  auto result = native_;
  if (suites_.suites_size())
    result.section(suites_);
  if (prompts_.prompts_size())
    result.section(prompts_);
  if (records_.records_size())
    result.section(records_);
  return result;
}
Declarations Plugin::declarations() const {
  return compose().declarations();
}
void Plugin::serve(string address) const {
  compose().serve(move(address));
}
} // namespace dokime::plugin

/* === Private Functions === */

/* === Entry Point === */
