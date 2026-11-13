// SPDX-License-Identifier: Apache-2.0
#include "dokime/dokime.hpp"
#include "thermal.pb.h"
#include <iostream>

struct Bench {
  double limit;
};

int main(int argc, char **argv) {
  try {
    dokime::Template value;
    *value.definition.mutable_metadata() = dokime::metadata("thermal.cycle", "Thermal Cycle");
    auto *parameter = value.definition.add_parameters();
    *parameter->mutable_metadata() = dokime::metadata("limit", "Limit");
    parameter->set_type(dokime::v1::VALUE_TYPE_NUMBER);
    parameter->mutable_expression()->mutable_literal()->set_number(85);
    value.streams = {"thermal.temperature"};
    value.initialize = [](dokime::Context &ctx) {
      ctx.set(std::make_shared<Bench>(Bench{ctx.parameters().at("limit").number()}));
      ctx.log(dokime::v1::SEVERITY_INFO, "Initialized");
    };
    value.steps.push_back(
        {dokime::metadata("ramp", "Ramp to Target"), [](dokime::Context &ctx) { ctx.progress(1, 2, "Ramp to Target"); }, 2});
    value.steps.push_back({dokime::metadata("hold", "Hold"), [](dokime::Context &ctx) {
                             ::dokime::plugin::host::v1::PromptsCreateRequest question;
                             question.set_prompt("door");
                             question.set_prompt_type("thermal.door");
                             auto answer = ctx.prompt(std::move(question));
                             thermal::v1::Door door;
                             door.set_closed(answer.action() == "yes" && answer.accepted());
                             ctx.record("thermal.door", door);
                             google::protobuf::Duration duration;
                             duration.set_seconds(2);
                             duration.set_nanos(500000000);
                             ctx.sleep(duration);
                             ctx.progress(2, 2, "Hold");
                           }});
    value.criteria.push_back({dokime::metadata("within_limit", "Temperature Within Limit"), [](dokime::Context &ctx) {
                                auto statistics = ctx.statistics("thermal.temperature", "celsius", "hold");
                                auto records = ctx.records("thermal.door", "hold");
                                auto door = harmos::decode<thermal::v1::Door>(records.last().payload());
                                return statistics.count() > 0 && statistics.max() <= ctx.get<Bench>()->limit && records.has_last() &&
                                       door.closed();
                              }});
    value.cleanup = [](dokime::Context &ctx) { ctx.log(dokime::v1::SEVERITY_INFO, "Cleaned Up"); };
    dokime::Stream<thermal::v1::Temperature> temperature("thermal.temperature");
    dokime::Observer<thermal::v1::Temperature> observer{temperature,
                                                        [](dokime::Context &ctx) {
                                                          thermal::v1::Temperature sample;
                                                          sample.set_celsius(ctx.get<Bench>()->limit - 1);
                                                          return sample;
                                                        },
                                                        {},
                                                        {}};
    observer.interval.set_seconds(1);
    dokime::Record door;
    *door.mutable_metadata() = dokime::metadata("thermal.door", "Door Check");
    dokime::Prompt prompt;
    *prompt.mutable_metadata() = dokime::metadata("thermal.door", "Door Check");
    prompt.set_message("Is the Door Closed?");
    prompt.set_archetype(dokime::v1::PROMPT_ARCHETYPE_CONFIRMATION);
    auto *yes = prompt.add_actions();
    yes->set_identifier("yes");
    yes->set_label("Yes");
    auto *no = prompt.add_actions();
    no->set_identifier("no");
    no->set_label("No");
    dokime::Plugin plugin("thermal", "1.0.0");
    plugin
        .requires_operation(
            harmos::Operation<::dokime::plugin::host::v1::PromptsCreateRequest, ::dokime::plugin::host::v1::PromptsCreateResponse>(
                "dokime.plugin.host.v1.Prompts/Create", harmos::wire::VISIBILITY_PUBLIC))
        .requires_operation(harmos::Operation<::dokime::plugin::host::v1::evidence::RecordsAppendRequest,
                                              ::dokime::plugin::host::v1::evidence::RecordsAppendResponse>(
            "dokime.plugin.host.v1.evidence.Records/Append", harmos::wire::VISIBILITY_PUBLIC))
        .requires_operation(harmos::Operation<::dokime::plugin::host::v1::evidence::RecordsListRequest,
                                              ::dokime::plugin::host::v1::evidence::RecordsListResponse>(
            "dokime.plugin.host.v1.evidence.Records/List", harmos::wire::VISIBILITY_PUBLIC))
        .requires_operation(harmos::Operation<::dokime::plugin::host::v1::EvidenceStatisticsRequest,
                                              ::dokime::plugin::host::v1::EvidenceStatisticsResponse>(
            "dokime.plugin.host.v1.Evidence/Statistics", harmos::wire::VISIBILITY_PUBLIC))
        .requires_operation(harmos::Operation<dokime::plugin::host::v1::executions::ProgressGetRequest,
                                              dokime::plugin::host::v1::executions::ProgressGetResponse>(
            "dokime.plugin.host.v1.executions.Progress/Get", harmos::wire::VISIBILITY_PUBLIC))
        .requires_operation(harmos::Operation<dokime::plugin::host::v1::executions::ProgressSetRequest,
                                              dokime::plugin::host::v1::executions::ProgressSetResponse>(
            "dokime.plugin.host.v1.executions.Progress/Set", harmos::wire::VISIBILITY_PUBLIC))
        .requires_operation(harmos::Operation<::dokime::plugin::host::v1::evidence::LogsAppendRequest,
                                              ::dokime::plugin::host::v1::evidence::LogsAppendResponse>(
            "dokime.plugin.host.v1.evidence.Logs/Append", harmos::wire::VISIBILITY_PUBLIC))
        .requires_operation(harmos::SLEEP)
        .stream(temperature)
        .observer(std::move(observer))
        .record<thermal::v1::Door>(std::move(door))
        .prompt(std::move(prompt))
        .suite({dokime::metadata("thermal", "Thermal"), {std::move(value)}});
    plugin.serve(argc == 2 && std::string(argv[1]) == "--remote" ? "127.0.0.1:0" : "");
  } catch (const std::exception &error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
