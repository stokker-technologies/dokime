/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "../support/plugin_host.hpp"
#include "dokime/payloads.hpp"
#include <catch2/catch_session.hpp>
#include <catch2/catch_test_macros.hpp>
#include <grpc/event_engine/event_engine.h>
#include <limits>

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
  const auto response = host.invoke<decltype(request)>("values." + name, request);
  REQUIRE(dokime::templates::detail::unpack<T>(response) == value);
}

/* === Public Functions === */

/* === Private Functions === */

namespace {
string address;
} // namespace

/* === Entry Point === */

TEST_CASE("Plain Operations Round Trip Through Public and Local Private Calls") {
  PluginHost host(address);
  const auto description = host.describe();
  for (const auto &operation : description.declarations().operations())
    REQUIRE(operation.visibility() == harmos::VISIBILITY_PUBLIC);
  roundtrip(host, "text", string("a value"));
  roundtrip(host, "text", string{});
  roundtrip(host, "flag", true);
  roundtrip(host, "flag", false);
  roundtrip(host, "count", numeric_limits<int64_t>::min());
  roundtrip(host, "count", numeric_limits<int64_t>::max());
  roundtrip(host, "number", 2.125);
  roundtrip(host, "number", -0.0);
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
