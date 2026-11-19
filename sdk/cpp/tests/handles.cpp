/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
#include <atomic>
#include <future>

// Third Party Imports
#include <catch2/catch_test_macros.hpp>

// Local Imports
#include "dokime/client.hpp"
#include "dokime/host/v1/cases.grpc.pb.h"
#include "dokime/host/v1/cycles.grpc.pb.h"
#include "dokime/host/v1/evidence/records.grpc.pb.h"
#include "dokime/host/v1/evidence/samples.grpc.pb.h"
#include "dokime/host/v1/executions.grpc.pb.h"
#include "dokime/host/v1/executions/progress.grpc.pb.h"
#include "dokime/host/v1/executions/schemas.grpc.pb.h"
#include "dokime/host/v1/prompts.grpc.pb.h"
#include "dokime/host/v1/prompts/answers.grpc.pb.h"

using grpc::ServerBuilder;
using grpc::ServerContext;
using grpc::ServerWriter;
using grpc::Status;
using grpc::StatusCode;
using std::string;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace {
class CasesService final : public dokime::host::v1::Cases::Service {
public:
  dokime::host::v1::CasesCreateRequest created;
  Status Create(ServerContext *, const dokime::host::v1::CasesCreateRequest *request,
                dokime::host::v1::CasesCreateResponse *response) override {
    created = *request;
    auto *value = response->add_cases();
    value->set_case_id("case");
    value->set_suite_id(request->plan().selections(0).suite_id());
    value->set_template_id(request->plan().selections(0).template_id());
    value->mutable_metadata()->set_label("Confirmation");
    return Status::OK;
  }
  Status Get(ServerContext *context, const dokime::host::v1::CasesGetRequest *request,
             dokime::host::v1::CasesGetResponse *response) override {
    if (request->name() == "missing") {
      harmos::model::Failure failure;
      failure.set_classification(harmos::CLASSIFICATION_REFUSED);
      failure.set_code("case_absent");
      failure.set_operation("cases.get");
      failure.set_message("Case Is Missing");
      failure.set_action("Materialize a Case Before Reading It");
      failure.mutable_cause()->set_code("selection_absent");
      (*failure.mutable_details())["case"] = request->name();
      failure.set_diagnostics("test-diagnostics");
      context->AddTrailingMetadata("grpc-status-details-bin", failure.SerializeAsString());
      return {StatusCode::FAILED_PRECONDITION, "Case Is Missing"};
    }
    auto *value = response->mutable_value()->mutable_summary();
    value->set_case_id(request->name());
    value->set_suite_id("capture");
    value->set_template_id("capture.confirmation");
    response->mutable_resolved_at()->add_positions()->set_sequence(7);
    return Status::OK;
  }
  Status Parameters(ServerContext *, const dokime::host::v1::CasesParametersRequest *request,
                    dokime::host::v1::CasesParametersResponse *response) override {
    if (request->at().positions(0).sequence() != 7)
      return {StatusCode::INVALID_ARGUMENT, "Retain Case Cut"};
    auto *value = response->add_parameters();
    value->set_name("limit");
    value->mutable_value()->set_unsigned_integer(UINT64_MAX);
    return Status::OK;
  }
};
class CyclesService final : public dokime::host::v1::Cycles::Service {
public:
  bool drift = false;
  bool repeat = false;
  dokime::host::v1::CyclesCreateRequest created;
  Status Create(ServerContext *, const dokime::host::v1::CyclesCreateRequest *request,
                dokime::host::v1::CyclesCreateResponse *response) override {
    created = *request;
    auto *value = response->mutable_cycle();
    value->set_cycle_id("cycle");
    *value->mutable_metadata() = request->metadata();
    value->set_phase(dokime::host::v1::CYCLE_PHASE_ACTIVE);
    return Status::OK;
  }
  Status List(ServerContext *, const dokime::host::v1::CyclesListRequest *request,
              dokime::host::v1::CyclesListResponse *response) override {
    const bool first = request->page().page_token().empty();
    if (!first && request->at().positions(0).sequence() != 9)
      return {StatusCode::INVALID_ARGUMENT, "Retain Cycle Cut"};
    response->mutable_resolved_at()->add_positions()->set_sequence(!first && drift ? 10 : 9);
    response->add_cycles()->set_cycle_id(first ? "first" : "second");
    response->set_next_page_token(first || repeat ? "next" : "");
    return Status::OK;
  }
  Status Seal(ServerContext *, const dokime::host::v1::CyclesSealRequest *, dokime::host::v1::CyclesSealResponse *response) override {
    response->set_seal("retained-seal");
    return Status::OK;
  }
};
class ExecutionsService final : public dokime::host::v1::Executions::Service {
public:
  std::atomic<int> starts = 0;
  std::atomic<int> stops = 0;
  std::atomic<int> cancels = 0;
  bool wrong_case = false;
  bool refuse_start = false;
  dokime::host::v1::ExecutionDetail value;
  dokime::host::v1::ExecutionsCreateRequest created;
  Status Create(ServerContext *, const dokime::host::v1::ExecutionsCreateRequest *request,
                dokime::host::v1::ExecutionsCreateResponse *response) override {
    created = *request;
    value.set_cycle_id(request->parent());
    value.mutable_summary()->set_execution_id(request->executions(0).execution_id());
    value.mutable_summary()->set_case_id(wrong_case ? "wrong" : request->executions(0).case_id());
    value.mutable_admitted_suite()->set_identifier("capture");
    value.mutable_admitted_template()->set_identifier("capture.confirmation");
    *response->add_executions() = value;
    return Status::OK;
  }
  Status Get(ServerContext *, const dokime::host::v1::ExecutionsGetRequest *, dokime::host::v1::ExecutionsGetResponse *response) override {
    *response->mutable_execution() = value;
    return Status::OK;
  }
  Status Start(ServerContext *, const dokime::host::v1::ExecutionsStartRequest *request,
               dokime::host::v1::ExecutionsStartResponse *response) override {
    if (refuse_start)
      return {StatusCode::FAILED_PRECONDITION, "Resolve Execution Admission Before Starting"};
    if (request->mode() != dokime::host::v1::EXECUTION_MODE_AUTOMATIC)
      return {StatusCode::INVALID_ARGUMENT, "Select Automatic Mode"};
    ++starts;
    *response->mutable_execution() = value;
    return Status::OK;
  }
  Status Stop(ServerContext *, const dokime::host::v1::ExecutionsStopRequest *,
              dokime::host::v1::ExecutionsStopResponse *response) override {
    ++stops;
    *response->mutable_execution() = value;
    return Status::OK;
  }
  Status Cancel(ServerContext *, const dokime::host::v1::ExecutionsCancelRequest *,
                dokime::host::v1::ExecutionsCancelResponse *response) override {
    ++cancels;
    *response->mutable_execution() = value;
    return Status::OK;
  }
  Status Seal(ServerContext *, const dokime::host::v1::ExecutionsSealRequest *,
              dokime::host::v1::ExecutionsSealResponse *response) override {
    response->mutable_outcome()->mutable_evaluated()->set_status(dokime::host::v1::RESULT_STATUS_PASSED);
    return Status::OK;
  }
};
class ProgressService final : public dokime::host::v1::executions::Progress::Service {
public:
  bool terminal = true;
  Status Watch(ServerContext *, const dokime::host::v1::executions::ProgressWatchRequest *,
               ServerWriter<dokime::host::v1::executions::ProgressWatchResponse> *writer) override {
    dokime::host::v1::executions::ProgressWatchResponse frame;
    frame.set_phase("Stepping");
    writer->Write(frame);
    if (terminal) {
      frame.set_phase("Finished");
      frame.set_sealed(true);
      writer->Write(frame);
    }
    return Status::OK;
  }
};
class SchemasService final : public dokime::host::v1::executions::Schemas::Service {
public:
  std::atomic<int> reads = 0;
  google::protobuf::FileDescriptorSet types;
  dokime::host::v1::FrozenSchemas bindings;
  SchemasService() {
    google::protobuf::BoolValue::descriptor()->file()->CopyTo(types.add_file());
    bindings.set_digest(harmos::plugin::detail::sha256(types.SerializeAsString()));
    auto *record = bindings.add_records();
    record->set_record_type("capture.confirmation.confirmation");
    record->set_message_name(google::protobuf::BoolValue::descriptor()->full_name());
    auto *stream = bindings.add_streams();
    stream->set_stream("capture.confirmation.samples");
    stream->set_message_name(google::protobuf::BoolValue::descriptor()->full_name());
  }
  Status Get(ServerContext *, const dokime::host::v1::executions::SchemasGetRequest *,
             dokime::host::v1::executions::SchemasGetResponse *response) override {
    ++reads;
    response->set_digest(bindings.digest());
    *response->mutable_types() = types;
    *response->mutable_records() = bindings.records();
    *response->mutable_streams() = bindings.streams();
    response->mutable_resolved_at()->set_sequence(1);
    return Status::OK;
  }
};
class RecordsService final : public dokime::host::v1::evidence::Records::Service {
public:
  string execution;
  bool outside = false;
  Status List(ServerContext *, const dokime::host::v1::evidence::RecordsListRequest *request,
              dokime::host::v1::evidence::RecordsListResponse *response) override {
    if (request->scope().execution() != execution)
      return {StatusCode::INVALID_ARGUMENT, "Select Bound Execution"};
    if (request->has_at() && request->at().positions(0).sequence() != 11)
      return {StatusCode::INVALID_ARGUMENT, "Retain Evidence Cut"};
    response->mutable_resolved_at()->add_positions()->set_sequence(11);
    response->set_count(2);
    auto *record = response->add_records();
    record->set_record_id("record");
    record->set_record_type(request->record_type());
    record->mutable_scope()->set_execution_id(outside ? "other" : execution);
    google::protobuf::BoolValue value;
    value.set_value(true);
    record->set_payload(value.SerializeAsString());
    *response->mutable_last() = *record;
    if (request->page().page_token().empty())
      response->set_next_page_token("next");
    return Status::OK;
  }
};
class SamplesService final : public dokime::host::v1::evidence::Samples::Service {
public:
  string execution;
  Status List(ServerContext *, const dokime::host::v1::evidence::SamplesListRequest *request,
              dokime::host::v1::evidence::SamplesListResponse *response) override {
    if (request->scope().execution_id() != execution || request->streams_size() != 1)
      return {StatusCode::INVALID_ARGUMENT, "Select Bound Stream"};
    if (request->has_at() && request->at().positions(0).sequence() != 11)
      return {StatusCode::INVALID_ARGUMENT, "Retain Evidence Cut"};
    response->mutable_resolved_at()->add_positions()->set_sequence(11);
    auto *sample = response->add_samples()->mutable_raw();
    sample->set_sample_id("sample");
    sample->set_stream(request->streams(0));
    sample->mutable_scope()->set_execution_id(execution);
    google::protobuf::BoolValue value;
    value.set_value(true);
    sample->set_payload(value.SerializeAsString());
    if (request->page().page_token().empty())
      response->set_next_page_token("next");
    return Status::OK;
  }
};
class PromptsService final : public dokime::host::v1::Prompts::Service {
  Status Watch(ServerContext *, const dokime::host::v1::PromptsWatchRequest *,
               ServerWriter<dokime::host::v1::PromptsWatchResponse> *writer) override {
    dokime::host::v1::PromptsWatchResponse frame;
    auto *prompt = frame.add_prompts()->mutable_prompt();
    prompt->set_prompt_id("question");
    prompt->mutable_scope()->set_execution_id("execution");
    prompt->mutable_definition()->set_message("Continue?");
    auto *action = prompt->mutable_definition()->add_actions();
    action->set_identifier("yes");
    action->set_label("Yes");
    writer->Write(frame);
    writer->Write(frame);
    return Status::OK;
  }
};
class AnswersService final : public dokime::host::v1::prompts::Answers::Service {
public:
  std::promise<void> answered;
  std::atomic<int> count = 0;
  Status Create(ServerContext *, const dokime::host::v1::prompts::AnswersCreateRequest *request,
                dokime::host::v1::prompts::AnswersCreateResponse *) override {
    if (request->parent() != "question" || request->action() != "yes")
      return {StatusCode::INVALID_ARGUMENT, "Answer the Bound Question"};
    if (++count == 1)
      answered.set_value();
    return Status::OK;
  }
};
class Host {
public:
  CasesService cases;
  CyclesService cycles;
  ExecutionsService executions;
  ProgressService progress;
  SchemasService schemas;
  RecordsService records;
  SamplesService samples;
  PromptsService prompts;
  AnswersService answers;
  std::unique_ptr<grpc::Server> server;
  string endpoint;
  Host() {
    ServerBuilder builder;
    int port = 0;
    builder.AddListeningPort("127.0.0.1:0", grpc::InsecureServerCredentials(), &port);
    builder.RegisterService(&cases);
    builder.RegisterService(&cycles);
    builder.RegisterService(&executions);
    builder.RegisterService(&progress);
    builder.RegisterService(&schemas);
    builder.RegisterService(&records);
    builder.RegisterService(&samples);
    builder.RegisterService(&prompts);
    builder.RegisterService(&answers);
    *executions.value.mutable_schemas() = schemas.bindings;
    server = builder.BuildAndStart();
    if (!server)
      throw std::runtime_error("Start Test Host: Open Its Listener");
    endpoint = "http://127.0.0.1:" + std::to_string(port);
  }
  ~Host() {
    server->Shutdown();
  }
};
} // namespace

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

