// SPDX-License-Identifier: Apache-2.0
#include "dokime/dokime.hpp"
#include "thermal.pb.h"
#include <grpc/event_engine/event_engine.h>
#include <iostream>

using dokime::Plugin;
using dokime::VALUE_TYPE_NUMBER;
using dokime::templates::Context;
using dokime::templates::Criterion;
using dokime::templates::metadata;
using dokime::templates::Observer;
using dokime::templates::Record;
using dokime::templates::Step;
using dokime::templates::Stream;
using dokime::templates::Template;
using dokime::templates::WindowObserver;
using dokime::templates::sampling::Window;
using google::protobuf::Duration;
using grpc_event_engine::experimental::ShutdownDefaultEventEngine;
using std::cerr;
using std::exception;
using std::move;
using std::string;
using std::uint64_t;
using thermal::v1::Counter;
using thermal::v1::CounterColumns;
using thermal::v1::Door;
using thermal::v1::Temperature;

class ThermalTestTemplate : public dokime::templates::TestTemplate {
public:
  inline static const Record<Door> door{"door", "Door Check"};
  inline static const Stream<Temperature> temperature_stream{"temperature", "Temperature [°C]"};
  inline static const Stream<Counter> counter_stream{"counter", "Exact Counter"};

  void initialize(Context &ctx) {
    limit_ = ctx.parameters().at("limit").number();
    ctx.log("Initialized");
  }
  void ramp(Context &ctx) {
    ctx.progress(1, 2, "Ramp to Target");
  }
  void hold(Context &ctx) {
    const auto answer = ctx.prompt("Is the Door Closed?", {"Yes", "No"});
    Door reading;
    reading.set_closed(answer == "Yes");
    ctx.record(door, reading);
    ctx.sleep(std::chrono::milliseconds(2500));
    ctx.progress(2, 2, "Hold");
  }
  bool within_limit(Context &ctx) {
    const auto statistics = ctx.statistics<&ThermalTestTemplate::hold>(temperature_stream, "celsius");
    const auto reading = ctx.records<&ThermalTestTemplate::hold>(door).last();
    const bool has_samples = statistics.count() > 0 && statistics.has_max() && statistics.max().has_number();
    return has_samples && statistics.max().number() <= limit_ && reading.closed();
  }
  void cleanup(Context &ctx) {
    ctx.log("Cleaned Up");
  }
  Temperature sample_temperature(Context &) {
    Temperature sample;
    sample.set_celsius(limit_ > 0 ? limit_ - 1 : 0);
    return sample;
  }
  CounterColumns sample_counter(Context &, const Window &window) {
    CounterColumns columns;
    for (uint64_t index = 0; index < window.count; ++index)
      columns.add_value(1152921504606847975ULL);
    return columns;
  }
  static Template definition() {
    auto value = Template::from<ThermalTestTemplate>("Thermal Cycle", "cycle");
    auto &parameter = value.parameters.emplace_back();
    *parameter.mutable_metadata() = metadata("limit", "Limit");
    parameter.set_type(VALUE_TYPE_NUMBER);
    parameter.mutable_expression()->mutable_literal()->set_number(85);
    value.record(door);
    value.steps.push_back(Step::method<&ThermalTestTemplate::ramp>("Ramp to Target", "ramp", 2));
    value.steps.push_back(Step::method<&ThermalTestTemplate::hold>("Hold", "hold"));
    value.criteria.push_back(Criterion::method<&ThermalTestTemplate::within_limit>("Temperature Within Limit", "within_limit"));
    Duration interval;
    interval.set_seconds(1);
    value.observer(Observer<Temperature>::method<&ThermalTestTemplate::sample_temperature>(temperature_stream, interval));

    // A 1 kHz Stream Answers Each Host Window at Once.
    interval.set_seconds(0);
    interval.set_nanos(1000000);
    value.observer(WindowObserver<Counter, CounterColumns>::method<&ThermalTestTemplate::sample_counter>(counter_stream, interval));
    return value;
  }

private:
  double limit_ = 0;
};

int main(int argc, char **argv) {
  int result = 0;
  try {
    auto value = ThermalTestTemplate::definition();
    Plugin plugin("thermal", "1.0.0", {{"thermal", {move(value)}, "Thermal"}});
    plugin.serve(argc == 2 && string(argv[1]) == "--remote" ? "127.0.0.1:0" : "");
  } catch (const exception &error) {
    cerr << error.what() << '\n';
    result = 1;
  }
  ShutdownDefaultEventEngine();
  return result;
}
