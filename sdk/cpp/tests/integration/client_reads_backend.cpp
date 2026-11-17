/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "../support/backend.hpp"
#include "../support/check.hpp"
#include "dokime/client.hpp"
#include "dokime/host/v1/evidence.grpc.pb.h"
#include "dokime/host/v1/executions/schemas.grpc.pb.h"
#include "dokime/host/v1/plots/snapshots.grpc.pb.h"
#include "harmos/harmos.hpp"
#include <atomic>
#include <catch2/catch_test_macros.hpp>
#include <google/protobuf/empty.pb.h>
#include <google/protobuf/wrappers.pb.h>

using dokime::Client;
using dokime::EvidenceScope;
using dokime::client::EvidenceStatisticsOptions;
using dokime::client::ExecutionDetail;
using dokime::client::FrozenSchemaCache;
using dokime::client::PlotRequest;
using dokime::client::StatusGetOptions;
using dokime::client::StatusLogsWatchOptions;
using dokime::host::v1::EvidenceStatisticsRequest;
using dokime::host::v1::EvidenceStatisticsResponse;
using dokime::host::v1::executions::SchemasGetRequest;
using dokime::host::v1::executions::SchemasGetResponse;
using dokime::host::v1::plots::SnapshotsListRequest;
using dokime::host::v1::plots::SnapshotsListResponse;
using google::protobuf::BoolValue;
using google::protobuf::Empty;
using google::protobuf::FileDescriptorSet;
using grpc::ClientContext;
using grpc::ServerBuilder;
using grpc::ServerContext;
using grpc::Status;
using grpc::StatusCode;
using harmos::client::PLUGIN_STATE_CONNECTING;
using harmos::client::PluginPlacement;
using harmos::client::PluginValue;

using std::atomic;
using std::make_shared;
using std::runtime_error;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

class Evidence final : public dokime::host::v1::Evidence::Service {
  Status Statistics(ServerContext *, const EvidenceStatisticsRequest *request, EvidenceStatisticsResponse *reply) override {
    if (request->scope().execution_id() != "execution" || request->stream() != "measurements")
      return {StatusCode::INVALID_ARGUMENT, "Read Statistics: Select Execution Measurements"};
    auto *statistics = reply->mutable_statistics();
    statistics->set_count(2);
    statistics->mutable_sum()->mutable_exact()->set_numerator("36893488147419103229");
    statistics->mutable_sum()->mutable_exact()->set_denominator("1");
    statistics->mutable_mean()->mutable_exact()->set_numerator("36893488147419103229");
    statistics->mutable_mean()->mutable_exact()->set_denominator("2");
    statistics->mutable_max()->set_unsigned_integer(18446744073709551615ULL);
    return Status::OK;
  }
};

class Snapshots final : public dokime::host::v1::plots::Snapshots::Service {
  Status List(ServerContext *, const SnapshotsListRequest *request, SnapshotsListResponse *reply) override {
    if (request->requests_size() != 1 || request->requests(0).stream() != "measurements")
      return {StatusCode::INVALID_ARGUMENT, "Read Snapshots: Select Measurements"};
    auto *payload = reply->add_envelopes()->mutable_payload();
    auto *total = payload->mutable_frame()->add_series()->add_totals()->mutable_exact();
    total->set_numerator("36893488147419103229");
    total->set_denominator("1");
    auto *mean = payload->add_summaries()->mutable_mean()->mutable_exact();
    mean->set_numerator("36893488147419103229");
    mean->set_denominator("2");
    return Status::OK;
  }
};

class Schemas : public dokime::host::v1::executions::Schemas::Service {
public:
  FileDescriptorSet types;
  atomic<int> calls = 0;
  Schemas() {
    BoolValue::descriptor()->file()->CopyTo(types.add_file());
    Empty::descriptor()->file()->CopyTo(types.add_file());
  }
  Status Get(ServerContext *context, const SchemasGetRequest *request, SchemasGetResponse *reply) override {
    if (!context->client_metadata().contains("authorization"))
      return {StatusCode::UNAUTHENTICATED, "Get Schemas: Supply Credentials"};
    ++calls;
    auto cut = reply->mutable_resolved_at();
    cut->mutable_history()->set_store("schemas");
    cut->mutable_history()->set_epoch(request->name());
    cut->set_sequence(1);
    const bool first = request->page().page_token().empty();
    if (!first && (request->at().SerializeAsString() != cut->SerializeAsString() || request->page().page_token() != "second"))
      return {StatusCode::INVALID_ARGUMENT, "Get Schemas: Continue the Same Read Cut"};
    reply->set_digest(harmos::plugin::detail::sha256(types.SerializeAsString()));
    *reply->mutable_types()->add_file() = types.file(first ? 0 : 1);
    reply->set_next_page_token(first ? "second" : "");
    return Status::OK;
  }
};

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

