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

using dokime::Plugin;
using dokime::operations::Context;
using dokime::operations::operation;
using dokime::operations::Visibility;
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
  const auto local = operation<T, T>("values.local." + name, Visibility::Private, [](Context &, T request) {
    return request;
  });
  plugin.operation(local);
  plugin.operation(operation<T, T>("values." + name, Visibility::Public, [local](Context &context, T request) {
    return context.operation(local)(request);
  }));
}

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

int main() {
  int result = 0;
  try {
    Plugin plugin("values", "1.0.0");
    echo<string>(plugin, "text");
    echo<bool>(plugin, "flag");
    echo<int64_t>(plugin, "count");
    echo<double>(plugin, "number");
    plugin.serve();
  } catch (const exception &error) {
    cerr << error.what() << '\n';
    result = 1;
  }
  ShutdownDefaultEventEngine();
  return result;
}
