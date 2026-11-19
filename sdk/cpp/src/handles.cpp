/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
#include <algorithm>
#include <set>

// Third Party Imports
#include <openssl/evp.h>

// Local Imports
#include "dokime/client.hpp"
#include "dokime/detail/client.hpp"
#include "dokime/detail/transfers.hpp"

using std::invalid_argument;
using std::make_shared;
using std::move;
using std::optional;
using std::runtime_error;
using std::shared_ptr;
using std::string;
using std::vector;
using std::chrono::duration_cast;
using std::chrono::nanoseconds;
using std::chrono::seconds;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

namespace dokime::detail {
string invoke_public(const shared_ptr<ClientState> &client, const string &identifier, const string &fingerprint, const string &payload) {
  client::OperationsGetOptions lookup;
  lookup.context = client->context();
  const auto operation = client->transport->operations.get(identifier, lookup);
  if (operation.contract().fingerprint() != fingerprint || operation.contract().visibility() != harmos::VISIBILITY_PUBLIC)
    throw invalid_argument("Invoke Operation: Select a Public Operation Matching the Declared Contract");
  client::InvocationsCreateOptions options;
  options.context = client->context();
  return client->transport->invocations.create(identifier, fingerprint, payload, options);
}
vector<std::uint8_t> archive(const shared_ptr<ClientState> &client, const client::EvidenceSubject &subject) {
  client::EvidenceArchiveOptions options;
  options.context = client->context();
  const auto result = client->transport->evidence.archive(subject, options);
  if (!result.has_file())
    throw runtime_error("Download Evidence: Seal the Subject and Read Its Retained Archive");
  return download(client, result.file());
}
Note create_note(const shared_ptr<ClientState> &client, const string &cycle, const string &execution, const string &text,
                 optional<string> request_id) {
  client::AnnotationOwner owner;
  if (execution.empty())
    owner.mutable_owner()->set_cycle_id(cycle);
  else {
    owner.mutable_owner()->mutable_execution()->set_cycle_id(cycle);
    owner.mutable_owner()->mutable_execution()->set_execution_id(execution);
  }
  client::NoteWritable writable;
  writable.set_message(text);
  client::EvidenceNotesCreateOptions options;
  options.request_id = mutation_identity(move(request_id));
  options.context = client->context();
  const auto result = client->transport->evidence.notes.create(owner, writable, options);
  const auto &value = result.note();
  Note note{value.note_id(), value.message(), {}, instant(value.created_at()), value.scope().cycle_id(), {}};
  if (value.has_author())
    note.author = value.author();
  if (!value.scope().execution_id().empty())
    note.execution = value.scope().execution_id();
  return note;
}
shared_ptr<grpc::ClientContext> ClientState::context() const {
  std::lock_guard lock(contexts_mutex);
  if (closed) {
    Failure failure;
    failure.classification = "Cancelled";
    failure.code = "client_closed";
    failure.message = "The Client Is Closed";
    failure.action = "Connect a New Client";
    throw Error(move(failure));
  }
  auto context = make_shared<grpc::ClientContext>();
  if (!access_key.empty())
    context->AddMetadata("authorization", "Bearer " + access_key);
  std::erase_if(contexts, [](const auto &context) {
    return context.expired();
  });
  contexts.push_back(context);
  return context;
}
string mutation_identity(optional<string> request_id) {
  if (request_id && request_id->empty())
    throw invalid_argument("Mutate Resource: Supply a Nonempty Request Identity");
  return request_id ? *request_id : harmos::detail::request_id();
}
std::chrono::sys_time<nanoseconds> instant(const google::protobuf::Timestamp &value) {
  return std::chrono::sys_time<nanoseconds>(seconds(value.seconds()) + nanoseconds(value.nanos()));
}
google::protobuf::Timestamp timestamp(std::chrono::sys_time<nanoseconds> value) {
  const auto elapsed = duration_cast<nanoseconds>(value.time_since_epoch()).count();
  auto whole = elapsed / 1000000000;
  auto fraction = elapsed % 1000000000;
  if (fraction < 0) {
    --whole;
    fraction += 1000000000;
  }
  google::protobuf::Timestamp result;
  result.set_seconds(whole);
  result.set_nanos(static_cast<int>(fraction));
  return result;
}
Scalar decode_scalar(const Value &value) {
  if (value.has_null())
    return {};
  if (value.has_boolean())
    return Scalar{value.boolean()};
  if (value.has_integer())
    return Scalar{value.integer()};
  if (value.has_unsigned_integer())
    return Scalar{value.unsigned_integer()};
  if (value.has_number())
    return Scalar{value.number()};
  if (value.has_text())
    return Scalar{value.text()};
  if (value.has_data())
    return Scalar{vector<std::uint8_t>(value.data().begin(), value.data().end())};
  if (value.has_exact())
    return Scalar{Scalar::Exact{value.exact().numerator(), value.exact().denominator(), value.exact().negative()}};
  if (value.has_datetime())
    return Scalar{instant(value.datetime())};
  if (value.has_list()) {
    vector<Scalar> values;
    for (const auto &item : value.list().values())
      values.push_back(decode_scalar(item));
    return Scalar{move(values)};
  }
  throw runtime_error("Decode Scalar: Read a Supported Host Value");
}
Value encode_scalar(const Scalar &value) {
  Value result;
  std::visit(
      [&](const auto &item) {
        if constexpr (std::is_same_v<std::decay_t<decltype(item)>, std::monostate>)
          result.mutable_null();
        else if constexpr (std::is_same_v<std::decay_t<decltype(item)>, bool>)
          result.set_boolean(item);
        else if constexpr (std::is_same_v<std::decay_t<decltype(item)>, std::int64_t>)
          result.set_integer(item);
        else if constexpr (std::is_same_v<std::decay_t<decltype(item)>, std::uint64_t>)
          result.set_unsigned_integer(item);
        else if constexpr (std::is_same_v<std::decay_t<decltype(item)>, double>)
          result.set_number(item);
        else if constexpr (std::is_same_v<std::decay_t<decltype(item)>, string>)
          result.set_text(item);
        else if constexpr (std::is_same_v<std::decay_t<decltype(item)>, vector<std::uint8_t>>)
          result.set_data(string(item.begin(), item.end()));
        else if constexpr (std::is_same_v<std::decay_t<decltype(item)>, Scalar::Exact>) {
          result.mutable_exact()->set_numerator(item.numerator);
          result.mutable_exact()->set_denominator(item.denominator);
          result.mutable_exact()->set_negative(item.negative);
        } else if constexpr (std::is_same_v<std::decay_t<decltype(item)>, std::chrono::sys_time<nanoseconds>>) {
          *result.mutable_datetime() = timestamp(item);
        } else {
          for (const auto &element : item)
            *result.mutable_list()->add_values() = encode_scalar(element);
        }
      },
      value.value);
  return result;
}
shared_ptr<ExecutionState> ExecutionState::bind(shared_ptr<ClientState> client, client::ExecutionDetail detail,
                                                shared_ptr<CycleState> cycle, shared_ptr<CaseState> test_case) {
  const auto &summary = detail.summary();
  if (summary.execution_id().empty() || detail.cycle_id().empty() || summary.case_id().empty())
    throw runtime_error("Bind Execution: Read a Complete Host Execution Binding");
  if ((cycle && cycle->summary.cycle_id() != detail.cycle_id()) || (test_case && test_case->identifier != summary.case_id()))
    throw runtime_error("Bind Execution: Read an Execution From the Requested Cycle and Case");
  if (!cycle) {
    cycle = make_shared<CycleState>();
    cycle->client = client;
    cycle->summary.set_cycle_id(detail.cycle_id());
  }
  if (!test_case) {
    test_case = make_shared<CaseState>();
    test_case->client = client;
    test_case->identifier = summary.case_id();
    test_case->suite = detail.admitted_suite().identifier();
    test_case->template_id = detail.admitted_template().identifier();
    test_case->cycle = detail.cycle_id();
  }
  return make_shared<ExecutionState>(ExecutionState{move(client), move(detail), move(cycle), move(test_case)});
}
void ExecutionState::observe(client::ExecutionDetail observed) {
  if (observed.summary().execution_id() != detail.summary().execution_id() || observed.cycle_id() != cycle->summary.cycle_id() ||
      observed.summary().case_id() != test_case->identifier)
    throw runtime_error("Observe Execution: Read the Original Execution, Cycle and Case Identities");
  detail = move(observed);
}
} // namespace dokime::detail

