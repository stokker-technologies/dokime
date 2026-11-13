# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""An Author Composes A Report And Reopens Its Generated Wire Message."""

# === Imports === #

# Standard Library Imports
import unittest
from datetime import UTC, datetime
from typing import Any, cast

# Third Party Imports
import dokime

# Local Imports
from dokime import plot
from dokime.v1 import message, message_type
from dokime.values import decode, encode

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class DocumentStory(unittest.TestCase):
    def test_compose_serialize_and_reopen(self):
        # Build A Contour And Playback Grid From A Declared Stream.
        samples = dokime.Stream("example.samples")
        reading = samples.field("temperature").scale(2.0)
        contour = (
            plot.contour(reading)
            .x(samples.field("x"))
            .y(samples.field("y"))
            .reduce(plot.Reduction.MEAN)
            .render(plot.ContourRender.BANDS)
            .label("Temperature")
        )
        grid = (
            plot.grid_chart(samples.field("value"))
            .row(samples.field("row"))
            .column(samples.field("column"))
            .frame(samples.field("frame"))
            .rows([message("GridAxisEntry", key="sensor", label="Sensor")])
            .columns([message("GridAxisEntry", key="site", label="Site")])
        )
        document = dokime.Document("Evidence").section(
            dokime.Section("Measurements").plot(contour).plot(grid).summary(dokime.Summary().tile("Maximum", reading.max()))
        )

        # Reopen The Same Wire Format The Host Consumes.
        reopened = message_type("Document").FromString(document.message.SerializeToString())
        surface, playback, summary = reopened.sections[0].cards
        self.assertEqual(surface.plot.z.transform.input.stream.identifier, "example.samples")
        self.assertEqual(surface.plot.z.transform.scale, 2.0)
        self.assertEqual(surface.plot.kind, plot.contour(reading).message.kind)
        self.assertEqual(playback.plot.grid.frame.stream.field, "frame")
        self.assertEqual(summary.summary.tiles[0].value.aggregate.input, reading.message)
        self.assertFalse(reading.message.HasField("aggregate"))

        # Preserve Parameter Precision, Explicit Null And Source Datetimes.
        values = [None, True, -(2**63), 2**64 - 1, 1.25, "reading", b"\x00\xff", [1, None], datetime(2026, 1, 2, tzinfo=UTC)]
        self.assertEqual([decode(encode(value)) for value in values], values)
        with self.assertRaises(ValueError):
            encode(datetime(2026, 1, 2))
        with self.assertRaises(ValueError):
            decode(message("Value"))
        with self.assertRaises(TypeError):
            plot.xy(reading).reduce(cast(Any, "mean"))


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
