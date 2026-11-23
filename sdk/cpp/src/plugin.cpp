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
#include <cctype>
#include <google/protobuf/empty.pb.h>
#include <google/protobuf/util/message_differencer.h>
#include <limits>
#include <regex>
#include <set>
#include <sstream>

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
using google::protobuf::Empty;
using std::all_of;
using std::find_if;
using std::function;
using std::isalpha;
using std::istringstream;
using std::isupper;
using std::optional;
using std::regex;
using std::regex_replace;
using std::replace;
using std::size_t;
using std::to_string;
using std::tolower;
using std::toupper;
using std::transform;
using std::vector;

using google::protobuf::util::MessageDifferencer;
using harmos::Declarations;
using harmos::OperationError;
using harmos::OperationPolicy;
using harmos::VISIBILITY_PROTECTED;
using harmos::plugin::Contribution;
using harmos::plugin::Operation;

using std::invalid_argument;
using std::lock_guard;
using std::make_shared;
using std::move;
using std::numeric_limits;
using std::set;
using std::shared_ptr;
using std::string;
using std::uint32_t;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime::templates {

Metadata metadata(string identifier, string label) {
  Metadata value;
  value.set_identifier(move(identifier));
  value.set_label(move(label));
  return value;
}
string build_identifier(string name, optional<string> explicit_identifier, string prefix, bool class_name) {
  string local;
  if (explicit_identifier) {
    local = *explicit_identifier;
  } else {
    if (class_name) {
      for (const auto &suffix : {string("TestTemplate"), string("Plugin"), string("Suite")})
        if (name.ends_with(suffix)) {
          name.resize(name.size() - suffix.size());
          break;
        }
    }
    name = regex_replace(name, regex("([A-Z]+)([A-Z][a-z])"), "$1_$2");
    name = regex_replace(name, regex("([a-z0-9])([A-Z])"), "$1_$2");
    for (const auto character : name)
      local += character >= 'A' && character <= 'Z' ? static_cast<char>(character + ('a' - 'A')) : character;
  }
  if (local.empty() || local.find_first_not_of(" \t\r\n") == string::npos)
    throw invalid_argument("Build Plugin: Supply a Nonempty Declaration Identifier");
  return prefix.empty() ? local : prefix + "." + local;
}
string build_label(string name) {
  for (const auto &suffix : {string("TestTemplate"), string("Plugin"), string("Suite")}) {
    if (!name.ends_with(suffix))
      continue;
    name.resize(name.size() - suffix.size());
    break;
  }
  name = regex_replace(name, regex("([A-Z]+)([A-Z][a-z])"), "$1 $2");
  name = regex_replace(name, regex("([a-z0-9])([A-Z])"), "$1 $2");
  replace(name.begin(), name.end(), '_', ' ');
  istringstream input(name);
  vector<string> words;
  for (string word; input >> word;)
    words.push_back(move(word));
  const set<string> minor{"a", "an", "the", "and", "but", "or", "nor", "as", "at", "by", "for", "in", "of", "on", "per", "to", "via"};
  string result;
  for (size_t index = 0; index < words.size(); ++index) {
    auto word = words[index];
    const bool uppercase = all_of(word.begin(), word.end(), [](unsigned char value) {
      return !isalpha(value) || isupper(value);
    });
    if (!uppercase) {
      transform(word.begin(), word.end(), word.begin(), [](unsigned char value) {
        return static_cast<char>(tolower(value));
      });
      if (index == 0 || index + 1 == words.size() || !minor.contains(word))
        word[0] = static_cast<char>(toupper(static_cast<unsigned char>(word[0])));
    }
    if (!result.empty())
      result += ' ';
    result += word;
  }
  return result;
}
PromptDeclaration::PromptDeclaration(string name, string text, vector<string> choices, optional<string> identifier)
    : Declaration(move(name), text, move(identifier)) {
  value.set_message(text);
  value.set_archetype(PROMPT_ARCHETYPE_CHOICE);
  value.mutable_metadata()->set_label(text);
  if (choices.empty())
    choices.push_back("OK");
  set<string> labels;
  size_t index = 0;
  for (const auto &choice : choices) {
    if (!labels.insert(choice).second)
      throw invalid_argument("Declare Prompt: Supply Distinct Action Labels");
    auto *action = value.add_actions();
    action->set_identifier(to_string(index++));
    action->set_label(choice);
  }
}
PromptDeclaration::PromptDeclaration(string name, Prompt value, optional<string> identifier)
    : Declaration(move(name), value.metadata().label(), identifier), value(move(value)) {
  if (!identifier && !this->value.metadata().identifier().empty())
    this->identifier = this->value.metadata().identifier();
}
Case::Case(shared_ptr<const detail::Bindings> bindings, const vector<Step> &steps, const vector<Criterion> &criteria)
    : bindings_(move(bindings)), steps_(steps), criteria_(criteria) {}
Case &Case::step(const Step &step, string label, optional<string> identifier) {
  const auto declared = find_if(steps_.begin(), steps_.end(), [&](const Step &value) {
    return value.key == step.key;
  });
  if (declared == steps_.end())
    throw invalid_argument("Materialize Step: Declare the Handler on This Template Before Adding It");
  const auto operation = bindings_->identifiers.at(step.key);
  const auto identity = identifier ? build_identifier(step.name, identifier, bindings_->template_id) : operation;
  for (const auto &existing : message.steps())
    if (existing.metadata().identifier() == identity)
      throw invalid_argument("Materialize Step: Supply Unique Local Instance Identifiers");
  auto *value = message.add_steps();
  *value->mutable_metadata() = metadata(identity, move(label));
  value->set_operation(operation);
  value->set_repeatable(declared->repeatable);
  return *this;
}
Case &Case::criterion(const Criterion &criterion, string label, optional<string> identifier) {
  const auto declared = find_if(criteria_.begin(), criteria_.end(), [&](const Criterion &value) {
    return value.key == criterion.key;
  });
  if (declared == criteria_.end())
    throw invalid_argument("Materialize Criterion: Declare the Handler on This Template Before Adding It");
  const auto operation = bindings_->identifiers.at(criterion.key);
  const auto identity = identifier ? build_identifier(criterion.name, identifier, bindings_->template_id) : operation;
  for (const auto &existing : message.criteria())
    if (existing.metadata().identifier() == identity)
      throw invalid_argument("Materialize Criterion: Supply Unique Local Instance Identifiers");
  auto *value = message.add_criteria();
  *value->mutable_metadata() = metadata(identity, move(label));
  value->set_operation(operation);
  return *this;
}
} // namespace dokime::templates
namespace dokime {
using templates::build_identifier;
using templates::Declaration;
using templates::HostOperation;
using templates::metadata;
using templates::Template;

template <class Q>
templates::Context Plugin::execution_context(harmos::plugin::Context &transport, const Q &request, const string &template_id,
                                             const shared_ptr<templates::detail::Executions> &executions,
                                             shared_ptr<const templates::detail::Bindings> bindings, const string &action, string item,
                                             shared_ptr<dokime::detail::PluginState> plugin) {
  const auto &origin = request.origin();
  const bool same_case = MessageDifferencer::Equals(request.test_case(), origin.plugin_run().test_case());
  const bool has_entry = !origin.plugin_run().plugin().empty();
  const bool same_template = origin.plugin_run().template_() == template_id;
  if (request.execution().empty() || transport.scope() != request.execution() || !origin.has_plugin_run() || !same_case || !same_template ||
      !has_entry)
    throw invalid_argument(action + ": Supply Matching Execution Scope, Template and Frozen Origin");
  auto resources = executions->open(request.execution());
  {
    lock_guard lock(resources->mutex);
    resources->test_case = request.test_case();
  }
  return templates::Context(transport, request.test_case(), move(item), move(resources), move(bindings), move(plugin));
}
Plugin::Plugin(string identifier, string version, vector<Suite> suites)
    : identifier_(identifier), version_(version), plugin_(make_shared<dokime::detail::PluginState>()),
      native_(move(identifier), move(version)), executions_(make_shared<templates::detail::Executions>()) {
  plugin_->plugin = this;
  context_operations();
  for (auto &suite : suites)
    this->suite(move(suite));
}
Plugin &Plugin::suite(Suite suite) {
  const auto suite_id = build_identifier(suite.name, suite.identifier, {}, true);
  for (const auto &existing : suites_.suites())
    if (existing.metadata().identifier() == suite_id)
      throw invalid_argument("Build Plugin: Supply Unique Suite Identifiers");

  // Validate Every Declaration Before Registering Operations.
  set<string> templates;
  vector<shared_ptr<templates::detail::Bindings>> bound;
  for (const auto &value : suite.templates) {
    const auto id = build_identifier(value.name, value.identifier, suite_id, true);
    if (!templates.insert(id).second)
      throw invalid_argument("Build Plugin: Supply Unique Template Identifiers");
    for (const auto &existing : suites_.suites())
      for (const auto &previous : existing.templates())
        if (previous.metadata().identifier() == id)
          throw invalid_argument("Build Plugin: Supply Unique Template Identifiers");
    auto bindings = make_shared<templates::detail::Bindings>();
    bindings->template_id = id;
    set<string> names;
    for (const auto &name : {"materialize", "initialize", "cleanup"})
      names.insert(build_identifier(name, {}, id));
    auto bind = [&](const Declaration &member, const string &owner) {
      const auto identity = build_identifier(member.name, member.identifier, owner);
      if (!names.insert(identity).second || !bindings->identifiers.emplace(member.key, identity).second)
        throw invalid_argument("Build Plugin: Give Every Template Member a Unique Declaration and Identifier: " + identity);
    };
    for (const auto &step : value.steps) {
      bind(step, id);
      if (!step.run || step.retries == numeric_limits<uint32_t>::max())
        throw invalid_argument("Declare Step: Supply a Handler and a Smaller Retry Count");
    }
    for (const auto &criterion : value.criteria) {
      bind(criterion, id);
      if (!criterion.evaluate)
        throw invalid_argument("Declare Criterion: Supply a Handler");
    }
    for (const auto &record : value.records)
      bind(record.declaration, id);
    // Streams Belong to the Plugin; an Identifier Already Under the Plugin Stays as Written.
    for (const auto &stream : value.streams) {
      const auto &identifier = stream.declaration.identifier;
      bind(stream.declaration, identifier && identifier->starts_with(identifier_ + ".") ? string() : identifier_);
    }
    for (const auto &prompt : value.prompts)
      bind(prompt, id);
    if (value.type != typeid(void))
      bindings_[value.type].emplace_back(suite_id, id);
    bound.push_back(move(bindings));
  }
  Registrar registrar(native_);
  dokime::detail::payloads::Suite declaration;
  *declaration.mutable_metadata() = metadata(suite_id, suite.label.empty() ? templates::build_label(suite.name) : suite.label);
  size_t index = 0;
  for (auto &value : suite.templates) {
    const auto bindings = bound[index++];
    const auto id = bindings->template_id;
    TestCaseTemplate definition;
    *definition.mutable_metadata() = metadata(id, value.label.empty() ? templates::build_label(value.name) : value.label);
    definition.mutable_parameters()->Assign(value.parameters.begin(), value.parameters.end());
    definition.mutable_documents()->Assign(value.documents.begin(), value.documents.end());
    definition.mutable_files()->Assign(value.files.begin(), value.files.end());
    for (auto &record : value.records) {
      const auto identity = bindings->identifiers.at(record.declaration.key);
      *record.contract.mutable_metadata() = metadata(identity, record.declaration.label.empty() ? identity : record.declaration.label);
      *records_.add_records() = move(record.contract);
      record.message(registrar);
    }
    for (auto &prompt : value.prompts) {
      const auto identity = bindings->identifiers.at(prompt.key);
      prompt.value.mutable_metadata()->set_identifier(identity);
      prompt.value.set_owner_suite(suite_id);
      prompt.value.set_template_(id);
      bindings->prompts.emplace(prompt.key, prompt.value);
      *prompts_.add_prompts() = move(prompt.value);
    }
    const auto state = executions_;
    const auto plugin = plugin_;
    TestCase test_case;
    for (auto &step : value.steps) {
      const auto operation_id = bindings->identifiers.at(step.key);
      auto *declared = test_case.add_steps();
      *declared->mutable_metadata() = metadata(operation_id, step.label);
      declared->set_operation(operation_id);
      declared->set_repeatable(step.repeatable);
      const auto item = operation_id;
      Operation<StepsExecuteRequest, StepsExecuteResponse> operation(
          operation_id, VISIBILITY_PROTECTED,
          [state, plugin, bindings, id, item, run = step.run](harmos::plugin::Context &transport, StepsExecuteRequest request) {
            bool matches = false;
            for (const auto &step : request.test_case().steps())
              matches = matches || (step.metadata().identifier() == request.step() && step.operation() == item);
            if (!matches)
              throw invalid_argument("Execute Step: Supply the Item Bound to This Operation");
            auto ctx = execution_context(transport, request, id, state, bindings, "Execute Step", request.step(), plugin);
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
      registrar.add(operation);
    }
    for (auto &criterion : value.criteria) {
      const auto operation_id = bindings->identifiers.at(criterion.key);
      auto *declared = test_case.add_criteria();
      *declared->mutable_metadata() = metadata(operation_id, criterion.label);
      declared->set_operation(operation_id);
      const auto item = operation_id;
      registrar.add(Operation<CriteriaEvaluateRequest, CriteriaEvaluateResponse>(
          operation_id, VISIBILITY_PROTECTED,
          [state, plugin, bindings, id, item, evaluate = criterion.evaluate](harmos::plugin::Context &transport,
                                                                             CriteriaEvaluateRequest request) {
            bool matches = false;
            for (const auto &criterion : request.test_case().criteria())
              matches = matches || (criterion.metadata().identifier() == request.criterion() && criterion.operation() == item);
            if (!matches)
              throw invalid_argument("Evaluate Criterion: Supply the Item Bound to This Operation");
            auto ctx = execution_context(transport, request, id, state, bindings, "Evaluate Criterion", request.criterion(), plugin);
            CriteriaEvaluateResponse response;
            response.set_passed(evaluate(ctx));
            return response;
          }));
    }
    for (const auto &stream : value.streams) {
      test_case.add_streams(bindings->identifiers.at(stream.declaration.key));
      stream.register_(registrar, bindings, state, plugin);
    }
    definition.set_materialize(build_identifier("materialize", {}, id));
    registrar.add(Operation<TestCasesMaterializeRequest, TestCasesMaterializeResponse>(
        build_identifier("materialize", {}, id), VISIBILITY_PROTECTED,
        [id, test_case, bindings, steps = value.steps, criteria = value.criteria,
         materialize = move(value.materialize)](harmos::plugin::Context &transport, TestCasesMaterializeRequest request) {
          try {
            transport.scope();
            throw invalid_argument("Materialize Test Case: Omit the Execution Scope");
          } catch (const OperationError &failure) {
            if (failure.value.code() != "scope_required")
              throw;
          }
          if (request.template_() != id)
            throw invalid_argument("Materialize Test Case: Supply the Bound Template Without an Execution Scope");
          auto result = test_case;
          *result.mutable_parameters() = request.parameters();
          if (materialize) {
            templates::Case custom(bindings, steps, criteria);
            *custom.message.mutable_parameters() = request.parameters();
            *custom.message.mutable_streams() = result.streams();
            materialize(custom, request.parameters());
            result = move(custom.message);
          }
          TestCasesMaterializeResponse response;
          *response.mutable_test_case() = move(result);
          return response;
        }));
    if (value.initialize) {
      definition.set_initialize(build_identifier("initialize", {}, id));
      registrar.add(Operation<ExecutionsInitializeRequest, ExecutionsInitializeResponse>(
          build_identifier("initialize", {}, id), VISIBILITY_PROTECTED,
          [state, plugin, bindings, id, initialize = move(value.initialize)](harmos::plugin::Context &transport,
                                                                             ExecutionsInitializeRequest request) {
            auto ctx = execution_context(transport, request, id, state, bindings, "Initialize Execution", {}, plugin);
            initialize(ctx);
            return ExecutionsInitializeResponse{};
          }));
    }
    // Always Register Cleanup So the Host Can Release Execution Resources.
    definition.set_cleanup(build_identifier("cleanup", {}, id));
    registrar.add(Operation<ExecutionsCleanupRequest, ExecutionsCleanupResponse>(
        build_identifier("cleanup", {}, id), VISIBILITY_PROTECTED,
        [state, plugin, bindings, id, cleanup = move(value.cleanup)](harmos::plugin::Context &transport, ExecutionsCleanupRequest request) {
          auto ctx = execution_context(transport, request, id, state, bindings, "Clean Up Execution", {}, plugin);
          ctx.termination_ = request.termination();
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
    *definition.mutable_steps() = test_case.steps();
    *declaration.add_templates() = move(definition);
  }
  *suites_.add_suites() = move(declaration);
  return *this;
}
Plugin &Plugin::context_operations() {
  Registrar registrar(native_);
  registrar.add(HostOperation::BackendEnvironment);
  registrar.add(HostOperation::ContentArtifactsCreate);
  registrar.add(HostOperation::FilesUploadsCreate);
  registrar.add(HostOperation::FilesUploadsAppend);
  registrar.add(HostOperation::FilesUploadsPublish);
  registrar.add(HostOperation::FilesUploadsDelete);
  registrar.add(HostOperation::FilesCapture);
  registrar.add(HostOperation::FilesCapturesStart);
  registrar.add(HostOperation::FilesCapturesStop);
  registrar.add(HostOperation::PromptsCreate);
  registrar.add(HostOperation::PluginsSettingsList);
  registrar.add(HostOperation::SecretsGet);
  registrar.add(HostOperation::EvidenceRecordsAppend);
  registrar.add(HostOperation::EvidenceRecordsList);
  registrar.add(HostOperation::EvidenceStatistics);
  registrar.add(HostOperation::ExecutionsProgressGet);
  registrar.add(HostOperation::ExecutionsProgressSet);
  registrar.add(HostOperation::EvidenceLogsAppend);
  registrar.add(HostOperation::EvidenceTraceabilityGet);
  registrar.add(HostOperation::EvidenceTraceabilitySet);
  registrar.add(HostOperation::DocumentsAppendParts);
  registrar.add(HostOperation::TimeGet);
  registrar.add(HostOperation::TimeSleep);
  return *this;
}
harmos::plugin::Plugin Plugin::compose() const {
  auto result = native_;
  Registrar registrar(result);
  registrar.add(Operation<Empty, Empty>(identifier_ + ".initialize", VISIBILITY_PROTECTED,
                                        [plugin = plugin_](harmos::plugin::Context &transport, const Empty &) {
                                          operations::Context context(transport, plugin);
                                          try {
                                            plugin->activate(context);
                                            context.plugin().initialize(context);
                                          } catch (...) {
                                            plugin->close();
                                            throw;
                                          }
                                          return Empty{};
                                        }));
  registrar.add(Operation<Empty, Empty>(identifier_ + ".finalize", VISIBILITY_PROTECTED,
                                        [plugin = plugin_, executions = executions_](harmos::plugin::Context &transport, const Empty &) {
                                          operations::Context context(transport, plugin);
                                          try {
                                            context.plugin().finalize(context);
                                          } catch (...) {
                                            executions->clear();
                                            plugin->close();
                                            throw;
                                          }
                                          executions->clear();
                                          plugin->close();
                                          return Empty{};
                                        }));
  if (suites_.suites_size())
    registrar.add(Contribution{suites_});
  if (prompts_.prompts_size())
    registrar.add(Contribution{prompts_});
  if (records_.records_size())
    registrar.add(Contribution{records_});
  if (secrets_.secrets_size())
    registrar.add(Contribution{secrets_});
  return result;
}
Plugin &Plugin::secrets(SecretsDeclarations secrets) {
  secrets_ = move(secrets);
  return *this;
}
Declarations Plugin::declarations() const {
  return compose().declarations();
}
PluginDescription Plugin::describe() const {
  PluginDescription description;
  description.identifier = identifier_;
  description.version = version_;
  for (const auto &suite : suites_.suites()) {
    SuiteDescription described;
    described.identifier = suite.metadata().identifier();
    described.label = suite.metadata().label();
    for (const auto &value : suite.templates()) {
      TemplateDescription item;
      item.identifier = value.metadata().identifier();
      item.label = value.metadata().label();
      for (const auto &step : value.steps())
        item.steps.push_back({step.metadata().identifier(), step.metadata().label()});
      described.templates.push_back(move(item));
    }
    description.suites.push_back(move(described));
  }
  for (const auto &record : records_.records())
    description.records.push_back({{record.metadata().identifier(), record.metadata().label()}, record.message()});
  description.prompts.assign(prompts_.prompts().begin(), prompts_.prompts().end());
  const auto catalog = declarations();
  description.lanes.assign(catalog.lanes().begin(), catalog.lanes().end());
  for (const auto &operation : catalog.operations())
    description.operations.push_back({operation.id(), operation.request(), operation.response(),
                                      static_cast<operations::Visibility>(operation.visibility()), operation.lane()});
  return description;
}
Plugin::Plugin(Plugin &&other) noexcept
    : identifier_(move(other.identifier_)), version_(move(other.version_)), plugin_(move(other.plugin_)), bindings_(move(other.bindings_)),
      native_(move(other.native_)), suites_(move(other.suites_)), prompts_(move(other.prompts_)), records_(move(other.records_)),
      secrets_(move(other.secrets_)), executions_(move(other.executions_)) {
  plugin_->plugin = this;
}
} // namespace dokime

/* === Private Functions === */

/* === Entry Point === */
