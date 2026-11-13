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

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

int main() {
  // Start a Backend in This Process.
  Backend backend;
  Plugins plugins;
  Logs log_service;
  grpc::ServerBuilder builder;
  builder.RegisterService(&backend);
  builder.RegisterService(&plugins);
  builder.RegisterService(&log_service);
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
  check(refused, "Authentication Failure Was Hidden");

  // Add a Remote Entry and Read Its Canonical Placement Through Generated Stubs.
  auto plugin_stub = client.stub<dokime::v1::Plugins>();
  dokime::v1::AddRemotePlugin request;
  request.set_url("http://127.0.0.1:7001");
  grpc::ClientContext adding;
  auto added = client.unary(*plugin_stub, &dokime::v1::Plugins::Stub::Add, request, adding);
  grpc::ClientContext listing;
  auto listed = client.unary(*plugin_stub, &dokime::v1::Plugins::Stub::List, dokime::v1::ListPlugins{}, listing);
  check(listed.plugins_size() == 1 && listed.plugins(0).entry() == added.entry(), "Plugin Entry Was Lost");
  check(listed.plugins(0).remote().url() == request.url(), "Remote Placement Was Lost");
  check(listed.plugins(0).state() == dokime::v1::PLUGIN_STATE_CONNECTING, "Plugin State Was Lost");

  // Retain the Generated Method's Server Refusal.
  grpc::ClientContext invalid;
  refused = false;
  try {
    client.unary(*plugin_stub, &dokime::v1::Plugins::Stub::Add, dokime::v1::AddRemotePlugin{}, invalid);
  } catch (const dokime::RpcError &error) {
    refused = error.status().error_code() == grpc::StatusCode::INVALID_ARGUMENT;
  }
  check(refused, "Plugin Refusal Was Hidden");
  server->Shutdown();
}
