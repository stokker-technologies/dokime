/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#include "dokime/plots.hpp"
#include <stdexcept>

int main() {
  using namespace dokime::authoring;
  auto input = field("scan", "temperature");
  auto surface = contour(input.scale(2).offset(-1))
                     .x(field("scan", "position_x"))
                     .y(field("scan", "position_y"))
                     .reduce(dokime::v1::REDUCTION_MEAN)
                     .render(dokime::v1::CONTOUR_RENDER_BANDS);
  dokime::v1::Metadata metadata;
  metadata.set_identifier("temperature.map");
  metadata.set_label("Temperature Map");
  surface.label(metadata);
  dokime::v1::ReferenceBand band;
  band.mutable_center()->set_parameter("target");
  band.mutable_tolerance()->set_parameter("tolerance");
  surface.reference_band(band);
  dokime::v1::Section target;
  target.mutable_metadata()->set_identifier("map");
  target.mutable_metadata()->set_label("Map");
  plot(target, surface);
  dokime::v1::Document document;
  document.mutable_metadata()->set_identifier("scan");
  document.mutable_metadata()->set_label("Scan");
  section(document, target);
  dokime::v1::Document decoded;
  if (!decoded.ParseFromString(document.SerializeAsString()))
    throw std::runtime_error("document decode failed");
  surface.message.clear_y();
  input.message.mutable_stream()->set_field("changed");
  const auto &captured = decoded.sections(0).cards(0).plot();
  if (captured.y_size() != 1 || captured.z().transform().input().transform().input().stream().field() != "temperature" ||
      captured.reference_bands(0).tolerance().parameter() != "tolerance")
    throw std::runtime_error("contour operands were not copied");
  const auto grid = grid_chart(field("scan", "value"))
                        .row(field("scan", "row"))
                        .column(field("scan", "column"))
                        .frame(field("scan", "timestamp"))
                        .reduce(dokime::v1::REDUCTION_MAX);
  if (grid.message.grid().frame().stream().field() != "timestamp" || grid.message.reduction() != dokime::v1::REDUCTION_MAX)
    throw std::runtime_error("grid bindings differ");
}
