# ***
#
# Starter Project
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Check the Plugin Description Without Starting a Host."""

# === Imports === #

# Standard Library Imports
import unittest

# Third Party Imports
from google.protobuf.descriptor_pool import Default
from google.protobuf.json_format import MessageToDict
from google.protobuf.message_factory import GetMessageClass

# Local Imports
from starter_project.plugin import ExamplePlugin

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class PluginDescription(unittest.TestCase):
    def test_declared_capture_and_echo(self) -> None:
        # Construct the Plugin and Inspect Its Fixed Description.
        plugin = ExamplePlugin()
        description = plugin.declarations()
        operations = {item.id for item in description.operations}
        self.assertEqual(
            operations,
            {
                "example.echo",
                "example.initialize",
                "example.finalize",
                "capture.confirmation.materialize",
                "capture.confirmation.cleanup",
                "capture.confirmation.confirm",
                "capture.confirmation.confirmed",
            },
        )
        self.assertEqual([suite.__name__ for suite in plugin.suites], ["CaptureSuite"])
        self.assertEqual([template.__name__ for suite in plugin.suites for template in suite.templates], ["ConfirmationTestTemplate"])

        # Decode the Suite Section With Its Own Published Descriptor Set.
        pool = Default()
        suites = []
        for section in description.sections:
            descriptor = pool.FindMessageTypeByName(section.type)
            value = GetMessageClass(descriptor).FromString(section.payload)
            if "suites" in descriptor.fields_by_name:
                suites.extend(MessageToDict(value)["suites"])
        self.assertEqual([suite["metadata"]["identifier"] for suite in suites], ["capture"])
        self.assertEqual(
            [template["metadata"]["identifier"] for suite in suites for template in suite["templates"]], ["capture.confirmation"]
        )
        self.assertEqual(
            [
                (step["metadata"]["identifier"], step["operation"])
                for suite in suites
                for template in suite["templates"]
                for step in template["steps"]
            ],
            [("capture.confirmation.confirm", "capture.confirmation.confirm")],
        )
        self.assertIn("capture.confirmation.confirmed", operations)
        self.assertEqual([item.id for item in description.streams], ["capture.confirmation.temperature"])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
