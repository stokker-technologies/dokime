# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Round-Trip Typed Values Through The Canonical Wire"""

# === Imports === #

# Standard Library Imports
import math
from datetime import UTC, datetime, timedelta

# Third Party Imports
import pytest

# Local Imports
import dokime_client
from dokime_client.client import Aggregate, DatetimeValue, _wire
from dokime_client.client._wire import decode, encode

from .canonical import _message, _value

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_number_values() -> None:
    for integer in [-(2**63), 2**63 - 1, 2**63, 2**64 - 1]:
        assert decode(_round_trip(integer)) == integer
    assert _wire.variant(_project(2**64 - 1).nodes[0].kind) == "Unsigned"
    zero = decode(_value(unsigned_value=0))
    assert _wire.variant(_project(zero).nodes[0].kind) == "Unsigned"

    for number in [float("nan"), float("inf"), float("-inf")]:
        result = decode(_round_trip(number))
        assert isinstance(result, float)
        assert math.isnan(result) if math.isnan(number) else result == number


def test_datetime_values() -> None:
    date = datetime(2026, 9, 28, 12, 13, 14, 123456, tzinfo=UTC)
    assert decode(_project(date)) == date

    # Nanosecond Precision Survives And Orders After The Microsecond Value
    precise = _value(datetime_value="2026-09-28T12:13:14.123456789Z")
    assert _project(decode(precise)) == precise
    decoded = decode(precise)
    assert isinstance(decoded, datetime)
    assert decoded != date
    assert decoded > date
    assert len({decoded, date}) == 2

    # Arithmetic Keeps The Precise Wire Representation
    replaced = decode(_project(decoded.replace(second=15)))
    shifted = decode(_project(decoded + timedelta(seconds=1)))
    assert isinstance(replaced, DatetimeValue)
    assert isinstance(shifted, DatetimeValue)
    assert replaced._wire_text is not None
    assert shifted._wire_text is not None
    assert replaced._wire_text.startswith("2026-09-28T12:13:15.")
    assert shifted._wire_text.startswith("2026-09-28T12:13:15.")


def test_aggregate_codec() -> None:
    request = dokime_client.DokimeQueryAggregateRequest(
        aggregates=[
            {"aggregate": Aggregate.percentile(95).to_wire(), "column": "duration_ms"},
            {"aggregate": Aggregate.count().to_wire()},
        ]
    )
    decoded = dokime_client.DokimeQueryAggregateRequest.FromString(request.SerializeToString())
    percentile, count = (entry.aggregate for entry in decoded.aggregates)

    assert percentile.WhichOneof("_harmos_variant") == "Percentile"
    assert percentile.Percentile.rank == 95
    assert Aggregate.from_wire(percentile) == Aggregate.percentile(95)
    assert Aggregate.from_wire(count) == Aggregate.count()
    assert str(Aggregate.percentile(95)) == "p95"
    # An Absent Member Reads As Count, As The Harmos Codec Decodes It.
    assert Aggregate.from_wire(type(count)()) == Aggregate.count()
    for rank in (-1, 101):
        with pytest.raises(ValueError, match="outside 0 to 100"):
            Aggregate.percentile(rank)


# === Private Functions === #


def _project(item):
    return _message("Evidence", "ListRecords", path="records.details", **encode(item))


def _round_trip(item):
    wire = _project(item)
    return type(wire).FromString(wire.SerializeToString())


# === Entry Point === #
