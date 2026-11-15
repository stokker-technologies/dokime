/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

#include <dokime/dokime.hpp>
#include <iostream>

using dokime::Analysis;
using dokime::AnalysisValues;
using dokime::MapValue;
using dokime::VALUE_TYPE_NUMBER;
using dokime::plugin::AnalysisSamples;
using dokime::plugin::Context;
using dokime::plugin::Plugin;
using dokime::plugin::Template;

using dokime::plugin::metadata;
using std::exception;
using std::invalid_argument;
using std::make_shared;
using std::move;
using std::string;
using std::vector;

/* === Macros === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Templates === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

int main(int argc, char **argv) {
  try {
    // Compose the Suite from Ordinary Values.
    Template value;
    *value.definition.mutable_metadata() = metadata("sample.check", "Check");
    value.initialize = [](Context &ctx) { ctx.set(make_shared<double>(0)); };
    value.steps.push_back({metadata("measure", "Measure"), [](Context &ctx) { *ctx.get<double>() = 42; }});
    value.criteria.push_back({metadata("valid", "Valid"), [](Context &ctx) { return *ctx.get<double>() == 42; }});
    Plugin plugin("sample", "0.1.0");
    plugin.suite({metadata("sample", "Sample"), {move(value)}});

    // Scale Raw Values While Preserving Their Source-Time Axis.
    Analysis analysis;
    *analysis.mutable_metadata() = metadata("sample.scale", "Scale");
    analysis.set_version(1);
    analysis.set_max_samples(1024);
    analysis.set_max_output_values(1024);
    auto *gain = analysis.add_parameters();
    *gain->mutable_metadata() = metadata("gain", "Gain");
    gain->set_type(VALUE_TYPE_NUMBER);
    auto *output = analysis.add_outputs();
    *output->mutable_metadata() = metadata("scaled", "Scaled");
    output->set_unit("V");
    output->set_axis("time");
    output->set_axis_unit("s");
    plugin.analysis(move(analysis), [](Context &, const AnalysisSamples &samples, const MapValue &parameters) {
      const auto gain = parameters.values().at("gain").number();
      AnalysisValues result;
      result.set_key("scaled");
      result.set_unit(samples.unit());
      result.set_axis_unit(samples.axis_unit());
      *result.mutable_axis() = samples.axis();
      for (const auto value : samples.values())
        result.add_values(value * gain);
      return vector<AnalysisValues>{move(result)};
    });

    // Empty Address Uses the Endpoint Passed by the Host in HARMOS_ADDRESS.
    if (argc == 3 && string(argv[1]) == "--address")
      plugin.serve(argv[2]);
    else if (argc == 1)
      plugin.serve();
    else
      throw invalid_argument("Usage: dokime-plugin [--address ADDRESS]");
  } catch (const exception &error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
