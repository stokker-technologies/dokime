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

#include "dokime/declarations.hpp"
#include <chrono>
#include <cmath>
#include <map>
#include <variant>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace dokime {
using harmos::plugin::Lane;

class Plugin;
class Client;
namespace detail {
struct PluginState {
  Plugin *plugin = nullptr;
};
} // namespace detail
} // namespace dokime

namespace dokime::operations {
namespace detail {
google::protobuf::Duration encode_duration(std::chrono::duration<double> duration);
}
enum class Visibility {
  Private = harmos::VISIBILITY_PRIVATE,
  Protected = harmos::VISIBILITY_PROTECTED,
  Public = harmos::VISIBILITY_PUBLIC
};
struct OperationPolicy {
  std::optional<std::chrono::duration<double>> timeout;
  std::uint32_t attempts = 1;
  std::chrono::duration<double> backoff{0};
};
struct SettingValue {
  std::variant<bool, std::int64_t, double, std::string, std::chrono::duration<double>> value;
};
template <class Q, class R> class OperationValue;
template <class Q, class R> class BoundOperation;
class Context {
public:
  template <class T = Plugin> T &plugin() const {
    if (!plugin_ || !plugin_->plugin)
      throw std::invalid_argument("Read Plugin State: Call From an Active Plugin Handler");
    return dynamic_cast<T &>(*plugin_->plugin);
  }
  std::uint32_t attempt() const {
    return transport_.attempt;
  }
  void sleep(std::chrono::duration<double> duration);
  std::chrono::system_clock::time_point now();
  std::string cancelled() {
    return transport_.cancelled();
  }
  std::string secret(const std::string &identifier);
  std::map<std::string, SettingValue> settings();
  template <class Q, class R> BoundOperation<Q, R> operation(const OperationValue<Q, R> &value) {
    return BoundOperation<Q, R>(*this, value);
  }

protected:
  Context(harmos::plugin::Context &transport, std::shared_ptr<dokime::detail::PluginState> plugin)
      : transport_(transport), plugin_(std::move(plugin)) {}
  harmos::plugin::Context transport_;
  std::shared_ptr<dokime::detail::PluginState> plugin_;

private:
  friend class dokime::Plugin;
  template <class Q, class R> friend class OperationValue;
  template <class Q, class R> friend class BoundOperation;
};

template <class Q, class R> class OperationValue {
public:
  OperationValue(std::string identifier, Visibility visibility, std::function<R(Context &, Q)> handler = {})
      : identifier_(std::move(identifier)), visibility_(visibility), handler_(std::move(handler)) {}
  OperationValue &policy(OperationPolicy policy) {
    if (policy.attempts == 0 || (policy.attempts > 1 && (!std::isfinite(policy.backoff.count()) || policy.backoff.count() <= 0)))
      throw std::invalid_argument("Declare Retry Policy: Supply Positive Attempts and Backoff");
    if (policy.timeout && (!std::isfinite(policy.timeout->count()) || policy.timeout->count() <= 0))
      throw std::invalid_argument("Declare Operation Policy: Supply a Positive Finite Timeout");
    policy_ = std::move(policy);
    return *this;
  }
  OperationValue &lane(const Lane &lane) {
    lane_ = lane;
    return *this;
  }
  OperationValue &replay_safe() {
    replay_safe_ = true;
    return *this;
  }

private:
  friend class dokime::Plugin;
  friend class dokime::Client;
  template <class A, class B> friend class BoundOperation;
  std::string identifier_;
  Visibility visibility_;
  std::function<R(Context &, Q)> handler_;
  std::optional<OperationPolicy> policy_;
  bool replay_safe_ = false;
  std::optional<Lane> lane_;
  auto native(std::shared_ptr<dokime::detail::PluginState> plugin) const {
    std::function<decltype(templates::detail::payload_message<R>())(harmos::plugin::Context &,
                                                                    decltype(templates::detail::payload_message<Q>()))>
        handler;
    if (handler_)
      handler = [plugin, run = handler_](harmos::plugin::Context &transport, decltype(templates::detail::payload_message<Q>()) request) {
        Context context(transport, plugin);
        return templates::detail::pack<R>(run(context, templates::detail::unpack<Q>(request)));
      };
    harmos::plugin::Operation<decltype(templates::detail::payload_message<Q>()), decltype(templates::detail::payload_message<R>())>
        operation(identifier_, static_cast<harmos::Visibility>(visibility_), std::move(handler));
    if (policy_) {
      harmos::OperationPolicy policy;
      if (policy_->timeout)
        *policy.mutable_timeout() = detail::encode_duration(*policy_->timeout);
      if (policy_->attempts > 1) {
        policy.mutable_retries()->set_total_attempts(policy_->attempts);
        *policy.mutable_retries()->mutable_initial() = detail::encode_duration(policy_->backoff);
      }
      operation.policy(policy);
    }
    if (lane_)
      operation.lane(*lane_);
    if (replay_safe_)
      operation.replay_safe();
    return operation;
  }
};
template <class Q, class R> class BoundOperation {
public:
  BoundOperation(Context &context, OperationValue<Q, R> operation)
      : invoke_([context, operation](const Q &request, std::optional<OperationPolicy> policy) mutable {
          auto selected = operation;
          if (policy)
            selected.policy(std::move(*policy));
          const auto native = selected.native(context.plugin_);
          const auto result = context.transport_.call(native, templates::detail::pack<Q>(request)).get();
          return templates::detail::unpack<R>(result);
        }) {}
  R operator()(const Q &request, std::optional<OperationPolicy> policy = {}) {
    return invoke_(request, std::move(policy));
  }

private:
  friend class dokime::Client;
  explicit BoundOperation(std::function<R(const Q &, std::optional<OperationPolicy>)> invoke) : invoke_(std::move(invoke)) {}
  std::function<R(const Q &, std::optional<OperationPolicy>)> invoke_;
};
template <class Q, class R> auto operation(std::string identifier, Visibility visibility, std::function<R(Context &, Q)> handler = {}) {
  return OperationValue<Q, R>(std::move(identifier), visibility, std::move(handler));
}
} // namespace dokime::operations

/* === Templates === */

/* === Public Declarations === */
