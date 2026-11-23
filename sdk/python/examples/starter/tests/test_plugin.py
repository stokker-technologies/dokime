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
        self.assertLessEqual(
            {
                "example.echo",
                "example.initialize",
                "example.finalize",
                "example.capture.confirmation.materialize",
                "example.capture.confirmation.cleanup",
                "example.capture.confirmation.confirm",
                "example.capture.confirmation.confirmed",
            },
            operations,
        )

        # Decode the Suite Section With Its Own Published Descriptor Set.
        pool = Default()
        suites = []
        for section in description.sections:
            descriptor = pool.FindMessageTypeByName(section.type)
            value = GetMessageClass(descriptor).FromString(section.payload)
            if "suites" in descriptor.fields_by_name:
                suites.extend(MessageToDict(value)["suites"])
        self.assertIn("example.capture", [suite["metadata"]["identifier"] for suite in suites])
        self.assertIn(
            "example.capture.confirmation", [template["metadata"]["identifier"] for suite in suites for template in suite["templates"]]
        )
        self.assertIn(
            ("example.capture.confirmation.confirm", "example.capture.confirmation.confirm"),
            [
                (step["metadata"]["identifier"], step["operation"])
                for suite in suites
                for template in suite["templates"]
                for step in template["steps"]
            ],
        )
        self.assertIn("example.temperature", [item.id for item in description.streams])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    unittest.main()
