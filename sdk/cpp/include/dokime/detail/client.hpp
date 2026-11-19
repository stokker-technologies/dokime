/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#pragma once

/* === Imports === */

// Standard Library Imports
#include <atomic>
#include <mutex>
#include <set>
#include <string_view>
#include <thread>

// Third Party Imports

// Local Imports
#include "dokime/_generated/client.hpp"
#include "dokime/handles.hpp"

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace dokime::detail {
class FrozenSchemaCache;
Failure project_failure(const harmos::model::Failure &value);
struct TemplateBinding {
  std::string plugin;
  std::string suite;
  std::string template_id;
};
struct RegistrationState;
struct ClientState {
  explicit ClientState(std::shared_ptr<grpc::Channel> channel) : channel(std::move(channel)), transport(std::in_place, this->channel) {}
  std::shared_ptr<grpc::Channel> channel;
  std::optional<Transport> transport;
  std::string access_key;
  std::atomic<bool> closed = false;
  mutable std::mutex contexts_mutex;
  mutable std::vector<std::weak_ptr<grpc::ClientContext>> contexts;
  std::map<std::type_index, std::vector<TemplateBinding>> bindings;
  std::shared_ptr<FrozenSchemaCache> schemas;
  std::recursive_mutex registrations_mutex;
  std::map<std::string, std::shared_ptr<RegistrationState>> registrations;
  std::mutex prompt_mutex;
  std::function<PromptAnswer(const PromptQuestion &)> prompt_handler;
  std::shared_ptr<grpc::ClientContext> prompt_context;
  std::jthread prompt_worker;
  std::exception_ptr prompt_failure;
  std::shared_ptr<grpc::ClientContext> context() const;
};
struct CaseState {
  std::shared_ptr<ClientState> client;
  std::string identifier;
  std::string label;
  std::string suite;
  std::string template_id;
  std::string cycle;
  std::optional<harmos::ReadCut> at;
};
struct CycleState {
  std::shared_ptr<ClientState> client;
  client::CycleSummary summary;
  std::string revision;
};
struct ExecutionState {
  std::shared_ptr<ClientState> client;
  client::ExecutionDetail detail;
  std::shared_ptr<CycleState> cycle;
  std::shared_ptr<CaseState> test_case;
  static std::shared_ptr<ExecutionState> bind(std::shared_ptr<ClientState> client, client::ExecutionDetail detail,
                                              std::shared_ptr<CycleState> cycle = {}, std::shared_ptr<CaseState> test_case = {});
  void observe(client::ExecutionDetail detail);
};
struct RegistrationState {
  std::shared_ptr<ClientState> client;
  client::host::PluginEntry entry;
  bool detached = false;
  std::shared_ptr<Plugin> plugin;
  std::optional<harmos::plugin::Listener> listener;
};
struct ProgressState {
  std::shared_ptr<ExecutionState> execution;
  std::optional<std::string> start;
  std::optional<harmos::ClientStream<client::ProgressWatchResponse>> watch;
  bool finished = false;
};
std::string mutation_identity(std::optional<std::string> request_id);
std::chrono::sys_time<std::chrono::nanoseconds> instant(const google::protobuf::Timestamp &value);
google::protobuf::Timestamp timestamp(std::chrono::sys_time<std::chrono::nanoseconds> value);
Scalar decode_scalar(const Value &value);
Value encode_scalar(const Scalar &value);
void validate_schema(const std::shared_ptr<ExecutionState> &execution, const std::string &identity, std::string_view message);
} // namespace dokime::detail

/* === Templates === */

/* === Public Declarations === */
