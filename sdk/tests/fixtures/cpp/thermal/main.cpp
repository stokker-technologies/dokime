// SPDX-License-Identifier: Apache-2.0
#include "dokime/dokime.hpp"
#include "thermal.pb.h"
#include <iostream>

using dokime::ExecutionProgressValue;
using dokime::Prompt;
using dokime::PROMPT_ARCHETYPE_CONFIRMATION;
using dokime::RecordContract;
using dokime::SEVERITY_INFO;
using dokime::VALUE_TYPE_NUMBER;
using dokime::plugin::Context;
using dokime::plugin::metadata;
using dokime::plugin::Observer;
using dokime::plugin::Plugin;
using dokime::plugin::SampleWindow;
using dokime::plugin::Stream;
using dokime::plugin::Template;
using dokime::plugin::WindowObserver;
using google::protobuf::Duration;
using harmos::plugin::decode;
using std::exception;
using std::invalid_argument;
using std::make_shared;
using std::move;
using std::string;
using thermal::v1::Counter;
using thermal::v1::CounterColumns;
using thermal::v1::Door;
using thermal::v1::Temperature;

struct Bench {
  double limit;
};

int main(int argc, char **argv) {
  try {
    Template value;
    *value.definition.mutable_metadata() = metadata("thermal.cycle", "Thermal Cycle");
    auto *parameter = value.definition.add_parameters();
    *parameter->mutable_metadata() = metadata("limit", "Limit");
    parameter->set_type(VALUE_TYPE_NUMBER);
    parameter->mutable_expression()->mutable_literal()->set_number(85);
    value.streams = {"thermal.temperature", "thermal.counter"};
    value.initialize = [](Context &ctx) {
      ctx.set(make_shared<Bench>(Bench{ctx.parameters().at("limit").number()}));
      auto host = ctx.host();
      host.evidence.logs.append(ctx.execution(), SEVERITY_INFO, "Initialized", host.time.get());
    };
    value.steps.push_back({metadata("ramp", "Ramp to Target"),
                           [](Context &ctx) {
                             auto host = ctx.host();
                             const auto current = host.executions.progress.get();
                             ExecutionProgressValue progress;
                             progress.set_completed(1);
                             progress.set_total(2);
                             progress.set_label("Ramp to Target");
                             host.executions.progress.set(progress, current.revision());
                           },
                           2});
    value.steps.push_back({metadata("hold", "Hold"), [](Context &ctx) {
                             auto host = ctx.host();
                             const auto answer = host.prompts.create("door", "thermal.door", {});
                             Door door;
                             door.set_closed(answer.action() == "yes" && answer.accepted());
                             host.evidence.records.append(ctx.execution(), "thermal.door", door.SerializeAsString(), host.time.get());
                             Duration duration;
                             duration.set_seconds(2);
                             duration.set_nanos(500000000);
                             host.time.sleep(duration);
                             const auto current = host.executions.progress.get();
                             ExecutionProgressValue progress;
                             progress.set_completed(2);
                             progress.set_total(2);
                             progress.set_label("Hold");
                             host.executions.progress.set(progress, current.revision());
                           }});
    value.criteria.push_back({metadata("within_limit", "Temperature Within Limit"), [](Context &ctx) {
                                auto host = ctx.host();
                                const auto measured = host.evidence.statistics("thermal.temperature", {}, {}, {}, "celsius", "hold");
                                const auto records = host.evidence.records.list("thermal.door", {}, {}, "hold", {});
                                if (!records.has_last())
                                  return false;
                                const auto door = decode<Door>(records.last().payload());
                                const auto &statistics = measured.statistics();
                                return statistics.count() > 0 && statistics.has_max() && statistics.max().has_number() &&
                                       statistics.max().number() <= ctx.get<Bench>()->limit && door.closed();
                              }});
    value.cleanup = [](Context &ctx) {
      auto host = ctx.host();
      host.evidence.logs.append(ctx.execution(), SEVERITY_INFO, "Cleaned Up", host.time.get());
    };
    Stream<Temperature> temperature("thermal.temperature");
    Observer<Temperature> observer{temperature,
                                   [](Context &ctx) {
                                     Temperature sample;
                                     try {
                                       sample.set_celsius(ctx.get<Bench>()->limit - 1);
                                     } catch (const invalid_argument &) {
                                       sample.set_celsius(0);
                                     }
                                     return sample;
                                   },
                                   {},
                                   {},
                                   {}};
    observer.interval.set_seconds(1);
    Stream<Counter> counter("thermal.counter");
    // A 1 kHz Stream Answers Each Host Window at Once.
    WindowObserver<Counter, CounterColumns> counter_observer{counter,
                                                             [](Context &, const SampleWindow &window) {
                                                               CounterColumns columns;
                                                               for (std::uint64_t index = 0; index < window.count; ++index)
                                                                 columns.add_value(1152921504606847975ULL);
                                                               return columns;
                                                             },
                                                             {},
                                                             {},
                                                             {}};
    counter_observer.interval.set_nanos(1000000);
    RecordContract door;
    *door.mutable_metadata() = metadata("thermal.door", "Door Check");
    Prompt prompt;
    *prompt.mutable_metadata() = metadata("thermal.door", "Door Check");
    prompt.set_message("Is the Door Closed?");
    prompt.set_archetype(PROMPT_ARCHETYPE_CONFIRMATION);
    auto *yes = prompt.add_actions();
    yes->set_identifier("yes");
    yes->set_label("Yes");
    auto *no = prompt.add_actions();
    no->set_identifier("no");
    no->set_label("No");
    Plugin plugin("thermal", "1.0.0");
    plugin.context_operations()
        .stream(temperature)
        .observer(move(observer))
        .stream(counter)
        .observer(move(counter_observer))
        .record<Door>(move(door))
        .prompt(move(prompt))
        .suite({metadata("thermal", "Thermal"), {move(value)}});
    plugin.serve(argc == 2 && string(argv[1]) == "--remote" ? "127.0.0.1:0" : "");
  } catch (const exception &error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
