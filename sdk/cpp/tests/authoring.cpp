/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "dokime/plugin.hpp"
#include <catch2/catch_test_macros.hpp>
#include <chrono>
#include <limits>

using dokime::Plugin;
using dokime::PromptDefinitions;
using dokime::RecordContracts;
using dokime::Suite;
using dokime::Suites;
using dokime::operations::operation;
using dokime::operations::Settings;
using dokime::operations::Visibility;
using dokime::templates::Case;
using dokime::templates::Context;
using dokime::templates::Criterion;
using dokime::templates::Observer;
using dokime::templates::PromptDeclaration;
using dokime::templates::Record;
using dokime::templates::Step;
using dokime::templates::Stream;
using dokime::templates::Template;
using harmos::Declarations;
using std::int64_t;
using std::invalid_argument;
using std::make_shared;
using std::numeric_limits;
using std::size_t;
using std::string;
using std::chrono::duration;
using std::chrono::milliseconds;
using std::chrono::seconds;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace {
class HTTPAccuracyTestTemplate : public dokime::templates::TestTemplate {
public:
  inline static const Record<bool> accepted_reading{"accepted_reading", "Unrelated Label"};
  inline static const PromptDeclaration question{"operator_choice", "Proceed?", {"Yes", "No"}};
  inline static const Stream<double> temperature{"outlet_temperature", "Unrelated Temperature Label"};
  void readVoltage(Context &) {}
  bool readingOK(Context &) {
    return true;
  }
  static Template definition() {
    auto value = Template::from<HTTPAccuracyTestTemplate>();
    value.record(accepted_reading);
    value.prompts.push_back(question);
    value.steps.push_back(Step::method<&HTTPAccuracyTestTemplate::readVoltage>("Completely Different Label", {}, 2));
    value.criteria.push_back(Criterion::method<&HTTPAccuracyTestTemplate::readingOK>("Another Label"));
    Observer<double> observer{temperature,
                              [](Context &) {
                                return 20.0;
                              },
                              {},
                              {},
                              {}};
    observer.interval.set_seconds(1);
    value.observer(observer);
    return value;
  }
};
class VoltageTestTemplate : public dokime::templates::TestTemplate {
public:
  inline static const Record<int64_t> reading{"reading", "Reading", "exact"};
  void read_voltage(Context &) {}
  static Template definition() {
    auto value = Template::from<VoltageTestTemplate>();
    value.record(reading);
    value.steps.push_back(Step::method<&VoltageTestTemplate::read_voltage>("Read", "take"));
    return value;
  }
};
} // namespace

/* === Templates === */

namespace {
template <class T> void roundtrip(const T &value) {
  const auto bytes = dokime::templates::detail::pack<T>(value).SerializeAsString();
  const auto decoded = dokime::templates::detail::decode<T>(bytes);
  REQUIRE(decoded == value);
}
} // namespace

/* === Public Functions === */

class AccuracySuite : public Suite {
public:
  static Suite definition() {
    auto value = Suite::from<AccuracySuite>();
    value.identifier = "example.accuracy";
    value.template_value(HTTPAccuracyTestTemplate::definition());
    return value;
  }
};
class ExamplePlugin : public Plugin {
public:
  ExamplePlugin() : Plugin(Plugin::from<ExamplePlugin>("1.0.0")) {
    suite(AccuracySuite::definition());
  }
};

TEST_CASE("Author Classes Describe Explicit Suite and Template Membership") {
  const ExamplePlugin plugin;
  const auto value = plugin.describe();
  REQUIRE(value.identifier == "example");
  REQUIRE(value.suites.at(0).identifier == "example.accuracy");
  REQUIRE(value.suites.at(0).label == "Accuracy");
  REQUIRE(value.suites.at(0).templates.at(0).identifier == "example.accuracy.http_accuracy");
  REQUIRE(value.suites.at(0).templates.at(0).label == "HTTP Accuracy");
  REQUIRE(value.suites.at(0).templates.at(0).steps.at(0).identifier == "example.accuracy.http_accuracy.read_voltage");
  REQUIRE(value.records.at(0).identifier == "example.accuracy.http_accuracy.accepted_reading");
}

TEST_CASE("Member Spelling Variants Produce the Same Operation Identity") {
  for (const auto &name : {"ReadVoltage", "readVoltage", "read_voltage"}) {
    Template value("VoltageTestTemplate");
    value.steps.push_back(Step(name, "Unrelated Label", [](Context &) {
    }));
    const Plugin plugin("names", "1.0.0", {{"names.accuracy", {value}}});
    REQUIRE(plugin.describe().suites.at(0).templates.at(0).steps.at(0).identifier == "names.accuracy.voltage.read_voltage");
  }
}

