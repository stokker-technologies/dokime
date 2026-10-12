# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from __future__ import annotations
from collections.abc import Callable
from collections.abc import Sequence
from datetime import UTC, datetime, timedelta
from typing import ClassVar
from typing import cast

# Third Party Imports
import pytest

# Local Imports
from dokime import Metadata
from dokime import ParameterDefinition
from dokime.core import record
from dokime.core.authoring import TestCaseTemplate
from dokime.core.authoring import TestSuite
from dokime.core.authoring import CredentialDefinition
from dokime.core.authoring import CredentialSchema
from dokime.core.authoring import add
from dokime.core.authoring import linspace
from dokime.core.authoring import parameter
from dokime.core.authoring import reference
from .._support import DynamicAuthoringTemplate
from .._support import fixture_suite

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def case_suite_builds():
    suite = fixture_suite().to_contract()

    assert suite["metadata"]["identifier"] == "fixture.suite"
    assert (
        suite["test_case_templates"][0]["metadata"]["identifier"] == "fixture.template"
    )


def case_parameter_contracts():
    parameter = ParameterDefinition(
        Metadata("distance", "Distance"),
        "float",
        default=1.25,
        unit={
            "metadata": {
                "identifier": "unit.mm",
                "label": "mm",
                "description": None,
            },
            "family": "custom",
            "symbol": "mm",
        },
    ).to_contract()
    unit = parameter["unit"]

    assert parameter["value_type"] == "number"
    assert parameter["expression"] == {
        "kind": "literal",
        "value": {"type": "number", "value": "1.25"},
    }
    assert parameter["extensions"] == {}
    assert parameter["list"] is None
    assert isinstance(unit, dict)
    assert unit["symbol"] == "mm"


def case_list_bounds():
    parameter = ParameterDefinition(
        Metadata("hold_times", "Hold times"),
        "list",
        default=[30, 60, 120],
        attributes={"element_type": "int", "min_items": 1, "max_items": 8},
    ).to_contract()

    # The parameter stays an array for every reader; the list block is what
    # carries the element type and the bounds.
    assert parameter["value_type"] == "array"
    assert parameter["list"] == {
        "element_type": "integer",
        "min_items": 1,
        "max_items": 8,
    }
    assert parameter["expression"] == {
        "kind": "literal",
        "value": {
            "type": "array",
            "value": [
                {"type": "integer", "value": str(item)} for item in [30, 60, 120]
            ],
        },
    }
    # The list attributes are the declaration, not opaque plugin metadata.
    assert parameter["extensions"] == {}


def case_credential_contracts():
    schema = CredentialSchema(
        credentials=[
            CredentialDefinition(
                "access_token",
                "Access token",
                description="API access token.",
                required=True,
                scopes=["fabric:write"],
            )
        ]
    )

    assert schema.to_contract() == {
        "credentials": [
            {
                "id": "access_token",
                "label": "Access token",
                "description": "API access token.",
                "required": True,
                "scopes": ["fabric:write"],
            }
        ]
    }


def case_param_cases():
    class ExpressionTemplate(TestCaseTemplate):
        distance = parameter(
            linspace(1.0, 3.0, 3), value_type=float, expansion="cartesian"
        )
        offset = parameter(
            add(reference("distance"), 1.0), value_type=float, expansion="cartesian"
        )

    parameters = {
        definition.name: definition
        for definition in ExpressionTemplate.parameter_definitions()
    }

    assert parameters["distance"].expression is not None
    assert parameters["distance"].expression.to_contract() == {
        "kind": "linspace",
        "start": 1.0,
        "stop": 3.0,
        "count": 3,
    }
    assert parameters["offset"].expression is not None
    assert parameters["offset"].expression.to_contract()["kind"] == "binary"

    cases = ExpressionTemplate.materialize(
        suite_id="fixture.registration",
    )
    assert len(cases) == 9
    assert sorted({cast(float, case.parameters["distance"]) for case in cases}) == [
        1.0,
        2.0,
        3.0,
    ]
    assert sorted({cast(float, case.parameters["offset"]) for case in cases}) == [
        2.0,
        3.0,
        4.0,
    ]


