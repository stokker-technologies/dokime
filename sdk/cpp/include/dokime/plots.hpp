/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#pragma once

// Standard Library Imports
#include <cstdint>
#include <string>
#include <utility>
#include <vector>

// Third Party Imports

// Local Imports
#include "dokime/messages.hpp"

/* === Macros === */

/* === Constants === */

/* === Types === */

namespace dokime::authoring {

/* === Classes === */

struct Expression {
  DataExpression message;
  Expression scale(double factor) const {
    Expression value;
    value.message.set_label(message.label());
    *value.message.mutable_transform()->mutable_input() = message;
    value.message.mutable_transform()->set_scale(factor);
    return value;
  }
  Expression offset(double offset) const {
    Expression value;
    value.message.set_label(message.label());
    *value.message.mutable_transform()->mutable_input() = message;
    value.message.mutable_transform()->set_offset(offset);
    return value;
  }
  Expression aggregate(Aggregate kind) const {
    Expression value;
    value.message.set_label(message.label());
    *value.message.mutable_aggregate()->mutable_input() = message;
    value.message.mutable_aggregate()->set_kind(kind);
    return value;
  }
  Expression mean() const {
    return aggregate(AGGREGATE_MEAN);
  }
  Expression std() const {
    return aggregate(AGGREGATE_STDDEV);
  }
};

struct Plot {
  static Plot line(std::string identifier, std::string label) {
    Plot result;
    result.message.set_kind(PLOT_KIND_TIME_SERIES);
    result.message.mutable_metadata()->set_identifier(std::move(identifier));
    result.message.mutable_metadata()->set_label(std::move(label));
    return result;
  }
  dokime::Plot message;
  Plot &x(const Expression &value) {
    *message.mutable_x() = value.message;
    return *this;
  }
  Plot &y(const Expression &value) {
    *message.add_y() = value.message;
    return *this;
  }
  Plot &series_by(const Expression &value) {
    *message.mutable_series_by() = value.message;
    return *this;
  }
  Plot &label(const Metadata &value) {
    *message.mutable_metadata() = value;
    return *this;
  }
  Plot &unit(std::string value) {
    message.set_unit(std::move(value));
    return *this;
  }
  Plot &input_samples(std::uint32_t count) {
    message.set_input_samples(count);
    return *this;
  }
  Plot &reduce(Reduction value) {
    message.set_reduction(value);
    return *this;
  }
  Plot &render(ContourRender value) {
    message.set_render(value);
    return *this;
  }
  Plot &arrangement(BarArrangement value) {
    message.set_arrangement(value);
    return *this;
  }
  Plot &interval(XyInterval value) {
    message.set_interval(value);
    return *this;
  }
  Plot &pareto(bool value) {
    message.set_pareto(value);
    return *this;
  }
  Plot &hole(double value) {
    message.set_hole(value);
    return *this;
  }
  Plot &row(const Expression &value) {
    *message.mutable_grid()->mutable_row() = value.message;
    return *this;
  }
  Plot &column(const Expression &value) {
    *message.mutable_grid()->mutable_column() = value.message;
    return *this;
  }
  Plot &frame(const Expression &value) {
    *message.mutable_grid()->mutable_frame() = value.message;
    return *this;
  }
  Plot &rows(const std::vector<GridAxisEntry> &entries) {
    message.mutable_grid()->clear_rows();
    for (const auto &entry : entries)
      *message.mutable_grid()->add_rows() = entry;
    return *this;
  }
  Plot &columns(const std::vector<GridAxisEntry> &entries) {
    message.mutable_grid()->clear_columns();
    for (const auto &entry : entries)
      *message.mutable_grid()->add_columns() = entry;
    return *this;
  }
  Plot &highlight(const GridHighlight &value) {
    *message.mutable_grid()->add_highlights() = value;
    return *this;
  }
  Plot &reference_line(const ReferenceLine &value) {
    *message.add_reference_lines() = value;
    return *this;
  }
  Plot &reference_band(const ReferenceBand &value) {
    *message.add_reference_bands() = value;
    return *this;
  }
};

/* === Templates === */

/* === Public Declarations === */

inline Card plot_card(const Plot &plot, std::uint32_t columns = 12) {
  Card result;
  *result.mutable_metadata() = plot.message.metadata();
  *result.mutable_plot() = plot.message;
  result.mutable_layout()->set_columns(columns);
  return result;
}
inline Card records_card(std::string identifier) {
  Card result;
  result.mutable_metadata()->set_identifier(std::move(identifier));
  result.mutable_metadata()->set_label("Records");
  result.set_evidence(EVIDENCE_CARD_RECORDS);
  return result;
}

inline Expression field(std::string stream, std::string name) {
  Expression value;
  value.message.mutable_stream()->set_identifier(std::move(stream));
  value.message.mutable_stream()->set_field(std::move(name));
  return value;
}

inline Plot time_series(const Expression &value) {
  Plot plot;
  plot.message.set_kind(PLOT_KIND_TIME_SERIES);
  *plot.message.add_y() = value.message;
  return plot;
}

inline Plot frequency_series(const Expression &value) {
  Plot plot;
  plot.message.set_kind(PLOT_KIND_FREQUENCY_SERIES);
  *plot.message.add_y() = value.message;
  return plot;
}

inline Plot xy(const Expression &value) {
  Plot plot;
  plot.message.set_kind(PLOT_KIND_XY);
  *plot.message.add_y() = value.message;
  return plot;
}

inline Plot trajectory(const Expression &value) {
  Plot plot;
  plot.message.set_kind(PLOT_KIND_TRAJECTORY);
  *plot.message.add_y() = value.message;
  return plot;
}

inline Plot contour(const Expression &value) {
  Plot plot;
  plot.message.set_kind(PLOT_KIND_CONTOUR);
  *plot.message.mutable_z() = value.message;
  return plot;
}

inline Plot grid_chart(const Expression &value) {
  Plot plot;
  plot.message.set_kind(PLOT_KIND_GRID_CHART);
  *plot.message.add_y() = value.message;
  return plot;
}

inline Plot bar(const Expression &value) {
  Plot plot;
  plot.message.set_kind(PLOT_KIND_BAR);
  *plot.message.add_y() = value.message;
  return plot;
}

inline Plot pie(const Expression &value) {
  Plot plot;
  plot.message.set_kind(PLOT_KIND_PIE);
  *plot.message.add_y() = value.message;
  return plot;
}

inline Plot histogram(const Expression &value) {
  Plot plot;
  plot.message.set_kind(PLOT_KIND_HISTOGRAM);
  *plot.message.mutable_x() = value.message;
  return plot;
}

inline Plot box_plot(const Expression &value) {
  Plot plot;
  plot.message.set_kind(PLOT_KIND_BOX_PLOT);
  *plot.message.add_y() = value.message;
  return plot;
}

inline Document &section(Document &document, const model::Section &value) {
  *document.add_sections() = value;
  return document;
}
inline model::Section &plot(model::Section &section, const Plot &value) {
  auto *card = section.add_cards();
  *card->mutable_metadata() = value.message.metadata();
  *card->mutable_plot() = value.message;
  return section;
}

} // namespace dokime::authoring
