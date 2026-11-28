# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Exact Scalars Survive Statistics and Plot Serialization."""

# === Imports === #

# Standard Library Imports
from fractions import Fraction
from unittest import TestCase

from dokime_sdk._generated.proto.host.v1.evidence_pb2 import Statistics

# Third Party Imports
# Local Imports
from dokime_sdk._generated.proto.host.v1.plots.snapshots_pb2 import PlotColumn
from dokime_sdk._generated.proto.model.v1.scalar_pb2 import Value
from dokime_sdk._values import decode, encode

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class ExactValuesStory(TestCase):
    def test_integers_and_rationals_remain_exact(self) -> None:
        values = [-(2**90), -(2**63), 2**63, 2**64 - 1, 2**90, Fraction(-7, 3), Fraction(2**100 + 1, 11)]
        for value in values:
            encoded = encode(value)
            reopened = Value.FromString(encoded.SerializeToString())
            self.assertEqual(decode(reopened), value)

    def test_statistics_distinguish_missing_values_from_exact_zero(self) -> None:
        statistics = Statistics(count=3, sum=encode(2**65 + 1), mean=encode(Fraction(2**65 + 1, 3)), min=encode(0))
        reopened = Statistics.FromString(statistics.SerializeToString())
        self.assertTrue(reopened.HasField("min"))
        self.assertFalse(reopened.HasField("median"))
        self.assertEqual(decode(reopened.sum), 2**65 + 1)
        self.assertEqual(decode(reopened.mean), Fraction(2**65 + 1, 3))
        self.assertEqual(decode(reopened.min), 0)

    def test_plot_totals_do_not_use_drawing_coordinates(self) -> None:
        totals = [2**64 - 1, Fraction(2**90 + 1, 7)]
        column = PlotColumn(totals=[encode(value) for value in totals])
        reopened = PlotColumn.FromString(column.SerializeToString())
        self.assertEqual([decode(value) for value in reopened.totals], totals)
        self.assertEqual(list(reopened.y), [])


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
