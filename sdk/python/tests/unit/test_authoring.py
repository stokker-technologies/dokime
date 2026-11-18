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
import base64
import unittest
import zlib
from collections.abc import Callable, Mapping
from datetime import timedelta
from typing import Any, ClassVar, cast

# Third Party Imports
# Local Imports
import dokime_sdk.operations
import dokime_sdk.templates
from dokime_sdk import Plugin, Scalar, Suite
from dokime_sdk.operations import Operation, OperationPolicy, RetryPolicy, Visibility, operation
from dokime_sdk.templates import Document, Materialization, Section, TestTemplate, criterion, materialize, record, step

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# Fixed Builder Description
_BUILDER_DESCRIPTION = "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"

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
    def test_class_composition_matches_fixed_builder_wire_description(self) -> None:
        plugin = ExamplePlugin()
        self.assertEqual(
            plugin.declarations().SerializeToString(deterministic=True), zlib.decompress(base64.b64decode(_BUILDER_DESCRIPTION))
        )

    def test_classes_bind_members_without_mutating_declarations(self) -> None:
        plugin = ExamplePlugin()
        template = plugin._suites[0].templates[0]
        self.assertEqual(plugin._identifier, "example")
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
        self.assertEqual(plugin._identifier, "http")
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
        policy = OperationPolicy(retries=RetryPolicy(2, timedelta()))
        self.assertRaises(ValueError, step, "Measure", retries=1, policy=policy)
        self.assertRaises(ValueError, Operation, "read", int, int, Visibility.PUBLIC, policy)
        self.assertRaises(ValueError, RetryPolicy, 0, timedelta())
        self.assertRaises(ValueError, OperationPolicy, timeout=timedelta())

    def test_operation_and_template_contexts_are_distinct(self) -> None:
        operations = dokime_sdk.operations.Context
        templates = dokime_sdk.templates.Context
        self.assertIsNot(operations, templates)
        self.assertTrue(issubclass(templates, operations))
        for name in ("operation", "sleep", "now", "cancelled", "secret", "settings"):
            self.assertTrue(callable(getattr(operations, name)))
        for name in (
            "prompt",
            "record",
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
            "prepare_effect",
            "confirm_effect",
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
