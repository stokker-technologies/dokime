# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Record schema declarations: field/record schemas and HubRecord."""

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
    DataExpression,
    JsonObject,
    _record_field_contract,
    _is_nullable,
    _title,
    value_type_name,
)

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class FieldDeclaration:
    name: str
    value_type: object = str
    label: str | None = None
    unit: str | None = None
    role: str | None = None
    description: str | None = None
    required: bool = True
    metadata: JsonObject = dataclass_field(default_factory=dict)

    @property
    def identifier(self) -> str:
        return self.name

    def to_contract(self) -> JsonObject:
        payload: JsonObject = {
            "identifier": self.name,
            "label": self.label or _title(self.name),
            "description": self.description,
            "value_type": value_type_name(self.value_type),
            "required": self.required,
            "nullable": _is_nullable(self.value_type),
            "unit": self.unit,
            "role": self.role,
            "metadata": dict(self.metadata),
        }
        return payload


@dataclass(frozen=True)
class RecordDeclaration:
    record_type: str
    fields: tuple[FieldDeclaration, ...] = ()
    label: str | None = None
    description: str | None = None
    default_level: str = "info"
    icon: str | None = None
    tone: str | None = None
    allow_extra_fields: bool = False
    metadata: JsonObject = dataclass_field(default_factory=dict)

    @property
    def identifier(self) -> str:
        return self.record_type

    def to_contract(self, *, template_id: str | None = None) -> JsonObject:
        return _native.record_contract(
            self.record_type,
            self.label,
            self.description,
            self.default_level,
            self.allow_extra_fields,
            [_record_field_contract(field) for field in self.fields],
            stable_id("template", template_id) if template_id is not None else None,
            self.icon,
            self.tone,
        )

    def to_expression(self) -> DataExpression:
        return DataExpression({"type": "record", "record_type": self.record_type})

    def count(self) -> DataExpression:
        return self.to_expression().count()

    def mean(self) -> DataExpression:
        return self.to_expression().mean()

    def median(self) -> DataExpression:
        return self.to_expression().median()

    def min(self) -> DataExpression:
        return self.to_expression().min()

    def max(self) -> DataExpression:
        return self.to_expression().max()

    def time_between(self, other_record_type: str) -> DataExpression:
        return self.to_expression().time_between(other_record_type)


@dataclass(frozen=True)
class HubRecord:
    record_type: str
    source: str = "python"
    message: str | None = None
    details: JsonObject = dataclass_field(default_factory=dict)
    scope: object | None = None
    template_id: str | None = None
    level: str = "info"
    timestamp: object | None = None
    session_id: str | None = None


# === Public Functions === #


def field(
    name: str,
    value_type: object = str,
    *,
    label: str | None = None,
    unit: str | None = None,
    role: str | None = None,
    description: str | None = None,
    required: bool = True,
    **metadata: object,
) -> FieldDeclaration:
    return FieldDeclaration(
        name=name,
        value_type=value_type,
        label=label,
        unit=unit,
        role=role,
        description=description,
        required=required,
        metadata=dict(metadata),
    )


# === Private Functions === #

# === Entry Point === #
