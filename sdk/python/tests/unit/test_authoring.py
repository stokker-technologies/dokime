# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Declaration Binding and Plain Payload Authoring Contracts."""

# === Imports === #

# Standard Library Imports
import json
import unittest
from collections.abc import Callable, Mapping
from datetime import timedelta
from pathlib import Path
from typing import ClassVar
from unittest.mock import Mock, patch

# Third Party Imports
from harmos import plugin

# Local Imports
import dokime_sdk.operations
import dokime_sdk.templates
from dokime_sdk import Lane, Plugin, Scalar, Suite
from dokime_sdk._generated import shared
from dokime_sdk.operations import OperationPolicy, RetryPolicy, Visibility, operation
from dokime_sdk.templates import (
    Document,
    Materialization,
    Section,
    TestTemplate,
    criterion,
    materialize,
    Observer,
    prompt,
    record,
    step,
    stream,
)

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #


class ConfirmationTestTemplate(TestTemplate):
    confirmation = record(bool, label="Confirmation", identifier='confirmation')

    @step("Confirm")
    async def confirm(self, ctx: dokime_sdk.templates.Context) -> None:
        pass

    @criterion("Confirmed")
    async def confirmed(self, ctx: dokime_sdk.templates.Context) -> bool:
        return True


class CaptureSuite(Suite):
    templates: ClassVar[list[type[TestTemplate]]] = [ConfirmationTestTemplate]


class ExamplePlugin(Plugin):
    version = "0.1.5"
    suites: ClassVar[list[type[Suite]]] = [CaptureSuite]
    records = (ConfirmationTestTemplate.confirmation,)