namespace dokime {
Client::Client(shared_ptr<detail::ClientState> state) : state_(move(state)) {}
Client::Client(Client &&) noexcept = default;
Client &Client::operator=(Client &&other) noexcept {
  if (this != &other) {
    try {
      close();
    } catch (...) {
    }
    state_ = move(other.state_);
  }
  return *this;
}
Client::~Client() {
  try {
    close();
  } catch (...) {
  }
}
void Client::close() {
  if (!state_)
    return;
  std::lock_guard registration_lock(state_->registrations_mutex);
  if (state_->closed)
    return;
  std::exception_ptr cleanup_failure;
  const auto registrations = state_->registrations;
  for (const auto &[identifier, registration] : registrations) {
    try {
      RegisteredPlugin(registration).unregister();
    } catch (...) {
      if (!cleanup_failure)
        cleanup_failure = std::current_exception();
      if (registration->listener) {
        registration->listener->close();
        registration->listener.reset();
      }
    }
  }
  state_->closed = true;
  state_->prompt_worker.request_stop();
  {
    std::lock_guard lock(state_->contexts_mutex);
    for (const auto &context : state_->contexts)
      if (const auto active = context.lock())
        active->TryCancel();
    state_->contexts.clear();
  }
  if (state_->prompt_worker.joinable())
    state_->prompt_worker.join();
  {
    std::lock_guard lock(state_->prompt_mutex);
    state_->prompt_handler = {};
    state_->prompt_context.reset();
  }
  state_->bindings.clear();
  state_->registrations.clear();
  state_->transport.reset();
  state_->channel.reset();
  if (cleanup_failure)
    std::rethrow_exception(cleanup_failure);
  if (state_->prompt_failure)
    std::rethrow_exception(state_->prompt_failure);
}
void Client::on_prompt(std::function<PromptAnswer(const PromptQuestion &)> handler) {
  const auto state = state_;
  state->context();
  {
    std::lock_guard lock(state->prompt_mutex);
    state->prompt_handler = move(handler);
    if (state->prompt_worker.joinable() || !state->prompt_handler)
      return;
    state->prompt_context = state->context();
  }
  state->prompt_worker = std::jthread([state](std::stop_token stop) {
    try {
      client::PromptsWatchOptions options;
      options.context = state->prompt_context;
      auto watch = state->transport->prompts.watch("", true, options);
      std::set<string> answered;
      while (const auto frame = watch.next()) {
        if (frame->has_resync_required())
          throw Error(detail::project_failure(frame->resync_required()));
        for (const auto &view : frame->prompts()) {
          const auto &prompt = view.prompt();
          std::function<PromptAnswer(const PromptQuestion &)> handler;
          {
            std::lock_guard lock(state->prompt_mutex);
            handler = state->prompt_handler;
          }
          if (!handler || view.has_response() || answered.contains(prompt.prompt_id()))
            continue;
          PromptQuestion question{prompt.prompt_id(), prompt.definition().message(), {}, prompt.scope().execution_id()};
          for (const auto &action : prompt.definition().actions())
            question.choices.push_back(action.label());
          const auto answer = handler(question);
          vector<string> actions;
          for (const auto &action : prompt.definition().actions())
            if (action.label() == answer.choice)
              actions.push_back(action.identifier());
          if (actions.size() != 1)
            throw invalid_argument("Answer Prompt: Choose One Supplied Choice Label");
          client::PromptsAnswersCreateOptions responding;
          responding.request_id = detail::mutation_identity({});
          responding.context = state->context();
          state->transport->prompts.answers.create(prompt.prompt_id(), actions.front(), {}, true, responding);
          answered.insert(prompt.prompt_id());
        }
      }
    } catch (const Error &error) {
      if (!stop.stop_requested() || error.failure.classification != "Cancelled")
        state->prompt_failure = std::current_exception();
    } catch (...) {
      state->prompt_failure = std::current_exception();
    }
  });
}
Plugins Client::plugins() const {
  return Plugins(state_);
}
Cases Client::cases() const {
  return Cases(state_);
}
Cycles Client::cycles() const {
  return Cycles(state_);
}
Executions Client::executions() const {
  return Executions(state_);
}
Cycle Client::cycle(const string &label, optional<string> request_id) {
  Metadata metadata;
  metadata.set_label(label);
  client::CyclesCreateOptions options;
  options.request_id = detail::mutation_identity(move(request_id));
  options.context = state_->context();
  const auto result = state_->transport->cycles.create({}, metadata, false, "", options);
  if (!result.has_cycle() || result.cycle().cycle_id().empty())
    throw runtime_error("Create Cycle: Read the Host-Assigned Cycle Identity");
  return Cycle(make_shared<detail::CycleState>(detail::CycleState{state_, result.cycle(), {}}));
}
Cases::Cases(shared_ptr<detail::ClientState> client) : client_(move(client)) {}
Case Cases::get(const string &identifier, const string &cycle) {
  client::CasesGetOptions options;
  options.context = client_->context();
  const auto result = client_->transport->cases.get(identifier, cycle, options);
  const auto &summary = result.value().summary();
  if (summary.case_id() != identifier)
    throw runtime_error("Read Case: Read the Requested Case Identity");
  return Case(make_shared<detail::CaseState>(detail::CaseState{client_, summary.case_id(), summary.metadata().label(), summary.suite_id(),
                                                               summary.template_id(), cycle, result.resolved_at()}));
}
vector<Case> Cases::materialize_type(std::type_index type, optional<string> plugin, optional<string> suite,
                                     const std::map<string, vector<Scalar>> &parameters, const std::map<string, optional<string>> &files,
                                     optional<string> request_id) {
  vector<detail::TemplateBinding> matches;
  if (const auto found = client_->bindings.find(type); found != client_->bindings.end())
    for (const auto &binding : found->second)
      if ((!plugin || *plugin == binding.plugin) && (!suite || *suite == binding.suite))
        matches.push_back(binding);
  if (matches.size() != 1)
    throw invalid_argument("Materialize Cases: Register One Unambiguous Plugin and Suite Binding for This Template");
  const auto &binding = matches.front();
  return materialize(binding.template_id, binding.plugin, binding.suite, parameters, files, move(request_id));
}
vector<Case> Cases::materialize(const string &template_id, const string &plugin, const string &suite,
                                const std::map<string, vector<Scalar>> &parameters, const std::map<string, optional<string>> &files,
                                optional<string> request_id) {
  if (template_id.empty() || plugin.empty() || suite.empty())
    throw invalid_argument("Materialize Cases: Supply Explicit Plugin, Suite and Template Identities");
  client::SelectionPlan plan;
  auto *selection = plan.add_selections();
  selection->set_suite_id(suite);
  selection->set_template_id(template_id);
  for (const auto &[name, values] : parameters) {
    auto *parameter = selection->add_parameters();
    parameter->set_key(name);
    for (const auto &value : values)
      *parameter->mutable_value()->add_values() = detail::encode_scalar(value);
  }
  for (const auto &[name, file] : files) {
    auto *input = selection->add_files();
    input->set_key(name);
    if (file)
      input->mutable_value()->mutable_kind()->set_value(*file);
    else
      input->mutable_value()->mutable_kind()->mutable_clear();
  }
  client::CasesCreateOptions options;
  options.request_id = detail::mutation_identity(move(request_id));
  options.context = client_->context();
  const auto result = client_->transport->cases.create(plan, false, options);
  vector<Case> cases;
  for (const auto &value : result.cases())
    cases.push_back(Case(make_shared<detail::CaseState>(
        detail::CaseState{client_, value.case_id(), value.metadata().label(), value.suite_id(), value.template_id(), {}, {}})));
  return cases;
}
Case::Case(shared_ptr<detail::CaseState> state) : state_(move(state)) {}
const string &Case::identifier() const {
  return state_->identifier;
}
const string &Case::label() const {
  return state_->label;
}
const string &Case::suite() const {
  return state_->suite;
}
const string &Case::template_id() const {
  return state_->template_id;
}
Case &Case::refresh() {
  client::CasesGetOptions options;
  options.context = state_->client->context();
  const auto result = state_->client->transport->cases.get(identifier(), state_->cycle, options);
  const auto &summary = result.value().summary();
  if (summary.case_id() != identifier())
    throw runtime_error("Refresh Case: Read the Original Case Identity");
  state_->label = summary.metadata().label();
  state_->suite = summary.suite_id();
  state_->template_id = summary.template_id();
  state_->at = result.resolved_at();
  return *this;
}
std::map<string, Scalar> Case::parameters() {
  if (!state_->at)
    refresh();
  client::CasesParametersOptions options;
  options.at = state_->at;
  options.context = state_->client->context();
  const auto result = state_->client->transport->cases.parameters(identifier(), state_->cycle, options);
  std::map<string, Scalar> values;
  for (const auto &value : result.parameters())
    values.emplace(value.name(), detail::decode_scalar(value.value()));
  return values;
}
Cycles::Cycles(shared_ptr<detail::ClientState> client) : client_(move(client)) {}
Cycle Cycles::get(const string &identifier) {
  client::CyclesGetOptions options;
  options.context = client_->context();
  const auto result = client_->transport->cycles.get(identifier, options);
  if (result.cycle().cycle_id() != identifier)
    throw runtime_error("Read Cycle: Read the Requested Cycle Identity");
  return Cycle(make_shared<detail::CycleState>(detail::CycleState{client_, result.cycle(), result.revision()}));
}
vector<Cycle> Cycles::list(const string &text, int page_size) {
  if (page_size < 0)
    throw invalid_argument("List Cycles: Supply a Nonnegative Page Size");
  client::CyclesListOptions options;
  options.page.emplace().set_page_size(page_size);
  client::CycleFilter filter;
  filter.set_text(text);
  vector<Cycle> cycles;
  std::set<string> tokens;
  while (true) {
    options.context = client_->context();
    const auto page = client_->transport->cycles.list({}, filter, {}, options);
    if (options.at && options.at->SerializeAsString() != page.resolved_at().SerializeAsString())
      throw runtime_error("List Cycles: Retain the Original Read Cut");
    options.at = page.resolved_at();
    for (const auto &summary : page.cycles())
      cycles.push_back(Cycle(make_shared<detail::CycleState>(detail::CycleState{client_, summary, {}})));
    if (page.next_page_token().empty())
      return cycles;
    if (!tokens.insert(page.next_page_token()).second)
      throw runtime_error("List Cycles: Continue With a New Page Token");
    options.page->set_page_token(page.next_page_token());
  }
}
Cycle::Cycle(shared_ptr<detail::CycleState> state) : state_(move(state)) {}
vector<std::uint8_t> Cycle::evidence() {
  client::EvidenceSubject subject;
  subject.set_cycle(identifier());
  return detail::archive(state_->client, subject);
}
Note Cycle::note(const string &text, optional<string> request_id) {
  return detail::create_note(state_->client, identifier(), "", text, move(request_id));
}
Artifact Cycle::artifact(const string &name, const vector<std::uint8_t> &data, const string &mime, optional<string> request_id) {
  ArtifactScope scope;
  scope.set_cycle_id(identifier());
  return detail::publish_artifact(state_->client, scope, name, data, mime, move(request_id));
}
const string &Cycle::identifier() const {
  return state_->summary.cycle_id();
}
const string &Cycle::label() const {
  return state_->summary.metadata().label();
}
string Cycle::phase() const {
  switch (state_->summary.phase()) {
  case client::CYCLE_PHASE_ACTIVE:
    return "Active";
  case client::CYCLE_PHASE_SEALING:
    return "Sealing";
  case client::CYCLE_PHASE_SEALED:
    return "Sealed";
  case client::CYCLE_PHASE_SEAL_FAILED:
    return "Seal Failed";
  default:
    return "Unspecified";
  }
}
Cycle &Cycle::refresh() {
  client::CyclesGetOptions options;
  options.context = state_->client->context();
  const auto result = state_->client->transport->cycles.get(identifier(), options);
  if (result.cycle().cycle_id() != identifier())
    throw runtime_error("Refresh Cycle: Read the Original Cycle Identity");
  state_->summary = result.cycle();
  state_->revision = result.revision();
  return *this;
}
Execution Cycle::execution(const Case &test_case, optional<string> request_id) {
  if (test_case.state_->client != state_->client)
    throw invalid_argument("Create Execution: Select a Case From This Client");
  const auto identity = detail::mutation_identity(move(request_id));
  client::ExecutionBinding binding;
  binding.set_case_id(test_case.identifier());
  // Match Python's UUIDv5 Under the URL Namespace.
  const unsigned char url_namespace[] = {0x6b, 0xa7, 0xb8, 0x11, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8};
  const string prefix(reinterpret_cast<const char *>(url_namespace), sizeof(url_namespace));
  const auto name = prefix + "dokime/execution/" + identifier() + "/" + identity;
  unsigned char bytes[EVP_MAX_MD_SIZE];
  unsigned int length = 0;
  if (EVP_Digest(name.data(), name.size(), bytes, &length, EVP_sha1(), nullptr) != 1 || length != 20)
    throw runtime_error("Create Execution Identity: Initialize the UUIDv5 Digest");
  static constexpr char digits[] = "0123456789abcdef";
  string execution_id;
  for (int index = 0; index < 16; ++index) {
    if (index == 4 || index == 6 || index == 8 || index == 10)
      execution_id += '-';
    auto byte = static_cast<unsigned char>(bytes[index]);
    if (index == 6)
      byte = (byte & 15) | 80;
    if (index == 8)
      byte = (byte & 63) | 128;
    execution_id += digits[byte >> 4];
    execution_id += digits[byte & 15];
  }
  binding.set_execution_id(execution_id);
  client::ExecutionsCreateOptions options;
  options.request_id = identity;
  options.context = state_->client->context();
  const auto result = state_->client->transport->executions.create(identifier(), {binding}, options);
  if (result.executions_size() != 1)
    throw runtime_error("Create Execution: Read Exactly One Bound Execution");
  return Execution(detail::ExecutionState::bind(state_->client, result.executions(0), state_, test_case.state_));
}
void Cycle::seal(optional<string> request_id) {
  client::CyclesSealOptions options;
  options.request_id = detail::mutation_identity(move(request_id));
  options.context = state_->client->context();
  const auto result = state_->client->transport->cycles.seal(identifier(), options);
  if (!result.seal().empty()) {
    state_->summary.set_phase(client::CYCLE_PHASE_SEALED);
    state_->summary.set_sealed(true);
    return;
  }
  client::CycleSelection selection;
  selection.add_cycle_ids(identifier());
  client::CyclesWatchOptions watching;
  watching.context = state_->client->context();
  auto watch = state_->client->transport->cycles.watch(selection, {}, {}, watching);
  while (const auto frame = watch.next()) {
    if (frame->has_resync_required())
      throw Error(detail::project_failure(frame->resync_required()));
    for (const auto &summary : frame->cycles()) {
      if (summary.cycle_id() != identifier())
        continue;
      state_->summary = summary;
      if (phase() == "Sealed")
        return;
      if (phase() == "Seal Failed")
        throw runtime_error("Seal Cycle: Resolve " + summary.seal_failure() + " and Retry Seal");
    }
  }
  throw runtime_error("Seal Cycle: Reconnect to Observe Seal Settlement");
}
Executions::Executions(shared_ptr<detail::ClientState> client) : client_(move(client)) {}
Execution Executions::get(const string &identifier) {
  client::ExecutionsGetOptions options;
  options.context = client_->context();
  const auto result = client_->transport->executions.get(identifier, options);
  if (result.execution().summary().execution_id() != identifier)
    throw runtime_error("Read Execution: Read the Requested Execution Identity");
  return Execution(detail::ExecutionState::bind(client_, result.execution()));
}
vector<Execution> Executions::list(const string &cycle, const string &template_id, const string &state, int page_size) {
  if (page_size < 0)
    throw invalid_argument("List Executions: Supply a Nonnegative Page Size");
  client::ExecutionSelection selection;
  if (!cycle.empty())
    selection.add_cycle_ids(cycle);
  if (!template_id.empty())
    selection.add_template_ids(template_id);
  if (!state.empty()) {
    if (state == "open" || state == "Open")
      selection.add_statuses(client::EXECUTION_STATE_OPEN);
    else if (state == "closed" || state == "Closed")
      selection.add_statuses(client::EXECUTION_STATE_CLOSED);
    else if (state == "sealed" || state == "Sealed")
      selection.add_statuses(client::EXECUTION_STATE_SEALED);
    else
      throw invalid_argument("List Executions: Select Open, Closed or Sealed");
  }
  client::ExecutionsListOptions options;
  options.page.emplace().set_page_size(page_size);
  vector<Execution> executions;
  std::set<string> tokens;
  while (true) {
    options.context = client_->context();
    const auto page = client_->transport->executions.list(cycle, selection, {}, {}, options);
    if (options.at && options.at->SerializeAsString() != page.resolved_at().SerializeAsString())
      throw runtime_error("List Executions: Retain the Original Read Cut");
    options.at = page.resolved_at();
    for (const auto &detail : page.executions())
      executions.push_back(Execution(detail::ExecutionState::bind(client_, detail)));
    if (page.next_page_token().empty())
      return executions;
    if (!tokens.insert(page.next_page_token()).second)
      throw runtime_error("List Executions: Continue With a New Page Token");
    options.page->set_page_token(page.next_page_token());
  }
}
Execution::Execution(shared_ptr<detail::ExecutionState> state) : state_(move(state)) {}
vector<std::uint8_t> Execution::evidence() {
  client::EvidenceSubject subject;
  subject.set_execution(identifier());
  return detail::archive(state_->client, subject);
}
Note Execution::note(const string &text, optional<string> request_id) {
  return detail::create_note(state_->client, state_->cycle->summary.cycle_id(), identifier(), text, move(request_id));
}
Artifact Execution::artifact(const string &name, const vector<std::uint8_t> &data, const string &mime, optional<string> request_id) {
  ArtifactScope scope;
  scope.set_execution_id(identifier());
  return detail::publish_artifact(state_->client, scope, name, data, mime, move(request_id));
}
const string &Execution::identifier() const {
  return state_->detail.summary().execution_id();
}
Cycle Execution::cycle() const {
  return Cycle(state_->cycle);
}
Case Execution::test_case() const {
  return Case(state_->test_case);
}
Execution &Execution::refresh() {
  client::ExecutionsGetOptions options;
  options.context = state_->client->context();
  state_->observe(state_->client->transport->executions.get(identifier(), options).execution());
  return *this;
}
void Execution::start(optional<string> request_id) {
  client::ExecutionsStartOptions options;
  options.request_id = detail::mutation_identity(move(request_id));
  options.context = state_->client->context();
  state_->observe(state_->client->transport->executions.start(identifier(), client::EXECUTION_MODE_AUTOMATIC, options).execution());
}
void Execution::stop(optional<string> request_id) {
  client::ExecutionsStopOptions options;
  options.request_id = detail::mutation_identity(move(request_id));
  options.context = state_->client->context();
  state_->observe(state_->client->transport->executions.stop(identifier(), options).execution());
}
void Execution::cancel(optional<string> request_id) {
  client::ExecutionsCancelOptions options;
  options.request_id = detail::mutation_identity(move(request_id));
  options.context = state_->client->context();
  state_->observe(state_->client->transport->executions.cancel(identifier(), options).execution());
}
Outcome Execution::seal(optional<string> request_id) {
  client::ExecutionsSealOptions options;
  options.request_id = detail::mutation_identity(move(request_id));
  options.context = state_->client->context();
  const auto result = state_->client->transport->executions.seal(identifier(), options);
  const auto &value = result.outcome();
  if (value.has_not_evaluated())
    return {ResultStatus::Pending, "not_evaluated", value.not_evaluated().reason(), value.not_evaluated().termination(), {}};
  if (value.has_evaluated()) {
    const auto status = static_cast<int>(value.evaluated().status());
    if (status < 0 || status > 5)
      throw runtime_error("Seal Execution: Read a Known Result Status");
    return {static_cast<ResultStatus>(status), "evaluated", value.evaluated().reason(), {}, {}};
  }
  if (value.has_declared()) {
    const auto status = static_cast<int>(value.declared().status());
    if (status < 0 || status > 5)
      throw runtime_error("Seal Execution: Read a Known Result Status");
    return {static_cast<ResultStatus>(status), "declared", value.declared().reason(), {}, value.declared().importer()};
  }
  throw runtime_error("Seal Execution: Read the Host's Stable Outcome");
}
ProgressStream Execution::watch() {
  auto state = std::make_unique<detail::ProgressState>();
  state->execution = state_;
  return ProgressStream(move(state));
}
ProgressStream Execution::run(optional<string> request_id) {
  auto state = std::make_unique<detail::ProgressState>();
  state->execution = state_;
  state->start = detail::mutation_identity(move(request_id));
  return ProgressStream(move(state));
}
ProgressStream::ProgressStream(std::unique_ptr<detail::ProgressState> state) : state_(move(state)) {}
ProgressStream::ProgressStream(ProgressStream &&) noexcept = default;
ProgressStream &ProgressStream::operator=(ProgressStream &&) noexcept = default;
ProgressStream::~ProgressStream() = default;
void ProgressStream::close() {
  if (!state_)
    return;
  state_->watch.reset();
  state_->finished = true;
}
optional<Progress> ProgressStream::next() try {
  if (!state_ || state_->finished)
    return {};
  const auto execution = state_->execution;
  if (state_->start) {
    const auto identity = move(*state_->start);
    state_->start.reset();
    client::ExecutionsStartOptions options;
    options.context = execution->client->context();
    options.request_id = identity;
    execution->observe(execution->client->transport->executions
                           .start(execution->detail.summary().execution_id(), client::EXECUTION_MODE_AUTOMATIC, options)
                           .execution());
  }
  if (!state_->watch) {
    client::ExecutionsProgressWatchOptions options;
    options.context = execution->client->context();
    state_->watch.emplace(execution->client->transport->executions.progress.watch(execution->detail.summary().execution_id(), options));
  }
  const auto frame = state_->watch->next();
  if (!frame) {
    close();
    throw runtime_error("Watch Execution: Reconnect to Observe the Required Sealed State");
  }
  Progress progress{frame->phase(), frame->value().completed(), frame->value().total(), frame->value().label(), frame->sealed()};
  if (progress.sealed)
    close();
  return progress;
} catch (...) {
  close();
  throw;
}
Plugins::Plugins(shared_ptr<detail::ClientState> client) : client_(move(client)) {}
RegisteredPlugin Plugins::register_owned(shared_ptr<Plugin> plugin, optional<string> address, optional<string> request_id) {
  std::lock_guard lock(client_->registrations_mutex);
  client_->context();
  const auto identifier = plugin->identifier();
  if (client_->registrations.contains(identifier))
    throw invalid_argument("Register Plugin: Unregister This Client's Existing Plugin First");
  const auto values = detail::configuration::load();
  const auto selected = detail::configuration::resolve("DOKIME_PLUGIN_ADDRESS", address, values);
  auto registration = make_shared<detail::RegistrationState>();
  registration->client = client_;
  registration->plugin = move(plugin);
  registration->listener.emplace(registration->plugin->compose().start(selected));
  const auto endpoint = "http://" + registration->listener->address();
  try {
    harmos::client::PluginPlacement placement;
    placement.mutable_remote()->set_url(endpoint);
    harmos::client::PluginValue value;
    value.set_enabled(true);
    client::PluginsCreateOptions options;
    options.context = client_->context();
    options.request_id = detail::mutation_identity(move(request_id));
    const auto created = client_->transport->plugins.create(identifier, placement, value, options);
    registration->entry = created.entry();
    if (registration->entry.name() != identifier)
      throw runtime_error("Register Plugin: Read the Requested Plugin Entry Identity");
    client_->registrations.emplace(identifier, registration);
    RegisteredPlugin registered(registration);
    auto ready = [&] {
      if (registration->entry.has_failure())
        throw Error(detail::project_failure(registration->entry.failure()));
      const auto state = registration->entry.state();
      if (state != harmos::client::PLUGIN_STATE_READY && state != harmos::client::PLUGIN_STATE_CONNECTING)
        throw runtime_error("Register Plugin: Enable the Plugin and Resolve Its Activation Failure");
      return state == harmos::client::PLUGIN_STATE_READY;
    };
    if (!ready()) {
      client::PluginsWatchOptions watching;
      watching.context = client_->context();
      watching.name = identifier;
      auto watch = client_->transport->plugins.watch(watching);
      while (const auto frame = watch.next()) {
        if (!frame->resync_required().empty())
          throw runtime_error("Register Plugin: Reconnect After Plugin Resynchronization");
        if (frame->has_snapshot()) {
          for (const auto &entry : frame->snapshot().entries())
            if (entry.name() == identifier)
              registration->entry = entry;
        }
        if (frame->has_change() && frame->change().entry().name() == identifier)
          registration->entry = frame->change().entry();
        if (ready())
          break;
      }
    }
    if (!ready())
      throw runtime_error("Register Plugin: Reconnect; Readiness Observation Ended Before Ready");
    for (const auto &[type, bindings] : registration->plugin->bindings_)
      for (const auto &[suite, template_id] : bindings)
        client_->bindings[type].push_back({identifier, suite, template_id});
    return registered;
  } catch (...) {
    const auto failure = std::current_exception();
    client::PluginsGetOptions lookup;
    lookup.context = client_->context();
    try {
      const auto observed = client_->transport->plugins.get(identifier, lookup);
      if (observed.entry().placement().remote().url() == endpoint) {
        client::PluginsDeleteOptions deleting;
        deleting.context = client_->context();
        client_->transport->plugins.delete_(identifier, observed.entry().revision(), deleting);
      }
    } catch (const Error &error) {
      const auto status = error.failure.details.find("grpc_status");
      if (status == error.failure.details.end() || status->second != "5") {
        registration->listener->close();
        throw;
      }
    }
    registration->listener->close();
    registration->listener.reset();
    client_->registrations.erase(identifier);
    std::rethrow_exception(failure);
  }
}
RegisteredPlugin Plugins::get(const string &identifier) {
  std::lock_guard lock(client_->registrations_mutex);
  if (const auto found = client_->registrations.find(identifier); found != client_->registrations.end()) {
    RegisteredPlugin registered(found->second);
    registered.refresh();
    return registered;
  }
  client::PluginsGetOptions options;
  options.context = client_->context();
  const auto result = client_->transport->plugins.get(identifier, options);
  if (result.entry().name() != identifier)
    throw runtime_error("Read Plugin: Read the Requested Entry Identity");
  return RegisteredPlugin(make_shared<detail::RegistrationState>(detail::RegistrationState{client_, result.entry(), false, {}, {}}));
}
RegisteredPlugin::RegisteredPlugin(shared_ptr<detail::RegistrationState> state) : state_(move(state)) {}
const string &RegisteredPlugin::identifier() const {
  return state_->entry.name();
}
string RegisteredPlugin::label() const {
  return state_->entry.description().id();
}
string RegisteredPlugin::version() const {
  return state_->entry.description().version();
}
string RegisteredPlugin::state() const {
  switch (state_->entry.state()) {
  case harmos::client::PLUGIN_STATE_CONNECTING:
    return "Connecting";
  case harmos::client::PLUGIN_STATE_READY:
    return "Ready";
  case harmos::client::PLUGIN_STATE_STOPPING:
    return "Stopping";
  case harmos::client::PLUGIN_STATE_STOPPED:
    return "Stopped";
  case harmos::client::PLUGIN_STATE_UNAVAILABLE:
    return "Unavailable";
  default:
    return "Unspecified";
  }
}
RegisteredPlugin &RegisteredPlugin::refresh() {
  client::PluginsGetOptions options;
  options.context = state_->client->context();
  const auto result = state_->client->transport->plugins.get(identifier(), options);
  if (result.entry().name() != identifier())
    throw runtime_error("Refresh Plugin: Read the Original Entry Identity");
  state_->entry = result.entry();
  return *this;
}
void RegisteredPlugin::unregister() {
  std::lock_guard lock(state_->client->registrations_mutex);
  if (state_->detached)
    return;
  try {
    refresh();
    if (state_->listener && state_->entry.placement().remote().url() != "http://" + state_->listener->address())
      throw runtime_error("Unregister Plugin: Resolve the Changed Placement Before Removing This Client's Registration");
    client::PluginsDeleteOptions options;
    options.context = state_->client->context();
    state_->client->transport->plugins.delete_(identifier(), state_->entry.revision(), options);
  } catch (const Error &error) {
    const auto status = error.failure.details.find("grpc_status");
    if (status == error.failure.details.end() || status->second != "5")
      throw;
  }
  if (state_->listener) {
    state_->listener->close();
    state_->listener.reset();
  }
  for (auto binding = state_->client->bindings.begin(); binding != state_->client->bindings.end();) {
    std::erase_if(binding->second, [&](const auto &value) {
      return value.plugin == identifier();
    });
    if (binding->second.empty())
      binding = state_->client->bindings.erase(binding);
    else
      ++binding;
  }
  state_->client->registrations.erase(identifier());
  state_->plugin.reset();
  state_->detached = true;
}
} // namespace dokime

/* === Private Functions === */

/* === Entry Point === */
