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
from typing import Any, cast

import dokime_sdk.templates

# Third Party Imports
# Local Imports
from dokime_sdk.templates import Document, GridAxisEntry, Section, Stream, Summary, plot_card, records_card

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class DocumentStory(unittest.TestCase):
    def test_telemetry_at_record_histograms_roundtrip(self) -> None:
        sampled = dokime_sdk.templates.field("readings", "value").at("status")
        section = Section("Connection Events")
        for event in ("disconnected", "connected"):
            plot = dokime_sdk.templates.histogram(sampled).filter(sampled.detail("event"), dokime_sdk.templates.FilterOperator.EQ, event)
            section.plot(plot)
        document = Document("Tracking").section(section).message
        reopened = type(document).FromString(document.SerializeToString())
        for card, event in zip(reopened.sections[0].cards, ("disconnected", "connected"), strict=True):
            self.assertEqual(card.plot.x.transform.at.record_type, "status")
            self.assertFalse(card.plot.x.transform.at.HasField("within_s"))
            self.assertEqual(card.plot.filters[0].expression.transform.input, sampled.message)
            self.assertEqual(card.plot.filters[0].expression.transform.detail, "event")
            self.assertEqual(card.plot.filters[0].value.text, event)

    def test_heat_strip_document_roundtrips_every_reducer(self) -> None:
        samples = Stream("example.samples")
        for reduction in (
            dokime_sdk.templates.Reduction.LAST,
            dokime_sdk.templates.Reduction.MAX,
            dokime_sdk.templates.Reduction.MEAN,
            dokime_sdk.templates.Reduction.COUNT,
        ):
            plot = dokime_sdk.templates.heat_strip(samples.field("value"), label="Heat Strip").cells(samples.field("cell"))
            plot.facet(samples.field("facet")).reduce(reduction)
            document = Document("Report").section(Section("Values").plot(plot)).message
            decoded = type(document).FromString(document.SerializeToString())
            self.assertEqual(decoded, document)
            captured = decoded.sections[0].cards[0].plot
            self.assertEqual(captured.cell.stream.field, "cell")
            self.assertEqual(captured.facet.stream.field, "facet")
            self.assertEqual(captured.reduction, int(reduction))

    def test_compose_serialize_and_reopen(self) -> None:
        # Build A Contour And Playback Grid From A Declared Stream.
        samples = Stream("example.samples")
        reading = samples.field("temperature").scale(2.0)
        contour = (
            dokime_sdk.templates.contour(reading)
            .x(samples.field("x"))
            .y(samples.field("y"))
            .reduce(dokime_sdk.templates.Reduction.MEAN)
            .render(dokime_sdk.templates.ContourRender.BANDS)
            .label("Temperature")
        )
        grid = (
            dokime_sdk.templates.grid_chart(samples.field("value"))
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
        self.assertEqual(playback.plot.grid.frame.stream.field, "frame")
        self.assertEqual(summary.summary.tiles[0].value.aggregate.input, reading.message)
        self.assertFalse(reading.message.HasField("aggregate"))

        self.assertRaises(TypeError, dokime_sdk.templates.xy(reading).reduce, cast(Any, "mean"))

    def test_plot_cards_roundtrip(self) -> None:
        line = dokime_sdk.templates.Plot.line("voltage", "Voltage").unit("V").y(dokime_sdk.templates.field("samples", "voltage"))
        card = plot_card(line, columns=6)
        line.message.y.clear()
        self.assertEqual(len(card.plot.y), 1)
        self.assertEqual(card.layout.columns, 6)
        self.assertEqual(records_card("records").WhichOneof("body"), "evidence")


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
