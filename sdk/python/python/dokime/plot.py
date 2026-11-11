# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Thin Plot Builders Over Canonical dokime.v1 Messages."""

# === Imports === #

# Standard Library Imports
from collections.abc import Iterable
from enum import IntEnum
from typing import Any, Self

# Third Party Imports
# Local Imports
from .v1 import enum_value, message

# Type Checking Imports

# === Public Exports === #

__all__ = [
    "Plot",
    "Expression",
    "Reduction",
    "ContourRender",
    "BarArrangement",
    "XyInterval",
    "PlotStyle",
    "AxisScale",
    "Aggregate",
    "field",
    "time_series",
    "frequency_series",
    "xy",
    "trajectory",
    "contour",
    "grid_chart",
    "bar",
    "pie",
    "histogram",
    "box_plot",
]

# === Constants === #

_RETIRED = {
    "grouped_bar": "bar(...).arrangement(BarArrangement.GROUPED)",
    "stacked_bar": "bar(...).arrangement(BarArrangement.STACKED)",
    "pareto": "bar(...).pareto(True)",
    "donut": "pie(...).hole(0.5)",
    "xy_interval": "xy(...).interval(XyInterval.SIGMA)",
    "heatmap": "contour(...)",
    "xy_value_heatmap": "contour(...)",
    "wifi_channel_occupancy": "grid_chart(...) with declared rows, columns and frames",
    "box_summary": "box_plot(...)",
    "aggregate_table": "a document Summary card",
}

# === Types === #

# === Classes === #


class Reduction(IntEnum):
    MEAN = enum_value("Reduction", "REDUCTION_MEAN")
    MIN = enum_value("Reduction", "REDUCTION_MIN")
    MAX = enum_value("Reduction", "REDUCTION_MAX")


class ContourRender(IntEnum):
    CELLS = enum_value("ContourRender", "CONTOUR_RENDER_CELLS")
    BANDS = enum_value("ContourRender", "CONTOUR_RENDER_BANDS")


class BarArrangement(IntEnum):
    GROUPED = enum_value("BarArrangement", "BAR_ARRANGEMENT_GROUPED")
    STACKED = enum_value("BarArrangement", "BAR_ARRANGEMENT_STACKED")


class XyInterval(IntEnum):
    SIGMA = enum_value("XyInterval", "XY_INTERVAL_SIGMA")


class PlotStyle(IntEnum):
    LINE = enum_value("PlotStyle", "PLOT_STYLE_LINE")
    POINTS = enum_value("PlotStyle", "PLOT_STYLE_POINTS")
    LINE_POINTS = enum_value("PlotStyle", "PLOT_STYLE_LINE_POINTS")
    STEP = enum_value("PlotStyle", "PLOT_STYLE_STEP")


class AxisScale(IntEnum):
    LINEAR = enum_value("AxisScale", "AXIS_SCALE_LINEAR")
    LOG = enum_value("AxisScale", "AXIS_SCALE_LOG")


class Aggregate(IntEnum):
    COUNT = enum_value("Aggregate", "AGGREGATE_COUNT")
    SUM = enum_value("Aggregate", "AGGREGATE_SUM")
    MIN = enum_value("Aggregate", "AGGREGATE_MIN")
    MAX = enum_value("Aggregate", "AGGREGATE_MAX")
    MEAN = enum_value("Aggregate", "AGGREGATE_MEAN")
    STDDEV = enum_value("Aggregate", "AGGREGATE_STDDEV")
    MEDIAN = enum_value("Aggregate", "AGGREGATE_MEDIAN")
    PERCENTILE = enum_value("Aggregate", "AGGREGATE_PERCENTILE")
    RMS = enum_value("Aggregate", "AGGREGATE_RMS")


