# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Plot Builders Over Canonical Model Values."""

# === Imports === #

# Standard Library Imports
from collections.abc import Iterable
from enum import IntEnum
from typing import Self, cast

# Third Party Imports
# Local Imports
from .._generated import shared
from .._generated.shared import (
    DataExpression,
    FieldReference,
    GridAxisEntry,
    GridHighlight,
    PlotKind,
    ReferenceBand,
    ReferenceLine,
    TransformExpression,
)

# Type Checking Imports

# === Public Exports === #


# === Constants === #

# === Types === #

# === Classes === #


class Reduction(IntEnum):
    MEAN = shared.Reduction.Value("REDUCTION_MEAN")
    MIN = shared.Reduction.Value("REDUCTION_MIN")
    MAX = shared.Reduction.Value("REDUCTION_MAX")


class ContourRender(IntEnum):
    CELLS = shared.ContourRender.Value("CONTOUR_RENDER_CELLS")
    BANDS = shared.ContourRender.Value("CONTOUR_RENDER_BANDS")


class BarArrangement(IntEnum):
    GROUPED = shared.BarArrangement.Value("BAR_ARRANGEMENT_GROUPED")
    STACKED = shared.BarArrangement.Value("BAR_ARRANGEMENT_STACKED")


class XyInterval(IntEnum):
    SIGMA = shared.XyInterval.Value("XY_INTERVAL_SIGMA")


class PlotStyle(IntEnum):
    LINE = shared.PlotStyle.Value("PLOT_STYLE_LINE")
    POINTS = shared.PlotStyle.Value("PLOT_STYLE_POINTS")
    LINE_POINTS = shared.PlotStyle.Value("PLOT_STYLE_LINE_POINTS")
    STEP = shared.PlotStyle.Value("PLOT_STYLE_STEP")


class AxisScale(IntEnum):
    LINEAR = shared.AxisScale.Value("AXIS_SCALE_LINEAR")
    LOG = shared.AxisScale.Value("AXIS_SCALE_LOG")


class Aggregate(IntEnum):
    COUNT = shared.Aggregate.Value("AGGREGATE_COUNT")
    SUM = shared.Aggregate.Value("AGGREGATE_SUM")
    MIN = shared.Aggregate.Value("AGGREGATE_MIN")
    MAX = shared.Aggregate.Value("AGGREGATE_MAX")
    MEAN = shared.Aggregate.Value("AGGREGATE_MEAN")
    STDDEV = shared.Aggregate.Value("AGGREGATE_STDDEV")
    MEDIAN = shared.Aggregate.Value("AGGREGATE_MEDIAN")
    PERCENTILE = shared.Aggregate.Value("AGGREGATE_PERCENTILE")
    RMS = shared.Aggregate.Value("AGGREGATE_RMS")


