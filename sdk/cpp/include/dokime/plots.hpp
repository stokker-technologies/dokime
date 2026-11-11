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
#include <string>
#include <utility>
#include <vector>

// Third Party Imports

// Local Imports
#include "dokime/v1/documents.pb.h"

/* === Macros === */

/* === Constants === */

/* === Types === */

namespace dokime::authoring {

/* === Classes === */

struct Expression {
    v1::DataExpression message;
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
    Expression aggregate(v1::Aggregate kind) const {
        Expression value;
        value.message.set_label(message.label());
        *value.message.mutable_aggregate()->mutable_input() = message;
        value.message.mutable_aggregate()->set_kind(kind);
        return value;
    }
    Expression mean() const { return aggregate(v1::AGGREGATE_MEAN); }
    Expression std() const { return aggregate(v1::AGGREGATE_STDDEV); }
};

struct Plot {
    v1::Plot message;
    Plot& x(const Expression& value) { *message.mutable_x() = value.message; return *this; }
    Plot& y(const Expression& value) { *message.add_y() = value.message; return *this; }
    Plot& series_by(const Expression& value) { *message.mutable_series_by() = value.message; return *this; }
    Plot& label(const v1::Metadata& value) { *message.mutable_metadata() = value; return *this; }
    Plot& unit(std::string value) { message.set_unit(std::move(value)); return *this; }
    Plot& reduce(v1::Reduction value) { message.set_reduction(value); return *this; }
    Plot& render(v1::ContourRender value) { message.set_render(value); return *this; }
    Plot& arrangement(v1::BarArrangement value) { message.set_arrangement(value); return *this; }
    Plot& interval(v1::XyInterval value) { message.set_interval(value); return *this; }
    Plot& pareto(bool value) { message.set_pareto(value); return *this; }
    Plot& hole(double value) { message.set_hole(value); return *this; }
    Plot& row(const Expression& value) { *message.mutable_grid()->mutable_row() = value.message; return *this; }
    Plot& column(const Expression& value) { *message.mutable_grid()->mutable_column() = value.message; return *this; }
    Plot& frame(const Expression& value) { *message.mutable_grid()->mutable_frame() = value.message; return *this; }
    Plot& rows(const std::vector<v1::GridAxisEntry>& entries) { message.mutable_grid()->clear_rows(); for (const auto& entry : entries) *message.mutable_grid()->add_rows() = entry; return *this; }
    Plot& columns(const std::vector<v1::GridAxisEntry>& entries) { message.mutable_grid()->clear_columns(); for (const auto& entry : entries) *message.mutable_grid()->add_columns() = entry; return *this; }
    Plot& highlight(const v1::GridHighlight& value) { *message.mutable_grid()->add_highlights() = value; return *this; }
};

/* === Templates === */

/* === Public Declarations === */

inline Expression field(std::string stream, std::string name) {
    Expression value;
    value.message.mutable_stream()->set_identifier(std::move(stream));
    value.message.mutable_stream()->set_field(std::move(name));
    return value;
}

inline Plot time_series(const Expression& value) {
    Plot plot;
    plot.message.set_kind(v1::PLOT_KIND_TIME_SERIES);
    *plot.message.add_y() = value.message;
    return plot;
}

inline Plot frequency_series(const Expression& value) {
    Plot plot;
    plot.message.set_kind(v1::PLOT_KIND_FREQUENCY_SERIES);
    *plot.message.add_y() = value.message;
    return plot;
}

inline Plot xy(const Expression& value) {
    Plot plot;
    plot.message.set_kind(v1::PLOT_KIND_XY);
    *plot.message.add_y() = value.message;
    return plot;
}

inline Plot trajectory(const Expression& value) {
    Plot plot;
    plot.message.set_kind(v1::PLOT_KIND_TRAJECTORY);
    *plot.message.add_y() = value.message;
    return plot;
}

inline Plot contour(const Expression& value) {
    Plot plot;
    plot.message.set_kind(v1::PLOT_KIND_CONTOUR);
    *plot.message.mutable_z() = value.message;
    return plot;
}

inline Plot grid_chart(const Expression& value) {
    Plot plot;
    plot.message.set_kind(v1::PLOT_KIND_GRID_CHART);
    *plot.message.add_y() = value.message;
    return plot;
}

inline Plot bar(const Expression& value) {
    Plot plot;
    plot.message.set_kind(v1::PLOT_KIND_BAR);
    *plot.message.add_y() = value.message;
    return plot;
}

inline Plot pie(const Expression& value) {
    Plot plot;
    plot.message.set_kind(v1::PLOT_KIND_PIE);
    *plot.message.add_y() = value.message;
    return plot;
}

inline Plot histogram(const Expression& value) {
    Plot plot;
    plot.message.set_kind(v1::PLOT_KIND_HISTOGRAM);
    *plot.message.mutable_x() = value.message;
    return plot;
}

inline Plot box_plot(const Expression& value) {
    Plot plot;
    plot.message.set_kind(v1::PLOT_KIND_BOX_PLOT);
    *plot.message.add_y() = value.message;
    return plot;
}

inline v1::Document& section(v1::Document& document, const v1::Section& value) { *document.add_sections() = value; return document; }
inline v1::Section& plot(v1::Section& section, const Plot& value) {
    auto* card = section.add_cards();
    *card->mutable_metadata() = value.message.metadata();
    *card->mutable_plot() = value.message;
    return section;
}

} // namespace dokime::authoring