class Expression:
    def __init__(self, value: Any) -> None:
        if value.DESCRIPTOR.full_name != "dokime.v1.DataExpression":
            raise TypeError("expected a dokime.v1.DataExpression")
        self.message = value

    def label(self, label: str) -> Self:
        value = message("DataExpression")
        value.CopyFrom(self.message)
        value.label = label
        return type(self)(value)

    def scale(self, factor: float) -> Self:
        return self._transform("scale", factor)

    def offset(self, value: float) -> Self:
        return self._transform("offset", value)

    def detail(self, name: str) -> Self:
        return self._transform("detail", name)

    def time_between(self, record: str) -> Self:
        return self._transform("time_between", record)

    def per_case(self) -> Self:
        return self._transform("per_case", True)

    def aggregate(self, kind: Aggregate, *, percentile: float = 0.0) -> Self:
        value = message("DataExpression", label=self.message.label)
        value.aggregate.input.CopyFrom(self.message)
        value.aggregate.kind = _enum(kind, Aggregate)
        value.aggregate.percentile = percentile
        return type(self)(value)

    def percentile(self, rank: float) -> Self:
        return self.aggregate(Aggregate.PERCENTILE, percentile=rank)

    def _transform(self, name: str, operand: Any) -> Self:
        value = message("DataExpression", label=self.message.label)
        value.transform.input.CopyFrom(self.message)
        setattr(value.transform, name, operand)
        return type(self)(value)

    def count(self) -> Self:
        return self.aggregate(Aggregate.COUNT)

    def sum(self) -> Self:
        return self.aggregate(Aggregate.SUM)

    def min(self) -> Self:
        return self.aggregate(Aggregate.MIN)

    def max(self) -> Self:
        return self.aggregate(Aggregate.MAX)

    def mean(self) -> Self:
        return self.aggregate(Aggregate.MEAN)

    def std(self) -> Self:
        return self.aggregate(Aggregate.STDDEV)

    def median(self) -> Self:
        return self.aggregate(Aggregate.MEDIAN)

    def rms(self) -> Self:
        return self.aggregate(Aggregate.RMS)


class Plot:
    def __init__(self, value: Any) -> None:
        if value.DESCRIPTOR.full_name != "dokime.v1.Plot":
            raise TypeError("expected a dokime.v1.Plot")
        self.message = value

    def x(self, value: Expression) -> Self:
        self.message.x.CopyFrom(_expression(value))
        return self

    def y(self, value: Expression) -> Self:
        self.message.y.add().CopyFrom(_expression(value))
        return self

    def series_by(self, value: Expression) -> Self:
        self.message.series_by.CopyFrom(_expression(value))
        return self

    def label(self, label: str) -> Self:
        from .documents import metadata

        identifier = self.message.metadata.identifier
        self.message.metadata.CopyFrom(metadata(label))
        if identifier:
            self.message.metadata.identifier = identifier
        return self

    def id(self, identifier: str) -> Self:
        self.message.metadata.identifier = identifier
        return self

    def unit(self, unit: str) -> Self:
        self.message.unit = unit
        return self

    def reduce(self, reduction: Reduction) -> Self:
        self.message.reduction = _enum(reduction, Reduction)
        return self

    def render(self, render: ContourRender) -> Self:
        self.message.render = _enum(render, ContourRender)
        return self

    def arrangement(self, arrangement: BarArrangement) -> Self:
        self.message.arrangement = _enum(arrangement, BarArrangement)
        return self

    def interval(self, interval: XyInterval) -> Self:
        self.message.interval = _enum(interval, XyInterval)
        return self

    def style(self, style: PlotStyle) -> Self:
        self.message.style = _enum(style, PlotStyle)
        return self

    def x_scale(self, scale: AxisScale) -> Self:
        self.message.x_scale = _enum(scale, AxisScale)
        return self

    def y_scale(self, scale: AxisScale) -> Self:
        self.message.y_scale = _enum(scale, AxisScale)
        return self

    def pareto(self, enabled: bool) -> Self:
        if type(enabled) is not bool:
            raise TypeError("pareto requires a Boolean")
        self.message.pareto = enabled
        return self

    def hole(self, fraction: float) -> Self:
        self.message.hole = fraction
        return self

    def sample_rate_hz(self, rate: float) -> Self:
        self.message.sample_rate_hz = rate
        return self

    def bins(self, count: int) -> Self:
        self.message.histogram_bins = count
        return self

    def display_points(self, count: int) -> Self:
        self.message.display_points = count
        return self

    def row(self, value: Expression) -> Self:
        self.message.grid.row.CopyFrom(_expression(value))
        return self

    def column(self, value: Expression) -> Self:
        self.message.grid.column.CopyFrom(_expression(value))
        return self

    def frame(self, value: Expression) -> Self:
        self.message.grid.frame.CopyFrom(_expression(value))
        return self

    def rows(self, entries: Iterable[Any]) -> Self:
        del self.message.grid.rows[:]
        self.message.grid.rows.extend(entries)
        return self

    def columns(self, entries: Iterable[Any]) -> Self:
        del self.message.grid.columns[:]
        self.message.grid.columns.extend(entries)
        return self

    def highlight(self, highlight: Any) -> Self:
        if highlight.DESCRIPTOR.full_name != "dokime.v1.GridHighlight":
            raise TypeError("expected a dokime.v1.GridHighlight")
        self.message.grid.highlights.add().CopyFrom(highlight)
        return self

    def summary_label(self, label: str) -> Self:
        self.message.grid.summary_label = label
        return self


