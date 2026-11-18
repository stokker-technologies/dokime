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
# Local Imports
from dokime_sdk import GridAxisEntry, Value, plot
from dokime_sdk.authoring import Stream
from dokime_sdk.documents import Document, Section, Summary, plot_card, records_card
from dokime_sdk.parameters import Parameter
from dokime_sdk.values import Scalar, decode, encode

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class DocumentStory(unittest.TestCase):
    def test_compose_serialize_and_reopen(self) -> None:
        # Build A Contour And Playback Grid From A Declared Stream.
        samples = Stream("example.samples")
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
            .rows([GridAxisEntry(key="sensor", label="Sensor")])
            .columns([GridAxisEntry(key="site", label="Site")])
        )
        document = Document("Evidence").section(
            Section("Measurements").plot(contour).plot(grid).summary(Summary().tile("Maximum", reading.max()))
        )

        # Reopen The Same Wire Format The Host Consumes.
        reopened = type(document.message).FromString(document.message.SerializeToString())
        surface, playback, summary = reopened.sections[0].cards
        self.assertEqual(surface.plot.z.transform.input.stream.identifier, "example.samples")
        self.assertEqual(surface.plot.z.transform.scale, 2.0)
        self.assertEqual(surface.plot.kind, plot.contour(reading).message.kind)
        self.assertEqual(playback.plot.grid.frame.stream.field, "frame")
        self.assertEqual(summary.summary.tiles[0].value.aggregate.input, reading.message)
        self.assertFalse(reading.message.HasField("aggregate"))

        # Preserve Parameter Precision, Explicit Null And Source Datetimes.
        values: list[Scalar] = [None, True, -(2**63), 2**64 - 1, 1.25, "reading", b"\x00\xff", [1, None], datetime(2026, 1, 2, tzinfo=UTC)]
        self.assertEqual([decode(encode(value)) for value in values], values)
        self.assertRaises(ValueError, encode, datetime(2026, 1, 2))
        self.assertRaises(ValueError, decode, Value())
        self.assertRaises(TypeError, plot.xy(reading).reduce, cast(Any, "mean"))

    def test_parameter_plot_card_shorthands_roundtrip(self) -> None:
        builder = Parameter.number("voltage", "Voltage").unit("V").default(230.0).range(0.0, 300.0)
        voltage = builder.build()
        builder.default(120.0)
        decoded = type(voltage).FromString(voltage.SerializeToString())
        self.assertEqual(decoded.expression.literal.number, 230.0)
        self.assertEqual((decoded.min, decoded.max), (0.0, 300.0))
        self.assertEqual(Parameter.count("samples", "Samples").default(100).build().expression.literal.unsigned_integer, 100)
        self.assertEqual(Parameter.count("large", "Large").default(2**64 - 1).build().expression.literal.unsigned_integer, 2**64 - 1)
        self.assertTrue(Parameter.flag("enabled", "Enabled").default(True).build().expression.literal.boolean)
        sweep = Parameter.sweep("loads", "Loads", [80.0, 100.0]).build()
        self.assertTrue(sweep.expansion.cartesian)
        self.assertEqual([value.literal.number for value in sweep.expression.array.values], [80.0, 100.0])
        ladder = Parameter.numbers("ladder", "Ladder", [10.0, 20.0]).items(1, 8).build()
        self.assertEqual([value.number for value in ladder.expression.literal.list.values], [10.0, 20.0])
        self.assertEqual((ladder.min_items, ladder.max_items), (1, 8))
        line = plot.Plot.line("voltage", "Voltage").unit("V").y(plot.field("samples", "voltage"))
        card = plot_card(line, columns=6)
        line.message.y.clear()
        self.assertEqual(len(card.plot.y), 1)
        self.assertEqual(card.layout.columns, 6)
        self.assertEqual(records_card("records").WhichOneof("body"), "evidence")


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
