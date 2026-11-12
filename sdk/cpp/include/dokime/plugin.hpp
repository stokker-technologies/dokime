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
#include "dokime/v1/catalog.pb.h"
#include <set>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Declarations === */

namespace dokime {
v1::Metadata metadata(std::string identifier, std::string label);

struct Step {
  v1::Metadata metadata;
  std::function<void(Context &)> run;
  std::uint32_t retries = 0;
  bool repeatable = false;
};
struct Criterion {
  v1::Metadata metadata;
  std::function<bool(Context &)> evaluate;
};
struct Template {
  v1::TestCaseTemplate definition;
  std::vector<Step> steps;
  std::vector<Criterion> criteria;
  std::vector<std::string> streams;
  std::function<void(Context &)> initialize;
  std::function<void(Context &)> cleanup;
  std::function<void(v1::TestCase &, const Parameters &)> materialize;
};
struct Suite {
  v1::Metadata metadata;
  std::vector<Template> templates;
};

class Plugin {
public:
  Plugin(std::string identifier, std::string version);
  Plugin &suite(Suite suite);
  Plugin &prompt(Prompt prompt);
  template <class T> Plugin &record(Record record) {
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
    // Include the Record Descriptor Without Inventing an Operation.
    if (record_schemas_.insert(std::string(T::descriptor()->full_name())).second)
      native_.section(T{});
    return *this;
  }
  template <class T> Plugin &stream(Stream<T> stream) {
    native_.stream(stream);
    return *this;
  }
  template <class T> Plugin &observer(harmos::Observer<T> observer) {
    native_.observer(observer);
    return *this;
  }
  template <class T> Plugin &observer(Observer<T> observer) {
    auto state = executions_;
    harmos::Observer<T> native(observer.stream, [state, sample = std::move(observer.sample)](harmos::Context &transport) {
      auto resources = state->open(transport.scope);
      v1::TestCase test_case;
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
    native_.observer(native);
    return *this;
  }
  template <class Q, class R> Plugin &operation(harmos::Operation<Q, R> operation) {
    native_.operation(operation);
    return *this;
  }
  template <class Q, class R> Plugin &requires_operation(harmos::Operation<Q, R> operation) {
    native_.requires_operation(operation);
    return *this;
  }
  template <class T> Plugin &section(T section) {
    native_.section(section);
    return *this;
  }
  Plugin &context_operations();
  harmos::wire::Catalog catalog() const;
  void serve(std::string address = {}) const;

private:
  harmos::Plugin compose() const;
  harmos::Plugin native_;
  std::string identifier_;
  std::set<std::string> record_schemas_;
  v1::Suites suites_;
  v1::PromptCatalog prompts_;
  v1::RecordContracts records_;
  std::shared_ptr<detail::Executions> executions_;
};
} // namespace dokime