# === Public Functions === #


def field(stream: str, name: str) -> Expression:
    return Expression(message("DataExpression", stream=message("FieldReference", identifier=stream, field=name)))


def time_series(value: Expression, *, label: str | None = None) -> Plot:
    plot = _new_plot("TIME_SERIES", value)
    return plot.label(label) if label is not None else plot


def frequency_series(value: Expression, *, label: str | None = None) -> Plot:
    plot = _new_plot("FREQUENCY_SERIES", value)
    return plot.label(label) if label is not None else plot


def xy(value: Expression, *, label: str | None = None) -> Plot:
    plot = _new_plot("XY", value)
    return plot.label(label) if label is not None else plot


def trajectory(value: Expression, *, label: str | None = None) -> Plot:
    plot = _new_plot("TRAJECTORY", value)
    return plot.label(label) if label is not None else plot


def contour(value: Expression, *, label: str | None = None) -> Plot:
    plot = _new_plot("CONTOUR", value)
    return plot.label(label) if label is not None else plot


def grid_chart(value: Expression, *, label: str | None = None) -> Plot:
    plot = _new_plot("GRID_CHART", value)
    return plot.label(label) if label is not None else plot


def bar(value: Expression, *, label: str | None = None) -> Plot:
    plot = _new_plot("BAR", value)
    return plot.label(label) if label is not None else plot


def pie(value: Expression, *, label: str | None = None) -> Plot:
    plot = _new_plot("PIE", value)
    return plot.label(label) if label is not None else plot


def histogram(value: Expression, *, label: str | None = None) -> Plot:
    plot = _new_plot("HISTOGRAM", value)
    return plot.label(label) if label is not None else plot


def box_plot(value: Expression, *, label: str | None = None) -> Plot:
    plot = _new_plot("BOX_PLOT", value)
    return plot.label(label) if label is not None else plot


# === Private Functions === #


def _new_plot(kind: str, value: Expression) -> Plot:
    result = message("Plot", kind=enum_value("PlotKind", f"PLOT_KIND_{kind}"))
    if kind == "CONTOUR":
        result.z.CopyFrom(_expression(value))
    elif kind == "HISTOGRAM":
        result.x.CopyFrom(_expression(value))
    else:
        result.y.add().CopyFrom(_expression(value))
    return Plot(result)


def _expression(value: Expression) -> Any:
    if not isinstance(value, Expression):
        raise TypeError("plot bindings require a field expression, not text")
    return value.message


def _enum(value: IntEnum, expected: type[IntEnum]) -> int:
    if type(value) is not expected:
        raise TypeError(f"expected {expected.__name__}, not text or an integer")
    return int(value)


def __getattr__(name: str) -> Any:
    if name in _RETIRED:
        raise AttributeError(f"{name}: use {_RETIRED[name]}")
    raise AttributeError(name)


# === Entry Point === #