class Expression:
    def __init__(self, value: DataExpression) -> None:
        if not isinstance(value, DataExpression):
            raise TypeError("Expected a DataExpression")
        self.message = value

    def label(self, label: str) -> Self:
        value = DataExpression()
        value.CopyFrom(self.message)
        value.label = label
        return type(self)(value)

    def scale(self, factor: float) -> Self:
        return self._transform(TransformExpression(scale=factor))

    def offset(self, value: float) -> Self:
        return self._transform(TransformExpression(offset=value))

    def detail(self, name: str) -> Self:
        return self._transform(TransformExpression(detail=name))

    def time_between(self, record: str) -> Self:
        return self._transform(TransformExpression(time_between=record))

    def per_case(self) -> Self:
        return self._transform(TransformExpression(per_case=True))

    def aggregate(self, kind: Aggregate, *, percentile: float = 0.0) -> Self:
        value = DataExpression(label=self.message.label)
        value.aggregate.input.CopyFrom(self.message)
        value.aggregate.kind = cast(shared.Aggregate, _enum(kind, Aggregate))
        value.aggregate.percentile = percentile
        return type(self)(value)

    def percentile(self, rank: float) -> Self:
        return self.aggregate(Aggregate.PERCENTILE, percentile=rank)

    def _transform(self, transform: TransformExpression) -> Self:
        value = DataExpression(label=self.message.label)
        value.transform.CopyFrom(transform)
        value.transform.input.CopyFrom(self.message)
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
    @classmethod
    def line(cls, identifier: str, label: str) -> "Plot":
        return cls(shared.Plot(metadata=shared.Metadata(identifier=identifier, label=label), kind=shared.PLOT_KIND_TIME_SERIES))

    def __init__(self, value: shared.Plot) -> None:
        if not isinstance(value, shared.Plot):
            raise TypeError("Expected a Plot")
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
        from ._documents import metadata

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
        self.message.reduction = cast(shared.Reduction, _enum(reduction, Reduction))
        return self

    def render(self, render: ContourRender) -> Self:
        self.message.render = cast(shared.ContourRender, _enum(render, ContourRender))
        return self

    def arrangement(self, arrangement: BarArrangement) -> Self:
        self.message.arrangement = cast(shared.BarArrangement, _enum(arrangement, BarArrangement))
        return self

    def interval(self, interval: XyInterval) -> Self:
        self.message.interval = cast(shared.XyInterval, _enum(interval, XyInterval))
        return self

    def style(self, style: PlotStyle) -> Self:
        self.message.style = cast(shared.PlotStyle, _enum(style, PlotStyle))
        return self

    def x_scale(self, scale: AxisScale) -> Self:
        self.message.x_scale = cast(shared.AxisScale, _enum(scale, AxisScale))
        return self

    def y_scale(self, scale: AxisScale) -> Self:
        self.message.y_scale = cast(shared.AxisScale, _enum(scale, AxisScale))
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

    def input_samples(self, count: int) -> Self:
        self.message.input_samples = count
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

    def rows(self, entries: Iterable[GridAxisEntry]) -> Self:
        del self.message.grid.rows[:]
        self.message.grid.rows.extend(entries)
        return self

    def columns(self, entries: Iterable[GridAxisEntry]) -> Self:
        del self.message.grid.columns[:]
        self.message.grid.columns.extend(entries)
        return self

    def highlight(self, highlight: GridHighlight) -> Self:
        if not isinstance(highlight, GridHighlight):
            raise TypeError("Expected a GridHighlight")
        self.message.grid.highlights.add().CopyFrom(highlight)
        return self

    def summary_label(self, label: str) -> Self:
        self.message.grid.summary_label = label
        return self

    def reference_line(self, line: ReferenceLine) -> Self:
        if not isinstance(line, ReferenceLine):
            raise TypeError("Expected a ReferenceLine message")
        self.message.reference_lines.add().CopyFrom(line)
        return self

    def markers(self, *records: str) -> Self:
        self.message.marker_records.extend(records)
        return self

    def reference_band(self, band: ReferenceBand) -> Self:
        if not isinstance(band, ReferenceBand):
            raise TypeError("Expected a ReferenceBand message")
        self.message.reference_bands.add().CopyFrom(band)
        return self


# === Public Functions === #


def field(stream: str, name: str) -> Expression:
    return Expression(DataExpression(stream=FieldReference(identifier=stream, field=name)))


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
    result = shared.Plot(kind=cast(PlotKind, PlotKind.Value(f"PLOT_KIND_{kind}")))
    if kind == "CONTOUR":
        result.z.CopyFrom(_expression(value))
    elif kind == "HISTOGRAM":
        result.x.CopyFrom(_expression(value))
    else:
        result.y.add().CopyFrom(_expression(value))
    return Plot(result)


def _expression(value: Expression) -> DataExpression:
    if not isinstance(value, Expression):
        raise TypeError("plot bindings require a field expression, not text")
    return value.message


def _enum(value: IntEnum, expected: type[IntEnum]) -> int:
    if type(value) is not expected:
        raise TypeError(f"expected {expected.__name__}, not text or an integer")
    return int(value)


# === Entry Point === #