def case_materializations_identities():
    from dokime import _native
    from dokime.suites import suite_from_authored_suite

    class Measurement(TestCaseTemplate):
        template_id = "bench.measurement"
        voltage = parameter(230.0, value_type=float)
        readings = parameter([1, 2, 3], value_type=list[int])

    class Bench(TestSuite):
        suite_id = "bench"
        members = (Measurement,)

    suite = suite_from_authored_suite(Bench).to_contract()
    template = suite["test_case_templates"][0]
    native = _native.materialize_template_cases_contract(
        "fixture.registration", dict(template), {}, {}, {}, 1000, False
    )
    cases = Measurement.materialize(
        suite_id="fixture.registration",
    )
    assert len(cases) == 1
    native_cases = cast(list[dict[str, object]], native["test_cases"])
    assert cases[0].case_id != native_cases[0]["test_case_id"]
    assert (
        cases[0].case_id
        != Measurement.case(
            suite_id="fixture.registration", voltage=230.0, readings=[1, 2, 3]
        ).case_id
    )


def case_registration_metadata():
    from dokime.suites import suite_from_authored_suite

    class ParameterMetadataTemplate(TestCaseTemplate):
        template_id: ClassVar[str | None] = "fixture.parameter_metadata"

        station = parameter("A-1", label="Station", choices=["A-1", "B-2"])

    class ParameterMetadataSuite(TestSuite):
        suite_id: ClassVar[str | None] = "fixture.parameter_suite"
        members: ClassVar[Sequence[type[TestCaseTemplate]]] = (
            ParameterMetadataTemplate,
        )

    suite = suite_from_authored_suite(ParameterMetadataSuite).to_contract()
    suite_again = suite_from_authored_suite(ParameterMetadataSuite).to_contract()
    template = suite["test_case_templates"][0]
    template_parameters = {
        cast(dict[str, object], parameter_contract["metadata"])["identifier"]: (
            parameter_contract
        )
        for parameter_contract in template["parameters"]
    }

    assert (
        suite_again["test_case_templates"][0]["test_case_template_id"]
        == template["test_case_template_id"]
    )
    assert len(str(template["test_case_template_id"])) == 36
    assert set(template_parameters) == {"station"}
    station_selection = cast(
        dict[str, object], template_parameters["station"]["selection"]
    )

    assert station_selection["choices"] == [
        {"type": "text", "value": item} for item in ["A-1", "B-2"]
    ]


def case_declaration_contracts():
    from dokime import stream
    from dokime.core._validation import stable_id

    @record(id="tracking.record")
    class Tracking:
        distance: float
        note: str | None

    @stream(id="tracking.samples")
    class Samples:
        distance: float

    class Template(TestCaseTemplate):
        template_id = "fixture.template"
        records = (Tracking,)
        streams = (Samples,)
        subscribes = (Samples, Tracking)

    (record_contract,) = Template.record_contract_definitions()
    (stream_contract,) = Template.telemetry_contract_definitions()
    assert Template.subscription_sources() == [
        {"kind": "stream", "identifier": "tracking.samples"},
        {"kind": "record", "identifier": "tracking.record"},
    ]
    from dokime.suites import suite_from_authored_suite

    class SubscribedSuite(TestSuite):
        suite_id: ClassVar[str | None] = "fixture.subscriptions"
        members: ClassVar[Sequence[type[TestCaseTemplate]]] = (Template,)

    contract = suite_from_authored_suite(SubscribedSuite).to_contract()
    assert (
        contract["test_case_templates"][0]["subscriptions"]
        == Template.subscription_sources()
    )
    assert (
        len(
            Template.materialize(
                suite_id="fixture.registration",
            )
        )
        == 1
    )

    expected = stable_id("template", "fixture.template")
    assert record_contract["template_id"] == stream_contract["template_id"] == expected
    assert record_contract["record_type"] == "tracking.record"
    assert stream_contract["stream"] == "tracking.samples"
    fields = record_contract["fields"]
    assert isinstance(fields, list)
    assert fields[0]["metadata"]["identifier"] == "distance"
    assert fields[1]["value_type"] == "text"
    assert fields[1]["nullable"] is True


