# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Telemetry schema declarations: stream/field schemas."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
from dataclasses import dataclass, field as dataclass_field

# Third Party Imports

# Local Imports
from dokime import _native
from dokime.core._validation import stable_id
from dokime.core.declarations.plots import (  # noqa: E402
    PlotDeclaration as PlotDeclaration,
    SigmaInterval as SigmaInterval,
    TimeSeriesPlot as TimeSeriesPlot,
    XyScatterPlot as XyScatterPlot,
    XyPlot as XyPlot,
    XyLinePlot as XyLinePlot,
    XyIntervalPlot as XyIntervalPlot,
    TrajectoryPlot as TrajectoryPlot,
    HeatmapPlot as HeatmapPlot,
    HistogramPlot as HistogramPlot,
    FrequencySeriesPlot as FrequencySeriesPlot,
    BarPlot as BarPlot,
    GroupedBarPlot as GroupedBarPlot,
    StackedBarPlot as StackedBarPlot,
    PiePlot as PiePlot,
    DonutPlot as DonutPlot,
    ParetoPlot as ParetoPlot,
    XyValueHeatmapPlot as XyValueHeatmapPlot,
    BoxSummaryDeclaration as BoxSummaryDeclaration,
    AggregateTablePlot as AggregateTablePlot,
)
from dokime.core.declarations import (
    JsonObject,
    _coerce_field,
    _telemetry_column_contract,
)
from dokime.core.declarations.record_schemas import FieldDeclaration

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class TelemetryFieldSource:
    stream: str
    field_name: str | None = None

    def __str__(self) -> str:
        return (
            self.stream
            if self.field_name is None
            else f"{self.stream}.{self.field_name}"
        )


@dataclass(frozen=True)
class TelemetryStreamDeclaration:
    stream: str
    label: str | None = None
    fields: tuple[FieldDeclaration, ...] = ()
    default_x: str | None = None
    allow_extra_fields: bool = False
    scalar: bool = False
    metadata: JsonObject = dataclass_field(default_factory=dict)

    @property
    def stream_id(self) -> str:
        return self.stream

    @property
    def id(self) -> str:
        return self.stream

    def get(self, field_name: str | None = None) -> TelemetryFieldSource:
        return TelemetryFieldSource(self.stream, field_name)

    def to_contract(self, *, template_id: str | None = None) -> JsonObject:
        return _native.telemetry_contract(
            self.stream,
            self.label,
            None,
            [_telemetry_column_contract(field) for field in self.fields],
            self.default_x,
            self.allow_extra_fields,
            self.scalar,
            stable_id("template", template_id) if template_id is not None else None,
        )


# === Public Functions === #


def telemetry_stream(
    stream: str,
    *,
    label: str | None = None,
    fields: list[FieldDeclaration | tuple[object, ...]]
    | tuple[FieldDeclaration | tuple[object, ...], ...] = (),
    default_x: str | None = None,
    allow_extra_fields: bool = False,
    scalar: bool = False,
    **metadata: object,
) -> TelemetryStreamDeclaration:
    return TelemetryStreamDeclaration(
        stream=stream,
        label=label,
        fields=tuple(_coerce_field(item) for item in fields),
        default_x=default_x,
        allow_extra_fields=allow_extra_fields,
        scalar=scalar,
        metadata=dict(metadata),
    )


# === Private Functions === #

# === Entry Point === #
