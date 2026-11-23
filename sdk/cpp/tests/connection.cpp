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
#include <filesystem>
#include <fstream>

// Third Party Imports
#include <catch2/catch_test_macros.hpp>

// Local Imports
#include "dokime/client.hpp"
#include "dokime/host/v1/status.grpc.pb.h"

using harmos::OperationError;
using std::string;
using std::filesystem::path;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

namespace {
class IdentityService final : public dokime::host::v1::Status::CallbackService {
public:
  string identity;
  string authorization;
  grpc::ServerUnaryReactor *Get(grpc::CallbackServerContext *context, const harmos::host::v1::StatusGetRequest *,
                                dokime::host::v1::StatusGetResponse *response) override {
    const auto found = context->client_metadata().find("authorization");
    if (found != context->client_metadata().end())
      authorization.assign(found->second.data(), found->second.length());
    response->mutable_status()->mutable_identity()->set_host_id(identity);
    auto *reactor = context->DefaultReactor();
    reactor->Finish(grpc::Status::OK);
    return reactor;
  }
};
class IdentityHost {
public:
  IdentityService status;
  std::unique_ptr<grpc::Server> server;
  string endpoint;
  explicit IdentityHost(string identity) {
    status.identity = std::move(identity);
    grpc::ServerBuilder builder;
    int port = 0;
    builder.AddListeningPort("127.0.0.1:0", grpc::InsecureServerCredentials(), &port);
    builder.RegisterService(&status);
    server = builder.BuildAndStart();
    if (!server)
      throw std::runtime_error("Start Identity Host: Open Its Listener");
    endpoint = "http://127.0.0.1:" + std::to_string(port);
  }
  ~IdentityHost() {
    server->Shutdown();
  }
};
class PairingFile {
public:
  path directory;
  path file;
  PairingFile(const string &first, const string &second) {
    directory = std::filesystem::temp_directory_path() / ("dokime-cpp-pairing-" + harmos::detail::request_id());
    std::filesystem::create_directory(directory);
    file = directory / "host.pair";
    std::ofstream output(file, std::ios::binary);
    output.exceptions(std::ios::badbit | std::ios::failbit);
    output << "{\"version\":1,\"endpoints\":[\"" << first << "\",\"" << second
           << "\"],\"backend_identifier\":\"intended\",\"label\":\"Test Host\",\"certificate_pem\":null,"
              "\"access_key\":\"01234567890123456789012345678901\"}";
    output.close();
  }
  ~PairingFile() {
    std::filesystem::remove_all(directory);
  }
};
} // namespace

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

TEST_CASE("Pairing Tries Endpoints in Order and Verifies Identity and Credentials") {
  IdentityHost wrong("other");
  IdentityHost intended("intended");
  PairingFile pairing(wrong.endpoint, intended.endpoint);
  dokime::ConnectionSettings settings;
  settings.remote = pairing.file.string();
  auto client = dokime::connect(settings);
  REQUIRE(wrong.status.authorization == "Bearer 01234567890123456789012345678901");
  REQUIRE(intended.status.authorization == wrong.status.authorization);
  client.close();
}
TEST_CASE("Pairing Failures Preserve Every Endpoint Cause") {
  IdentityHost first("first");
  IdentityHost second("second");
  PairingFile pairing(first.endpoint, second.endpoint);
  dokime::ConnectionSettings settings;
  settings.remote = pairing.file.string();
  try {
    dokime::connect(settings);
    FAIL("Pairing Must Refuse Different Host Identities");
  } catch (const OperationError &error) {
    REQUIRE(error.value.has_cause());
    REQUIRE(error.value.cause().details().at("connection_endpoint") == first.endpoint);
    REQUIRE(error.value.cause().has_cause());
    REQUIRE(error.value.cause().cause().details().at("connection_endpoint") == second.endpoint);
  }
}
TEST_CASE("Direct Connections Refuse Unpinned HTTPS and Non-Origin Targets") {
  for (const auto &target : {"https://127.0.0.1:7000", "http://user@127.0.0.1:7000", "http://127.0.0.1:7000/path",
                             "http://127.0.0.1:7000?query", "http://127.0.0.1", "http://127.0.0.1:0"}) {
    dokime::ConnectionSettings settings;
    settings.remote = target;
    REQUIRE_THROWS(dokime::connect(settings));
  }
}