def case_document_plots():
    from dokime import (
        BarPlot,
        BoxSummaryDeclaration,
        Document,
        FrequencySeriesPlot,
        HeatmapPlot,
        HistogramPlot,
        Section,
        TimeSeriesPlot,
        TrajectoryPlot,
        XyIntervalPlot,
        XyLinePlot,
        XyPlot,
        XyScatterPlot,
        dimension,
        telemetry,
    )

    plots = [
        TimeSeriesPlot("Distance", source="tracking.distance_m", x="timestamp"),
        BarPlot("Counts", source="tracking.count", x="sample_index"),
        XyPlot("XY Alias", x="position_x_m", y="position_y_m"),
        XyScatterPlot("XY Scatter", x="position_x_m", y="position_y_m"),
        XyLinePlot("XY Line", x="position_x_m", y="position_y_m"),
        XyIntervalPlot("XY Interval", x="distance_m", y="error_m"),
        TrajectoryPlot("Trajectory", x="position_x_m", y="position_y_m"),
        HeatmapPlot(
            "Congestion Map",
            source="tracking",
            x="position_x_m",
            y="position_y_m",
            z="host.wifi.channel_occupancy.wifi_network_congestion_score",
        ),
        FrequencySeriesPlot("Frequency", source="tracking.distance_m", x="timestamp"),
        HistogramPlot(
            "Reacquire Time", source="tracking.target_reacquire_time_s", bins=16
        ),
        BoxSummaryDeclaration("Distance Summary", source="tracking.distance_m"),
        BarPlot("Median Distance")
        .x(dimension("serial_number"))
        .y(telemetry("tracking", "distance_m").median()),
        BarPlot("Samples By Serial")
        .x(dimension("serial_number"))
        .series_by(dimension("serial_number"))
        .y(telemetry("tracking", "distance_m").count()),
    ]
    contracts = [plot.to_contract() for plot in plots]
    plot_types = [contract["data"]["plot"]["plot_type"] for contract in contracts]

    assert plot_types == [
        "time_series",
        "bar",
        "xy_scatter",
        "xy_scatter",
        "xy_line",
        "xy_interval",
        "trajectory",
        "xy_value_heatmap",
        "frequency_series",
        "histogram",
        "box_summary",
        "bar",
        "bar",
    ]
    assert contracts[7]["data"]["plot"]["z_stream"] == "host.wifi.channel_occupancy"
    assert contracts[7]["data"]["plot"]["z_field"] == "wifi_network_congestion_score"
    assert contracts[9]["data"]["plot"]["histogram"]["bins"] == 16
    assert contracts[11]["data"]["plot"]["x_expression"]["source"] == {
        "type": "dimension",
        "name": "serial_number",
    }
    assert contracts[11]["data"]["plot"]["y_expressions"][0]["transforms"] == [
        {"type": "median"}
    ]
    cycle_document = (
        Document("default", "Default Document")
        .section(
            Section("fixture.cycle.tracking", "Tracking")
            .plot(plots[11])
            .plot(plots[12])
        )
        .section(Section("fixture.cycle.distribution", "Distribution").plot(plots[9]))
    )
    document = Document("fixture.execution", "Fixture Execution").section(
        Section("fixture.execution.tracking", "Tracking").plot(plots[0])
    )

    document_contract = cast(dict[str, object], document.to_contract())
    cycle_document_contract = cast(dict[str, object], cycle_document.to_contract())
    document_metadata = cast(dict[str, object], document_contract["metadata"])
    cycle_document_metadata = cast(
        dict[str, object], cycle_document_contract["metadata"]
    )
    assert document_metadata["identifier"] == "fixture.execution"
    assert cycle_document_metadata["identifier"] == "default"
    assert "views" not in document_contract
    document_cards = cast(list[dict[str, object]], document_contract["cards"])
    document_layout = cast(dict[str, object], document_contract["layout"])
    document_nodes = cast(list[dict[str, object]], document_layout["nodes"])
    assert len(document_cards) == 1
    assert "layout" not in document_cards[0]
    assert list(document_layout) == ["nodes"]
    document_section_node = document_nodes[0]
    document_placement = cast(
        dict[str, object],
        cast(
            list[dict[str, object]],
            cast(dict[str, object], document_section_node["data"])["children"],
        )[0],
    )
    assert document_section_node["type"] == "section"
    assert document_placement["type"] == "card"

    cycle_layout = cast(dict[str, object], cycle_document_contract["layout"])
    cycle_nodes = cast(list[dict[str, object]], cycle_layout["nodes"])
    cycle_cards = cast(list[dict[str, object]], cycle_document_contract["cards"])
    card_labels = {
        str(cast(dict[str, object], card["metadata"])["identifier"]): cast(
            dict[str, object], card["metadata"]
        )["label"]
        for card in cycle_cards
    }

    assert [node["type"] for node in cycle_nodes] == ["section", "section"]
    assert [
        cast(dict[str, object], node["data"])["node_id"] for node in cycle_nodes
    ] == ["fixture.cycle.tracking", "fixture.cycle.distribution"]

    tracking_children = cast(
        list[dict[str, object]],
        cast(dict[str, object], cycle_nodes[0]["data"])["children"],
    )
    distribution_children = cast(
        list[dict[str, object]],
        cast(dict[str, object], cycle_nodes[1]["data"])["children"],
    )

    assert [child["type"] for child in tracking_children] == ["card", "card"]
    assert [
        card_labels[str(cast(dict[str, object], child["data"])["card_id"])]
        for child in tracking_children
    ] == ["Median Distance", "Samples By Serial"]
    assert [
        card_labels[str(cast(dict[str, object], child["data"])["card_id"])]
        for child in distribution_children
    ] == ["Reacquire Time"]


