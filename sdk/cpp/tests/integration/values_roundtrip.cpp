/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
#include <cstdint>
#include <limits>
#include <string>

// Third Party Imports
#include <catch2/catch_session.hpp>
#include <catch2/catch_test_macros.hpp>
#include <grpc/event_engine/event_engine.h>

// Local Imports
#include "../support/plugin_host.hpp"
#include "dokime/messages.hpp"
#include "dokime/payloads.hpp"

using dokime::Metadata;
using grpc_event_engine::experimental::ShutdownDefaultEventEngine;
using std::int64_t;
using std::numeric_limits;
using std::string;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

template <class T> void roundtrip(PluginHost &host, const string &name, const T &value) {
  auto request = dokime::templates::detail::pack(value);
  host.child = [&name, &request](const harmos::plugin::v1::ChildRequest &child) -> string {
    REQUIRE(child.operation() == "values.local." + name);
    REQUIRE(child.payload() == request.SerializeAsString());
    REQUIRE_FALSE(child.fingerprint().empty());
    return child.payload();
  };
  const auto response = host.invoke<decltype(request)>("values." + name, request);
  REQUIRE(dokime::templates::detail::unpack<T>(response) == value);
}

/* === Public Functions === */

/* === Private Functions === */

namespace {
string address;
} // namespace

/* === Entry Point === */

TEST_CASE("Plain Operations Round Trip Through Host Routed Private Calls") {
  PluginHost host(address);
  roundtrip(host, "text", string("a value"));
  roundtrip(host, "text", string{});
  roundtrip(host, "flag", true);
  roundtrip(host, "flag", false);
  roundtrip(host, "count", numeric_limits<int64_t>::min());
  roundtrip(host, "count", numeric_limits<int64_t>::max());
  roundtrip(host, "number", 2.125);
  roundtrip(host, "number", -0.0);
  host.child = [](const harmos::plugin::v1::ChildRequest &request) -> string {
    REQUIRE(request.operation() == "device.measure");
    REQUIRE(request.payload().empty());
    REQUIRE(request.fingerprint().empty());
    const auto payload = harmos::json(request.json());
    REQUIRE(payload.fields().at("identifier").string_value() == "reading");
    return R"({"identifier":"reading","label":"Reading"})";
  };
  Metadata request;
  request.set_identifier("reading");
  const auto response = host.invoke<Metadata>("values.untyped", request);
  REQUIRE(response.identifier() == "reading");
  REQUIRE(response.label() == "Reading");
  host.stop();
}

int main(int argc, char **argv) {
  if (argc != 3)
    return 2;
  address = argv[1];
  const auto result = Catch::Session{}.run(1, argv);
  ShutdownDefaultEventEngine();
  return result;
}