class AuthoringStory(unittest.TestCase):
    def test_at_builders_and_record_binding(self) -> None:
        templates = dokime_sdk.templates
        sampled = templates.field("readings", "value").label("Distance").at("reading")
        bounded = sampled.within_s(0.5)
        self.assertFalse(sampled.message.transform.at.HasField("within_s"))
        self.assertEqual(bounded.message.transform.at.within_s, 0.5)
        self.assertEqual(bounded.message.label, "Distance")
        self.assertRaises(ValueError, templates.field("readings", "value").within_s, 0.5)
        plot = templates.histogram(sampled).filter(sampled.detail("event"), templates.FilterOperator.EQ, "disconnected")
        document = Document("Tracking").section(Section("Events").plot(plot))

        class ReadingsTestTemplate(TestTemplate):
            reading = record(str, identifier="reading")
            documents = (document,)

        class ReadingsSuite(Suite):
            templates = (ReadingsTestTemplate,)

        class ReadingsPlugin(Plugin):
            version = "1.0.0"
            suites = (ReadingsSuite,)
            records = (ReadingsTestTemplate.reading,)

        bound = ReadingsPlugin()._suites[0].templates[0].documents[0].message.sections[0].cards[0].plot
        self.assertEqual(bound.x.transform.at.record_type, "readings.reading")
        self.assertEqual(bound.filters[0].expression.transform.input.transform.at.record_type, bound.x.transform.at.record_type)
        self.assertEqual(sampled.message.transform.at.record_type, "reading")

    def test_plugin_root_qualifies_explicit_and_derived_members_once(self) -> None:
        @operation("measure")
        async def measure(ctx: dokime_sdk.operations.Context, request: int) -> int:
            return request

        @operation("example.read")
        async def read(ctx: dokime_sdk.operations.Context, request: int) -> int:
            return request

        class TrackingTestTemplateStatus(Observer):
            identifier = "example.status"
            label = "Status"
            row = int

            async def sample(self, ctx: dokime_sdk.templates.Context) -> int:
                return 1


        class TrackingTestTemplate(TestTemplate):
            uses_streams = (TrackingTestTemplateStatus,)
            identifier = "example.layout.tracking"
            reading = record(int, identifier="example.reading")
            question = prompt("Continue?", identifier="example.question")
            documents = (Document("Report").id("example.report"),)

            @step("Read", identifier="example.take")
            async def take(self, ctx: dokime_sdk.templates.Context) -> None:
                pass


            @materialize
            def materialize(self, case: Materialization, parameters: Mapping[str, Scalar]) -> None:
                case.step("Read", self.take, identifier="example.channel")

        class LayoutSuite(Suite):
            identifier = "example.layout"
            templates = (TrackingTestTemplate,)

        class QualifiedPluginSystem(Observer):
            identifier = "example.system"
            label = "System"
            row = int

            async def sample(self, ctx: dokime_sdk.operations.Context) -> int:
                return 1


        class QualifiedPlugin(Plugin):
            identifier = "example"
            version = "1.0.0"
            suites = (LayoutSuite,)
            records = (TrackingTestTemplate.reading,)
            operations = (measure, read)
            observers = (TrackingTestTemplateStatus, QualifiedPluginSystem)


        plugin = QualifiedPlugin()
        template = plugin._suites[0].templates[0]
        self.assertEqual(plugin._suites[0].identifier, "example.layout")
        self.assertEqual(template.identifier, "example.layout.tracking")
        self.assertEqual(template.resolve(TrackingTestTemplate.reading), "example.reading")
        self.assertEqual(template.resolve(TrackingTestTemplate.question), "example.question")
        self.assertEqual(template.resolve(TrackingTestTemplate.take), "example.take")
        self.assertEqual(template.resolve(TrackingTestTemplateStatus), "example.status")
        self.assertEqual(template.definition().documents[0].metadata.identifier, "example.report")
        self.assertEqual(template.build({}).message.steps[0].metadata.identifier, "example.channel")
        declarations = plugin.declarations()
        self.assertEqual({item.id for item in declarations.streams}, {"example.status", "example.system"})
        registered = {item.id for item in declarations.operations}
        self.assertTrue({"example.measure", "example.read", "example.take", "example.layout.tracking.materialize"} <= registered)
        self.assertTrue(all(identifier.startswith("example.") for identifier in registered))
        required = {item.id for item in declarations.requires}
        self.assertTrue({"measure", "example.read"}.isdisjoint(required))

    def test_document_queries_preserve_declaration_binding(self) -> None:
        templates = dokime_sdk.templates
        declaration = record(int, identifier="reading")
        reading = declaration.field()
        section = Section("Readings").repeat(reading, order_by=reading.max()).traceability()
        plot = templates.time_series(templates.field("samples", "value")).markers("reading")
        document = Document("Report").section(section.plot(plot))
        reopened = type(document.message).FromString(document.message.SerializeToString())
        repeated = reopened.sections[0]
        self.assertEqual(repeated.repeat.value.record.identifier, "reading")
        self.assertEqual(repeated.repeat.order_by.aggregate.input, reading.message)
        self.assertEqual(repeated.cards[0].evidence, dokime_sdk.templates._documents.shared.EVIDENCE_CARD_TRACEABILITY)
        self.assertEqual(list(repeated.cards[1].plot.marker_records), ["reading"])
        section.repeat(reading)
        self.assertFalse(section.message.repeat.HasField("order_by"))
        self.assertRaises(ValueError, record(int).field)

        class ReadingsTestTemplate(TestTemplate):
            reading = declaration
            documents = (document,)

        class ReadingsSuite(Suite):
            templates = (ReadingsTestTemplate,)

        class ReadingsPlugin(Plugin):
            version = "1.0.0"
            suites = (ReadingsSuite,)

        bound = ReadingsPlugin()._suites[0].templates[0]
        cards = bound.documents[0].message.sections[0].cards
        self.assertEqual(bound.documents[0].message.sections[0].repeat.value.record.identifier, "readings.reading")
        self.assertEqual(list(cards[1].plot.marker_records), ["readings.reading"])
        self.assertEqual(reading.message.record.identifier, "reading")

    def test_lanes_preserve_declarations_and_operation_assignments(self) -> None:
        instrument = Lane("instrument", serial=True)

        @operation("example.measure", lane=instrument)
        async def measure(ctx: dokime_sdk.operations.Context, value: float) -> float:
            return value

        class LanePlugin(Plugin):
            version = "1.0.0"
            lanes: ClassVar[list[Lane]] = [instrument]
            operations: ClassVar[list[Callable[..., object]]] = [measure]

        declarations = LanePlugin().declarations()
        self.assertEqual(len(declarations.lanes), 1)
        self.assertEqual(declarations.lanes[0].name, "instrument")
        self.assertTrue(declarations.lanes[0].serial)
        selected = next(item for item in declarations.operations if item.id == "lane.example.measure")
        self.assertEqual(selected.lane, "instrument")

        class MissingLanePlugin(Plugin):
            version = "1.0.0"
            operations: ClassVar[list[Callable[..., object]]] = [measure]

        self.assertRaises(ValueError, MissingLanePlugin)

    def test_plugin_version_uses_owning_editable_metadata_or_explicit_override(self) -> None:
        class InstalledPlugin(Plugin):
            pass

        owning = Mock()
        owning.version = "2.3.4"
        owning.read_text.return_value = json.dumps({"url": Path(__file__).resolve().parent.as_uri(), "dir_info": {"editable": True}})
        unrelated = Mock()
        unrelated.version = "9.9.9"
        unrelated.read_text.return_value = json.dumps(
            {"url": (Path(__file__).resolve().parent / "unrelated").as_uri(), "dir_info": {"editable": True}}
        )
        packages = patch("dokime_sdk._plugin.packages_distributions", return_value={})
        packages.start()
        self.addCleanup(packages.stop)
        distributions = patch("dokime_sdk._plugin.distributions", return_value=[unrelated, owning])
        installed = distributions.start()
        self.addCleanup(distributions.stop)
        self.assertEqual(InstalledPlugin()._version, "2.3.4")
        parent = Mock()
        parent.version = "0.1.0"
        parent.read_text.return_value = json.dumps({"url": Path(__file__).resolve().parents[2].as_uri(), "dir_info": {"editable": True}})
        installed.return_value = [parent, unrelated, owning]
        self.assertEqual(InstalledPlugin()._version, "2.3.4")
        installed.return_value = [unrelated]
        self.assertRaises(ValueError, InstalledPlugin)
        installed.return_value = [owning, owning]
        self.assertRaises(ValueError, InstalledPlugin)
        InstalledPlugin.version = "5.6.7"
        self.assertEqual(InstalledPlugin()._version, "5.6.7")

    def test_classes_bind_members_without_mutating_declarations(self) -> None:
        plugin = ExamplePlugin()
        template = plugin._suites[0].templates[0]
        self.assertEqual(plugin.identifier, "example")
        self.assertEqual(template.identifier, "example.capture.confirmation")
        self.assertEqual(template.resolve(ConfirmationTestTemplate.confirmation), "example.confirmation")
        self.assertEqual(template.resolve(ConfirmationTestTemplate().confirm), "example.capture.confirmation.confirm")
        self.assertIsNone(ConfirmationTestTemplate.identifier)
        self.assertEqual(ConfirmationTestTemplate.confirmation.identifier, "confirmation")

        class OtherSuite(Suite):
            templates: ClassVar[list[type[TestTemplate]]] = [ConfirmationTestTemplate]

        class OtherPlugin(ExamplePlugin):
            suites: ClassVar[list[type[Suite]]] = [OtherSuite]
            records = (ConfirmationTestTemplate.confirmation,)

        other = OtherPlugin()._suites[0].templates[0]
        self.assertEqual(other.resolve(ConfirmationTestTemplate.confirmation), "other.confirmation")
        self.assertEqual(template.identifier, "example.capture.confirmation")
        self.assertRaises(ValueError, other.resolve, "other.confirmation")

    def test_acronyms_suffixes_and_local_overrides(self) -> None:
        class ReadHTTPVoltageTestTemplate(TestTemplate):
            label = "Explicit Label"
            reading = record(int, identifier="rms")

            @step("Read", identifier="take")
            async def measure(self, ctx: dokime_sdk.templates.Context) -> None:
                pass

        class HTTPAccuracySuite(Suite):
            templates: ClassVar[list[type[TestTemplate]]] = [ReadHTTPVoltageTestTemplate]

        class HTTPPlugin(ExamplePlugin):
            suites: ClassVar[list[type[Suite]]] = [HTTPAccuracySuite]
            records = (ReadHTTPVoltageTestTemplate.reading,)

        plugin = HTTPPlugin()
        bound = plugin._suites[0]
        template = bound.templates[0]
        self.assertEqual(plugin.identifier, "http")
        self.assertEqual(bound.identifier, "http.http_accuracy")
        self.assertEqual(bound.label, "HTTP Accuracy")
        self.assertEqual(template.identifier, "http.http_accuracy.read_http_voltage")
        self.assertEqual(template.resolve(ReadHTTPVoltageTestTemplate.reading), "http.rms")
        self.assertEqual(template.resolve(ReadHTTPVoltageTestTemplate.measure), "http.http_accuracy.read_http_voltage.take")

    def test_duplicate_empty_invalid_and_reused_members_fail(self) -> None:
        for identity in ("", "already..qualified", "Uppercase", "has space"):
            declaration = type("InvalidSuite", (CaptureSuite,), {"identifier": identity})
            plugin = type("InvalidPlugin", (ExamplePlugin,), {"suites": [declaration]})
            self.assertRaises(ValueError, plugin)
        class DuplicatePlugin(ExamplePlugin):
            suites: ClassVar[list[type[Suite]]] = [CaptureSuite, CaptureSuite]
            records = (ConfirmationTestTemplate.confirmation, ConfirmationTestTemplate.confirmation,)

        self.assertRaises(ValueError, DuplicatePlugin)

    def test_materialization_uses_a_pure_instance_and_immutable_inputs(self) -> None:
        class ChannelsTestTemplate(TestTemplate):
            @step("Read")
            async def read(self, ctx: dokime_sdk.templates.Context, *, channel: int) -> None:
                pass

            @materialize
            def channels(self, case: Materialization, parameters: Mapping[str, Scalar]) -> None:
                channels = parameters["channels"]
                if not isinstance(channels, list):
                    raise TypeError("Materialize Channels: Supply a List")
                for channel in channels:
                    case.step("Read", self.read, identifier=f"read_{channel}", channel=channel)

        class ChannelsSuite(Suite):
            templates: ClassVar[list[type[TestTemplate]]] = [ChannelsTestTemplate]

        class ChannelsPlugin(ExamplePlugin):
            suites: ClassVar[list[type[Suite]]] = [ChannelsSuite]

        template = ChannelsPlugin()._suites[0].templates[0]
        parameters: dict[str, Scalar] = {"channels": [1, 2]}
        case = template.build(parameters)
        parameters["channels"] = []
        self.assertEqual(case.parameters["channels"], [1, 2])
        self.assertEqual(
            [item.metadata.identifier for item in case.message.steps],
            ["channels.channels.channels.read_1", "channels.channels.channels.read_2"],
        )
        self.assertEqual(
            {identifier: arguments for (_, identifier), (_, arguments) in case.bindings.items()},
            {"channels.channels.channels.read_1": {"channel": 1}, "channels.channels.channels.read_2": {"channel": 2}},
        )
        self.assertRaises(ValueError, case.step, "Duplicate", ChannelsTestTemplate.read, identifier="read_1")

    def test_operation_typing_comes_from_annotations(self) -> None:
        @operation(visibility=Visibility.PUBLIC)
        async def readVoltage(ctx: dokime_sdk.operations.Context, request: int) -> bool:
            return request > 0

        class TypedPlugin(ExamplePlugin):
            operations: ClassVar[list[Callable[..., object]]] = [readVoltage]

        self.assertIsInstance(vars(readVoltage)["__dokime_operation__"], plugin.Operation)
        declaration = next(item for item in TypedPlugin().declarations().operations if item.id == "typed.read_voltage")
        self.assertEqual(declaration.request, "google.protobuf.Int64Value")
        self.assertEqual(declaration.response, "google.protobuf.BoolValue")

        async def ambiguous(ctx: dokime_sdk.operations.Context, request: int | str) -> bool:
            return True

        self.assertRaises(TypeError, operation(visibility=Visibility.PUBLIC), ambiguous)

    def test_policy_has_one_retry_source_and_requires_replay_safety(self) -> None:
        policy = OperationPolicy(retries=RetryPolicy(2, timedelta(milliseconds=100)))
        self.assertRaises(ValueError, step, "Measure", retries=1, policy=policy)
        self.assertRaises(ValueError, RetryPolicy, 0, timedelta())
        self.assertRaises(ValueError, OperationPolicy, timeout=timedelta())

    def test_retry_backoff_is_positive_at_operation_declaration(self) -> None:
        async def measure(ctx: dokime_sdk.templates.Context) -> None:
            pass

        async def evaluate(ctx: dokime_sdk.templates.Context) -> bool:
            return True

        self.assertRaisesRegex(ValueError, r"measure:.*policy\.retries\.initial", step("Measure", retries=1), measure)
        for initial in (timedelta(), timedelta(milliseconds=-1)):
            policy = OperationPolicy(retries=RetryPolicy(2, initial))
            error = r"measure:.*retries\.initial"
            self.assertRaisesRegex(ValueError, error, step("Measure", policy=policy), measure)
            self.assertRaisesRegex(ValueError, r"evaluate:.*retries\.initial", criterion("Evaluate", policy=policy), evaluate)

    def test_document_declarations_bind_copies_in_each_template_home(self) -> None:
        class ReportTestTemplate(TestTemplate):
            report = Document("Report").section(Section("Measurements"))
            documents: ClassVar[list[Document]] = [report]

        class FirstSuite(Suite):
            templates: ClassVar[list[type[TestTemplate]]] = [ReportTestTemplate]

        class SecondSuite(FirstSuite):
            pass

        class ReportPlugin(ExamplePlugin):
            suites: ClassVar[list[type[Suite]]] = [FirstSuite, SecondSuite]

        plugin = ReportPlugin()
        first, second = [suite.templates[0] for suite in plugin._suites]
        self.assertEqual(first.resolve(ReportTestTemplate.report), "report.first.report.report")
        self.assertEqual(second.resolve(ReportTestTemplate.report), "report.second.report.report")
        first_section = first.documents[0].message.sections[0]
        declared_section = ReportTestTemplate.report.message.sections[0]
        first_section.metadata.identifier = "changed"
        self.assertNotEqual(first_section.metadata.identifier, declared_section.metadata.identifier)
        self.assertEqual(second.definition().documents[0].sections[0], declared_section)
        self.assertEqual(ReportTestTemplate.report.message.metadata.identifier, "report")
        self.assertRaises(ValueError, first.resolve, Document("Unknown"))
        ReportTestTemplate.documents = [ReportTestTemplate.report, ReportTestTemplate.report]
        self.assertRaises(ValueError, ReportPlugin)


class HeatStripTests(unittest.TestCase):
    def test_heat_strip_authoring_preserves_defaults_and_operands(self) -> None:
        samples = dokime_sdk.templates.Stream("example.samples")
        plot = dokime_sdk.templates.heat_strip(samples.field("value")).cells(samples.field("cell")).facet(samples.field("facet"))
        self.assertEqual(plot.message.kind, shared.PLOT_KIND_HEAT_STRIP)
        self.assertEqual(plot.message.reduction, shared.REDUCTION_LAST)
        self.assertEqual(plot.message.cell.stream.field, "cell")
        self.assertEqual(plot.message.facet.stream.field, "facet")
        self.assertRaises(TypeError, plot.cells, "cell")
        self.assertRaises(TypeError, plot.reduce, "mean")
        self.assertRaises(TypeError, plot.reduce, 4)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