def case_plot_contract():
    import pytest

    from dokime import TimeSeriesPlot
    from dokime import XyLinePlot

    time_series = TimeSeriesPlot(
        "Temperature",
        source="soak.celsius",
        y_format={"kind": "fixed", "decimals": 1},
        x_format={"kind": "elapsed_since_run", "unit": "seconds"},
    )
    contract = cast(dict[str, object], time_series.to_contract())
    plot = cast(dict[str, object], cast(dict[str, object], contract["data"])["plot"])

    assert plot["y_format"] == {"kind": "fixed", "decimals": 1}
    assert plot["x_format"] == {"kind": "elapsed_since_run", "unit": "seconds"}

    # A number format on a time_series plot's x axis is refused the same way
    # the Rust builder refuses it — one validation, read from either side.
    with pytest.raises(ValueError):
        TimeSeriesPlot(
            "Temperature",
            source="soak.celsius",
            x_format={"kind": "fixed", "decimals": 1},
        ).to_contract()

    xy_line = XyLinePlot(
        "Position",
        x="position_x_m",
        y="position_y_m",
        x_format={"kind": "scientific", "significant_digits": 3},
    )
    xy_contract = cast(dict[str, object], xy_line.to_contract())
    xy_plot = cast(
        dict[str, object], cast(dict[str, object], xy_contract["data"])["plot"]
    )
    assert xy_plot["x_format"] == {
        "kind": "scientific",
        "significant_digits": 3,
    }


