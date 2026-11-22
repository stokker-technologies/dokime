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
#include <atomic>
#include <chrono>
#include <cmath>
#include <functional>
#include <map>
#include <memory>
#include <mutex>
#include <type_traits>
#include <typeindex>
#include <vector>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace dokime::templates {
enum class HostOperation;
}

namespace dokime::operations {
class Context;
class Resource {
public:
  virtual ~Resource() = default;
  virtual void open(Context &) {}
  virtual void close() {}
};

class Settings {
public:
  Settings() = default;
  explicit Settings(harmos::SettingValues values) : values_(std::move(values)) {}
  virtual ~Settings() = default;
  const std::vector<harmos::SettingDeclaration> &declarations() const {
    return declarations_;
  }

protected:
  template <class T>
  T setting(std::string name, std::string title, T initial, std::optional<T> minimum = {}, std::optional<T> maximum = {},
            std::string description = {}) {
    harmos::SettingDeclaration declaration;
    declaration.set_name(name);
    declaration.set_title(std::move(title));
    declaration.set_description(std::move(description));
    *declaration.mutable_default_() = encode(initial);
    if (minimum)
      *declaration.mutable_minimum() = encode(*minimum);
    if (maximum)
      *declaration.mutable_maximum() = encode(*maximum);
    declarations_.push_back(std::move(declaration));
    if (!values_)
      return initial;
    const auto saved = values_->values().find(name);
    if (saved == values_->values().end())
      throw std::invalid_argument("Read Settings: Configure " + name + ", Then Retry the Plugin");
    const auto &value = saved->second;
    if (value.value_case() != encode(initial).value_case())
      throw std::invalid_argument("Read Settings: Save the Declared Type for " + name + ", Then Retry the Plugin");
    if constexpr (std::is_same_v<T, bool>)
      return value.flag();
    else if constexpr (std::is_same_v<T, std::int64_t>)
      return value.integer();
    else if constexpr (std::is_same_v<T, double>)
      return value.number();
    else if constexpr (std::is_same_v<T, std::string>)
      return value.text();
    else
      return std::chrono::duration<double>(value.duration().seconds()) + std::chrono::nanoseconds(value.duration().nanos());
  }

private:
  std::optional<harmos::SettingValues> values_;
  std::vector<harmos::SettingDeclaration> declarations_;
  static harmos::SettingValue encode(bool value) {
    harmos::SettingValue result;
    result.set_flag(value);
    return result;
  }
  static harmos::SettingValue encode(std::int64_t value) {
    harmos::SettingValue result;
    result.set_integer(value);
    return result;
  }
  static harmos::SettingValue encode(double value) {
    harmos::SettingValue result;
    result.set_number(value);
    return result;
  }
  static harmos::SettingValue encode(const std::string &value) {
    harmos::SettingValue result;
    result.set_text(value);
    return result;
  }
  static harmos::SettingValue encode(std::chrono::duration<double> value);
};
} // namespace dokime::operations

namespace dokime {
using harmos::json;
using harmos::plugin::Lane;

class Plugin;
class Client;
class Registrar {
public:
  Registrar &add(templates::HostOperation operation);
  template <class T> Registrar &add(const T &value) {
    native_.add([&value](harmos::plugin::Registrar &registrar) {
      registrar.add(value);
    });
    return *this;
  }

private:
  friend class Plugin;
  explicit Registrar(harmos::plugin::Plugin &native) : native_(native) {}
  harmos::plugin::Plugin &native_;
};
namespace detail {
struct ResourceEntry {
  std::mutex gate;
  std::shared_ptr<operations::Resource> value;
};
struct PluginState {
  Plugin *plugin = nullptr;
  std::atomic_bool closed = false;
  std::map<std::type_index, std::shared_ptr<ResourceEntry>> resources;
  std::mutex opened_gate;
  std::vector<std::shared_ptr<operations::Resource>> opened;
  std::map<std::type_index, std::function<std::shared_ptr<const operations::Settings>(const harmos::SettingValues &)>> settings;
  std::map<std::type_index, std::shared_ptr<const operations::Settings>> snapshots;
  void activate(operations::Context &context);
  void close();
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
  template <class T> std::shared_ptr<T> resource() {
    static_assert(std::is_base_of_v<Resource, T>, "Declare Resource: Derive From Resource");
    const auto declared = plugin_->resources.find(typeid(T));
    if (declared == plugin_->resources.end())
      throw std::invalid_argument("Open Resource: Declare the Resource Class on This Plugin");
    const auto &entry = declared->second;
    std::lock_guard lock(entry->gate);
    if (plugin_->closed)
      throw std::invalid_argument("Open Resource: Restart the Deactivated Plugin, Then Retry");
    if (!entry->value) {
      auto value = std::make_shared<T>();
      value->open(*this);
      entry->value = value;
      std::lock_guard opened_lock(plugin_->opened_gate);
      plugin_->opened.push_back(std::move(value));
    }
    return std::static_pointer_cast<T>(entry->value);
  }
  template <class T> const T &settings() const {
    static_assert(std::is_base_of_v<Settings, T>, "Declare Settings: Derive From Settings");
    const auto snapshot = plugin_->snapshots.find(typeid(T));
    if (snapshot == plugin_->snapshots.end())
      throw std::invalid_argument("Read Settings: Declare the Settings Class and Activate the Plugin");
    return *std::static_pointer_cast<const T>(snapshot->second);
  }
  google::protobuf::Struct operation(const std::string &identifier, const google::protobuf::Struct &payload = {}) {
    return transport_.operation(identifier, payload);
  }
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
  friend struct dokime::detail::PluginState;
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
          const auto result = context.transport_.operation(native, templates::detail::pack<Q>(request)).get();
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