TEST_CASE("Bound Handles Compose Materialization, Lazy Run and Seal") {
  Host host;
  dokime::ConnectionSettings settings;
  settings.remote = host.endpoint;
  auto client = dokime::connect(settings);
  auto cycle = client.cycle("Capture", "cycle-request");
  const auto cases = client.cases().materialize("capture.confirmation", "example", "capture", {{"limit", {dokime::Scalar{UINT64_MAX}}}},
                                                {{"source", std::nullopt}}, "case-request");
  REQUIRE(cases.size() == 1);
  REQUIRE(host.cycles.created.origins().empty());
  REQUIRE_FALSE(host.cycles.created.continue_automatically());
  REQUIRE(host.cases.created.plan().selections(0).parameters(0).value().values(0).unsigned_integer() == UINT64_MAX);
  REQUIRE(host.cases.created.plan().selections(0).files(0).value().kind().has_clear());
  auto test_case = client.cases().get(cases.front().identifier());
  REQUIRE(std::get<std::uint64_t>(test_case.parameters().at("limit").value) == UINT64_MAX);
  auto execution = cycle.execution(test_case, "execution-request");
  const auto first_id = execution.identifier();
  REQUIRE(cycle.execution(test_case, "execution-request").identifier() == first_id);
  REQUIRE(cycle.execution(test_case, "another-request").identifier() != first_id);
  execution = cycle.execution(test_case, "execution-request");
  REQUIRE(execution.cycle().identifier() == cycle.identifier());
  REQUIRE(execution.test_case().identifier() == test_case.identifier());
  auto progress = execution.run();
  REQUIRE(host.executions.starts == 0);
  REQUIRE_FALSE(progress.next()->sealed);
  REQUIRE(host.executions.starts == 1);
  REQUIRE(progress.next()->sealed);
  REQUIRE_FALSE(progress.next());
  REQUIRE(execution.seal().status == dokime::ResultStatus::Passed);
  REQUIRE(execution.seal().status == dokime::ResultStatus::Passed);
  auto reopened = client.executions().get(execution.identifier());
  REQUIRE(reopened.cycle().identifier() == cycle.identifier());
  REQUIRE(reopened.test_case().template_id() == "capture.confirmation");
  auto watch = execution.watch();
  REQUIRE_FALSE(watch.next()->sealed);
  watch.close();
  REQUIRE(host.executions.starts == 1);
  REQUIRE(host.executions.stops == 0);
  REQUIRE(host.executions.cancels == 0);
  execution.stop();
  execution.cancel();
  REQUIRE(host.executions.stops == 1);
  REQUIRE(host.executions.cancels == 1);
  cycle.seal();
  REQUIRE(cycle.phase() == "Sealed");
  client.close();
  client.close();
  REQUIRE_THROWS_AS(execution.refresh(), dokime::Error);
}
TEST_CASE("Bound Lists Retain Cuts and Refuse Repeated Tokens") {
  Host host;
  dokime::ConnectionSettings settings;
  settings.remote = host.endpoint;
  auto client = dokime::connect(settings);
  const auto cycles = client.cycles().list("", 1);
  REQUIRE(cycles.size() == 2);
  REQUIRE(cycles[0].identifier() == "first");
  REQUIRE(cycles[1].identifier() == "second");
  host.cycles.drift = true;
  REQUIRE_THROWS_AS(client.cycles().list("", 1), std::runtime_error);
  host.cycles.drift = false;
  host.cycles.repeat = true;
  REQUIRE_THROWS_AS(client.cycles().list("", 1), std::runtime_error);
}
TEST_CASE("Bound Calls Preserve Structured Host Failures") {
  Host host;
  dokime::ConnectionSettings settings;
  settings.remote = host.endpoint;
  auto client = dokime::connect(settings);
  try {
    client.cases().get("missing");
    FAIL("Read Missing Case Must Fail");
  } catch (const dokime::Error &error) {
    REQUIRE(error.failure.classification == "Refused");
    REQUIRE(error.failure.code == "case_absent");
    REQUIRE(error.failure.operation == "cases.get");
    REQUIRE(error.failure.cause);
    REQUIRE(error.failure.cause->code == "selection_absent");
    REQUIRE(error.failure.details.at("case") == "missing");
    REQUIRE(error.failure.diagnostics == "test-diagnostics");
  }
}
TEST_CASE("Bound Readers Decode Frozen Types and Retain Evidence Cuts") {
  Host host;
  dokime::ConnectionSettings settings;
  settings.remote = host.endpoint;
  auto client = dokime::connect(settings);
  auto cycle = client.cycle("Capture");
  const auto test_case = client.cases().get("case");
  auto execution = cycle.execution(test_case);
  host.records.execution = execution.identifier();
  host.samples.execution = execution.identifier();
  const dokime::templates::Record<bool> confirmation{"confirmation"};
  auto records = execution.records(confirmation);
  REQUIRE(records.count() == 2);
  REQUIRE(records.last());
  REQUIRE(records.read(2).size() == 2);
  auto record_pages = records.pages(1);
  REQUIRE(record_pages.next()->size() == 1);
  REQUIRE(record_pages.next()->size() == 1);
  REQUIRE_FALSE(record_pages.next());
  REQUIRE(host.schemas.reads == 1);
  const dokime::templates::Stream<bool> stream{"samples"};
  auto samples = execution.samples(stream);
  const auto rows = samples.collect();
  REQUIRE(rows.size() == 2);
  REQUIRE(rows.front().value);
  REQUIRE(rows.front().stream == "capture.confirmation.samples");
  REQUIRE(host.schemas.reads == 1);
  REQUIRE_THROWS_AS(samples.collect(1), std::invalid_argument);
  auto wrong_type = execution.records<string>("capture.confirmation.confirmation");
  REQUIRE_THROWS_AS(wrong_type.last(), std::invalid_argument);
  host.records.outside = true;
  REQUIRE_THROWS_AS(records.last(), std::runtime_error);
}
TEST_CASE("The Client Answers Each Pending Prompt Once") {
  Host host;
  dokime::ConnectionSettings settings;
  settings.remote = host.endpoint;
  auto client = dokime::connect(settings);
  auto answered = host.answers.answered.get_future();
  client.on_prompt([](const dokime::PromptQuestion &question) {
    if (question.identifier != "question" || question.execution != "execution" || question.choices != std::vector<string>{"Yes"})
      throw std::invalid_argument("Answer Test Prompt: Retain the Host Question and Choices");
    return dokime::PromptAnswer{"Yes"};
  });
  answered.get();
  client.close();
  REQUIRE(host.answers.count == 1);
}
TEST_CASE("Bound Executions Refuse Foreign Cases and Incomplete Observation") {
  Host host;
  dokime::ConnectionSettings settings;
  settings.remote = host.endpoint;
  auto client = dokime::connect(settings);
  auto other = dokime::connect(settings);
  auto cycle = client.cycle("Capture");
  const auto foreign = other.cases().get("case");
  REQUIRE_THROWS_AS(cycle.execution(foreign), std::invalid_argument);
  const auto test_case = client.cases().get("case");
  host.executions.wrong_case = true;
  REQUIRE_THROWS_AS(cycle.execution(test_case), std::runtime_error);
  host.executions.wrong_case = false;
  auto execution = cycle.execution(test_case);
  host.progress.terminal = false;
  auto watch = execution.watch();
  REQUIRE(watch.next());
  REQUIRE_THROWS_AS(watch.next(), std::runtime_error);
  REQUIRE(host.executions.starts == 0);
  host.executions.refuse_start = true;
  auto run = execution.run();
  REQUIRE_THROWS_AS(run.next(), dokime::Error);
  host.executions.refuse_start = false;
  REQUIRE_FALSE(run.next());
}