def case_authored_contract():
    from dokime import Document
    from dokime import Section
    from dokime import TimeSeriesPlot
    from dokime import steps
    from dokime.core import CaseTemplate

    document = (
        Document("fixture.document", "Fixture Document")
        .section(
            Section("fixture.document.tracking", "Tracking").plot(
                TimeSeriesPlot("Distance", source="tracking.distance_m", x="timestamp")
            )
        )
        .section(Section("fixture.document.steps", "Steps").card(steps("Steps")))
    )

    template = CaseTemplate(
        metadata=Metadata("fixture.document.template", "Fixture Document Template"),
        document=document.to_contract(),
    ).to_contract()
    contract = cast(dict[str, object], template["document"])
    layout = cast(dict[str, object], contract["layout"])
    nodes = cast(list[dict[str, object]], layout["nodes"])

    assert list(layout) == ["nodes"]
    assert [cast(dict[str, object], node["data"])["node_id"] for node in nodes] == [
        "fixture.document.tracking",
        "fixture.document.steps",
    ]
    assert [
        cast(dict[str, object], card["metadata"])["identifier"]
        for card in cast(list[dict[str, object]], contract["cards"])
    ] == ["plot.distance", "card.steps"]


def case_materialized_retained():
    cases = DynamicAuthoringTemplate.materialize(
        suite_id="fixture.registration",
    )

    assert len(cases) == 1
    assert cases[0].case_id
    assert (
        cases[0].case_id
        != DynamicAuthoringTemplate.materialize(
            suite_id="fixture.registration",
        )[0].case_id
    )


def case_template_paths():
    from dokime.core import CaseTemplate
    from dokime.suites import suite_from_authored_suite

    definition: dict[str, object] = {
        "prompt_type": "fixture.confirm",
        "schema_version": 1,
        "archetype": "ConfirmationPrompt",
        "title": "Confirm the fixture",
        "message": None,
        "fields": [],
        "actions": [
            {
                "id": "confirm",
                "label": "Confirm",
                "emphasis": "primary",
                "requires_condition": False,
            }
        ],
        "condition": None,
        "rules": [],
        "owner": None,
        "template_identifier": None,
        "examples": [],
    }

    class Questions(TestCaseTemplate):
        template_id: ClassVar[str | None] = "fixture.questions"
        prompt_definitions: ClassVar[list[dict[str, object]]] = [definition]

    class QuestionsSuite(TestSuite):
        suite_id: ClassVar[str | None] = "fixture.questions.suite"
        members: ClassVar[Sequence[type[TestCaseTemplate]]] = (Questions,)

    direct = CaseTemplate(
        Metadata("fixture.direct", "Direct"), prompt_definitions=[definition]
    ).to_contract()
    authored = suite_from_authored_suite(QuestionsSuite).to_contract()
    assert direct["prompt_definitions"] == [definition]
    assert authored["test_case_templates"][0]["prompt_definitions"] == [definition]
    assert (
        fixture_suite().to_contract()["test_case_templates"][0]["prompt_definitions"]
        == []
    )
    assert "prompts" not in direct


