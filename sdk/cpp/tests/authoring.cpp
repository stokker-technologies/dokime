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
#include <limits>

using dokime::PromptDefinitions;
using dokime::RecordContracts;
using dokime::Suites;
using dokime::plugin::Case;
using dokime::plugin::Context;
using dokime::plugin::Criterion;
using dokime::plugin::Observer;
using dokime::plugin::operation;
using dokime::plugin::Plugin;
using dokime::plugin::PromptDeclaration;
using dokime::plugin::Record;
using dokime::plugin::Step;
using dokime::plugin::Stream;
using dokime::plugin::Suite;
using dokime::plugin::Template;
using dokime::plugin::Visibility;
using harmos::Declarations;
using std::int64_t;
using std::invalid_argument;
using std::make_shared;
using std::numeric_limits;
using std::size_t;
using std::string;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace {
class HTTPAccuracySuite {
public:
  inline static const Record<bool> accepted_reading{"accepted_reading", "Unrelated Label"};
  inline static const PromptDeclaration question{"operator_choice", "Proceed?", {"Yes", "No"}};
  inline static const Stream<double> temperature{"outlet_temperature", "Unrelated Temperature Label"};
  void readVoltage(Context &) {}
  bool readingOK(Context &) {
    return true;
  }
  static void declare(Template &value) {
    value.record(accepted_reading);
    value.prompts.push_back(question);
    value.steps.push_back(Step::method<&HTTPAccuracySuite::readVoltage>("Completely Different Label", {}, 2));
    value.criteria.push_back(Criterion::method<&HTTPAccuracySuite::readingOK>("Another Label"));
    Observer<double> observer{temperature,
                              [](Context &) {
                                return 20.0;
                              },
                              {},
                              {},
                              {}};
    observer.interval.set_seconds(1);
    value.observer(observer);
  }
};
class VoltageTemplate {
public:
  inline static const Record<int64_t> reading{"reading", "Reading", "exact"};
  void read_voltage(Context &) {}
  static void declare(Template &value) {
    value.record(reading);
    value.steps.push_back(Step::method<&VoltageTemplate::read_voltage>("Read", "take"));
  }
};
} // namespace

/* === Templates === */

namespace {
template <class T> T section(const Declarations &declarations) {
  for (const auto &section : declarations.sections()) {
    if (section.type() != T::descriptor()->full_name())
      continue;
    T value;
    REQUIRE(value.ParseFromString(section.payload()));
    return value;
  }
  throw invalid_argument("Read Declarations: Register the Requested Section");
}
template <class T> void roundtrip(const T &value) {
  const auto bytes = dokime::plugin::detail::pack<T>(value).SerializeAsString();
  const auto decoded = dokime::plugin::detail::decode<T>(bytes);
  REQUIRE(decoded == value);
}
} // namespace

/* === Public Functions === */

TEST_CASE("Code Names Qualify Every Template Member Independently of Labels") {
  const Plugin plugin("naming", "1.0.0", {Suite::from<HTTPAccuracySuite>("Not an Identifier")});
  const auto declarations = plugin.declarations();
  const auto suites = section<Suites>(declarations);
  const auto &value = suites.suites(0).templates(0);
  REQUIRE(suites.suites(0).metadata().identifier() == "http_accuracy");
  REQUIRE(value.metadata().identifier() == "http_accuracy");
  REQUIRE(value.steps(0).metadata().identifier() == "http_accuracy.read_voltage");
  REQUIRE(value.steps(0).operation() == "http_accuracy.read_voltage");
  REQUIRE(section<RecordContracts>(declarations).records(0).metadata().identifier() == "http_accuracy.accepted_reading");
  REQUIRE(section<RecordContracts>(declarations).records(0).message() == "google.protobuf.BoolValue");
  const auto prompts = section<PromptDefinitions>(declarations);
  REQUIRE(prompts.prompts(0).metadata().identifier() == "http_accuracy.operator_choice");
  REQUIRE(prompts.prompts(0).owner_suite() == "http_accuracy");
  REQUIRE(prompts.prompts(0).template_() == "http_accuracy");
  REQUIRE(declarations.streams(0).id() == "http_accuracy.outlet_temperature");
  bool retry = false;
  bool criterion = false;
  for (const auto &operation : declarations.operations()) {
    REQUIRE(operation.visibility() == harmos::VISIBILITY_PROTECTED);
    if (operation.id() == "http_accuracy.read_voltage")
      retry = operation.policy().retries().total_attempts() == 3;
    criterion = criterion || operation.id() == "http_accuracy.reading_ok";
  }
  REQUIRE(retry);
  REQUIRE(criterion);
}

