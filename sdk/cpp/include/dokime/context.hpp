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
template <class Q, class R> class OperationValue;
struct HostRoot;
enum class HostOperation;

template <class T> struct Stream : Declaration {
  using Declaration::Declaration;
};
// One Host Grant: Sample i Is Due at start + i * period. Window Names the Evidence Time Selection.
namespace sampling {
using harmos::plugin::Window;
}
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
  std::function<C(Context &, const sampling::Window &)> sample;
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
  Context(harmos::plugin::Context &transport, TestCase test_case, std::string item, std::shared_ptr<detail::Resources> resources,
          std::shared_ptr<const detail::Bindings> bindings = {});
  const google::protobuf::Map<std::string, Value> &parameters() const {
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
      throw std::invalid_argument("Read Execution Resource: Initialize the Resource Before Reading It");
    return std::static_pointer_cast<T>(found->second);
  }
  void sleep(google::protobuf::Duration duration) {
    transport_.sleep(duration);
  }
  google::protobuf::Timestamp now() {
    return transport_.now();
  }
  template <class Q, class R>
  R operation(const OperationValue<Q, R> &operation, const std::type_identity_t<Q> &request,
              std::optional<harmos::OperationPolicy> policy = {}) {
    const auto native = operation.native(std::make_shared<detail::Executions>());
    const auto result = transport_.call(native, detail::pack<Q>(request), std::move(policy)).get();
    return detail::unpack<R>(result);
  }
  HostRoot host();
  std::string resolve(const Declaration &declaration) const;
  template <auto Method> std::string resolve() const {
    Declaration declaration(detail::method_name<Method>());
    declaration.key = detail::Member<Method>::key;
    return resolve(declaration);
  }
  template <class T> std::shared_ptr<T> instance() {
    std::lock_guard lock(resources_->mutex);
    auto &instance = resources_->values[typeid(T)];
    if (!instance)
      instance = std::make_shared<T>();
    return std::static_pointer_cast<T>(instance);
  }
  std::string prompt(std::string text, std::vector<std::string> choices = {});
  std::string prompt(const PromptDeclaration &declaration);
  template <class T> RecordsAppendResponse record(const Record<T> &declaration, const std::type_identity_t<T> &value);
  template <class T> class RecordsQuery {
  public:
    RecordsQuery(Context &context, const Record<T> &declaration, std::string step)
        : context_(context), declaration_(declaration), step_(std::move(step)) {}
    T last();

  private:
    Context &context_;
    Record<T> declaration_;
    std::string step_;
  };
  template <class T> RecordsQuery<T> records(const Record<T> &declaration) {
    resolve(declaration);
    return RecordsQuery<T>(*this, declaration, {});
  }
  template <auto Method, class T> RecordsQuery<T> records(const Record<T> &declaration) {
    resolve(declaration);
    return RecordsQuery<T>(*this, declaration, resolve<Method>());
  }
  template <class T> EvidenceStatisticsResponse statistics(const Stream<T> &stream, std::string field = {});
  template <auto Method, class T> EvidenceStatisticsResponse statistics(const Stream<T> &stream, std::string field = {});

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
  std::shared_ptr<const detail::Bindings> bindings_;
  std::string ask(Prompt definition, std::string identity, std::string prompt_type);
};
} // namespace dokime::plugin

#include "dokime/_generated/host.hpp"

namespace dokime::plugin {
template <class T> RecordsAppendResponse Context::record(const Record<T> &declaration, const std::type_identity_t<T> &value) {
  const auto kind = resolve(declaration);
  const auto message = detail::pack<T>(value);
  return host().evidence.records.append(execution(), kind, message.SerializeAsString(), now());
}
template <class T> T Context::RecordsQuery<T>::last() {
  const auto response = context_.host().evidence.records.list(context_.resolve(declaration_), {}, {}, step_, {});
  if (!response.has_last())
    throw std::invalid_argument("Read Last Record: Append a Matching Record Before Reading It");
  return detail::decode<T>(response.last().payload());
}
template <class T> EvidenceStatisticsResponse Context::statistics(const Stream<T> &stream, std::string field) {
  return host().evidence.statistics(resolve(stream), {}, {}, {}, field, {});
}
template <auto Method, class T> EvidenceStatisticsResponse Context::statistics(const Stream<T> &stream, std::string field) {
  return host().evidence.statistics(resolve(stream), {}, {}, {}, field, resolve<Method>());
}
} // namespace dokime::plugin
