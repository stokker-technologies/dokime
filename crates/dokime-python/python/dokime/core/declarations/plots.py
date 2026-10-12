# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Plot declarations: PlotDeclaration + its leaf plot/table subclasses + SigmaInterval."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
from copy import copy
from dataclasses import dataclass, field as dataclass_field
from typing import Self, cast

# Third Party Imports

# Local Imports
from dokime import _native
from dokime.core.declarations import (
    JsonObject,
    _dotted_source_stream,
    _expression_contract,
    _expression_contracts,
    _field_list,
    _field_name,
    _filter_contracts,
    _histogram_options,
    _interval_options,
    _mapping_like,
    _object_list,
    _optional_expression_contract,
    _optional_float,
    _optional_format,
    _plot_options,
    _source_stream,
    _source_field,
    _string_list,
    telemetry,
)

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class PlotDeclaration:
    label: str
    source: object | None = None
    x_value: object | None = None
    y_value: object | list[object] | None = None
    z_value: object | None = None
    series_by_value: object | None = None
    filters: tuple[JsonObject, ...] = ()
    group: str | None = None
    unit: str | None = None
    plot_type: str = "plot"
    options: JsonObject = dataclass_field(default_factory=dict)

    def id(self, identifier: str) -> Self:
        return self._replace_option(identifier=identifier)

    def analyze(
        self, identifier: str, *, version: int = 1, parameters: JsonObject | None = None
    ) -> Self:
        """Select an admitted analysis over the original evidence."""
        return self._replace_option(
            analysis={
                "id": identifier,
                "version": version,
                "parameters": dict(parameters or {}),
            }
        )

    def group_by(self, expression: object) -> Self:
        return self.series_by(expression)

    def series_by(self, expression: object) -> Self:
        return self._replace(
            label=self.label,
            source=self.source,
            x_value=self.x_value,
            y_value=self.y_value,
            z_value=self.z_value,
            series_by_value=expression,
            filters=self.filters,
            group=self.group,
            unit=self.unit,
            plot_type=self.plot_type,
            options=dict(self.options),
        )

    def with_group(self, group: str) -> Self:
        return self._replace(
            label=self.label,
            source=self.source,
            x_value=self.x_value,
            y_value=self.y_value,
            z_value=self.z_value,
            series_by_value=self.series_by_value,
            filters=self.filters,
            group=group,
            unit=self.unit,
            plot_type=self.plot_type,
            options=dict(self.options),
        )

    def with_unit(self, unit: str) -> Self:
        return self._replace(
            label=self.label,
            source=self.source,
            x_value=self.x_value,
            y_value=self.y_value,
            z_value=self.z_value,
            series_by_value=self.series_by_value,
            filters=self.filters,
            group=self.group,
            unit=unit,
            plot_type=self.plot_type,
            options=dict(self.options),
        )

    def style(self, style: str) -> Self:
        return self._replace_option(style=style)

    def marker_records(self, record_types: list[str] | tuple[str, ...]) -> Self:
        return self._replace_option(marker_record_types=list(record_types))

    def x(self, expression: object) -> Self:
        return self.x_axis(expression)

    def x_axis(self, expression: object) -> Self:
        return self._replace(
            label=self.label,
            source=self.source,
            x_value=expression,
            y_value=self.y_value,
            z_value=self.z_value,
            series_by_value=self.series_by_value,
            filters=self.filters,
            group=self.group,
            unit=self.unit,
            plot_type=self.plot_type,
            options=dict(self.options),
        )

    def y(self, expression: object) -> Self:
        return self.y_axis(expression)

    def y_axis(self, expression: object) -> Self:
        values = (
            [*cast(list[object], self.y_value)]
            if isinstance(self.y_value, list)
            else []
        )
        if self.y_value is not None and not isinstance(self.y_value, list):
            values.append(self.y_value)
        values.append(expression)
        return self._replace(
            label=self.label,
            source=self.source,
            x_value=self.x_value,
            y_value=values,
            z_value=self.z_value,
            series_by_value=self.series_by_value,
            filters=self.filters,
            group=self.group,
            unit=self.unit,
            plot_type=self.plot_type,
            options=dict(self.options),
        )

    def z(self, expression: object) -> Self:
        return self.z_axis(expression)

    def z_axis(self, expression: object) -> Self:
        return self._replace(
            label=self.label,
            source=self.source,
            x_value=self.x_value,
            y_value=self.y_value,
            z_value=expression,
            series_by_value=self.series_by_value,
            filters=self.filters,
            group=self.group,
            unit=self.unit,
            plot_type=self.plot_type,
            options=dict(self.options),
        )

    def filter(
        self, expression: object, operator: str, value: object | None = None
    ) -> Self:
        return self._replace(
            label=self.label,
            source=self.source,
            x_value=self.x_value,
            y_value=self.y_value,
            z_value=self.z_value,
            series_by_value=self.series_by_value,
            filters=(
                *self.filters,
                {
                    "expression": _expression_contract(expression),
                    "operator": operator,
                    "value": value,
                },
            ),
            group=self.group,
            unit=self.unit,
            plot_type=self.plot_type,
            options=dict(self.options),
        )

    def to_contract(self) -> JsonObject:
        options = _plot_options(self.options)
        source = options.pop("source", self.source)
        x = options.pop("x", options.pop("x_value", self.x_value))
        y = options.pop("y", options.pop("y_value", self.y_value))
        z = options.pop(
            "z", options.pop("z_field", options.pop("z_value", self.z_value))
        )
        series_by_value = options.pop(
            "series_by", options.pop("series_by_value", self.series_by_value)
        )
        z_stream = options.pop(
            "z_stream", _source_stream(z) or _dotted_source_stream(z)
        )
        group = options.pop("group", self.group)
        unit = options.pop("unit", self.unit)
        identifier = options.pop("identifier", options.pop("plot_id", None))
        live_window_s = options.pop("live_window_s", options.pop("window_s", None))
        x_format = options.pop("x_format", None)
        y_format = options.pop("y_format", None)
        analysis = options.pop("analysis", None)

        return _native.document_plot_contract(
            self.label,
            self.plot_type,
            None if identifier is None else str(identifier),
            _source_stream(source),
            _field_name(x),
            _field_list(y, fallback=_source_field(source)),
            None if z_stream is None else str(z_stream),
            _field_name(z),
            _field_name(series_by_value),
            _optional_expression_contract(x),
            _expression_contracts(y),
            _optional_expression_contract(z),
            _optional_expression_contract(series_by_value),
            [*self.filters, *_filter_contracts(options.pop("filters", ()))],
            None if group is None else str(group),
            None if unit is None else str(unit),
            None if options.get("style") is None else str(options.pop("style")),
            None if options.get("x_scale") is None else str(options.pop("x_scale")),
            None if options.get("y_scale") is None else str(options.pop("y_scale")),
            _mapping_like(options.pop("fft", None)),
            _interval_options(options.pop("interval", None)),
            _histogram_options(options, unit),
            _string_list(options.pop("marker_record_types", ())),
            _object_list(options.pop("reference_lines", ())),
            _object_list(options.pop("reference_bands", ())),
            _optional_float(live_window_s),
            _optional_format(x_format),
            _optional_format(y_format),
            None if analysis is None else _mapping_like(analysis),
        )

    def _initialize(self, label: str, plot_type: str, kwargs: JsonObject) -> None:
        options = dict(kwargs)
        PlotDeclaration.__init__(
            self,
            label=label,
            plot_type=plot_type,
            source=options.pop("source", None),
            x_value=options.pop("x", None),
            y_value=options.pop("y", None),
            z_value=options.pop("z", None),
            series_by_value=options.pop("series_by", None),
            filters=tuple(_filter_contracts(options.pop("filters", ()))),
            group=cast(str | None, options.pop("group", None)),
            unit=cast(str | None, options.pop("unit", None)),
            options=options,
        )

    def _replace(self, **updates: object) -> Self:
        result = copy(self)
        for name, value in updates.items():
            object.__setattr__(result, name, value)
        return result

    def _replace_option(self, **updates: object) -> Self:
        options = dict(self.options)
        options.update(updates)
        return self._replace(
            label=self.label,
            source=self.source,
            x_value=self.x_value,
            y_value=self.y_value,
            z_value=self.z_value,
            series_by_value=self.series_by_value,
            filters=self.filters,
            group=self.group,
            unit=self.unit,
            plot_type=self.plot_type,
            options=options,
        )


