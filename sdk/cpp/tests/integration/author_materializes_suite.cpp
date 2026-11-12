/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include "../support/check.hpp"
#include "dokime/plugin.hpp"

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

int main() {
  // Compose a Suite and Hand Its Values to the Plugin.
  dokime::Template value;
  *value.definition.mutable_metadata() = dokime::metadata("sample.measure", "Measure");
  value.steps.push_back({dokime::metadata("capture", "Capture"), [](dokime::Context &) {}, 2});
  value.criteria.push_back({dokime::metadata("valid", "Valid"), [](dokime::Context &) { return true; }});
  dokime::Plugin plugin("sample", "1.0.0");
  plugin.suite({dokime::metadata("sample", "Sample"), {value}});

  // Inspect the Frozen Catalog and Its Private Unary Contracts.
  auto catalog = plugin.catalog();
  dokime::v1::Suites suites;
  for (const auto &section : catalog.sections())
    if (section.type() == "dokime.v1.Suites")
      check(suites.ParseFromString(section.payload()), "Suite Did Not Decode");
  check(suites.suites_size() == 1 && suites.suites(0).templates(0).materialize() == "sample.measure.materialize",
        "Suite Was Not Registered");
  bool retry = false;
  for (const auto &operation : catalog.operations()) {
    check(operation.visibility() == harmos::wire::VISIBILITY_PRIVATE, "An Authoring Operation Became Public");
    if (operation.id() == "sample.measure.step.capture")
      retry = operation.policy().attempts() == 3;
  }
  check(retry, "Step Retry Policy Was Lost");

  // Correct a Duplicate Identifier Before Serving.
  bool duplicate = false;
  try {
    plugin.suite({dokime::metadata("sample", "Sample"), {value}});
  } catch (const std::invalid_argument &) {
    duplicate = true;
  }
  check(duplicate, "Duplicate Suite Was Accepted");
}
