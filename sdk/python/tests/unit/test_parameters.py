# SPDX-License-Identifier: Apache-2.0

"""Typed Parameters Bind Once and Use the Host's Existing Constraint Contract."""

# === Imports === #

# Standard Library Imports
from unittest import TestCase

# Third Party Imports
from harmos import plugin
from harmos._generated.proto.plugin.v1.invocation_pb2 import Request

# Local Imports
from dokime_sdk import Plugin, Suite
from dokime_sdk.templates import Context, Parameter, TestTemplate

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class RadioTemplate(TestTemplate):
    channel = Parameter.integer("Wi-Fi Channel", choices=(1, 6, 11), default=6)
    country = Parameter.text("Country Code", choices=("CH", "DE"))
    power = Parameter.number("Power", min=0.0, max=10.0, default=5.0)
    enabled = Parameter.flag("Enabled", default=False)
    channels = Parameter.list("Channels", int, default=[1, 6], min_items=1, max_items=3)
    count = Parameter.unsigned_integer("Count", default=2**64 - 1)
    loads = Parameter.sweep("Loads", (1.0, 2.0))


class RadioSuite(Suite):
    templates = (RadioTemplate,)


class RadioPlugin(Plugin):
    version = "1.0.0"
    suites = (RadioSuite,)


class ParameterTests(TestCase):
    def test_attributes_freeze_constraints_in_the_host_template_contract(self) -> None:
        owner = RadioPlugin()
        parameters = {value.metadata.identifier: value for value in owner._suites[0].templates[0].definition().parameters}
        self.assertEqual([value.integer for value in parameters["channel"].choices], [1, 6, 11])
        self.assertEqual(parameters["channel"].expression.literal.integer, 6)
        self.assertEqual([value.text for value in parameters["country"].choices], ["CH", "DE"])
        self.assertEqual((parameters["power"].min, parameters["power"].max), (0.0, 10.0))
        self.assertEqual(parameters["power"].expression.literal.number, 5.0)
        self.assertTrue(parameters["enabled"].expression.literal.HasField("boolean"))
        self.assertEqual(parameters["count"].expression.literal.unsigned_integer, 2**64 - 1)
        self.assertEqual((parameters["channels"].min_items, parameters["channels"].max_items), (1, 3))
        self.assertEqual([value.integer for value in parameters["channels"].expression.literal.list.values], [1, 6])
        self.assertTrue(parameters["loads"].expansion.cartesian)
        parameters["channel"].ClearField("choices")
        self.assertEqual(len(RadioTemplate.channel._definition().choices), 3)

    def test_context_returns_typed_values_and_rejects_foreign_declarations(self) -> None:
        owner = RadioPlugin()
        transport = plugin.Context(Request(scope="execution", attempt=1), owner.identifier, {})
        context = Context(transport, owner, {"channel": 11, "country": "CH", "power": 1.5, "enabled": False}, owner._suites[0].templates[0])
        self.assertEqual(context.parameter(RadioTemplate.channel), 11)
        self.assertEqual(context.parameter(RadioTemplate.country), "CH")
        self.assertEqual(context.parameter(RadioTemplate.power), 1.5)
        self.assertIs(context.parameter(RadioTemplate.enabled), False)
        self.assertRaises(ValueError, context.parameter, Parameter.integer("Foreign"))
        self.assertFalse(hasattr(context, "parameters"))

    def test_declarations_reject_ambiguous_types_and_constraints(self) -> None:
        self.assertRaises(TypeError, Parameter.integer, "Channel", choices=(True,))
        self.assertRaises(ValueError, Parameter.number, "Power", min=2.0, max=1.0)
        self.assertRaises(ValueError, Parameter.text, "Country", choices=("CH",), default="DE")
        self.assertRaises(TypeError, Parameter.list, "Channels", int, default=[True])
        self.assertRaises(ValueError, Parameter.list, "Channels", int, min_items=2, max_items=1)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
