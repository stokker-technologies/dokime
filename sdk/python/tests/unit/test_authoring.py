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
import ast
import unittest
from pathlib import Path
from typing import Self

# Third Party Imports
from google.protobuf.wrappers_pb2 import BoolValue

# Local Imports
from dokime import Context, Plugin, Prompt, PromptDefinitions, Template, criterion, observer, prompt, record, step, stream, suite
from dokime._payloads import message_type, pack, unpack
from dokime.authoring import Case, Suite, materialize
from dokime.values import Scalar

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class AuthoringStory(unittest.TestCase):
    def test_code_names_generate_qualified_identifiers_independently_of_labels(self) -> None:
        @suite(label="A Label That Is Not an Identifier")
        class HTTPAccuracySuite:
            accepted_reading = record(bool, label="Unrelated Record Label")
            operator_choice = prompt("Proceed?", choices=["Yes", "No"])

            @step("Completely Different Label")
            async def read_voltage(self, ctx: Context) -> None:
                pass

            @criterion("Another Label")
            def readingOK(self, ctx: Context) -> bool:
                return True

            @stream("Unrelated Stream Label")
            @observer()
            def outlet_temperature(self, ctx: Context) -> float:
                return 20.0

        plugin = Plugin("naming", "1.0.0", suites=[HTTPAccuracySuite])
        template = plugin.suites[0].templates[0]
        case = template.build({}).message
        self.assertEqual(template.identifier, "http_accuracy")
        self.assertEqual(case.steps[0].metadata.identifier, "http_accuracy.read_voltage")
        self.assertEqual(case.criteria[0].metadata.identifier, "http_accuracy.readingok")
        self.assertEqual(list(case.streams), ["http_accuracy.outlet_temperature"])
        self.assertEqual(template.resolve(HTTPAccuracySuite.accepted_reading), "http_accuracy.accepted_reading")
        self.assertEqual(template.resolve(HTTPAccuracySuite().read_voltage), case.steps[0].metadata.identifier)
        sections = plugin.declarations().sections
        prompts = next(section for section in sections if section.type.endswith("PromptDefinitions"))
        value = PromptDefinitions.FromString(prompts.payload)
        self.assertEqual(value.prompts[0].metadata.identifier, "http_accuracy.operator_choice")

    def test_grouped_templates_bind_once_without_mutating_shared_declarations(self) -> None:
        class VoltageTemplate:
            reading = record(int, identifier="exact", label="Reading")

            @step("Measure", identifier="take")
            def measure(self, ctx: Context) -> None:
                pass

        local = Template(VoltageTemplate, identifier="voltage")
        first = Plugin("one", "1.0.0", suites=[Suite("accuracy", [local])]).suites[0].templates[0]
        second = Plugin("two", "1.0.0", suites=[Suite("production", [local])]).suites[0].templates[0]
        self.assertEqual(first.identifier, "accuracy.voltage")
        self.assertEqual(first.resolve(VoltageTemplate.reading), "accuracy.voltage.exact")
        self.assertEqual(first.definition().steps[0].operation, "accuracy.voltage.take")
        self.assertEqual(second.resolve(VoltageTemplate.reading), "production.voltage.exact")
        self.assertEqual(local.identifier, "voltage")
        self.assertRaises(ValueError, first.resolve, "accuracy.voltage.exact")

    def test_explicit_suite_and_generated_template_names(self) -> None:
        @suite(identifier="chosen", label="Ignored for Naming")
        class RenamedSuite:
            @step("Ignored")
            def read_value(self, ctx: Context) -> None:
                pass

        single = Plugin("one", "1.0.0", suites=[RenamedSuite]).suites[0].templates[0]
        grouped = Plugin("two", "1.0.0", suites=[Suite("accuracy", [Template(RenamedSuite)])]).suites[0].templates[0]
        self.assertEqual(single.identifier, "chosen")
        self.assertEqual(single.resolve(RenamedSuite.read_value), "chosen.read_value")
        self.assertEqual(grouped.identifier, "accuracy.renamed")

    def test_duplicate_and_empty_declarations_fail_at_construction(self) -> None:
        @suite()
        class Repeated:
            first = record(bool, identifier="same")
            second = record(str, identifier="same")

        @suite()
        class Empty:
            reading = record(int, identifier="")

        @suite()
        class CrossKind:
            reading = record(int, identifier="measure")

            @step("Measure")
            def measure(self, ctx: Context) -> None:
                pass

        @suite(identifier="")
        class EmptySuite:
            pass

        for declaration in (Repeated, Empty, CrossKind, EmptySuite):
            self.assertRaises(ValueError, Plugin, "invalid", "1.0.0", suites=[declaration])
        self.assertRaises(ValueError, Plugin, "invalid", "1.0.0", suites=[Repeated, Repeated])

    def test_materialized_members_qualify_and_bind_instance_methods(self) -> None:
        @suite()
        class VoltageAccuracy:
            @step("Read")
            def read_voltage(self, ctx: Context, *, channel: int) -> None:
                pass

            @criterion("Accept")
            def accept_voltage(self, ctx: Context, *, channel: int) -> bool:
                return True

            @materialize
            def channels(cls: type[Self], case: Case, parameters: dict[str, Scalar]) -> None:
                channels = parameters["channels"]
                if not isinstance(channels, list):
                    raise TypeError("Materialize Voltages: Supply a List of Channels")
                instance = cls()
                for channel in channels:
                    case.step("Read", instance.read_voltage, identifier=f"read_{channel}", channel=channel)
                    case.criterion("Accept", instance.accept_voltage, identifier=f"accept_{channel}", channel=channel)

        template = Plugin("voltage", "1.0.0", suites=[VoltageAccuracy]).suites[0].templates[0]
        case = template.build({"channels": [1, 2]})
        self.assertEqual([item.metadata.identifier for item in case.message.steps], ["voltage_accuracy.read_1", "voltage_accuracy.read_2"])
        self.assertEqual(
            [item.metadata.identifier for item in case.message.criteria], ["voltage_accuracy.accept_1", "voltage_accuracy.accept_2"]
        )
        self.assertEqual([item.operation for item in case.message.steps], ["voltage_accuracy.read_voltage"] * 2)
        self.assertEqual([item.operation for item in case.message.criteria], ["voltage_accuracy.accept_voltage"] * 2)
        self.assertEqual(case.bindings["step", "voltage_accuracy.read_2"], (VoltageAccuracy.read_voltage, {"channel": 2}))
        self.assertEqual(case.bindings["criterion", "voltage_accuracy.accept_2"], (VoltageAccuracy.accept_voltage, {"channel": 2}))
        self.assertEqual(template.resolve(VoltageAccuracy().read_voltage), "voltage_accuracy.read_voltage")

    def test_structured_prompt_fields_survive_template_binding(self) -> None:
        definition = Prompt(
            message="Enter a Reading",
            fields=[{"metadata": {"identifier": "reading", "label": "Reading"}, "type": "VALUE_TYPE_NUMBER", "required": True}],
            actions=[{"identifier": "save", "label": "Save"}],
        )

        @suite()
        class ReadingSuite:
            question = prompt(definition)

        definition.fields.clear()
        plugin = Plugin("prompts", "1.0.0", suites=[ReadingSuite])
        section = next(item for item in plugin.declarations().sections if item.type.endswith("PromptDefinitions"))
        bound = PromptDefinitions.FromString(section.payload).prompts[0]
        self.assertEqual(bound.metadata.identifier, "reading.question")
        self.assertEqual(bound.template, "reading")
        self.assertTrue(bound.fields[0].required)
        self.assertEqual(bound.fields[0].metadata.identifier, "reading")
        self.assertEqual(bound.actions[0].identifier, "save")

    def test_primitive_and_structured_payloads_roundtrip(self) -> None:
        values = [False, True, "a value", -(2**63), 2**63 - 1, 2.125, BoolValue(value=True)]
        for value in values:
            row = type(value)
            encoded = pack(row, value).SerializeToString()
            decoded = message_type(row).FromString(encoded)
            self.assertEqual(unpack(row, decoded), value)
            self.assertIs(type(unpack(row, decoded)), row)
        self.assertRaises(TypeError, pack, int, True)
        self.assertRaises(TypeError, record, bytes)

    def test_examples_import_the_product_surface(self) -> None:
        examples = Path(__file__).resolve().parents[2] / "examples"
        imported = []
        for source in examples.rglob("*.py"):
            tree = ast.parse(source.read_text(encoding="utf-8"))
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    imported.extend((source, item.name) for item in node.names)
                if isinstance(node, ast.ImportFrom) and node.module:
                    imported.append((source, node.module))
        self.assertTrue(any(name == "dokime" for _, name in imported))
        self.assertEqual([(path, name) for path, name in imported if name.split(".")[0] == "harmos"], [])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