TEST_CASE("Client Reads the Backend") {
  // Start a Backend in This Process.
  Backend backend;
  Plugins plugins;
  Suites suites;
  Logs log_service;
  Schemas schema_service;
  Evidence evidence_service;
  Snapshots snapshot_service;
  ServerBuilder builder;
  builder.RegisterService(&backend);
  builder.RegisterService(&plugins);
  builder.RegisterService(&suites);
  builder.RegisterService(&log_service);
  builder.RegisterService(&schema_service);
  builder.RegisterService(&evidence_service);
  builder.RegisterService(&snapshot_service);
  auto server = builder.BuildAndStart();
  check(bool(server), "Fake Backend Did Not Start");
  auto channel = server->InProcessChannel({});
  Client client(channel);

  // Pair and Read Its Identity Through the Generated Stub.
  StatusGetOptions authenticated;
  authenticated.context = make_shared<ClientContext>();
  authenticated.context->AddMetadata("authorization", "Bearer test");
  auto identity = client.status.get(authenticated);
  check(identity.status().identity().host_id() == "bench", "Backend Identity Was Lost");

  // Watch Evidence Until the Backend Closes the Stream.
  StatusLogsWatchOptions watching;
  watching.context = make_shared<ClientContext>();
  watching.context->AddMetadata("authorization", "Bearer test");
  auto logs = client.status.logs.watch(1024 * 1024, 1024, watching);
  const auto frame = logs.next();
  check(frame && frame->events(0).target() == "execution", "Log Was Lost");
  check(!logs.next(), "Log Stream Did Not Finish");

  // Read Without Pairing and Retain the Backend Refusal.
  bool refused = false;
  try {
    client.status.get();
  } catch (const runtime_error &) {
    refused = true;
  }
  check(refused, "Authentication Failure Was Hidden");

  // Read a Product Registry Through Its Top-Level Client Root.
  auto suite = client.suites.get("sample", "", 0);
  check(suite.value().metadata().identifier() == "sample", "Suite Registry Read Was Lost");

  // Retain a Product Resource Failure Through the Root.
  refused = false;
  try {
    client.suites.get("", "", 0);
  } catch (const runtime_error &) {
    refused = true;
  }
  check(refused, "Missing Suite Became an Empty Success");

  // Create a Remote Entry and Read Its Canonical Library Placement.
  PluginPlacement placement;
  placement.mutable_remote()->set_url("http://127.0.0.1:7001");
  PluginValue value;
  value.set_enabled(true);
  auto added = client.plugins.create("sample", placement, value);
  auto listed = client.plugins.list();
  check(listed.entries_size() == 1 && listed.entries(0).name() == added.entry().name(), "Plugin Entry Was Lost");
  check(listed.entries(0).placement().remote().url() == placement.remote().url(), "Remote Placement Was Lost");
  check(listed.entries(0).state() == PLUGIN_STATE_CONNECTING, "Plugin State Was Lost");

  // Retain the Generated Method's Server Refusal.
  refused = false;
  try {
    client.plugins.create("invalid", {}, value);
  } catch (const runtime_error &) {
    refused = true;
  }
  check(refused, "Plugin Refusal Was Hidden");
  // Read Exact Statistics and Plot Values Through the Generated Client.
  EvidenceScope scope;
  scope.set_execution_id("execution");
  EvidenceStatisticsOptions statistics_options;
  statistics_options.scope = scope;
  const auto measured = client.evidence.statistics("measurements", {}, {}, {}, "", "", statistics_options);
  const auto &statistics = measured.statistics();
  check(statistics.has_sum() && statistics.sum().has_exact() && statistics.sum().exact().numerator() == "36893488147419103229",
        "Statistics Sum Was Coerced");
  check(statistics.mean().has_exact() && statistics.mean().exact().denominator() == "2", "Statistics Mean Was Coerced");
  check(statistics.max().unsigned_integer() == 18446744073709551615ULL && !statistics.has_min(), "Typed Statistics Presence Was Lost");
  PlotRequest plot_request;
  plot_request.set_stream("measurements");
  const auto plots = client.plots.snapshots.list({plot_request});
  const auto &payload = plots.envelopes(0).payload();
  check(payload.frame().series(0).totals(0).exact().numerator() == statistics.sum().exact().numerator(), "Category Total Was Coerced");
  check(payload.summaries(0).mean().has_exact() && payload.summaries(0).mean().exact().denominator() == "2" &&
            !payload.summaries(0).has_std(),
        "Typed Plot Summary Presence Was Lost");

  // Fetch Complete Descriptor Pages, Then Reuse Them for Another Execution.
  FrozenSchemaCache schemas;
  ExecutionDetail detail;
  detail.mutable_summary()->set_execution_id("first");
  detail.mutable_schemas()->set_digest(harmos::plugin::detail::sha256(schema_service.types.SerializeAsString()));
  const auto configure = [](ClientContext &context) {
    context.AddMetadata("authorization", "Bearer test");
  };
  const auto types = schemas.get(client, detail, configure);
  check(types->file_size() == 2 && types->file(0).name() == "google/protobuf/wrappers.proto" &&
            types->file(1).name() == "google/protobuf/empty.proto",
        "Descriptor Pages Were Lost");
  const auto calls = schema_service.calls.load();
  check(calls == types->file_size(), "Descriptors Were Fetched More Than Once");
  detail.mutable_summary()->set_execution_id("second");
  const auto reused = schemas.get(client, detail, configure);
  check(types == reused && schema_service.calls.load() == calls, "Equal Descriptor Digests Were Fetched Again");
  server->Shutdown();
}