TEST_CASE("Structured Prompts Keep Fields Actions and Independent Template Bindings") {
  REQUIRE_THROWS_AS(PromptDeclaration("question", "Proceed?", {"Yes", "Yes"}), invalid_argument);
  dokime::Prompt definition;
  auto *field = definition.add_fields();
  field->mutable_metadata()->set_identifier("reading");
  field->set_type(dokime::VALUE_TYPE_NUMBER);
  field->set_required(true);
  definition.add_actions()->set_identifier("save");
  const PromptDeclaration question("question", definition);
  definition.clear_fields();
  Template local("ReadingTestTemplate");
  local.prompts.push_back(question);
  const Plugin first("one", "1.0.0", {{"one.accuracy", {local}}});
  const Plugin second("two", "1.0.0", {{"two.production", {local}}});
  const auto first_prompts = first.describe().prompts;
  const auto second_prompts = second.describe().prompts;
  const auto &bound = first_prompts.at(0);
  REQUIRE(bound.metadata().identifier() == "one.accuracy.reading.question");
  REQUIRE(bound.fields(0).required());
  REQUIRE(bound.fields(0).metadata().identifier() == "reading");
  REQUIRE(bound.actions(0).identifier() == "save");
  REQUIRE(second_prompts.at(0).metadata().identifier() == "two.production.reading.question");
  REQUIRE(question.value.metadata().identifier().empty());
}

TEST_CASE("Grouped Templates Bind Independently and Explicit Identifiers Win") {
  const auto local = VoltageTestTemplate::definition();
  const Plugin first("one", "1.0.0", {{"one.accuracy", {local}}});
  const Plugin second("two", "1.0.0", {{"two.production", {local}}});
  const auto first_suites = first.describe().suites;
  REQUIRE(first_suites.at(0).templates.at(0).identifier == "one.accuracy.voltage");
  REQUIRE(first_suites.at(0).templates.at(0).steps.at(0).identifier == "one.accuracy.voltage.take");
  REQUIRE(first.describe().records.at(0).identifier == "one.accuracy.voltage.exact");
  REQUIRE(second.describe().records.at(0).identifier == "two.production.voltage.exact");
  REQUIRE_FALSE(local.identifier.has_value());
  REQUIRE(VoltageTestTemplate::reading.identifier == "exact");
}

TEST_CASE("Duplicate Empty and Lifecycle Declaration Collisions Are Refused") {
  Template repeated("Repeated");
  repeated.record(Record<bool>("first", {}, "same"));
  repeated.record(Record<string>("second", {}, "same"));
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"invalid.suite", {repeated}}}), invalid_argument);
  Template empty("Empty");
  empty.record(Record<int64_t>("reading", {}, ""));
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"invalid.suite", {empty}}}), invalid_argument);
  Template cross_kind("CrossKind");
  cross_kind.record(Record<bool>("reading", {}, "measure"));
  cross_kind.steps.push_back(Step("measure", "Measure", [](Context &) {
  }));
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"invalid.suite", {cross_kind}}}), invalid_argument);
  Template lifecycle("Lifecycle");
  lifecycle.record(Record<bool>("cleanup"));
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"invalid.suite", {lifecycle}}}), invalid_argument);
  const auto value = VoltageTestTemplate::definition();
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"invalid.suite", {value, value}}}), invalid_argument);
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"invalid.suite", {value}, {}, ""}}), invalid_argument);
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"invalid.suite", {value}}, {"invalid.suite", {value}}}), invalid_argument);
  Plugin recoverable("valid", "1.0.0");
  REQUIRE_THROWS_AS(recoverable.suite({"valid.suite", {cross_kind}}), invalid_argument);
  REQUIRE_NOTHROW(recoverable.suite({"valid.suite", {value}}).describe());
}

TEST_CASE("Primitive Payloads Share Record and Operation Contracts") {
  roundtrip(false);
  roundtrip(true);
  roundtrip(string("a value"));
  roundtrip(numeric_limits<int64_t>::min());
  roundtrip(numeric_limits<int64_t>::max());
  roundtrip(2.125);
  roundtrip(17);
  const auto large_integer = dokime::templates::detail::pack<int64_t>(numeric_limits<int64_t>::max()).SerializeAsString();
  REQUIRE_THROWS_AS(dokime::templates::detail::decode<int>(large_integer), invalid_argument);
  google::protobuf::BoolValue structured;
  structured.set_value(true);
  REQUIRE(dokime::templates::detail::decode<google::protobuf::BoolValue>(structured.SerializeAsString()).value());
  Template value("ValuesTestTemplate");
  value.record(Record<string>("text")).record(Record<bool>("flag")).record(Record<int64_t>("count")).record(Record<double>("number"));
  Plugin plugin("values", "1.0.0", {{"values.values", {value}}});
  plugin.operation(operation<string, string>("values.echo", Visibility::Public, [](dokime::operations::Context &, string request) {
    return request;
  }));
  plugin.operation(operation<bool, bool>("values.flag", Visibility::Public, [](dokime::operations::Context &, bool request) {
    return request;
  }));
  plugin.operation(operation<int64_t, int64_t>("values.count", Visibility::Public, [](dokime::operations::Context &, int64_t request) {
    return request;
  }));
  plugin.operation(operation<double, double>("values.number", Visibility::Public, [](dokime::operations::Context &, double request) {
    return request;
  }));
  const auto declarations = plugin.describe();
  const auto &contracts = declarations.records;
  size_t matched = 0;
  for (const auto &operation : declarations.operations) {
    if (operation.visibility != Visibility::Public)
      continue;
    for (const auto &record : contracts) {
      if (record.message != operation.request)
        continue;
      REQUIRE(operation.response == record.message);
      ++matched;
    }
  }
  REQUIRE(matched == 4);
}

