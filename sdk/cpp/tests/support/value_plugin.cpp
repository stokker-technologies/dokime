/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "dokime/plugin.hpp"
#include <grpc/event_engine/event_engine.h>
#include <iostream>

using dokime::plugin::Context;
using dokime::plugin::operation;
using dokime::plugin::Plugin;
using dokime::plugin::Visibility;
using grpc_event_engine::experimental::ShutdownDefaultEventEngine;
using std::cerr;
using std::exception;
using std::int64_t;
using std::string;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

template <class T> void echo(Plugin &plugin, string name) {
  const auto local = operation<T, T>("local." + name, Visibility::Private, [](Context &, T request) {
    return request;
  });
  plugin.operation(local);
  plugin.operation(operation<T, T>("values." + name, Visibility::Public, [local](Context &context, T request) {
    return context.operation(local, request);
  }));
}

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

int main(int argc, char **argv) {
  int result = 0;
  try {
    Plugin plugin("values", "1.0.0");
    echo<string>(plugin, "text");
    echo<bool>(plugin, "flag");
    echo<int64_t>(plugin, "count");
    echo<double>(plugin, "number");
    plugin.serve(argc == 3 && string(argv[1]) == "--address" ? argv[2] : "");
  } catch (const exception &error) {
    cerr << error.what() << '\n';
    result = 1;
  }
  ShutdownDefaultEventEngine();
  return result;
}
