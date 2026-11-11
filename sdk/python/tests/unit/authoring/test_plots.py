# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Canonical Document Builder Semantics."""

# === Imports === #

# Standard Library Imports
from dataclasses import dataclass

# Local Imports
import dokime

# Third Party Imports
import pytest
from dokime import plot
from dokime.v1 import message

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_canonical_contour_and_grid_builders() -> None:
    @dokime.stream(id="scan", label="Scan", window=32)
    @dataclass
    class Scan:
        temperature: float

    value = Scan.field("temperature").scale(1.0)
    surface = (
        plot.contour(value)
        .x(Scan.field("position_x"))
        .y(Scan.field("position_y"))
        .label("Temperature Map")
        .unit("°C")
        .reduce(plot.Reduction.MEAN)
        .render(plot.ContourRender.BANDS)
    )
    surface.reference_band(
        message(
            "ReferenceBand", center=message("DataExpression", parameter="target_y"), tolerance=message("DataExpression", parameter="band_y")
        )
    )
    grid = (
        plot.grid_chart(Scan.field("value"))
        .row(Scan.field("row_key"))
        .column(Scan.field("column_key"))
        .frame(Scan.field("timestamp"))
        .rows([message("GridAxisEntry", key="sensor", label="Sensor")])
        .columns([message("GridAxisEntry", key="site", label="Site")])
        .reduce(plot.Reduction.MAX)
        .label("Sensor Grid")
    )
    document = dokime.Document("Scan").section(dokime.Section("Map").plot(surface).plot(grid))
    decoded = message("Document")
    decoded.ParseFromString(bytes(document._definition()["message"]))
    first, second = decoded.sections[0].cards
    assert first.plot.DESCRIPTOR.full_name == "dokime.v1.Plot"
    assert first.plot.kind == 18
    assert first.plot.z.transform.scale == 1.0
    assert first.plot.z.transform.input.stream.identifier == "scan"
    assert first.plot.z.transform.input.stream.field == "temperature"
    assert first.plot.reduction == int(plot.Reduction.MEAN)
    assert first.plot.render == int(plot.ContourRender.BANDS)
    assert first.plot.reference_bands[0].center.parameter == "target_y"
    assert first.plot.reference_bands[0].tolerance.parameter == "band_y"
    assert second.plot.grid.frame.stream.field == "timestamp"
    assert second.plot.reduction == int(plot.Reduction.MAX)
    assert not value.message.HasField("aggregate")
    assert value.mean().offset(2.0).message.transform.input.aggregate.input == value.message


@pytest.mark.parametrize(
    "method,argument",
    [
        ("reduce", "mean"),
        ("render", "bands"),
        ("arrangement", "grouped"),
        ("interval", "sigma"),
        ("pareto", 1),
    ],
)
def test_options_refuse_untyped_values(method: str, argument: object) -> None:
    with pytest.raises(TypeError):
        getattr(plot.xy(plot.field("scan", "value")), method)(argument)


@pytest.mark.parametrize(
    "name,replacement",
    [
        ("heatmap", "contour"),
        ("xy_value_heatmap", "contour"),
        ("wifi_channel_occupancy", "grid_chart"),
        ("grouped_bar", "arrangement"),
        ("stacked_bar", "arrangement"),
        ("donut", "hole"),
        ("pareto", "bar"),
        ("xy_interval", "interval"),
        ("box_summary", "box_plot"),
        ("aggregate_table", "Summary"),
    ],
)
def test_retired_constructors_name_replacement(name: str, replacement: str) -> None:
    with pytest.raises(AttributeError, match=replacement):
        getattr(plot, name)


def test_all_kinds_and_typed_options() -> None:
    value = plot.field("scan", "value")
    assert {
        getattr(plot, name)(value).message.kind
        for name in ("time_series", "frequency_series", "xy", "trajectory", "contour", "grid_chart", "bar", "pie", "histogram", "box_plot")
    } == {1, 2, 5, 8, 10, 14, 15, 18, 19, 20}
    assert not plot.bar(value).message.HasField("arrangement")
    assert plot.bar(value).arrangement(plot.BarArrangement.GROUPED).message.arrangement == 1
    assert plot.bar(value).arrangement(plot.BarArrangement.STACKED).message.arrangement == 2
    assert plot.bar(value).pareto(True).message.pareto
    assert plot.pie(value).hole(0.5).message.hole == 0.5
    assert plot.xy(value).interval(plot.XyInterval.SIGMA).message.interval == 1
    assert plot.contour(value).id("stable").label("Temperature Map").message.metadata.identifier == "stable"


# === Private Functions === #

# === Entry Point === #