TEST_CASE("Member Spelling Variants Produce the Same Operation Identity") {
  for (const auto &name : {"ReadVoltage", "readVoltage", "read_voltage"}) {
    Template value("VoltageTemplate");
    value.steps.push_back(Step(name, "Unrelated Label", [](Context &) {
    }));
    const Plugin plugin("names", "1.0.0", {{"accuracy", {value}}});
    REQUIRE(section<Suites>(plugin.declarations()).suites(0).templates(0).steps(0).operation() == "accuracy.voltage.read_voltage");
  }
}

TEST_CASE("Structured Prompts Keep Fields Actions and Independent Template Bindings") {
  dokime::Prompt definition;
  auto *field = definition.add_fields();
  field->mutable_metadata()->set_identifier("reading");
  field->set_type(dokime::VALUE_TYPE_NUMBER);
  field->set_required(true);
  definition.add_actions()->set_identifier("save");
  const PromptDeclaration question("question", definition);
  definition.clear_fields();
  Template local("ReadingTemplate");
  local.prompts.push_back(question);
  const Plugin first("one", "1.0.0", {{"accuracy", {local}}});
  const Plugin second("two", "1.0.0", {{"production", {local}}});
  const auto first_prompts = section<PromptDefinitions>(first.declarations());
  const auto second_prompts = section<PromptDefinitions>(second.declarations());
  const auto &bound = first_prompts.prompts(0);
  REQUIRE(bound.metadata().identifier() == "accuracy.reading.question");
  REQUIRE(bound.fields(0).required());
  REQUIRE(bound.fields(0).metadata().identifier() == "reading");
  REQUIRE(bound.actions(0).identifier() == "save");
  REQUIRE(second_prompts.prompts(0).metadata().identifier() == "production.reading.question");
  REQUIRE(question.value.metadata().identifier().empty());
}

TEST_CASE("Grouped Templates Bind Independently and Explicit Identifiers Win") {
  const auto local = Template::from<VoltageTemplate>();
  const Plugin first("one", "1.0.0", {{"accuracy", {local}}});
  const Plugin second("two", "1.0.0", {{"production", {local}}});
  const auto first_suites = section<Suites>(first.declarations());
  REQUIRE(first_suites.suites(0).templates(0).metadata().identifier() == "accuracy.voltage");
  REQUIRE(first_suites.suites(0).templates(0).steps(0).operation() == "accuracy.voltage.take");
  REQUIRE(section<RecordContracts>(first.declarations()).records(0).metadata().identifier() == "accuracy.voltage.exact");
  REQUIRE(section<RecordContracts>(second.declarations()).records(0).metadata().identifier() == "production.voltage.exact");
  REQUIRE_FALSE(local.identifier.has_value());
  REQUIRE(VoltageTemplate::reading.identifier == "exact");
  const Plugin chosen("three", "1.0.0", {Suite::from<VoltageTemplate>("Ignored", "chosen")});
  REQUIRE(section<Suites>(chosen.declarations()).suites(0).templates(0).metadata().identifier() == "chosen");
}

