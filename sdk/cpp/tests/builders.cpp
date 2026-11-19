/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#include "dokime/parameters.hpp"
#include "dokime/plots.hpp"
#include "dokime/plugin.hpp"
#include <algorithm>
#include <catch2/catch_test_macros.hpp>
using dokime::CONTOUR_RENDER_BANDS;
using dokime::Document;
using dokime::Metadata;
using dokime::REDUCTION_MAX;
using dokime::REDUCTION_MEAN;
using dokime::ReferenceBand;
using std::find_if;
using std::invalid_argument;

TEST_CASE("Documents Preserve Composed Plot Operands") {
  using namespace dokime::authoring;
  auto input = field("scan", "temperature");
  auto surface = contour(input.scale(2).offset(-1))
                     .x(field("scan", "position_x"))
                     .y(field("scan", "position_y"))
                     .reduce(REDUCTION_MEAN)
                     .render(CONTOUR_RENDER_BANDS);
  Metadata metadata;
  metadata.set_identifier("temperature.map");
  metadata.set_label("Temperature Map");
  surface.label(metadata);
  ReferenceBand band;
  band.mutable_center()->set_parameter("target");
  band.mutable_tolerance()->set_parameter("tolerance");
  surface.reference_band(band);
  dokime::model::Section target;
  target.mutable_metadata()->set_identifier("map");
  target.mutable_metadata()->set_label("Map");
  plot(target, surface);
  Document document;
  document.mutable_metadata()->set_identifier("scan");
  document.mutable_metadata()->set_label("Scan");
  section(document, target);
  Document decoded;
  REQUIRE(decoded.ParseFromString(document.SerializeAsString()));
  surface.message.clear_y();
  input.message.mutable_stream()->set_field("changed");
  const auto &captured = decoded.sections(0).cards(0).plot();
  REQUIRE(captured.y_size() == 1);
  REQUIRE(captured.z().transform().input().transform().input().stream().field() == "temperature");
  REQUIRE(captured.reference_bands(0).tolerance().parameter() == "tolerance");
  const auto grid = grid_chart(field("scan", "value"))
                        .row(field("scan", "row"))
                        .column(field("scan", "column"))
                        .frame(field("scan", "timestamp"))
                        .reduce(REDUCTION_MAX);
  REQUIRE(grid.message.grid().frame().stream().field() == "timestamp");
  REQUIRE(grid.message.reduction() == REDUCTION_MAX);
}

TEST_CASE("Parameter Builders Preserve Types Expansion and Bounds") {
  const auto voltage = dokime::authoring::Parameter::number("voltage", "Voltage").unit("V").default_value(230.0).range(0.0, 300.0).build();
  const auto count = dokime::authoring::Parameter::count("samples", "Samples").default_value(UINT64_MAX).build();
  const auto flag = dokime::authoring::Parameter::flag("enabled", "Enabled").default_value(true).build();
  const auto sweep = dokime::authoring::Parameter::sweep("load", "Load", {80.0, 100.0}).build();
  const auto ladder = dokime::authoring::Parameter::numbers("steps", "Steps", {10.0, 20.0}).items(1, 8).build();
  dokime::Parameter decoded;
  REQUIRE(decoded.ParseFromString(voltage.SerializeAsString()));
  REQUIRE(decoded.expression().literal().number() == 230.0);
  REQUIRE(decoded.min() == 0.0);
  REQUIRE(decoded.max() == 300.0);
  REQUIRE(count.expression().literal().unsigned_integer() == UINT64_MAX);
  REQUIRE(flag.expression().literal().boolean());
  REQUIRE(sweep.expansion().cartesian());
  REQUIRE(sweep.expression().array().values_size() == 2);
  REQUIRE(ladder.expression().literal().list().values(1).number() == 20.0);
  REQUIRE(ladder.min_items() == 1);
  REQUIRE(ladder.max_items() == 8);
  auto line = dokime::authoring::Plot::line("voltage", "Voltage").unit("V").y(dokime::authoring::field("samples", "voltage"));
  const auto card = dokime::authoring::plot_card(line, 6);
  line.message.clear_y();
  REQUIRE(card.plot().y_size() == 1);
  REQUIRE(card.layout().columns() == 6);
  REQUIRE(dokime::authoring::records_card("records").evidence() == dokime::EVIDENCE_CARD_RECORDS);
}

TEST_CASE("Lanes Preserve Declarations and Operation Assignments", "[builders]") {
  const auto instrument = dokime::Lane::serial("instrument");
  auto operation = dokime::operations::operation<double, double>("example.measure", dokime::operations::Visibility::Public,
                                                                 [](dokime::operations::Context &, double value) {
                                                                   return value;
                                                                 });
  operation.lane(instrument);
  dokime::Plugin plugin("example", "1.0.0");
  plugin.lane(instrument).operation(operation);
  const auto description = plugin.describe();
  REQUIRE(description.lanes.size() == 1);
  REQUIRE(description.lanes[0].name() == "instrument");
  REQUIRE(description.lanes[0].serial());
  const auto selected = find_if(description.operations.begin(), description.operations.end(), [](const auto &value) {
    return value.identifier == "example.measure";
  });
  REQUIRE(selected != description.operations.end());
  REQUIRE(selected->lane == "instrument");

  dokime::Plugin missing("missing", "1.0.0");
  missing.operation(operation);
  REQUIRE_THROWS_AS(missing.describe(), invalid_argument);
}
