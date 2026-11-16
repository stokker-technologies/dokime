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

struct Step {
  Metadata metadata;
  std::function<void(Context &)> run;
  std::uint32_t retries = 0;
  bool repeatable = false;
};
struct Criterion {
  Metadata metadata;
  std::function<bool(Context &)> evaluate;
};
struct Template {
  TestCaseTemplate definition;
  std::vector<Step> steps;
  std::vector<Criterion> criteria;
  std::vector<std::string> streams;
  std::function<void(Context &)> initialize;
  std::function<void(Context &)> cleanup;
  std::function<void(TestCase &, const Parameters &)> materialize;
};
struct Suite {
  Metadata metadata;
  std::vector<Template> templates;
};

class Plugin {
public:
  Plugin(std::string identifier, std::string version);
  Plugin &suite(Suite suite);
  Plugin &prompt(Prompt prompt);
  template <class T> Plugin &record(RecordContract record) {
    if (record.metadata().identifier().empty())
      throw std::invalid_argument("Record Identifier Is Required");
    for (const auto &existing : records_.records())
      if (existing.metadata().identifier() == record.metadata().identifier())
        throw std::invalid_argument("Duplicate Record Identifier");
    if (record.message().empty())
      record.set_message(T::descriptor()->full_name());
    if (record.message() != T::descriptor()->full_name())
      throw std::invalid_argument("Record Message Differs from Its Contract");
    *records_.add_records() = std::move(record);
    native_.message<T>();
    return *this;
  }
  template <class T> Plugin &stream(Stream<T> stream) {
    native_.stream(stream);
    return *this;
  }
  template <class T> Plugin &observer(harmos::plugin::Observer<T> observer) {
    native_.observer(observer);
    return *this;
  }
  template <class T> Plugin &observer(Observer<T> observer) {
    auto state = executions_;
    harmos::plugin::Observer<T> native(observer.stream, [state, sample = std::move(observer.sample)](harmos::plugin::Context &transport) {
      auto resources = state->open(transport.scope());
      TestCase test_case;
      {
        std::lock_guard lock(resources->mutex);
        test_case = resources->test_case;
      }
      Context ctx(transport, std::move(test_case), {}, resources);
      return sample(ctx);
    });
    native.every(observer.interval);
    if (!observer.interval_setting.empty())
      native.interval_setting(std::move(observer.interval_setting));
    if (!observer.window_setting.empty())
      native.window_setting(std::move(observer.window_setting));
    native_.observer(native);
    return *this;
  }
  template <class T, class C> Plugin &observer(WindowObserver<T, C> observer) {
    auto state = executions_;
    auto native = harmos::plugin::Observer<T>::template window<C>(
        observer.stream, [state, sample = std::move(observer.sample)](harmos::plugin::Context &transport, const SampleWindow &window) {
          auto resources = state->open(transport.scope());
          TestCase test_case;
          {
            std::lock_guard lock(resources->mutex);
            test_case = resources->test_case;
          }
          Context ctx(transport, std::move(test_case), {}, resources);
          return sample(ctx, window);
        });
    native.every(observer.interval);
    if (!observer.interval_setting.empty())
      native.interval_setting(std::move(observer.interval_setting));
    if (!observer.window_setting.empty())
      native.window_setting(std::move(observer.window_setting));
    native_.observer(native);
    return *this;
  }
  template <class Q, class R> Plugin &operation(harmos::plugin::Operation<Q, R> operation) {
    native_.operation(operation);
    return *this;
  }
  template <class Q, class R> Plugin &requires_operation(harmos::plugin::Operation<Q, R> operation) {
    native_.requires_operation(operation);
    return *this;
  }
  template <class T> Plugin &message() {
    native_.message<T>();
    return *this;
  }
  template <class T> Plugin &section(T section) {
    native_.section(section);
    return *this;
  }
  Plugin &context_operations();
  harmos::Declarations declarations() const;
  Plugin &requires_operation(HostOperation operation);
  void serve(std::string address = {}) const;

private:
  harmos::plugin::Plugin compose() const;
  harmos::plugin::Plugin native_;
  std::string identifier_;
  dokime::Suites suites_;
  PromptDefinitions prompts_;
  RecordContracts records_;
  std::shared_ptr<detail::Executions> executions_;
};
} // namespace dokime::plugin
