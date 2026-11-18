// SPDX-License-Identifier: Apache-2.0
#include "dokime/dokime.hpp"
#include "thermal.pb.h"
#include <grpc/event_engine/event_engine.h>
#include <iostream>

using dokime::ExecutionProgressValue;
using dokime::SEVERITY_INFO;
using dokime::VALUE_TYPE_NUMBER;
using dokime::plugin::Context;
using dokime::plugin::Criterion;
using dokime::plugin::metadata;
using dokime::plugin::Observer;
using dokime::plugin::Plugin;
using dokime::plugin::Record;
using dokime::plugin::Step;
using dokime::plugin::Stream;
using dokime::plugin::Template;
using dokime::plugin::WindowObserver;
using dokime::plugin::sampling::Window;
using google::protobuf::Duration;
using grpc_event_engine::experimental::ShutdownDefaultEventEngine;
using std::cerr;
using std::exception;
using std::invalid_argument;
using std::make_shared;
using std::move;
using std::string;
using std::uint64_t;
using thermal::v1::Counter;
using thermal::v1::CounterColumns;
using thermal::v1::Door;
using thermal::v1::Temperature;

struct Bench {
  double limit;
};

class Thermal {
public:
  inline static const Record<Door> door{"door", "Door Check"};
  inline static const Stream<Temperature> temperature_stream{"temperature", "Temperature [°C]"};
  inline static const Stream<Counter> counter_stream{"counter", "Exact Counter"};

  void initialize(Context &ctx) {
    ctx.set(make_shared<Bench>(Bench{ctx.parameters().at("limit").number()}));
    auto host = ctx.host();
    host.evidence.logs.append(ctx.execution(), SEVERITY_INFO, "Initialized", host.time.get());
  }
  void ramp(Context &ctx) {
    auto host = ctx.host();
    const auto current = host.executions.progress.get();
    ExecutionProgressValue progress;
    progress.set_completed(1);
    progress.set_total(2);
    progress.set_label("Ramp to Target");
    host.executions.progress.set(progress, current.revision());
  }
  void hold(Context &ctx) {
    const auto answer = ctx.prompt("Is the Door Closed?", {"Yes", "No"});
    Door reading;
    reading.set_closed(answer == "Yes");
    ctx.record(door, reading);
    Duration duration;
    duration.set_seconds(2);
    duration.set_nanos(500000000);
    ctx.sleep(duration);
    auto host = ctx.host();
    const auto current = host.executions.progress.get();
    ExecutionProgressValue progress;
    progress.set_completed(2);
    progress.set_total(2);
    progress.set_label("Hold");
    host.executions.progress.set(progress, current.revision());
  }
  bool within_limit(Context &ctx) {
    const auto measured = ctx.statistics<&Thermal::hold>(temperature_stream, "celsius");
    const auto reading = ctx.records<&Thermal::hold>(door).last();
    const auto &statistics = measured.statistics();
    const bool has_samples = statistics.count() > 0 && statistics.has_max() && statistics.max().has_number();
    return has_samples && statistics.max().number() <= ctx.get<Bench>()->limit && reading.closed();
  }
  void cleanup(Context &ctx) {
    auto host = ctx.host();
    host.evidence.logs.append(ctx.execution(), SEVERITY_INFO, "Cleaned Up", host.time.get());
  }
  static void declare(Template &value) {
    auto &parameter = value.parameters.emplace_back();
    *parameter.mutable_metadata() = metadata("limit", "Limit");
    parameter.set_type(VALUE_TYPE_NUMBER);
    parameter.mutable_expression()->mutable_literal()->set_number(85);
    value.record(door);
    value.steps.push_back(Step::method<&Thermal::ramp>("Ramp to Target", "ramp", 2));
    value.steps.push_back(Step::method<&Thermal::hold>("Hold", "hold"));
    value.criteria.push_back(Criterion::method<&Thermal::within_limit>("Temperature Within Limit", "within-limit"));
    Observer<Temperature> observer{temperature_stream,
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
    value.observer(move(observer));

    // A 1 kHz Stream Answers Each Host Window at Once.
    WindowObserver<Counter, CounterColumns> counter_observer{counter_stream,
                                                             [](Context &, const Window &window) {
                                                               CounterColumns columns;
                                                               for (uint64_t index = 0; index < window.count; ++index)
                                                                 columns.add_value(1152921504606847975ULL);
                                                               return columns;
                                                             },
                                                             {},
                                                             {},
                                                             {}};
    counter_observer.interval.set_nanos(1000000);
    value.observer(move(counter_observer));
  }
};

int main(int argc, char **argv) {
  int result = 0;
  try {
    auto value = Template::from<Thermal>("Thermal Cycle", "cycle");
    Plugin plugin("thermal", "1.0.0", {{"thermal", {move(value)}, "Thermal"}});
    plugin.serve(argc == 2 && string(argv[1]) == "--remote" ? "127.0.0.1:0" : "");
  } catch (const exception &error) {
    cerr << error.what() << '\n';
    result = 1;
  }
  ShutdownDefaultEventEngine();
  return result;
}