def case_typed_cases():
    import pytest
    from dokime.suites import suite_from_authored_suite

    class BoundedLists(TestCaseTemplate):
        template_id: ClassVar[str | None] = "fixture.bounded_lists"
        holds: list[int] = parameter([1, 5], min_items=1, max_items=3, min=1, max=5)
        empty: list[str] = parameter([])
        temperature = parameter(20.0, min=-10, max=30)
        stations: list[str] = parameter(["A", "B"], min_items=1)

    class BoundedSuite(TestSuite):
        suite_id: ClassVar[str | None] = "fixture.bounded_suite"
        members: ClassVar[Sequence[type[TestCaseTemplate]]] = (BoundedLists,)

    suite = suite_from_authored_suite(BoundedSuite).to_contract()
    template = suite["test_case_templates"][0]
    declarations = {
        entry["metadata"]["identifier"]: entry for entry in template["parameters"]
    }
    assert declarations["holds"]["list"] == {
        "element_type": "integer",
        "min_items": 1,
        "max_items": 3,
    }
    assert declarations["holds"]["min"] == 1.0
    assert declarations["holds"]["max"] == 5.0
    empty_list = declarations["empty"]["list"]
    assert empty_list is not None
    assert empty_list["element_type"] == "text"
    stations_list = declarations["stations"]["list"]
    assert stations_list is not None
    assert stations_list["element_type"] == "text"
    cases = BoundedLists.materialize(
        suite_id="fixture.registration",
    )
    assert len(cases) == 1
    assert cases[0].parameters["holds"] == [1, 5]
    assert cases[0].parameters["empty"] == []
    with pytest.raises(ValueError, match="outside"):
        BoundedLists.materialize(
            suite_id="fixture.registration", case_overrides={"temperature": [31.0]}
        )
    with pytest.raises(ValueError, match="outside"):
        BoundedLists.materialize(
            suite_id="fixture.registration", case_overrides={"holds": [[1, 6]]}
        )


def case_param_attributes():
    declared = ParameterDefinition(
        Metadata("levels", "Levels"),
        "array",
        default=[0.0, 1.0],
        element_type="number",
        min_items=1,
        max_items=4,
        min=0.0,
        max=1.0,
    ).to_contract()
    assert declared["list"] == {
        "element_type": "number",
        "min_items": 1,
        "max_items": 4,
    }
    assert declared["min"] == 0.0
    assert declared["max"] == 1.0
    assert declared["extensions"] == {}


def case_metadata_dates():
    date = datetime(2026, 9, 28, 12, 13, 14, 123456, tzinfo=UTC)
    metadata = Metadata(
        "sensor",
        "Sensor",
        tags=["lab", "lab"],
        attributes={"serial": 2**64 - 1, "gain": float("inf"), "when": date},
        created_at=date,
    ).to_contract()
    assert metadata["tags"] == ["lab"]
    assert metadata["updated_at"] is None
    assert metadata["created_at"] == "2026-09-28T12:13:14.123456Z"
    attributes = cast(dict[str, object], metadata["attributes"])
    assert attributes["serial"] == {"type": "unsigned", "value": str(2**64 - 1)}
    assert attributes["gain"] == {"type": "number", "value": "inf"}
    assert attributes["when"] == {
        "type": "datetime",
        "value": "2026-09-28T12:13:14.123456Z",
    }


def case_materialized_nanoseconds():
    from dokime.core.authoring.parameters import _value_from_contract

    encoded = {"type": "datetime", "value": "2026-09-28T12:13:14.123456789Z"}
    value = _value_from_contract(encoded)
    assert isinstance(value, datetime)
    assert value != datetime(2026, 9, 28, 12, 13, 14, 123456, tzinfo=UTC)
    parameter = ParameterDefinition(
        Metadata("when", "When"), "datetime", default=value
    ).to_contract()
    expression = cast(dict[str, object], parameter["expression"])
    assert expression["value"] == encoded
    metadata = Metadata("sensor", "Sensor", created_at=value).to_contract()
    assert metadata["created_at"] == encoded["value"]
    edited = Metadata(
        "sensor", "Sensor", created_at=value.replace(second=15)
    ).to_contract()
    assert str(edited["created_at"]).startswith("2026-09-28T12:13:15.")
    advanced = Metadata(
        "sensor", "Sensor", created_at=value + timedelta(seconds=1)
    ).to_contract()
    assert str(advanced["created_at"]).startswith("2026-09-28T12:13:15.")


