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
#include <google/protobuf/util/message_differencer.h>
#include <limits>
#include <set>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime {
namespace {
void identifier(const std::string &value) {
  if (value.empty())
    throw std::invalid_argument("Identifier Is Required");
}
template <class Q>
Context execution_context(harmos::Context &transport, const Q &request, const std::string &template_id, const std::string &plugin_id,
                          const std::shared_ptr<detail::Executions> &executions, const std::string &action, std::string item = {}) {
  const auto &origin = request.origin();
  const bool same_case = google::protobuf::util::MessageDifferencer::Equals(request.test_case(), origin.plugin_run().test_case());
  const bool same_plugin = origin.plugin_run().plugin() == plugin_id;
  const bool same_template = origin.plugin_run().template_() == template_id;
  if (request.execution().empty() || transport.scope() != request.execution() || !origin.has_plugin_run() || !same_case || !same_template ||
      !same_plugin)
    throw std::invalid_argument(action + ": Execution Scope, Template and Frozen Origin Must Agree");
  auto resources = executions->open(request.execution());
  {
    std::lock_guard lock(resources->mutex);
    resources->test_case = request.test_case();
  }
  return Context(transport, request.test_case(), std::move(item), std::move(resources));
}
} // namespace

v1::Metadata metadata(std::string identifier, std::string label) {
  v1::Metadata value;
  value.set_identifier(std::move(identifier));
  value.set_label(std::move(label));
  return value;
}
Plugin::Plugin(std::string identifier, std::string version)
    : native_(identifier, std::move(version)), identifier_(std::move(identifier)), executions_(std::make_shared<detail::Executions>()) {
  native_.on_stop([state = executions_] { state->clear(); });
}

