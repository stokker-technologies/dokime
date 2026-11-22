# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Templates Reference Existing Qualified Streams Without Registering Producers."""

# === Imports === #

# Standard Library Imports
import unittest

# Third Party Imports
# Local Imports
from dokime_sdk import Plugin, Suite
from dokime_sdk.templates import Document, Section, Stream, TestTemplate, time_series

# Type Checking Imports

# === Public Exports === #

# === Constants === #

SYSTEM = Stream("peer_fixture.system")
LOCAL = Stream("capture_fixture.system")

# === Types === #

# === Classes === #


class Check(TestTemplate):
    uses_streams = (SYSTEM, LOCAL, SYSTEM)
    documents = (Document("Evidence").section(Section("System").plot(time_series(SYSTEM.field("value")))),)


class CaptureSuite(Suite):
    templates = (Check,)


class CapturePlugin(Plugin):
    version = "1.0.0"
    suites = (CaptureSuite,)


class UsesStreamsTests(unittest.TestCase):
    def test_uses_streams_preserve_identifiers_and_documents_without_producers(self) -> None:
        plugin = CapturePlugin()
        template = plugin._suites[0].templates[0]
        self.assertEqual(template.resolve(SYSTEM), "peer_fixture.system")
        self.assertEqual(template.resolve(LOCAL), "capture_fixture.system")
        self.assertEqual(list(template.build({}).message.streams), ["peer_fixture.system", "capture_fixture.system"])
        self.assertEqual(len(plugin.declarations().streams), 0)
        plot = template.definition().documents[0].sections[0].cards[0].plot
        self.assertEqual(plot.y[0].stream.identifier, "peer_fixture.system")
        self.assertEqual(plot.y[0].stream.field, "value")
        self.assertRaises(ValueError, template.resolve, Stream("unlisted.system"))

    def test_uses_streams_require_explicit_qualified_identifiers(self) -> None:
        for identifier in (None, "", "system", "peer..system", "Peer.system"):

            class InvalidCheck(TestTemplate):
                uses_streams = (Stream(identifier),)

            class InvalidSuite(Suite):
                templates = (InvalidCheck,)

            class InvalidPlugin(Plugin):
                version = "1.0.0"
                suites = (InvalidSuite,)

            self.assertRaises(ValueError, InvalidPlugin)


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
