/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "dokime/templates/context.hpp"

using std::invalid_argument;
using std::lock_guard;
using std::make_shared;
using std::move;
using std::optional;
using std::shared_ptr;
using std::string;
using std::unordered_map;
using std::vector;
using std::chrono::duration_cast;
using std::chrono::nanoseconds;
using std::chrono::system_clock;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime::templates {
shared_ptr<detail::Resources> detail::Executions::open(const string &execution) {
  lock_guard lock(mutex);
  auto &resources = values[execution];
  if (!resources)
    resources = make_shared<Resources>();
  return resources;
}
void detail::Executions::close(const string &execution) {
  lock_guard lock(mutex);
  values.erase(execution);
}
void detail::Executions::clear() {
  unordered_map<string, shared_ptr<Resources>> released;
  {
    lock_guard lock(mutex);
    values.swap(released);
  }
}
Context::Context(harmos::plugin::Context &transport, TestCase test_case, string item, shared_ptr<detail::Resources> resources,
                 shared_ptr<const detail::Bindings> bindings, shared_ptr<dokime::detail::PluginState> plugin)
    : operations::Context(transport, move(plugin)), test_case_(move(test_case)), item_(move(item)), resources_(move(resources)),
      bindings_(move(bindings)) {}

void Context::log(string text, Severity severity) {
  host().evidence.logs.append(execution(), severity, text, transport_.now());
}
void Context::progress(std::uint64_t completed, std::uint64_t total, string label) {
  const auto current = host().executions.progress.get();
  ExecutionProgressValue value;
  value.set_completed(completed);
  value.set_total(total);
  value.set_label(move(label));
  host().executions.progress.set(value, current.revision());
}
void Context::traceability(string identifier, Value value, string label, string source, string unit) {
  HostEvidenceTraceabilityGetOptions options;
  options.entry = identifier;
  const auto current = host().evidence.traceability.get(options);
  TraceabilityEntry entry;
  entry.mutable_metadata()->set_identifier(identifier);
  entry.mutable_metadata()->set_label(move(label));
  *entry.mutable_value() = move(value);
  entry.set_source(move(source));
  entry.set_unit(move(unit));
  host().evidence.traceability.set(entry, current.revision(), identifier);
}
void Context::document(string document, model::Section part, string section) {
  HostDocumentsAppendPartsOptions options;
  options.subsection = move(part);
  host().documents.append_parts(execution(), document, section, move(options));
}
void Context::document(string document, Card part, string section) {
  HostDocumentsAppendPartsOptions options;
  options.card = move(part);
  host().documents.append_parts(execution(), document, section, move(options));
}
string Context::prepare_effect(string resource, string inverse_operation, const google::protobuf::Message &inverse, string description) {
  return host().effects.create(resource, inverse_operation, inverse.SerializeAsString(), description).name();
}
void Context::confirm_effect(string ticket, optional<system_clock::time_point> at, bool restored) {
  const auto current = host().effects.get(ticket);
  auto value = current.value();
  const auto applied = at ? *at : now();
  const auto instant = applied.time_since_epoch();
  const auto elapsed = duration_cast<nanoseconds>(instant).count();
  value.mutable_applied_at()->set_seconds(elapsed / 1000000000);
  value.mutable_applied_at()->set_nanos(static_cast<int>(elapsed % 1000000000));
  value.set_restored(restored);
  host().effects.set(ticket, value, current.revision());
}

string Context::resolve(const Declaration &declaration) const {
  if (!bindings_)
    throw invalid_argument("Resolve Declaration: Call From a Template Operation");
  const auto found = bindings_->identifiers.find(declaration.key);
  if (found == bindings_->identifiers.end())
    throw invalid_argument("Resolve Declaration: Supply a Member of This Template");
  return found->second;
}
string Context::ask(Prompt definition, string identity, string prompt_type) {
  HostPromptsCreateOptions options;
  options.definition = definition;
  const auto response = host().prompts.create(identity, prompt_type, {}, move(options));
  if (!response.accepted())
    throw invalid_argument("Create Prompt: Choose an Accepted Operator Answer");
  for (const auto &action : definition.actions())
    if (action.identifier() == response.action())
      return action.label();
  throw invalid_argument("Read Prompt Answer: Choose One of the Declared Actions");
}
string Context::prompt(string text, vector<string> choices) {
  PromptDeclaration declaration("question", move(text), move(choices));
  if (!bindings_)
    throw invalid_argument("Create Prompt: Call From a Template Operation");
  const auto identity = build_identifier("question", {}, bindings_->template_id);
  declaration.value.mutable_metadata()->set_identifier(identity);
  return ask(move(declaration.value), identity, {});
}
string Context::prompt(const PromptDeclaration &declaration) {
  const auto identity = resolve(declaration);
  return ask(bindings_->prompts.at(declaration.key), identity, identity);
}
} // namespace dokime::templates

/* === Private Functions === */

/* === Entry Point === */
