/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#pragma once

/* === Imports === */

// Standard Library Imports
#include <functional>
#include <map>
#include <memory>
#include <typeindex>

// Third Party Imports

// Local Imports
#include "dokime/configuration.hpp"
#include "dokime/errors.hpp"
#include "dokime/plugin.hpp"
#include "dokime/values.hpp"

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace dokime {
namespace detail {
struct ClientState;
struct CaseState;
struct CycleState;
struct ExecutionState;
struct RegistrationState;
struct ProgressState;
struct EvidenceRows;
std::string invoke_public(const std::shared_ptr<ClientState> &client, const std::string &identifier, const std::string &fingerprint,
                          const std::string &payload);
} // namespace detail
class Client;
class Plugins;
class RegisteredPlugin;
class Cases;
class Case;
class Cycles;
class Cycle;
class Executions;
class Execution;
template <class T> class RecordsQuery;
template <class T> class SamplesQuery;

class ProgressStream {
public:
  ProgressStream(ProgressStream &&) noexcept;
  ProgressStream &operator=(ProgressStream &&) noexcept;
  ProgressStream(const ProgressStream &) = delete;
  ~ProgressStream();
  std::optional<Progress> next();
  void close();

private:
  friend class Execution;
  explicit ProgressStream(std::unique_ptr<detail::ProgressState> state);
  std::unique_ptr<detail::ProgressState> state_;
};
class Case {
public:
  const std::string &identifier() const;
  const std::string &label() const;
  const std::string &suite() const;
  const std::string &template_id() const;
  std::map<std::string, Scalar> parameters();
  Case &refresh();

private:
  friend class Cases;
  friend class Cycle;
  friend class Execution;
  explicit Case(std::shared_ptr<detail::CaseState> state);
  std::shared_ptr<detail::CaseState> state_;
};
class Cases {
public:
  template <class T>
  std::vector<Case> materialize(std::optional<std::string> plugin = {}, std::optional<std::string> suite = {},
                                const std::map<std::string, std::vector<Scalar>> &parameters = {},
                                const std::map<std::string, std::optional<std::string>> &files = {},
                                std::optional<std::string> request_id = {}) {
    return materialize_type(typeid(T), std::move(plugin), std::move(suite), parameters, files, std::move(request_id));
  }
  std::vector<Case> materialize(const std::string &template_id, const std::string &plugin, const std::string &suite,
                                const std::map<std::string, std::vector<Scalar>> &parameters = {},
                                const std::map<std::string, std::optional<std::string>> &files = {},
                                std::optional<std::string> request_id = {});
  Case get(const std::string &identifier, const std::string &cycle = "");

private:
  friend class Client;
  explicit Cases(std::shared_ptr<detail::ClientState> client);
  std::vector<Case> materialize_type(std::type_index type, std::optional<std::string> plugin, std::optional<std::string> suite,
                                     const std::map<std::string, std::vector<Scalar>> &parameters,
                                     const std::map<std::string, std::optional<std::string>> &files, std::optional<std::string> request_id);
  std::shared_ptr<detail::ClientState> client_;
};
class Cycle {
public:
  const std::string &identifier() const;
  const std::string &label() const;
  std::string phase() const;
  Execution execution(const Case &test_case, std::optional<std::string> request_id = {});
  Cycle &refresh();
  void seal(std::optional<std::string> request_id = {});
  std::vector<std::uint8_t> evidence();
  Note note(const std::string &text, std::optional<std::string> request_id = {});
  Artifact artifact(const std::string &name, const std::vector<std::uint8_t> &data, const std::string &mime = "application/octet-stream",
                    std::optional<std::string> request_id = {});

private:
  friend class Client;
  friend class Cycles;
  friend class Execution;
  explicit Cycle(std::shared_ptr<detail::CycleState> state);
  std::shared_ptr<detail::CycleState> state_;
};
class Cycles {
public:
  Cycle get(const std::string &identifier);
  std::vector<Cycle> list(const std::string &text = "", int page_size = 0);

private:
  friend class Client;
  explicit Cycles(std::shared_ptr<detail::ClientState> client);
  std::shared_ptr<detail::ClientState> client_;
};
class Execution {
public:
  const std::string &identifier() const;
  Cycle cycle() const;
  Case test_case() const;
  Execution &refresh();
  void start(std::optional<std::string> request_id = {});
  ProgressStream watch();
  ProgressStream run(std::optional<std::string> request_id = {});
  void stop(std::optional<std::string> request_id = {});
  void cancel(std::optional<std::string> request_id = {});
  Outcome seal(std::optional<std::string> request_id = {});
  std::vector<std::uint8_t> evidence();
  Note note(const std::string &text, std::optional<std::string> request_id = {});
  Artifact artifact(const std::string &name, const std::vector<std::uint8_t> &data, const std::string &mime = "application/octet-stream",
                    std::optional<std::string> request_id = {});
  template <class T> RecordsQuery<T> records(const templates::Record<T> &declaration, const std::string &step = "");
  template <class T> RecordsQuery<T> records(const std::string &record_type, const std::string &step = "");
  template <class T>
  SamplesQuery<T> samples(const templates::Stream<T> &stream, const std::vector<std::string> &fields = {},
                          std::pair<std::optional<std::chrono::sys_time<std::chrono::nanoseconds>>,
                                    std::optional<std::chrono::sys_time<std::chrono::nanoseconds>>>
                              between = {});
  template <class T>
  SamplesQuery<T> samples(const std::string &stream, const std::vector<std::string> &fields = {},
                          std::pair<std::optional<std::chrono::sys_time<std::chrono::nanoseconds>>,
                                    std::optional<std::chrono::sys_time<std::chrono::nanoseconds>>>
                              between = {});

private:
  friend class Cycle;
  friend class Executions;
  template <class T> friend class RecordsQuery;
  template <class T> friend class SamplesQuery;
  explicit Execution(std::shared_ptr<detail::ExecutionState> state);
  std::shared_ptr<detail::ExecutionState> state_;
};
class Executions {
public:
  Execution get(const std::string &identifier);
  std::vector<Execution> list(const std::string &cycle = "", const std::string &template_id = "", const std::string &state = "",
                              int page_size = 0);

private:
  friend class Client;
  explicit Executions(std::shared_ptr<detail::ClientState> client);
  std::shared_ptr<detail::ClientState> client_;
};
class RegisteredPlugin {
public:
  const std::string &identifier() const;
  std::string label() const;
  std::string version() const;
  std::string state() const;
  RegisteredPlugin &refresh();
  void unregister();

private:
  friend class Plugins;
  explicit RegisteredPlugin(std::shared_ptr<detail::RegistrationState> state);
  std::shared_ptr<detail::RegistrationState> state_;
};
class Plugins {
public:
  RegisteredPlugin get(const std::string &identifier);

private:
  friend class Client;
  explicit Plugins(std::shared_ptr<detail::ClientState> client);
  std::shared_ptr<detail::ClientState> client_;
};
class Client {
public:
  Client(Client &&) noexcept;
  Client &operator=(Client &&) noexcept;
  Client(const Client &) = delete;
  ~Client();
  Plugins plugins() const;
  Cases cases() const;
  Cycles cycles() const;
  Executions executions() const;
  Cycle cycle(const std::string &label, std::optional<std::string> request_id = {});
  template <class Q, class R> operations::BoundOperation<Q, R> operation(const operations::OperationValue<Q, R> &value) {
    harmos::plugin::Plugin description("contract", "0.0.0");
    description.operation(value.native({}));
    const auto contract = description.declarations().operations(0);
    const auto client = state_;
    const auto configured_policy = value.policy_.has_value();
    return operations::BoundOperation<Q, R>(
        [client, contract, configured_policy](const Q &request, std::optional<operations::OperationPolicy> policy) {
          if (policy || configured_policy)
            throw Error(Failure{"Refused",
                                "host_capability_required",
                                "The Host Has No Client Invocation Policy Contract",
                                "Publish the Product Invocation Policy Contract and Regenerate Bindings",
                                "invocations.create",
                                {},
                                {},
                                {},
                                {}});
          const auto message = templates::detail::pack<Q>(request);
          const auto result = detail::invoke_public(client, contract.id(), contract.fingerprint(), message.SerializeAsString());
          return templates::detail::decode<R>(result);
        });
  }
  void on_prompt(std::function<PromptAnswer(const PromptQuestion &)> handler);
  void close();

private:
  friend Client connect(ConnectionSettings);
  explicit Client(std::shared_ptr<detail::ClientState> state);
  std::shared_ptr<detail::ClientState> state_;
};
Client connect(ConnectionSettings settings = {});
} // namespace dokime

/* === Templates === */

/* === Public Declarations === */
