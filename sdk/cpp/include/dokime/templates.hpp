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

#include "dokime/templates/context.hpp"

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Declarations === */

namespace dokime::templates {
class TestTemplate {};
struct Step : Declaration {
  std::function<void(Context &)> run;
  std::uint32_t retries = 0;
  bool repeatable = false;
  Step(std::string name, std::string label, std::function<void(Context &)> run, std::optional<std::string> identifier = {},
       std::uint32_t retries = 0, bool repeatable = false)
      : Declaration(std::move(name), std::move(label), std::move(identifier)), run(std::move(run)), retries(retries),
        repeatable(repeatable) {}
  template <auto Method>
  static Step method(std::string label, std::optional<std::string> identifier = {}, std::uint32_t retries = 0, bool repeatable = false) {
    Step value(
        detail::method_name<Method>(), std::move(label),
        [](Context &context) {
          auto instance = context.instance<decltype(detail::member_owner(Method))>();
          ((*instance).*Method)(context);
        },
        std::move(identifier), retries, repeatable);
    value.key = detail::Member<Method>::key;
    return value;
  }
};
struct Criterion : Declaration {
  std::function<bool(Context &)> evaluate;
  Criterion(std::string name, std::string label, std::function<bool(Context &)> evaluate, std::optional<std::string> identifier = {})
      : Declaration(std::move(name), std::move(label), std::move(identifier)), evaluate(std::move(evaluate)) {}
  template <auto Method> static Criterion method(std::string label, std::optional<std::string> identifier = {}) {
    Criterion value(
        detail::method_name<Method>(), std::move(label),
        [](Context &context) {
          auto instance = context.instance<decltype(detail::member_owner(Method))>();
          return ((*instance).*Method)(context);
        },
        std::move(identifier));
    value.key = detail::Member<Method>::key;
    return value;
  }
};
class Case {
public:
  Case(std::shared_ptr<const detail::Bindings> bindings, const std::vector<Step> &steps, const std::vector<Criterion> &criteria);
  Case &step(const Step &step, std::string label, std::optional<std::string> identifier = {});
  Case &criterion(const Criterion &criterion, std::string label, std::optional<std::string> identifier = {});
  TestCase message;

private:
  std::shared_ptr<const detail::Bindings> bindings_;
  std::vector<Step> steps_;
  std::vector<Criterion> criteria_;
};
struct Template : Declaration {
  std::type_index type{typeid(void)};
  std::vector<Parameter> parameters;
  std::vector<Document> documents;
  std::vector<FileParameter> files;
  std::vector<Step> steps;
  std::vector<Criterion> criteria;
  std::vector<PromptDeclaration> prompts;
  std::function<void(Context &)> initialize;
  std::function<void(Context &)> cleanup;
  std::function<void(Case &, const google::protobuf::Map<std::string, Value> &)> materialize;
  Template(std::string name, std::string label = {}, std::optional<std::string> identifier = {})
      : Declaration(std::move(name), std::move(label), std::move(identifier)) {}
  template <class T> static Template from(std::string label = {}, std::optional<std::string> identifier = {}) {
    static_assert(std::is_base_of_v<TestTemplate, T>, "Declare Template: Derive the Author Class From TestTemplate");
    Template value(detail::type_name<T>(), std::move(label), std::move(identifier));
    value.type = typeid(T);
    if constexpr (requires(T &instance, Context &context) { instance.initialize(context); })
      value.initialize = [](Context &context) {
        context.instance<T>()->initialize(context);
      };
    if constexpr (requires(T &instance, Context &context) { instance.cleanup(context); })
      value.cleanup = [](Context &context) {
        context.instance<T>()->cleanup(context);
      };
    return value;
  }
  Template &step(Step value) {
    steps.push_back(std::move(value));
    return *this;
  }
  Template &criterion(Criterion value) {
    criteria.push_back(std::move(value));
    return *this;
  }
  template <class T> Template &record(Record<T> record) {
    RecordContract contract;
    contract.set_message(decltype(detail::payload_message<T>())::descriptor()->full_name());
    records.push_back({std::move(record), std::move(contract), [](Registrar &registrar) {
                         registrar.add(harmos::plugin::Descriptor<decltype(detail::payload_message<T>())>{});
                       }});
    return *this;
  }
  template <class T> Template &observer(Observer<T> observer) {
    auto stream = observer.stream;
    streams.push_back({stream, [observer = std::move(observer)](Registrar &registrar, std::shared_ptr<const detail::Bindings> bindings,
                                                                std::shared_ptr<detail::Executions> executions,
                                                                std::shared_ptr<dokime::detail::PluginState> plugin) {
                         harmos::plugin::Stream<decltype(detail::payload_message<T>())> stream(
                             bindings->identifiers.at(observer.stream.key));
                         stream.description(observer.stream.label);
                         registrar.add(stream);
                         harmos::plugin::Observer<decltype(detail::payload_message<T>())> value(
                             stream, [executions, bindings, plugin, sample = observer.sample](harmos::plugin::Context &transport) {
                               auto resources = executions->open(transport.scope());
                               TestCase test_case;
                               {
                                 std::lock_guard lock(resources->mutex);
                                 test_case = resources->test_case;
                               }
                               Context context(transport, std::move(test_case), {}, resources, bindings, plugin);
                               return detail::pack<T>(sample(context));
                             });
                         value.every(observer.interval);
                         if (!observer.interval_setting.empty())
                           value.interval_setting(observer.interval_setting);
                         if (!observer.window_setting.empty())
                           value.window_setting(observer.window_setting);
                         registrar.add(value);
                       }});
    return *this;
  }
  template <class T, class C> Template &observer(WindowObserver<T, C> observer) {
    auto stream = observer.stream;
    streams.push_back({stream, [observer = std::move(observer)](Registrar &registrar, std::shared_ptr<const detail::Bindings> bindings,
                                                                std::shared_ptr<detail::Executions> executions,
                                                                std::shared_ptr<dokime::detail::PluginState> plugin) {
                         harmos::plugin::Stream<decltype(detail::payload_message<T>())> stream(
                             bindings->identifiers.at(observer.stream.key));
                         stream.description(observer.stream.label);
                         registrar.add(stream);
                         auto value = harmos::plugin::Observer<decltype(detail::payload_message<T>())>::template window<C>(
                             stream, [executions, bindings, plugin, sample = observer.sample](harmos::plugin::Context &transport,
                                                                                              const harmos::plugin::Window &window) {
                               auto resources = executions->open(transport.scope());
                               TestCase test_case;
                               {
                                 std::lock_guard lock(resources->mutex);
                                 test_case = resources->test_case;
                               }
                               Context context(transport, std::move(test_case), {}, resources, bindings, plugin);
                               const sampling::Window selected{window.start, window.period, window.count, window.sequence};
                               return sample(context, selected);
                             });
                         value.every(observer.interval);
                         if (!observer.interval_setting.empty())
                           value.interval_setting(observer.interval_setting);
                         if (!observer.window_setting.empty())
                           value.window_setting(observer.window_setting);
                         registrar.add(value);
                       }});
    return *this;
  }

private:
  friend class dokime::Plugin;
  struct RecordDefinition {
    Declaration declaration;
    RecordContract contract;
    std::function<void(Registrar &)> message;
  };
  struct StreamDefinition {
    Declaration declaration;
    std::function<void(Registrar &, std::shared_ptr<const detail::Bindings>, std::shared_ptr<detail::Executions>,
                       std::shared_ptr<dokime::detail::PluginState>)>
        register_;
  };
  std::vector<RecordDefinition> records;
  std::vector<StreamDefinition> streams;
};
} // namespace dokime::templates
