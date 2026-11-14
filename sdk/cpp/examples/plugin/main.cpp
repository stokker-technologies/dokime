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

// Local Imports
#include <dokime/plugin.hpp>

using std::exception;
using std::invalid_argument;
using std::string;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

using dokime::plugin::Plugin;

int main(int argc, char **argv) {
  try {
    // Create the Plugin.
    Plugin plugin("sample", "0.1.0");

    // Serve the Placement Chosen by the Host or the Operator.
    if (argc == 3 && string(argv[1]) == "--address")
      plugin.serve(argv[2]);
    else if (argc == 1)
      plugin.serve();
    else
      throw invalid_argument("Serve Plugin: Expected [--address ADDRESS]");
  } catch (const exception &error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
