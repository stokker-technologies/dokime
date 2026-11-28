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
from .._generated.proto.model.v1.expressions_pb2 import Aggregate as AggregateMessage
from .._generated.proto.model.v1.expressions_pb2 import AtExpression, DataExpression, FieldReference, TransformExpression
from .._generated.proto.model.v1.expressions_pb2 import FilterOperator as FilterOperatorMessage
from .._generated.proto.model.v1.metadata_pb2 import Metadata
from .._generated.proto.model.v1.plots_pb2 import (
    PLOT_KIND_TIME_SERIES,
    GridAxisEntry,
    GridHighlight,
    PlotKind,
    ReferenceBand,
    ReferenceLine,
)
from .._generated.proto.model.v1.plots_pb2 import AxisScale as AxisScaleMessage
from .._generated.proto.model.v1.plots_pb2 import BarArrangement as BarArrangementMessage
from .._generated.proto.model.v1.plots_pb2 import ContourRender as ContourRenderMessage
from .._generated.proto.model.v1.plots_pb2 import Plot as PlotMessage
from .._generated.proto.model.v1.plots_pb2 import PlotStyle as PlotStyleMessage
from .._generated.proto.model.v1.plots_pb2 import Reduction as ReductionMessage
from .._generated.proto.model.v1.plots_pb2 import XyInterval as XyIntervalMessage
from .._values import Scalar, encode

# Type Checking Imports

# === Public Exports === #


# === Constants === #

# === Types === #

# === Classes === #


class Reduction(IntEnum):
    MEAN = ReductionMessage.Value("REDUCTION_MEAN")
    MIN = ReductionMessage.Value("REDUCTION_MIN")
    MAX = ReductionMessage.Value("REDUCTION_MAX")
    LAST = ReductionMessage.Value("REDUCTION_LAST")
    COUNT = ReductionMessage.Value("REDUCTION_COUNT")


class ContourRender(IntEnum):
    CELLS = ContourRenderMessage.Value("CONTOUR_RENDER_CELLS")
    BANDS = ContourRenderMessage.Value("CONTOUR_RENDER_BANDS")


class BarArrangement(IntEnum):
    GROUPED = BarArrangementMessage.Value("BAR_ARRANGEMENT_GROUPED")
    STACKED = BarArrangementMessage.Value("BAR_ARRANGEMENT_STACKED")


class XyInterval(IntEnum):
    SIGMA = XyIntervalMessage.Value("XY_INTERVAL_SIGMA")


class PlotStyle(IntEnum):
    LINE = PlotStyleMessage.Value("PLOT_STYLE_LINE")
    POINTS = PlotStyleMessage.Value("PLOT_STYLE_POINTS")
    LINE_POINTS = PlotStyleMessage.Value("PLOT_STYLE_LINE_POINTS")
    STEP = PlotStyleMessage.Value("PLOT_STYLE_STEP")


class AxisScale(IntEnum):
    LINEAR = AxisScaleMessage.Value("AXIS_SCALE_LINEAR")
    LOG = AxisScaleMessage.Value("AXIS_SCALE_LOG")


class Aggregate(IntEnum):
    COUNT = AggregateMessage.Value("AGGREGATE_COUNT")
    SUM = AggregateMessage.Value("AGGREGATE_SUM")
    MIN = AggregateMessage.Value("AGGREGATE_MIN")
    MAX = AggregateMessage.Value("AGGREGATE_MAX")
    MEAN = AggregateMessage.Value("AGGREGATE_MEAN")
    STDDEV = AggregateMessage.Value("AGGREGATE_STDDEV")
    MEDIAN = AggregateMessage.Value("AGGREGATE_MEDIAN")
    PERCENTILE = AggregateMessage.Value("AGGREGATE_PERCENTILE")
    RMS = AggregateMessage.Value("AGGREGATE_RMS")


