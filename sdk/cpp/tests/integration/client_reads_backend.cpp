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
#include "harmos/harmos.hpp"
#include <atomic>
#include <catch2/catch_test_macros.hpp>
#include <google/protobuf/empty.pb.h>
#include <google/protobuf/wrappers.pb.h>

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

class Schemas : public dokime::client::v1::executions::Schemas::Service {
public:
  google::protobuf::FileDescriptorSet types;
  std::atomic<int> calls = 0;
  Schemas() {
    google::protobuf::BoolValue::descriptor()->file()->CopyTo(types.add_file());
    google::protobuf::Empty::descriptor()->file()->CopyTo(types.add_file());
  }
  grpc::Status Get(grpc::ServerContext *context, const dokime::client::v1::executions::SchemasGetRequest *request,
                   dokime::client::v1::executions::SchemasGetResponse *reply) override {
    if (!context->client_metadata().contains("authorization"))
      return {grpc::StatusCode::UNAUTHENTICATED, "Get Schemas: Supply Credentials"};
    ++calls;
    auto cut = reply->mutable_resolved_at();
    cut->mutable_history()->set_store("schemas");
    cut->mutable_history()->set_epoch(request->name());
    cut->set_sequence(1);
    const bool first = request->page().page_token().empty();
    if (!first && (request->at().SerializeAsString() != cut->SerializeAsString() || request->page().page_token() != "second"))
      return {grpc::StatusCode::INVALID_ARGUMENT, "Get Schemas: Continue the Same Read Cut"};
    reply->set_digest(harmos::detail::sha256(types.SerializeAsString()));
    *reply->mutable_types()->add_file() = types.file(first ? 0 : 1);
    reply->set_next_page_token(first ? "second" : "");
    return grpc::Status::OK;
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
  grpc::ServerBuilder builder;
  builder.RegisterService(&backend);
  builder.RegisterService(&plugins);
  builder.RegisterService(&suites);
  builder.RegisterService(&log_service);
  builder.RegisterService(&schema_service);
  auto server = builder.BuildAndStart();
  check(bool(server), "Fake Backend Did Not Start");
  auto channel = server->InProcessChannel({});
  dokime::Client client(channel);

  // Pair and Read Its Identity Through the Generated Stub.
  grpc::ClientContext authenticated;
  authenticated.AddMetadata("authorization", "Bearer test");
  auto identity = client.status.get(authenticated, {});
  check(identity.status().identity().host_id() == "bench", "Backend Identity Was Lost");

  // Watch Evidence Until the Backend Closes the Stream.
  grpc::ClientContext watching;
  watching.AddMetadata("authorization", "Bearer test");
  auto logs = client.status.logs.watch(watching, {});
  harmos::v1::status::LogsWatchResponse frame;
  check(logs->Read(&frame) && frame.events(0).target() == "execution", "Log Was Lost");
  check(!logs->Read(&frame) && logs->Finish().ok(), "Log Stream Did Not Finish");

  // Read Without Pairing and Retain the Backend's Actionable Failure.
  grpc::ClientContext unpaired;
  bool refused = false;
  try {
    client.status.get(unpaired, {});
  } catch (const std::runtime_error &) {
    refused = true;
  }
  grpc::ClientContext checking_refusal;
  harmos::v1::StatusGetResponse refused_response;
  auto status_stub = client.stub<harmos::v1::Status>();
  auto failure = status_stub->Get(&checking_refusal, {}, &refused_response);
  check(refused && failure.error_code() == grpc::StatusCode::UNAUTHENTICATED, "Authentication Failure Was Hidden");

  // Read a Product Registry Through Its Top-Level Client Root.
  grpc::ClientContext reading_suite;
  dokime::client::v1::SuitesGetRequest suite_request;
  suite_request.set_name("sample");
  auto suite = client.suites.get(reading_suite, suite_request);
  check(suite.value().metadata().identifier() == suite_request.name(), "Suite Registry Read Was Lost");

  // Retain a Product Resource Failure Through the Root.
  grpc::ClientContext missing_suite;
  refused = false;
  try {
    client.suites.get(missing_suite, {});
  } catch (const std::runtime_error &) {
    refused = true;
  }
  check(refused, "Missing Suite Became an Empty Success");

  // Add a Remote Entry and Read Its Canonical Placement Through Generated Stubs.
  auto plugin_stub = client.stub<dokime::client::v1::Plugins>();
  dokime::client::v1::AddRemotePlugin request;
  request.set_url("http://127.0.0.1:7001");
  grpc::ClientContext adding;
  auto added = client.unary(*plugin_stub, &dokime::client::v1::Plugins::Stub::Add, request, adding);
  grpc::ClientContext listing;
  auto listed = client.unary(*plugin_stub, &dokime::client::v1::Plugins::Stub::List, dokime::client::v1::ListPlugins{}, listing);
  check(listed.plugins_size() == 1 && listed.plugins(0).entry() == added.entry(), "Plugin Entry Was Lost");
  check(listed.plugins(0).remote().url() == request.url(), "Remote Placement Was Lost");
  check(listed.plugins(0).state() == dokime::client::v1::PLUGIN_STATE_CONNECTING, "Plugin State Was Lost");

  // Retain the Generated Method's Server Refusal.
  grpc::ClientContext invalid;
  refused = false;
  try {
    client.unary(*plugin_stub, &dokime::client::v1::Plugins::Stub::Add, dokime::client::v1::AddRemotePlugin{}, invalid);
  } catch (const dokime::RpcError &error) {
    refused = error.status().error_code() == grpc::StatusCode::INVALID_ARGUMENT;
  }
  check(refused, "Plugin Refusal Was Hidden");
  // Fetch Complete Descriptor Pages, Then Reuse Them for Another Execution.
  dokime::FrozenSchemaCache schemas;
  dokime::client::v1::ExecutionDetail detail;
  detail.mutable_summary()->set_execution_id("first");
  detail.mutable_schemas()->set_digest(harmos::detail::sha256(schema_service.types.SerializeAsString()));
  const auto configure = [](grpc::ClientContext &context) { context.AddMetadata("authorization", "Bearer test"); };
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
