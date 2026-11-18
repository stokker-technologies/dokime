/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "dokime/context.hpp"

using std::invalid_argument;
using std::lock_guard;
using std::make_shared;
using std::move;
using std::shared_ptr;
using std::string;
using std::unordered_map;
using std::vector;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime::plugin {
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
                 shared_ptr<const detail::Bindings> bindings)
    : transport_(transport), test_case_(move(test_case)), item_(move(item)), resources_(move(resources)), bindings_(move(bindings)) {}

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
} // namespace dokime::plugin

/* === Private Functions === */

/* === Entry Point === */
