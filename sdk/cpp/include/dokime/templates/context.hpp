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
#include "dokime/operations.hpp"
#include <mutex>
#include <optional>
#include <typeindex>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Declarations === */

namespace dokime::templates {
class Context;
struct Step;
struct Criterion;
struct Template;

struct HostRoot;
enum class HostOperation;

template <class T> struct Stream : Declaration {
  using Declaration::Declaration;
};
// One Host Grant: Sample i Is Due at start + i * period. Window Names the Evidence Time Selection.
namespace sampling {
struct Window {
  google::protobuf::Timestamp start;
  google::protobuf::Duration period;
  std::uint64_t count = 0;
  std::uint64_t sequence = 0;
};
} // namespace sampling
// Samples One Row per Call; Each Host Window Calls It Once per Sample, in Order.
template <class T> struct Observer {
  Stream<T> stream;
  std::function<T(Context &)> sample;
  google::protobuf::Duration interval;
  std::string interval_setting;
  // A Plugin Duration Setting That Replaces the Host's Window Length for This Stream.
  std::string window_setting;
  template <auto Method> static Observer method(Stream<T> stream, google::protobuf::Duration interval);
};
// Answers a Whole Host Window per Call; C Repeats Every Field of T With One Value per Sample.
template <class T, class C> struct WindowObserver {
  Stream<T> stream;
  std::function<C(Context &, const sampling::Window &)> sample;
  google::protobuf::Duration interval;
  std::string interval_setting;
  std::string window_setting;
  template <auto Method> static WindowObserver method(Stream<T> stream, google::protobuf::Duration interval);
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

class Context : public operations::Context {
  Context(harmos::plugin::Context &transport, TestCase test_case, std::string item, std::shared_ptr<detail::Resources> resources,
          std::shared_ptr<const detail::Bindings> bindings = {}, std::shared_ptr<dokime::detail::PluginState> plugin = {});

public:
  const google::protobuf::Map<std::string, Value> &parameters() const {
    return test_case_.parameters();
  }
  std::optional<Termination> termination() const {
    return termination_;
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
  std::string resolve(const Declaration &declaration) const;
  template <auto Method> std::string resolve() const {
    Declaration declaration(detail::method_name<Method>());
    declaration.key = detail::Member<Method>::key;
    return resolve(declaration);
  }
  void log(std::string text, Severity severity = SEVERITY_INFO);
  void progress(std::uint64_t completed, std::uint64_t total, std::string label = {});
  void traceability(std::string identifier, Value value, std::string label = {}, std::string source = {}, std::string unit = {});
  void document(std::string document, model::Section part, std::string section = {});
  void document(std::string document, Card part, std::string section = {});
  std::string prepare_effect(std::string resource, std::string inverse_operation, const google::protobuf::Message &inverse,
                             std::string description = {});
  void confirm_effect(std::string ticket, std::optional<std::chrono::system_clock::time_point> at = {}, bool restored = false);
  std::string prompt(std::string text, std::vector<std::string> choices = {});
  std::string prompt(const PromptDeclaration &declaration);
  template <class T> std::uint64_t record(const Record<T> &declaration, const std::type_identity_t<T> &value);
  template <class T> class RecordsQuery {
  public:
    RecordsQuery(Context &context, const Record<T> &declaration, std::string step)
        : context_(context), declaration_(declaration), step_(std::move(step)) {}
    T last();
    std::uint64_t count();

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
  template <class T> Statistics statistics(const Stream<T> &stream, std::string field = {});
  template <auto Method, class T> Statistics statistics(const Stream<T> &stream, std::string field = {});

private:
  friend class dokime::Plugin;
  friend struct Step;
  friend struct Criterion;
  friend struct Template;
  template <class T> friend struct Observer;
  template <class T, class C> friend struct WindowObserver;
  template <class T> std::shared_ptr<T> instance() {
    std::lock_guard lock(resources_->mutex);
    auto &instance = resources_->values[typeid(T)];
    if (!instance)
      instance = std::make_shared<T>();
    return std::static_pointer_cast<T>(instance);
  }
  template <class Q, class R>
  harmos::plugin::Call<R> call(const harmos::plugin::Operation<Q, R> &operation, const Q &request,
                               std::optional<harmos::OperationPolicy> policy = {}) {
    return transport_.call(operation, request, std::move(policy));
  }
  template <class Q, class R>
  harmos::plugin::Watching<R> watch(const harmos::plugin::Operation<Q, R> &operation, const Q &request, std::size_t max_item_bytes) {
    return transport_.watch(operation, request, max_item_bytes);
  }

  friend struct HostBackendResource;
  friend struct HostContentResource;
  friend struct HostContentArtifactsResource;
  friend struct HostDocumentsResource;
  friend struct HostEffectsResource;
  friend struct HostEvidenceResource;
  friend struct HostEvidenceLogsResource;
  friend struct HostEvidenceRecordsResource;
  friend struct HostEvidenceTraceabilityResource;
  friend struct HostExecutionsResource;
  friend struct HostExecutionsProgressResource;
  friend struct HostFilesResource;
  friend struct HostFilesCapturesResource;
  friend struct HostFilesUploadsResource;
  friend struct HostInvocationsResource;
  friend struct HostOperationsResource;
  friend struct HostPluginsResource;
  friend struct HostPluginsSettingsResource;
  friend struct HostPromptsResource;
  friend struct HostSecretsResource;
  friend struct HostTimeResource;
  HostRoot host();
  std::optional<Termination> termination_;
  TestCase test_case_;
  std::string item_;
  std::shared_ptr<detail::Resources> resources_;
  std::shared_ptr<const detail::Bindings> bindings_;
  std::string ask(Prompt definition, std::string identity, std::string prompt_type);
};
} // namespace dokime::templates

#include "dokime/_generated/host.hpp"

namespace dokime::templates {
template <class T> template <auto Method> Observer<T> Observer<T>::method(Stream<T> stream, google::protobuf::Duration interval) {
  return {std::move(stream),
          [](Context &context) {
            auto instance = context.instance<decltype(detail::member_owner(Method))>();
            return ((*instance).*Method)(context);
          },
          std::move(interval),
          {},
          {}};
}
template <class T, class C>
template <auto Method>
WindowObserver<T, C> WindowObserver<T, C>::method(Stream<T> stream, google::protobuf::Duration interval) {
  return {std::move(stream),
          [](Context &context, const sampling::Window &window) {
            auto instance = context.instance<decltype(detail::member_owner(Method))>();
            return ((*instance).*Method)(context, window);
          },
          std::move(interval),
          {},
          {}};
}
template <class T> std::uint64_t Context::record(const Record<T> &declaration, const std::type_identity_t<T> &value) {
  const auto kind = resolve(declaration);
  const auto message = detail::pack<T>(value);
  return host().evidence.records.append(execution(), kind, message.SerializeAsString(), transport_.now()).ordinal();
}
template <class T> T Context::RecordsQuery<T>::last() {
  const auto response = context_.host().evidence.records.list(context_.resolve(declaration_), {}, {}, step_, {});
  if (!response.has_last())
    throw std::invalid_argument("Read Last Record: Append a Matching Record Before Reading It");
  return detail::decode<T>(response.last().payload());
}
template <class T> std::uint64_t Context::RecordsQuery<T>::count() {
  return context_.host().evidence.records.list(context_.resolve(declaration_), {}, {}, step_, {}).count();
}
template <class T> Statistics Context::statistics(const Stream<T> &stream, std::string field) {
  return host().evidence.statistics(resolve(stream), {}, {}, {}, field, {}).statistics();
}
template <auto Method, class T> Statistics Context::statistics(const Stream<T> &stream, std::string field) {
  return host().evidence.statistics(resolve(stream), {}, {}, {}, field, resolve<Method>()).statistics();
}
} // namespace dokime::templates
