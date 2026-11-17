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
#include <iostream>

// Third Party Imports
#include <grpc/event_engine/event_engine.h>

// Local Imports
#include <dokime/client.hpp>

using dokime::Client;
using grpc_event_engine::experimental::ShutdownDefaultEventEngine;

using std::exception;
using std::invalid_argument;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

int main(int argc, char **argv) {
  int result = 0;
  try {
    // Connect to the Selected Host.
    if (argc != 2)
      throw invalid_argument("Read Status: Supply HOST:PORT");
    auto client = Client::connect(argv[1]);

    // Read Status and Plugin Entries Through One Composed Client.
    const auto status = client.status.get();
    const auto plugins = client.plugins.list();
    std::cout << status.status().identity().application_name() << '\n';
    std::cout << plugins.entries_size() << " Plugins\n";
  } catch (const exception &error) {
    std::cerr << error.what() << '\n';
    result = 1;
  }
  // Drain EventEngine Work Before Windows Begins DLL Teardown.
  ShutdownDefaultEventEngine();
  return result;
}
