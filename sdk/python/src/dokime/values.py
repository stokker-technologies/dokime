# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Lossless Scalar Conversion At The Protobuf Boundary."""

# === Imports === #

# Standard Library Imports
from datetime import UTC, datetime
from fractions import Fraction
from typing import Any

# Third Party Imports
from google.protobuf.timestamp_pb2 import Timestamp

# Local Imports
from .v1 import message

# Type Checking Imports

# === Public Exports === #

__all__ = ["decode", "encode"]

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def encode(value: Any) -> Any:
    if value is None:
        return message("Value", null={})
    if isinstance(value, bool):
        return message("Value", boolean=value)
    if isinstance(value, Fraction):
        return message(
            "Value", exact={"negative": value < 0, "numerator": str(abs(value.numerator)), "denominator": str(value.denominator)}
        )
    if isinstance(value, int):
        return message("Value", **{"unsigned_integer" if value >= 2**63 else "integer": value})
    if isinstance(value, (float, str, bytes)):
        field = {float: "number", str: "text", bytes: "data"}[type(value)]
        return message("Value", **{field: value})
    if isinstance(value, (list, tuple)):
        return message("Value", list={"values": [encode(item) for item in value]})
    if isinstance(value, datetime):
        if value.tzinfo is None:
            raise ValueError("Datetime values require a timezone")
        timestamp = Timestamp()
        timestamp.FromDatetime(value)
        return message("Value", datetime=timestamp)
    raise TypeError(f"Unsupported Dokime value: {type(value).__name__}")


def decode(value: Any) -> Any:
    kind = value.WhichOneof("kind")
    if kind is None:
        raise ValueError("Dokime Value has no kind")
    if kind == "null":
        return None
    if kind == "list":
        return [decode(item) for item in value.list.values]
    if kind == "exact":
        return Fraction((-1 if value.exact.negative else 1) * int(value.exact.numerator), int(value.exact.denominator))
    if kind == "datetime":
        return value.datetime.ToDatetime(tzinfo=UTC)
    return getattr(value, kind)


# === Private Functions === #

# === Entry Point === #
