/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "../plugin.hpp"
#include "../plugin/suites/capture/confirmation.hpp"
#include <chrono>
#include <dokime/client.hpp>
#include <filesystem>
#include <fstream>
#include <iostream>

using dokime::ResultStatus;
using starter_project::ConfirmationTestTemplate;
using starter_project::ExamplePlugin;
using std::ios;
using std::ofstream;
using std::streamsize;
using std::to_string;
using std::chrono::duration_cast;
using std::chrono::milliseconds;
using std::chrono::system_clock;

/* === Macros === */
/* === Constants === */
/* === Types === */
/* === Classes === */
/* === Templates === */
/* === Public Functions === */
/* === Private Functions === */
/* === Entry Point === */

int main() {
  // Initialize Client
  auto client = dokime::connect();

  // Initialize & Register Plugin
  client.plugins().register_plugin(ExamplePlugin{});

  // Initialize Test Cycle
  const auto started = duration_cast<milliseconds>(system_clock::now().time_since_epoch()).count();
  auto cycle = client.cycle("Capture " + to_string(started));

  // Initialize Test Cases
  const auto cases = client.cases().materialize<ConfirmationTestTemplate>();
  for (const auto &test_case : cases) {
    // Initialize Test Execution
    auto execution = cycle.execution(test_case);

    // Run Test Execution
    auto progress = execution.run();
    while (const auto update = progress.next()) {
      std::cout << update->phase << '\n';
    }

    // Seal Test Execution & Report Outcome
    const auto outcome = execution.seal();
    std::cout << "Passed: " << (outcome.status == ResultStatus::Passed) << '\n';
  }

  // Seal Test Cycle
  cycle.seal();

  // Save Evidence
  const auto evidence = cycle.evidence();
  std::filesystem::create_directories("_output");
  const auto archive = "_output/" + to_string(started) + "-" + cycle.identifier() + ".zip";
  ofstream output;
  output.exceptions(ios::badbit | ios::failbit);
  output.open(archive, ios::binary);
  output.write(reinterpret_cast<const char *>(evidence.data()), static_cast<streamsize>(evidence.size()));
  output.close();
  std::cout << "Saved " << archive << '\n';

  // Close Client
  client.close();
}