Plugin &Plugin::suite(Suite suite) {
  identifier(suite.metadata.identifier());
  for (const auto &existing : suites_.suites())
    if (existing.metadata().identifier() == suite.metadata.identifier())
      throw std::invalid_argument("Duplicate Suite Identifier");
  v1::Suite declaration;
  *declaration.mutable_metadata() = std::move(suite.metadata);
  for (auto &value : suite.templates) {
    const auto id = value.definition.metadata().identifier();
    identifier(id);
    for (const auto &existing : suites_.suites())
      for (const auto &previous : existing.templates())
        if (previous.metadata().identifier() == id)
          throw std::invalid_argument("Duplicate Template Identifier");
    for (const auto &previous : declaration.templates())
      if (previous.metadata().identifier() == id)
        throw std::invalid_argument("Duplicate Template Identifier");
    const auto state = executions_;
    const auto plugin_id = identifier_;
    v1::TestCase test_case;
    std::set<std::string> steps;
    std::size_t step_index = 0;
    for (auto &step : value.steps) {
      identifier(step.metadata.identifier());
      if (!step.run || !steps.insert(step.metadata.identifier()).second)
        throw std::invalid_argument("Step Handler Is Required and Identifiers Must Be Unique");
      if (step.retries == std::numeric_limits<std::uint32_t>::max())
        throw std::invalid_argument("Retry Count Is Too Large");
      const auto operation_id = id + ".step." + std::to_string(step_index++);
      auto *declared = test_case.add_steps();
      *declared->mutable_metadata() = step.metadata;
      declared->set_operation(operation_id);
      declared->set_repeatable(step.repeatable);
      const auto item = step.metadata.identifier();
      harmos::Operation<plugin::v1::StepsExecuteRequest, plugin::v1::StepsExecuteResponse> operation(
          operation_id, harmos::wire::VISIBILITY_PROTECTED,
          [state, id, plugin_id, item, run = std::move(step.run)](harmos::Context &transport, plugin::v1::StepsExecuteRequest request) {
            if (request.step() != item)
              throw std::invalid_argument("Execute Step: Identifier Does Not Match Its Operation");
            auto ctx = execution_context(transport, request, id, plugin_id, state, "Execute Step", item);
            run(ctx);
            return plugin::v1::StepsExecuteResponse{};
          });
      if (step.retries) {
        harmos::wire::OperationPolicy policy;
        policy.mutable_retries()->set_total_attempts(step.retries + 1);
        policy.mutable_retries()->mutable_initial()->set_nanos(100000000);
        operation.replay_safe();
        operation.policy(policy);
      }
      native_.operation(operation);
    }
    std::set<std::string> criteria;
    std::size_t criterion_index = 0;
    for (auto &criterion : value.criteria) {
      identifier(criterion.metadata.identifier());
      if (!criterion.evaluate || !criteria.insert(criterion.metadata.identifier()).second)
        throw std::invalid_argument("Criterion Handler Is Required and Identifiers Must Be Unique");
      const auto operation_id = id + ".criterion." + std::to_string(criterion_index++);
      auto *declared = test_case.add_criteria();
      *declared->mutable_metadata() = criterion.metadata;
      declared->set_operation(operation_id);
      const auto item = criterion.metadata.identifier();
      native_.operation(harmos::Operation<plugin::v1::CriteriaEvaluateRequest, plugin::v1::CriteriaEvaluateResponse>(
          operation_id, harmos::wire::VISIBILITY_PROTECTED,
          [state, id, plugin_id, item, evaluate = std::move(criterion.evaluate)](harmos::Context &transport,
                                                                                 plugin::v1::CriteriaEvaluateRequest request) {
            if (request.criterion() != item)
              throw std::invalid_argument("Evaluate Criterion: Identifier Does Not Match Its Operation");
            auto ctx = execution_context(transport, request, id, plugin_id, state, "Evaluate Criterion", item);
            plugin::v1::CriteriaEvaluateResponse response;
            response.set_passed(evaluate(ctx));
            return response;
          }));
    }
    for (auto &stream : value.streams)
      test_case.add_streams(std::move(stream));
    value.definition.set_materialize(id + ".materialize");
    native_.operation(harmos::Operation<plugin::v1::TestCasesMaterializeRequest, plugin::v1::TestCasesMaterializeResponse>(
        id + ".materialize", harmos::wire::VISIBILITY_PROTECTED,
        [id, test_case, materialize = std::move(value.materialize)](harmos::Context &transport,
                                                                    plugin::v1::TestCasesMaterializeRequest request) {
          try {
            transport.scope();
            throw std::invalid_argument("Materialize Requires No Execution Scope");
          } catch (const harmos::Failure &failure) {
            if (failure.value.code() != "scope_required")
              throw;
          }
          if (request.template_() != id)
            throw std::invalid_argument("Materialize Requires Its Template and No Execution Scope");
          auto result = test_case;
          *result.mutable_parameters() = request.parameters();
          if (materialize)
            materialize(result, request.parameters());
          plugin::v1::TestCasesMaterializeResponse response;
          *response.mutable_test_case() = std::move(result);
          return response;
        }));
    if (value.initialize) {
      value.definition.set_initialize(id + ".initialize");
      native_.operation(harmos::Operation<plugin::v1::ExecutionsInitializeRequest, plugin::v1::ExecutionsInitializeResponse>(
          id + ".initialize", harmos::wire::VISIBILITY_PROTECTED,
          [state, id, plugin_id, initialize = std::move(value.initialize)](harmos::Context &transport,
                                                                           plugin::v1::ExecutionsInitializeRequest request) {
            auto ctx = execution_context(transport, request, id, plugin_id, state, "Initialize Execution");
            try {
              initialize(ctx);
            } catch (...) {
              state->close(request.execution());
              throw;
            }
            return plugin::v1::ExecutionsInitializeResponse{};
          }));
    }
    // Always Register Cleanup So the Host Can Release Execution Resources.
    value.definition.set_cleanup(id + ".cleanup");
    native_.operation(harmos::Operation<plugin::v1::ExecutionsCleanupRequest, plugin::v1::ExecutionsCleanupResponse>(
        id + ".cleanup", harmos::wire::VISIBILITY_PROTECTED,
        [state, id, plugin_id, cleanup = std::move(value.cleanup)](harmos::Context &transport,
                                                                   plugin::v1::ExecutionsCleanupRequest request) {
          auto ctx = execution_context(transport, request, id, plugin_id, state, "Clean Up Execution");
          try {
            if (cleanup)
              cleanup(ctx);
          } catch (...) {
            state->close(request.execution());
            throw;
          }
          state->close(request.execution());
          return plugin::v1::ExecutionsCleanupResponse{};
        }));
    *declaration.add_templates() = std::move(value.definition);
  }
  *suites_.add_suites() = std::move(declaration);
  return *this;
}
Plugin &Plugin::prompt(Prompt prompt) {
  identifier(prompt.metadata().identifier());
  for (const auto &existing : prompts_.prompts())
    if (existing.metadata().identifier() == prompt.metadata().identifier())
      throw std::invalid_argument("Duplicate Prompt Identifier");
  *prompts_.add_prompts() = std::move(prompt);
  return *this;
}
Plugin &Plugin::context_operations() {
  Context::declare_requirements(native_);
  return *this;
}
harmos::Plugin Plugin::compose() const {
  auto result = native_;
  if (suites_.suites_size())
    result.section(suites_);
  if (prompts_.prompts_size())
    result.section(prompts_);
  if (records_.records_size())
    result.section(records_);
  return result;
}
harmos::wire::Catalog Plugin::catalog() const { return compose().catalog(); }
void Plugin::serve(std::string address) const { compose().serve(std::move(address)); }
} // namespace dokime

/* === Private Functions === */

/* === Entry Point === */