def case_expansion_pairs():
    import pytest

    class Expanded(TestCaseTemplate):
        template_id = "fixture.expanded"
        names: list[str] = parameter(["A", "B"])
        batches: list[str] = parameter([["A", "B"], ["C"]], expansion="cartesian")
        enabled: bool = parameter([False, True], expansion="cartesian")
        x: int = parameter([1, 2], expansion={"kind": "zip", "group": "point"})
        y: int = parameter([10, 20], expansion={"kind": "zip", "group": "point"})

    cases = Expanded.materialize(
        suite_id="fixture.registration",
    )
    assert len(cases) == 8
    for case in cases:
        assert case.parameters["names"] == ["A", "B"]
        assert case.parameters["batches"] in (["A", "B"], ["C"])
        x = case.parameters["x"]
        assert isinstance(x, int)
        assert case.parameters["y"] == x * 10

    class InvalidSingle(TestCaseTemplate):
        template_id = "fixture.invalid_single"
        enabled = parameter([False, True], value_type=bool)

    with pytest.raises(ValueError):
        InvalidSingle.materialize(
            suite_id="fixture.registration",
        )


def case_plot_original():
    from dokime import TimeSeriesPlot, telemetry

    first = telemetry("tracking", "distance_m")
    second = telemetry("tracking", "error_m")
    original = TimeSeriesPlot("Signals", y=first, style="line_points")
    chained = (
        original.y(second)
        .filter(first, "gt", 0)
        .filter(second, "lt", 10)
        .id("signals")
        .with_unit("m")
        .with_group("Tracking")
    )
    data = chained.to_contract()["data"]
    assert isinstance(data, dict)
    plot = data["plot"]
    assert len(plot["y_expressions"]) == 2
    assert len(plot["filters"]) == 2
    assert [item["value"] for item in plot["filters"]] == [
        {"type": "integer", "value": "0"},
        {"type": "integer", "value": "10"},
    ]
    assert plot["style"] == "line_points"
    assert plot["unit"] == "m"
    assert plot["group"] == "Tracking"
    assert chained.options == {"style": "line_points", "identifier": "signals"}
    original_data = original.to_contract()["data"]
    assert isinstance(original_data, dict)
    assert len(original_data["plot"]["y_expressions"]) == 1
    assert original.filters == ()


def case_unsupported_explicitly():
    import pytest
    from dokime import (
        AggregateTablePlot,
        DonutPlot,
        GroupedBarPlot,
        ParetoPlot,
        PiePlot,
        StackedBarPlot,
    )

    for author in (
        AggregateTablePlot,
        DonutPlot,
        GroupedBarPlot,
        ParetoPlot,
        PiePlot,
        StackedBarPlot,
    ):
        with pytest.raises(ValueError, match="unsupported document plot type"):
            author("Unsupported", x="index", y="value").to_contract()


def case_heatmap_author():
    from dokime import HeatmapPlot, XyValueHeatmapPlot, WifiChannelOccupancyPlot

    for author in (HeatmapPlot, XyValueHeatmapPlot):
        data = author("Heat", x="x", y="y", z="z").to_contract()["data"]
        assert isinstance(data, dict)
        plot = data["plot"]
        assert plot["plot_type"] == "xy_value_heatmap"
    data = WifiChannelOccupancyPlot("Wi-Fi").to_contract()["data"]
    assert isinstance(data, dict)
    wifi = data["plot"]
    assert wifi["plot_type"] == "wifi_channel_occupancy"
    assert wifi["stream"] == "host.wifi.channel_occupancy"
    assert wifi["y"] == ["network_count"]
    assert wifi["series_by"] == "wifi_channel"
    assert wifi["z_field"] == "wifi_network_congestion_score"
    assert wifi["style"] == "points"


def case_wifi_expressions():
    import pytest
    from dokime import WifiChannelOccupancyPlot, telemetry

    plot = WifiChannelOccupancyPlot("Wi-Fi")
    expression = telemetry("host.wifi", "wifi_rx_rate_mbps")
    for invalid in (
        plot.x(expression),
        plot.y(expression),
        plot.z(expression),
        plot.series_by(expression),
    ):
        with pytest.raises(ValueError, match="uses fixed host columns"):
            invalid.to_contract()


