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

#include "dokime/messages.hpp"
#include "harmos/harmos.hpp"
#include <mutex>
#include <optional>
#include <typeindex>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Declarations === */

namespace dokime::plugin {
class Context;
struct HostRoot;
enum class HostOperation;
using Parameters = google::protobuf::Map<std::string, Value>;
template <class T> using Stream = harmos::plugin::Stream<T>;
// One Host Grant: Sample i Is Due at start + i * period. Window Names the Evidence Time Selection.
using SampleWindow = harmos::plugin::Window;
// Samples One Row per Call; Each Host Window Calls It Once per Sample, in Order.
template <class T> struct Observer {
  Stream<T> stream;
  std::function<T(Context &)> sample;
  google::protobuf::Duration interval;
  std::string interval_setting;
  // A Plugin Duration Setting That Replaces the Host's Window Length for This Stream.
  std::string window_setting;
};
// Answers a Whole Host Window per Call; C Repeats Every Field of T With One Value per Sample.
template <class T, class C> struct WindowObserver {
  Stream<T> stream;
  std::function<C(Context &, const SampleWindow &)> sample;
  google::protobuf::Duration interval;
  std::string interval_setting;
  std::string window_setting;
};

namespace detail {
struct Resources {
  std::mutex mutex;
  TestCase test_case;
  std::unordered_map<std::type_index, std::shared_ptr<void>> values;
};
struct Executions {
  std::mutex mutex;
  std::unordered_map<std::string, std::shared_ptr<Resources>> values;
  std::shared_ptr<Resources> open(const std::string &execution);
  void close(const std::string &execution);
  void clear();
};
} // namespace detail

class Context {
public:
  Context(harmos::plugin::Context &transport, TestCase test_case, std::string item, std::shared_ptr<detail::Resources> resources);
  const Parameters &parameters() const {
    return test_case_.parameters();
  }
  const std::string &execution() const {
    return transport_.scope();
  }
  const std::string &item() const {
    return item_;
  }
  std::uint32_t attempt() const {
    return transport_.attempt;
  }
  template <class T> void set(std::shared_ptr<T> value) {
    std::lock_guard lock(resources_->mutex);
    resources_->values[typeid(T)] = std::move(value);
  }
  template <class T> std::shared_ptr<T> get() const {
    std::lock_guard lock(resources_->mutex);
    auto found = resources_->values.find(typeid(T));
    if (found == resources_->values.end())
      throw std::invalid_argument("Execution Resource Is Missing");
    return std::static_pointer_cast<T>(found->second);
  }
  void sleep(google::protobuf::Duration duration) {
    transport_.sleep(duration);
  }
  google::protobuf::Timestamp now() {
    return transport_.now();
  }
  HostRoot host();
  template <class Q, class R>
  harmos::plugin::Call<R> call(const harmos::plugin::Operation<Q, R> &operation, const Q &request,
                               std::optional<harmos::OperationPolicy> policy = {}) {
    return transport_.call(operation, request, std::move(policy));
  }
  template <class Q, class R>
  harmos::plugin::Watching<R> watch(const harmos::plugin::Operation<Q, R> &operation, const Q &request, std::size_t max_item_bytes) {
    return transport_.watch(operation, request, max_item_bytes);
  }

private:
  harmos::plugin::Context &transport_;
  TestCase test_case_;
  std::string item_;
  std::shared_ptr<detail::Resources> resources_;
};
} // namespace dokime::plugin

#include "dokime/_generated/host.hpp"
