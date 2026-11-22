# SPDX-License-Identifier: Apache-2.0

"""Record Expressions Bind Both Event Identities Without String-Based Transforms."""

# === Imports === #

# Standard Library Imports
from unittest import TestCase

# Third Party Imports
# Local Imports
from dokime_sdk import Plugin, Suite
from dokime_sdk.templates import Document, Record, Section, TestTemplate, time_series

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class RecordExpressionTests(TestCase):
    def test_time_since_binds_both_record_declarations_without_mutating_the_source(self) -> None:
        began = Record(str, identifier="began")
        ended = Record(str, identifier="ended")
        expression = ended.time_since(began)
        document = Document("Durations").section(Section("Events").plot(time_series(expression)))

        class TimingTestTemplate(TestTemplate):
            start = began
            stop = ended
            documents = (document,)

        class TimingSuite(Suite):
            templates = (TimingTestTemplate,)

        class TimingPlugin(Plugin):
            version = "1.0.0"
            suites = (TimingSuite,)

        bound = TimingPlugin()._suites[0].templates[0].documents[0].message.sections[0].cards[0].plot.y[0]
        self.assertEqual(bound.transform.time_between, "timing.timing.timing.began")
        self.assertEqual(bound.transform.input.record.identifier, "timing.timing.timing.ended")
        self.assertEqual(expression.message.transform.time_between, "began")
        self.assertEqual(expression.message.transform.input.record.identifier, "ended")
        self.assertFalse(hasattr(expression, "time_between"))
        self.assertRaises(ValueError, ended.time_since, Record(str))
        self.assertRaises(ValueError, Record(str).time_since, began)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