def case_plot_boundary():
    from datetime import datetime, timezone
    from dokime import TimeSeriesPlot, UnsignedValue, telemetry

    expression = telemetry("samples", "value")
    values = [
        False,
        -5,
        UnsignedValue(2**63),
        1.25,
        "ready",
        datetime(2026, 1, 1, tzinfo=timezone.utc),
        [1, "two"],
        {"enabled": True},
        None,
    ]
    kinds = [
        "bool",
        "integer",
        "unsigned",
        "number",
        "text",
        "datetime",
        "array",
        "object",
        None,
    ]

    for value, kind in zip(values, kinds, strict=True):
        for plot in (
            TimeSeriesPlot("Filtered", y=expression).filter(expression, "eq", value),
            TimeSeriesPlot(
                "Filtered",
                y=expression,
                filters=[
                    {
                        "expression": expression.to_contract(),
                        "operator": "eq",
                        "value": value,
                    }
                ],
            ),
        ):
            data = plot.to_contract()["data"]
            assert isinstance(data, dict)
            encoded = data["plot"]["filters"][0]["value"]
            if kind is None:
                assert encoded is None
            else:
                assert encoded["type"] == kind


# === Private Functions === #

# === Entry Point === #


def case_plot_analysis():
    from dokime import TimeSeriesPlot, UnsignedValue

    original = TimeSeriesPlot("Distance", source="tracking.distance_m", x="timestamp")
    parameters = {"points": UnsignedValue(64), "enabled": True, "label": "Signal"}
    selected = original.analyze("custom.analysis", version=2, parameters=parameters)
    parameters["points"] = UnsignedValue(128)
    data = selected.to_contract()["data"]
    assert isinstance(data, dict)
    assert data["plot"]["analysis"] == {
        "id": "custom.analysis",
        "version": 2,
        "parameters": {
            "points": {"type": "unsigned", "value": "64"},
            "enabled": {"type": "bool", "value": True},
            "label": {"type": "text", "value": "Signal"},
        },
    }
    assert "analysis" not in original.options


@pytest.mark.parametrize(
    "scenario",
    [
        pytest.param(case_suite_builds, id="suite_builds"),
        pytest.param(case_parameter_contracts, id="parameter_contracts"),
        pytest.param(case_list_bounds, id="list_bounds"),
        pytest.param(case_credential_contracts, id="credential_contracts"),
        pytest.param(case_param_cases, id="param_cases"),
        pytest.param(
            case_materializations_identities, id="materializations_identities"
        ),
        pytest.param(case_registration_metadata, id="registration_metadata"),
        pytest.param(case_declaration_contracts, id="declaration_contracts"),
        pytest.param(case_document_plots, id="document_plots"),
        pytest.param(case_plot_contract, id="plot_contract"),
        pytest.param(case_authored_contract, id="authored_contract"),
        pytest.param(case_materialized_retained, id="materialized_retained"),
        pytest.param(case_template_paths, id="template_paths"),
        pytest.param(case_typed_cases, id="typed_cases"),
        pytest.param(case_param_attributes, id="param_attributes"),
        pytest.param(case_metadata_dates, id="metadata_dates"),
        pytest.param(case_materialized_nanoseconds, id="materialized_nanoseconds"),
        pytest.param(case_expansion_pairs, id="expansion_pairs"),
        pytest.param(case_plot_original, id="plot_original"),
        pytest.param(case_unsupported_explicitly, id="unsupported_explicitly"),
        pytest.param(case_heatmap_author, id="heatmap_author"),
        pytest.param(case_wifi_expressions, id="wifi_expressions"),
        pytest.param(case_plot_boundary, id="plot_boundary"),
        pytest.param(case_plot_analysis, id="plot_analysis"),
    ],
)
def test_contract(scenario: Callable[..., None]) -> None:
    scenario()
