/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#include "dokime/plots.hpp"
#include <catch2/catch_test_macros.hpp>
using dokime::CONTOUR_RENDER_BANDS;
using dokime::Document;
using dokime::Metadata;
using dokime::REDUCTION_MAX;
using dokime::REDUCTION_MEAN;
using dokime::ReferenceBand;
using dokime::Section;

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
  Section target;
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