TEST_CASE("Duplicate Empty and Lifecycle Declaration Collisions Are Refused") {
  Template repeated("Repeated");
  repeated.record(Record<bool>("first", {}, "same"));
  repeated.record(Record<string>("second", {}, "same"));
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"suite", {repeated}}}), invalid_argument);
  Template empty("Empty");
  empty.record(Record<int64_t>("reading", {}, ""));
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"suite", {empty}}}), invalid_argument);
  Template cross_kind("CrossKind");
  cross_kind.record(Record<bool>("reading", {}, "measure"));
  cross_kind.steps.push_back(Step("measure", "Measure", [](Context &) {
  }));
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"suite", {cross_kind}}}), invalid_argument);
  Template lifecycle("Lifecycle");
  lifecycle.record(Record<bool>("cleanup"));
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"suite", {lifecycle}}}), invalid_argument);
  const auto value = Template::from<VoltageTemplate>();
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"suite", {value, value}}}), invalid_argument);
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"suite", {value}, {}, ""}}), invalid_argument);
  REQUIRE_THROWS_AS(Plugin("invalid", "1.0.0", {{"suite", {value}}, {"suite", {value}}}), invalid_argument);
  Plugin recoverable("valid", "1.0.0");
  REQUIRE_THROWS_AS(recoverable.suite({"suite", {cross_kind}}), invalid_argument);
  REQUIRE_NOTHROW(recoverable.suite({"suite", {value}}).declarations());
}

TEST_CASE("Primitive Payloads Share Record and Operation Contracts") {
  roundtrip(false);
  roundtrip(true);
  roundtrip(string("a value"));
  roundtrip(numeric_limits<int64_t>::min());
  roundtrip(numeric_limits<int64_t>::max());
  roundtrip(2.125);
  roundtrip(17);
  const auto large_integer = dokime::plugin::detail::pack<int64_t>(numeric_limits<int64_t>::max()).SerializeAsString();
  REQUIRE_THROWS_AS(dokime::plugin::detail::decode<int>(large_integer), invalid_argument);
  google::protobuf::BoolValue structured;
  structured.set_value(true);
  REQUIRE(dokime::plugin::detail::decode<google::protobuf::BoolValue>(structured.SerializeAsString()).value());
  Template value("ValuesTemplate");
  value.record(Record<string>("text")).record(Record<bool>("flag")).record(Record<int64_t>("count")).record(Record<double>("number"));
  Plugin plugin("values", "1.0.0", {{"values", {value}}});
  plugin.operation(operation<string, string>("values.echo", Visibility::Public, [](Context &, string request) {
    return request;
  }));
  plugin.operation(operation<bool, bool>("values.flag", Visibility::Public, [](Context &, bool request) {
    return request;
  }));
  plugin.operation(operation<int64_t, int64_t>("values.count", Visibility::Public, [](Context &, int64_t request) {
    return request;
  }));
  plugin.operation(operation<double, double>("values.number", Visibility::Public, [](Context &, double request) {
    return request;
  }));
  const auto declarations = plugin.declarations();
  const auto contracts = section<RecordContracts>(declarations);
  size_t matched = 0;
  for (const auto &operation : declarations.operations()) {
    if (operation.visibility() != harmos::VISIBILITY_PUBLIC)
      continue;
    for (const auto &record : contracts.records()) {
      if (record.message() != operation.request())
        continue;
      REQUIRE(operation.response() == record.message());
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
  auto bindings = make_shared<dokime::plugin::detail::Bindings>();
  bindings->template_id = "accuracy.voltage";
  bindings->identifiers.emplace(step.key, "accuracy.voltage.read_voltage");
  bindings->identifiers.emplace(criterion.key, "accuracy.voltage.accept_voltage");
  Case value(bindings, {step}, {criterion});
  value.step(step, "First Read", "read_1").step(step, "Second Read", "read_2");
  value.criterion(criterion, "Accept", "accept_1");
  REQUIRE(value.message.steps(0).metadata().identifier() == "accuracy.voltage.read_1");
  REQUIRE(value.message.steps(1).metadata().identifier() == "accuracy.voltage.read_2");
  REQUIRE(value.message.steps(1).operation() == "accuracy.voltage.read_voltage");
  REQUIRE(value.message.criteria(0).metadata().identifier() == "accuracy.voltage.accept_1");
  REQUIRE_THROWS_AS(value.step(step, "Repeated", "read_2"), invalid_argument);
  REQUIRE_THROWS_AS(value.step(step, "Empty", ""), invalid_argument);
  REQUIRE_THROWS_AS(value.step(Step("Other", "Other",
                                    [](Context &) {
                                    }),
                               "Other"),
                    invalid_argument);
}

/* === Private Functions === */

/* === Entry Point === */
