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

#include "dokime/context.hpp"

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Declarations === */

namespace dokime::plugin {
Metadata metadata(std::string identifier, std::string label);

enum class Visibility {
  Private = harmos::VISIBILITY_PRIVATE,
  Protected = harmos::VISIBILITY_PROTECTED,
  Public = harmos::VISIBILITY_PUBLIC
};

template <class Q, class R> class OperationValue {
public:
  OperationValue(std::string identifier, Visibility visibility, std::function<R(Context &, Q)> handler = {})
      : identifier_(std::move(identifier)), visibility_(visibility), handler_(std::move(handler)) {}
  OperationValue &policy(harmos::OperationPolicy policy) {
    policy_ = std::move(policy);
    return *this;
  }
  OperationValue &replay_safe() {
    replay_safe_ = true;
    return *this;
  }

private:
  friend class Plugin;
  friend class Context;
  std::string identifier_;
  Visibility visibility_;
  std::function<R(Context &, Q)> handler_;
  std::optional<harmos::OperationPolicy> policy_;
  bool replay_safe_ = false;
  auto native(std::shared_ptr<detail::Executions> executions) const {
    std::function<decltype(detail::payload_message<R>())(harmos::plugin::Context &, decltype(detail::payload_message<Q>()))> handler;
    if (handler_)
      handler = [executions, run = handler_](harmos::plugin::Context &transport, decltype(detail::payload_message<Q>()) request) {
        std::shared_ptr<detail::Resources> resources;
        try {
          resources = executions->open(transport.scope());
        } catch (const harmos::Failure &failure) {
          if (failure.value.code() != "scope_required")
            throw;
          resources = std::make_shared<detail::Resources>();
        }
        Context context(transport, {}, {}, resources);
        return detail::pack<R>(run(context, detail::unpack<Q>(request)));
      };
    harmos::plugin::Operation<decltype(detail::payload_message<Q>()), decltype(detail::payload_message<R>())> operation(
        identifier_, static_cast<harmos::Visibility>(visibility_), std::move(handler));
    if (policy_)
      operation.policy(*policy_);
    if (replay_safe_)
      operation.replay_safe();
    return operation;
  }
};

template <class Q, class R> auto operation(std::string identifier, Visibility visibility, std::function<R(Context &, Q)> handler = {}) {
  return OperationValue<Q, R>(std::move(identifier), visibility, std::move(handler));
}

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
    Template value(detail::type_name<T>(), std::move(label), std::move(identifier));
    T::declare(value);
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
  template <class T> Template &record(Record<T> record) {
    RecordContract contract;
    contract.set_message(decltype(detail::payload_message<T>())::descriptor()->full_name());
    records.push_back({std::move(record), std::move(contract), [](harmos::plugin::Plugin &native) {
                         native.message<decltype(detail::payload_message<T>())>();
                       }});
    return *this;
  }
  template <class T> Template &observer(Observer<T> observer) {
    auto stream = observer.stream;
    streams.push_back(
        {stream, [observer = std::move(observer)](harmos::plugin::Plugin &native, std::shared_ptr<const detail::Bindings> bindings,
                                                  std::shared_ptr<detail::Executions> executions) {
           harmos::plugin::Stream<decltype(detail::payload_message<T>())> stream(bindings->identifiers.at(observer.stream.key));
           stream.description(observer.stream.label);
           native.stream(stream);
           harmos::plugin::Observer<decltype(detail::payload_message<T>())> value(
               stream, [executions, bindings, sample = observer.sample](harmos::plugin::Context &transport) {
                 auto resources = executions->open(transport.scope());
                 TestCase test_case;
                 {
                   std::lock_guard lock(resources->mutex);
                   test_case = resources->test_case;
                 }
                 Context context(transport, std::move(test_case), {}, resources, bindings);
                 return detail::pack<T>(sample(context));
               });
           value.every(observer.interval);
           if (!observer.interval_setting.empty())
             value.interval_setting(observer.interval_setting);
           if (!observer.window_setting.empty())
             value.window_setting(observer.window_setting);
           native.observer(value);
         }});
    return *this;
  }
  template <class T, class C> Template &observer(WindowObserver<T, C> observer) {
    auto stream = observer.stream;
    streams.push_back(
        {stream, [observer = std::move(observer)](harmos::plugin::Plugin &native, std::shared_ptr<const detail::Bindings> bindings,
                                                  std::shared_ptr<detail::Executions> executions) {
           harmos::plugin::Stream<decltype(detail::payload_message<T>())> stream(bindings->identifiers.at(observer.stream.key));
           stream.description(observer.stream.label);
           native.stream(stream);
           auto value = harmos::plugin::Observer<decltype(detail::payload_message<T>())>::template window<C>(
               stream,
               [executions, bindings, sample = observer.sample](harmos::plugin::Context &transport, const sampling::Window &window) {
                 auto resources = executions->open(transport.scope());
                 TestCase test_case;
                 {
                   std::lock_guard lock(resources->mutex);
                   test_case = resources->test_case;
                 }
                 Context context(transport, std::move(test_case), {}, resources, bindings);
                 return sample(context, window);
               });
           value.every(observer.interval);
           if (!observer.interval_setting.empty())
             value.interval_setting(observer.interval_setting);
           if (!observer.window_setting.empty())
             value.window_setting(observer.window_setting);
           native.observer(value);
         }});
    return *this;
  }

private:
  friend class Plugin;
  struct RecordDefinition {
    Declaration declaration;
    RecordContract contract;
    std::function<void(harmos::plugin::Plugin &)> message;
  };
  struct StreamDefinition {
    Declaration declaration;
    std::function<void(harmos::plugin::Plugin &, std::shared_ptr<const detail::Bindings>, std::shared_ptr<detail::Executions>)> register_;
  };
  std::vector<RecordDefinition> records;
  std::vector<StreamDefinition> streams;
};
struct Suite : Declaration {
  std::vector<Template> templates;
  bool single = false;
  Suite(std::string name, std::vector<Template> templates, std::string label = {}, std::optional<std::string> identifier = {})
      : Declaration(std::move(name), std::move(label), std::move(identifier)), templates(std::move(templates)) {}
  template <class T> static Suite from(std::string label = {}, std::optional<std::string> identifier = {}) {
    auto value = Template::from<T>(std::move(label), identifier);
    Suite suite(value.name, {value}, value.label, std::move(identifier));
    suite.single = true;
    return suite;
  }
};

class Plugin {
public:
  Plugin(std::string identifier, std::string version, std::vector<Suite> suites = {});
  Plugin &suite(Suite suite);
  template <class Q, class R> Plugin &operation(const OperationValue<Q, R> &operation) {
    native_.operation(operation.native(executions_));
    return *this;
  }
  template <class Q, class R> Plugin &requires_operation(harmos::plugin::Operation<Q, R> operation) {
    native_.requires_operation(operation);
    return *this;
  }
  template <class T> Plugin &message() {
    native_.message<decltype(detail::payload_message<T>())>();
    return *this;
  }
  Plugin &secrets(SecretsDeclarations secrets);
  harmos::Declarations declarations() const;
  Plugin &requires_operation(HostOperation operation);
  void serve(std::string address = {}) const;

private:
  Plugin &context_operations();
  harmos::plugin::Plugin compose() const;
  harmos::plugin::Plugin native_;
  dokime::Suites suites_;
  PromptDefinitions prompts_;
  RecordContracts records_;
  SecretsDeclarations secrets_;
  std::shared_ptr<detail::Executions> executions_;
};
} // namespace dokime::plugin
