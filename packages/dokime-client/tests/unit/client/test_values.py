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
from datetime import UTC, datetime, timedelta, timezone
from typing import Any, cast

# Third Party Imports
import pytest

# Local Imports
import dokime_client
from dokime_client.client import Aggregate, DatetimeValue, UnsignedValue, _contracts, _wire
from dokime_client.client._wire import decode, encode

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def test_number_values() -> None:
    for integer in [-(2**63), 2**63 - 1, 2**63, 2**64 - 1]:
        assert decode(_round_trip(integer), 64) == integer
    assert _wire.variant(_project(2**64 - 1).nodes[0].kind) == "Unsigned"
    unsigned_zero = UnsignedValue(0)
    zero = decode(_project(unsigned_zero), 64)
    assert _wire.variant(_project(zero).nodes[0].kind) == "Unsigned"

    for number in [float("nan"), float("inf"), float("-inf")]:
        result = decode(_round_trip(number), 64)
        assert isinstance(result, float)
        assert math.isnan(result) if math.isnan(number) else result == number


def test_datetime_values() -> None:
    date = datetime(2026, 9, 28, 12, 13, 14, 123456, tzinfo=UTC)
    assert decode(_project(date), 64) == date

    # Nanosecond Precision Survives And Orders After The Microsecond Value
    precise = _project(DatetimeValue.from_rfc3339("2026-09-28T12:13:14.123456789Z"))
    assert _project(decode(precise, 64)) == precise
    decoded = decode(precise, 64)
    assert isinstance(decoded, DatetimeValue)
    assert decoded != date
    assert decoded > date
    assert len({decoded, date}) == 2

    # Transformations Preserve The Exact Wire Fraction Unless Explicitly Replaced.
    for transformed, expected in (
        (decoded.replace(second=15), "2026-09-28T12:13:15.123456789+00:00"),
        (decoded + timedelta(seconds=1), "2026-09-28T12:13:15.123456789+00:00"),
        (timedelta(seconds=1) + decoded, "2026-09-28T12:13:15.123456789+00:00"),
        (decoded - timedelta(hours=13, microseconds=123457), "2026-09-27T23:13:13.999999789+00:00"),
        (decoded + timedelta(days=1, microseconds=876544), "2026-09-29T12:13:15.000000789+00:00"),
        (decoded.replace(microsecond=0), "2026-09-28T12:13:14.000000000+00:00"),
        (decoded.replace(2026, 9, 28, 12, 13, 15, 0), "2026-09-28T12:13:15.000000000+00:00"),
        (decoded.astimezone(timezone(timedelta(hours=5, minutes=30))), "2026-09-28T12:13:14.123456789+00:00"),
    ):
        assert transformed._wire_text is None
        received = decode(_project(transformed), 64)
        assert isinstance(received, DatetimeValue)
        assert received._wire_text == expected

    # An Unchanged Timezone Returns The Original; Datetime Differences Stay Microsecond Timedeltas.
    assert decoded.astimezone(decoded.tzinfo) is decoded
    assert decoded._wire_text == "2026-09-28T12:13:14.123456789+00:00"
    assert decoded - date == timedelta(0)
    with pytest.raises(TypeError):
        decoded + cast(Any, 1)
    with pytest.raises(OverflowError):
        decoded.replace(year=9999) + timedelta(days=366)


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
        pytest.raises(ValueError, Aggregate.percentile, rank).match("outside 0 to 100")


# === Private Functions === #


def _project(item):
    return _contracts.nested("Evidence", "ListRecords", "records.details", direction="response", **encode(item, 64))


def _round_trip(item):
    wire = _project(item)
    return type(wire).FromString(wire.SerializeToString())


# === Entry Point === #
