/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "dokime/dokime.hpp"
#include <iostream>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

struct Bench {
  double temperature = 42;
};

/* === Private Functions === */

/* === Entry Point === */

int main(int argc, char **argv) {
  try {
    // Compose the Cross-Language Evidence Journey.
    dokime::Template value;
    *value.definition.mutable_metadata() = dokime::metadata("sdk.journey", "SDK Journey");
    value.streams = {"sdk.temperature"};
    value.initialize = [](dokime::Context &ctx) { ctx.set(std::make_shared<Bench>()); };
    value.steps.push_back({dokime::metadata("capture", "Capture"), [](dokime::Context &ctx) {
                             dokime::v1::OperatorQuestion question;
                             question.set_prompt("door");
                             *question.mutable_definition()->mutable_metadata() = dokime::metadata("door", "Door");
                             question.mutable_definition()->set_message("Close the Door");
                             auto *action = question.mutable_definition()->add_actions();
                             action->set_identifier("continue");
                             action->set_label("Continue");
                             auto answer = ctx.prompt(std::move(question));
                             dokime::v1::Value record;
                             record.set_boolean(answer.accepted() && answer.action() == "continue");
                             ctx.record("sdk.door", record);
                             google::protobuf::Duration duration;
                             duration.set_seconds(1);
                             ctx.sleep(duration);
                             ctx.progress(1, 1, "Captured");
                           }});
    value.criteria.push_back({dokime::metadata("within_limit", "Within Limit"), [](dokime::Context &ctx) {
                                auto statistics = ctx.statistics("sdk.temperature", "number", "capture");
                                auto records = ctx.records("sdk.door");
                                auto door = harmos::decode<dokime::v1::Value>(records.last().payload());
                                return statistics.count() > 0 && statistics.max() == 42 && records.count() == 1 && door.boolean();
                              }});
    value.cleanup = [](dokime::Context &ctx) {
      ctx.get<Bench>();
      ctx.log(dokime::v1::SEVERITY_INFO, "Cleanup Complete");
    };
    dokime::Stream<dokime::v1::Value> temperature("sdk.temperature");
    dokime::Observer<dokime::v1::Value> observer{temperature,
                                                 [](dokime::Context &ctx) {
                                                   dokime::v1::Value sample;
                                                   sample.set_number(ctx.get<Bench>()->temperature);
                                                   return sample;
                                                 },
                                                 {},
                                                 {}};
    observer.interval.set_seconds(1);
    dokime::Record door;
    *door.mutable_metadata() = dokime::metadata("sdk.door", "Door");
    dokime::Plugin plugin("sdk.cpp", "1.0.0");
    plugin.context_operations()
        .stream(temperature)
        .observer(std::move(observer))
        .record<dokime::v1::Value>(std::move(door))
        .suite({dokime::metadata("sdk", "SDK"), {std::move(value)}});

    // Serve Under Managed or Remote Placement.
    if (argc == 3 && std::string(argv[1]) == "--address")
      plugin.serve(argv[2]);
    else if (argc == 1)
      plugin.serve();
    else
      throw std::invalid_argument("Usage: dokime-system-suite [--address ADDRESS]");
  } catch (const std::exception &error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
