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
from typing import Any, ClassVar, cast
from unittest.mock import Mock, patch

# Third Party Imports
# Local Imports
import dokime_sdk.operations
import dokime_sdk.templates
from dokime_sdk import Lane, Plugin, Scalar, Suite
from dokime_sdk.operations import Operation, OperationPolicy, RetryPolicy, Visibility, operation
from dokime_sdk.templates import Document, Materialization, Section, TestTemplate, criterion, materialize, record, step

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #


class ConfirmationTestTemplate(TestTemplate):
    confirmation = record(bool, label="Confirmation")

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


class AuthoringStory(unittest.TestCase):
    def test_declaration_builders_preserve_parameters_and_document_queries(self) -> None:
        templates = dokime_sdk.templates
        parameter = templates.Parameter.text("serial", "Serial").required().build()
        self.assertTrue(parameter.required)
        self.assertFalse(parameter.HasField("expression"))
        values = templates.Parameter.list("channels", "Channels", int).default([1, 2]).items(1, 4).build()
        self.assertEqual([value.integer for value in values.expression.literal.list.values], [1, 2])
        choices = templates.Parameter.choice("mode", "Mode", ["fast", "slow"]).default("fast").build()
        self.assertIn(choices.expression.literal, choices.choices)
        self.assertRaises(ValueError, templates.Parameter.choice("mode", "Mode", ["fast"]).default("other").build)
        self.assertRaises(ValueError, templates.Parameter.list("channels", "Channels", int).default(["wrong"]).build)
        self.assertRaises(ValueError, templates.Parameter.choice, "mode", "Mode", [])
        self.assertRaises(ValueError, templates.Parameter.choice, "mode", "Mode", [True, 1])

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
        self.assertEqual(bound.documents[0].message.sections[0].repeat.value.record.identifier, "readings.readings.reading")
        self.assertEqual(list(cards[1].plot.marker_records), ["readings.readings.reading"])
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
        selected = next(item for item in declarations.operations if item.id == "example.measure")
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
        self.assertEqual(template.identifier, "capture.confirmation")
        self.assertEqual(template.resolve(ConfirmationTestTemplate.confirmation), "capture.confirmation.confirmation")
        self.assertEqual(template.resolve(ConfirmationTestTemplate().confirm), "capture.confirmation.confirm")
        self.assertIsNone(ConfirmationTestTemplate.identifier)
        self.assertIsNone(ConfirmationTestTemplate.confirmation.identifier)

        class OtherSuite(Suite):
            templates: ClassVar[list[type[TestTemplate]]] = [ConfirmationTestTemplate]

        class OtherPlugin(ExamplePlugin):
            suites: ClassVar[list[type[Suite]]] = [OtherSuite]

        other = OtherPlugin()._suites[0].templates[0]
        self.assertEqual(other.resolve(ConfirmationTestTemplate.confirmation), "other.confirmation.confirmation")
        self.assertEqual(template.identifier, "capture.confirmation")
        self.assertRaises(ValueError, other.resolve, "other.confirmation.confirmation")

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

        plugin = HTTPPlugin()
        bound = plugin._suites[0]
        template = bound.templates[0]
        self.assertEqual(plugin.identifier, "http")
        self.assertEqual(bound.identifier, "http_accuracy")
        self.assertEqual(bound.label, "HTTP Accuracy")
        self.assertEqual(template.identifier, "http_accuracy.read_http_voltage")
        self.assertEqual(template.resolve(ReadHTTPVoltageTestTemplate.reading), "http_accuracy.read_http_voltage.rms")
        self.assertEqual(template.resolve(ReadHTTPVoltageTestTemplate.measure), "http_accuracy.read_http_voltage.take")

    def test_duplicate_empty_qualified_and_reused_members_fail(self) -> None:
        for identity in ("", "already.qualified", "Uppercase", "has space"):
            declaration = type("InvalidSuite", (CaptureSuite,), {"identifier": identity})
            plugin = type("InvalidPlugin", (ExamplePlugin,), {"suites": [declaration]})
            self.assertRaises(ValueError, plugin)
        for members in (
            {"a": record(bool, identifier="same"), "b": record(str, identifier="same")},
            {"reading": record(bool, identifier="cleanup")},
        ):
            template = type("InvalidTestTemplate", (TestTemplate,), members)
            suite = type("InvalidSuite", (Suite,), {"templates": [template]})
            plugin = type("InvalidPlugin", (ExamplePlugin,), {"suites": [suite]})
            self.assertRaises(ValueError, plugin)

        class DuplicatePlugin(ExamplePlugin):
            suites: ClassVar[list[type[Suite]]] = [CaptureSuite, CaptureSuite]

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
            [item.metadata.identifier for item in case.message.steps], ["channels.channels.read_1", "channels.channels.read_2"]
        )
        self.assertEqual([item.operation for item in case.message.steps], ["channels.channels.read"] * 2)
        self.assertRaises(ValueError, case.step, "Duplicate", ChannelsTestTemplate.read, identifier="read_1")

    def test_operation_typing_comes_from_annotations(self) -> None:
        @operation(visibility=Visibility.PUBLIC)
        async def readVoltage(ctx: dokime_sdk.operations.Context, request: int) -> bool:
            return request > 0

        class TypedPlugin(ExamplePlugin):
            operations: ClassVar[list[Callable[..., object]]] = [readVoltage]

        declaration = next(item for item in TypedPlugin().declarations().operations if item.id == "read_voltage")
        self.assertEqual(declaration.request, "google.protobuf.Int64Value")
        self.assertEqual(declaration.response, "google.protobuf.BoolValue")
        self.assertRaises(TypeError, Operation, "unsupported", bytes, bool, Visibility.PUBLIC)

        async def ambiguous(ctx: dokime_sdk.operations.Context, request: int | str) -> bool:
            return True

        self.assertRaises(TypeError, operation(visibility=Visibility.PUBLIC), ambiguous)
        self.assertRaises(TypeError, Operation, "wrong_visibility", int, int, cast(Any, 2))

    def test_policy_has_one_retry_source_and_requires_replay_safety(self) -> None:
        policy = OperationPolicy(retries=RetryPolicy(2, timedelta(milliseconds=100)))
        self.assertRaises(ValueError, step, "Measure", retries=1, policy=policy)
        self.assertRaises(ValueError, Operation, "read", int, int, Visibility.PUBLIC, policy)
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
            self.assertRaisesRegex(ValueError, error, Operation, "measure", int, int, Visibility.PUBLIC, policy, replay_safe=True)

    def test_operation_and_template_contexts_are_distinct(self) -> None:
        operations = dokime_sdk.operations.Context
        templates = dokime_sdk.templates.Context
        self.assertIsNot(operations, templates)
        self.assertTrue(issubclass(templates, operations))
        for name in ("operation", "record", "sleep", "now", "cancelled", "secret", "settings"):
            self.assertTrue(callable(getattr(operations, name)))
        for name in (
            "prompt",
            "records",
            "statistics",
            "progress",
            "log",
            "traceability",
            "document",
            "artifact",
            "capture",
            "start_capture",
            "stop_capture",
            "environment",
        ):
            self.assertFalse(hasattr(operations, name))
            self.assertTrue(callable(getattr(templates, name)))

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
        self.assertEqual(first.resolve(ReportTestTemplate.report), "first.report.report")
        self.assertEqual(second.resolve(ReportTestTemplate.report), "second.report.report")
        first.documents[0].message.sections[0].metadata.label = "Changed"
        self.assertEqual(ReportTestTemplate.report.message.sections[0].metadata.label, "Measurements")
        self.assertEqual(second.definition().documents[0].sections[0].metadata.label, "Measurements")
        self.assertEqual(ReportTestTemplate.report.message.metadata.identifier, "report")
        self.assertRaises(ValueError, first.resolve, Document("Unknown"))
        ReportTestTemplate.documents = [ReportTestTemplate.report, ReportTestTemplate.report]
        self.assertRaises(ValueError, ReportPlugin)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
