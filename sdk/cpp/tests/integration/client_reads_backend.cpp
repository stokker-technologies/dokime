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
#include "journey.grpc.pb.h"

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
  grpc::ServerBuilder builder;
  builder.RegisterService(&backend);
  auto server = builder.BuildAndStart();
  check(bool(server), "Fake Backend Did Not Start");
  auto channel = server->InProcessChannel({});
  dokime::Client client(channel);
  auto stub = client.stub<dokime::sdk::test::BackendJourney>();

  // Pair and Read Its Identity Through the Generated Stub.
  grpc::ClientContext authenticated;
  authenticated.AddMetadata("authorization", "Bearer test");
  auto identity = client.unary(*stub, &dokime::sdk::test::BackendJourney::Stub::Identity, dokime::v1::GetBackendIdentity{}, authenticated);
  check(identity.backend_id() == "bench" && identity.epoch() == "boot", "Backend Identity Was Lost");

  // Watch Evidence Until the Backend Closes the Stream.
  grpc::ClientContext watching;
  auto logs = stub->Logs(&watching, {});
  dokime::v1::LogFrame frame;
  check(logs->Read(&frame) && frame.target() == "execution", "Log Was Lost");
  check(!logs->Read(&frame) && logs->Finish().ok(), "Log Stream Did Not Finish");

  // Read Without Pairing and Retain the Backend's Actionable Failure.
  grpc::ClientContext unpaired;
  bool refused = false;
  try {
    client.unary(*stub, &dokime::sdk::test::BackendJourney::Stub::Identity, dokime::v1::GetBackendIdentity{}, unpaired);
  } catch (const dokime::RpcError &error) {
    refused = error.status().error_code() == grpc::StatusCode::UNAUTHENTICATED;
  }
  check(refused, "Authentication Failure Was Hidden");
  server->Shutdown();
}