class FilterOperator(IntEnum):
    EQ = FilterOperatorMessage.Value("FILTER_OPERATOR_EQ")
    NE = FilterOperatorMessage.Value("FILTER_OPERATOR_NE")
    GT = FilterOperatorMessage.Value("FILTER_OPERATOR_GT")
    GE = FilterOperatorMessage.Value("FILTER_OPERATOR_GE")
    LT = FilterOperatorMessage.Value("FILTER_OPERATOR_LT")
    LE = FilterOperatorMessage.Value("FILTER_OPERATOR_LE")
    CONTAINS = FilterOperatorMessage.Value("FILTER_OPERATOR_CONTAINS")
    EXISTS = FilterOperatorMessage.Value("FILTER_OPERATOR_EXISTS")


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

    def at(self, record_type: str) -> Self:
        return self._transform(TransformExpression(at=AtExpression(record_type=record_type)))

    def within_s(self, tolerance_s: float) -> Self:
        if self.message.WhichOneof("kind") != "transform" or self.message.transform.WhichOneof("kind") != "at":
            raise ValueError("Set Within Seconds: Apply at Before within_s")
        value = DataExpression()
        value.CopyFrom(self.message)
        value.transform.at.within_s = tolerance_s
        return type(self)(value)

    def per_case(self) -> Self:
        return self._transform(TransformExpression(per_case=True))

    def aggregate(self, kind: Aggregate, *, percentile: float = 0.0) -> Self:
        value = DataExpression(label=self.message.label)
        value.aggregate.input.CopyFrom(self.message)
        value.aggregate.kind = cast(AggregateMessage, _enum(kind, Aggregate))
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
        return cls(PlotMessage(metadata=Metadata(identifier=identifier, label=label), kind=PLOT_KIND_TIME_SERIES))

    def __init__(self, value: PlotMessage) -> None:
        if not isinstance(value, PlotMessage):
            raise TypeError("Expected a Plot")
        self.message = value

    def x(self, value: Expression) -> Self:
        self.message.x.CopyFrom(_expression(value))
        return self

    def filter(self, expression: Expression, operator: FilterOperator, value: Scalar = None) -> Self:
        self.message.filters.add(
            expression=_expression(expression),
            operator=cast(FilterOperatorMessage, _enum(operator, FilterOperator)),
            value=encode(value),
        )
        return self

    def y(self, value: Expression) -> Self:
        self.message.y.add().CopyFrom(_expression(value))
        return self

    def cells(self, value: Expression) -> Self:
        self.message.cell.CopyFrom(_expression(value))
        return self

    def facet(self, value: Expression) -> Self:
        self.message.facet.CopyFrom(_expression(value))
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
        self.message.reduction = cast(ReductionMessage, _enum(reduction, Reduction))
        return self

    def render(self, render: ContourRender) -> Self:
        self.message.render = cast(ContourRenderMessage, _enum(render, ContourRender))
        return self

    def arrangement(self, arrangement: BarArrangement) -> Self:
        self.message.arrangement = cast(BarArrangementMessage, _enum(arrangement, BarArrangement))
        return self

    def interval(self, interval: XyInterval) -> Self:
        self.message.interval = cast(XyIntervalMessage, _enum(interval, XyInterval))
        return self

    def style(self, style: PlotStyle) -> Self:
        self.message.style = cast(PlotStyleMessage, _enum(style, PlotStyle))
        return self

    def x_scale(self, scale: AxisScale) -> Self:
        self.message.x_scale = cast(AxisScaleMessage, _enum(scale, AxisScale))
        return self

    def y_scale(self, scale: AxisScale) -> Self:
        self.message.y_scale = cast(AxisScaleMessage, _enum(scale, AxisScale))
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


def heat_strip(value: Expression, *, label: str | None = None) -> Plot:
    plot = _new_plot("HEAT_STRIP", value).reduce(Reduction.LAST)
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
    result = PlotMessage(kind=cast(PlotKind, PlotKind.Value(f"PLOT_KIND_{kind}")))
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