class SigmaInterval:
    def __init__(self, **options: object) -> None:
        self.options = dict(options)


class TimeSeriesPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "time_series", kwargs)


class XyScatterPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "xy_scatter", kwargs)


class XyPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "xy_scatter", kwargs)


class XyLinePlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "xy_line", kwargs)


class XyIntervalPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "xy_interval", kwargs)


class TrajectoryPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "trajectory", kwargs)


class HeatmapPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "heatmap", kwargs)


class WifiChannelOccupancyPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        options = {
            "source": telemetry("host.wifi.channel_occupancy", "network_count"),
            "x": "timestamp",
            "y": "network_count",
            "series_by": "wifi_channel",
            "z": "wifi_network_congestion_score",
            "style": "points",
            **kwargs,
        }
        self._initialize(label, "wifi_channel_occupancy", options)


class HistogramPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "histogram", kwargs)


class FrequencySeriesPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "frequency_series", kwargs)


class BarPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "bar", kwargs)


class GroupedBarPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "grouped_bar", kwargs)


class StackedBarPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "stacked_bar", kwargs)


class PiePlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "pie", kwargs)


class DonutPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "donut", kwargs)


class ParetoPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "pareto", kwargs)


class XyValueHeatmapPlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "xy_value_heatmap", kwargs)


class BoxSummaryDeclaration(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "box_summary", kwargs)


class AggregateTablePlot(PlotDeclaration):
    def __init__(self, label: str, **kwargs: object) -> None:
        self._initialize(label, "aggregate_table", kwargs)


# === Public Functions === #

# === Private Functions === #


# === Entry Point === #