TEST_CASE("Materialized Instances Share Qualification and Refuse Undeclared Handlers") {
  const auto step = Step("ReadVoltage", "Read", [](Context &) {
  });
  const auto criterion = Criterion("AcceptVoltage", "Accept", [](Context &) {
    return true;
  });
  auto bindings = make_shared<dokime::templates::detail::Bindings>();
  bindings->template_id = "one.accuracy.voltage";
  bindings->identifiers.emplace(step.key, "one.accuracy.voltage.read_voltage");
  bindings->identifiers.emplace(criterion.key, "one.accuracy.voltage.accept_voltage");
  Case value(bindings, {step}, {criterion});
  value.step(step, "First Read", "read_1").step(step, "Second Read", "read_2");
  value.criterion(criterion, "Accept", "accept_1");
  REQUIRE(value.message.steps(0).metadata().identifier() == "one.accuracy.voltage.read_1");
  REQUIRE(value.message.steps(1).metadata().identifier() == "one.accuracy.voltage.read_2");
  REQUIRE(value.message.steps(1).operation() == "one.accuracy.voltage.read_voltage");
  REQUIRE(value.message.criteria(0).metadata().identifier() == "one.accuracy.voltage.accept_1");
  REQUIRE_THROWS_AS(value.step(step, "Repeated", "read_2"), invalid_argument);
  REQUIRE_THROWS_AS(value.step(step, "Empty", ""), invalid_argument);
  REQUIRE_THROWS_AS(value.step(Step("Other", "Other",
                                    [](Context &) {
                                    }),
                               "Other"),
                    invalid_argument);
}

TEST_CASE("Retry Policies Require Positive Backoff Before Composition") {
  auto echo = operation<string, string>("policy.echo", Visibility::Public, [](dokime::operations::Context &, string request) {
    return request;
  });
  dokime::operations::OperationPolicy policy;
  policy.attempts = 2;
  REQUIRE_THROWS_AS(echo.policy(policy), invalid_argument);
  policy.backoff = milliseconds(20);
  policy.timeout = seconds(3);
  echo.replay_safe().policy(policy);
  Plugin plugin("policy", "1.0.0");
  REQUIRE_NOTHROW(plugin.operation(echo));
  policy.backoff = duration<double>(numeric_limits<double>::infinity());
  REQUIRE_THROWS_AS(echo.policy(policy), invalid_argument);
}

TEST_CASE("Typed Settings Preserve Signed Durations and Inclusive Bounds") {
  class ClockSettings : public Settings {
  public:
    using Settings::Settings;
    const duration<double> offset = setting<duration<double>>("offset", "Clock Offset", duration<double>(-1.25), seconds(-2), seconds(0));
  };
  class RequiredSettings : public Settings {
  public:
    using Settings::Settings;
    const string mode = setting<string>("mode", "Mode", "first", {}, {}, "", {"first", "second"}, true);
  };
  const RequiredSettings required;
  REQUIRE(required.declarations().front().required());
  REQUIRE(required.declarations().front().choices_size() == 2);
  const ClockSettings defaults;
  const auto &declaration = defaults.declarations().front();
  REQUIRE(declaration.default_().duration().seconds() == -1);
  REQUIRE(declaration.default_().duration().nanos() == -250000000);
  REQUIRE(declaration.minimum().duration().seconds() == -2);
  REQUIRE(declaration.maximum().has_duration());
  harmos::SettingValues saved;
  auto *saved_duration = (*saved.mutable_values())["offset"].mutable_duration();
  saved_duration->set_seconds(-1);
  saved_duration->set_nanos(-500000000);
  const ClockSettings snapshot(saved);
  REQUIRE(snapshot.offset.count() == -1.5);
}

TEST_CASE("Plugin Records Share One Contract Across Templates and Observers") {
  const Record<bool> event("event", "Event");
  Plugin plugin("events", "1.0.0");
  plugin.record(event);
  plugin.operation(operation<bool, bool>("events.write", Visibility::Public, [event](dokime::operations::Context &context, bool value) {
    context.record(event, value);
    return value;
  }));
  const auto description = plugin.describe();
  REQUIRE(description.records.size() == 1);
  REQUIRE(description.records.front().identifier == "events.event");
  REQUIRE_THROWS_AS(plugin.record(event), invalid_argument);
}

/* === Private Functions === */

/* === Entry Point === */
