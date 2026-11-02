# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Server Telemetry Statistics"""

# === Imports === #

# Standard Library Imports
import math
from collections.abc import Sequence
from dataclasses import dataclass
from typing import Any

# Third Party Imports
# Local Imports
from ..._wire.dokime.v1.QueryMessages.ListExecutions import messages_pb2 as contracts
from .. import _wire
from ..aggregate import Aggregate
from ..errors import ProtocolError

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True, slots=True)
class Statistic:
    name: str
    field: str
    aggregate: Aggregate
    value: int | float | None


@dataclass(frozen=True, slots=True)
class QualityCounts:
    field: str
    finite: int
    missing: int
    null: int
    non_finite: int


@dataclass(frozen=True, slots=True)
class Statistics:
    schema: contracts.Row7
    values: tuple[Statistic, ...]
    selected_rows: int
    quality_counts: tuple[QualityCounts, ...]
    snapshot: contracts.Row8
    complete: bool


# === Public Functions === #


def outputs(field: str, aggregates: Sequence[Aggregate]) -> list[dict[str, Any]]:
    if not isinstance(field, str):
        raise TypeError("aggregate field must be a literal field ID; use an empty field only for row count")
    if isinstance(aggregates, (str, bytes)) or not isinstance(aggregates, Sequence) or not aggregates:
        raise TypeError("Use a nonempty sequence of Aggregate objects, for example [Aggregate.mean()]")
    result: list[dict[str, Any]] = []
    names: set[str] = set()
    for aggregate in aggregates:
        if not isinstance(aggregate, Aggregate):
            raise TypeError("Use Aggregate.mean(), Aggregate.percentile(95), or another Aggregate object")
        if aggregate.member not in ("Count", "Sum", "Mean", "Median", "Min", "Max", "Rms", "Std", "Percentile"):
            raise ValueError("Use a declared Aggregate member")
        if aggregate.member == "Percentile" and (type(aggregate.rank) is not int or not 0 <= aggregate.rank <= 100):
            raise ValueError("Aggregate.percentile requires an integral rank from 0 to 100")
        if not field and aggregate.member != "Count":
            raise ValueError("Only Aggregate.count() can use an empty field for row count")
        name = str(aggregate)
        if name in names:
            raise ValueError(f"Duplicate aggregate output {name!r}; request each aggregate once")
        names.add(name)
        result.append({"alias": name, "field": field, "aggregate": aggregate.to_wire()})
    return result


def decode(response: Any, requested: list[dict[str, Any]]) -> Statistics:
    if not response.complete or any(not response.HasField(name) for name in ("schema", "snapshot")):
        raise ProtocolError("Statistics are incomplete or missing their captured schema/snapshot")
    schema = contracts.Row7()
    snapshot = contracts.Row8()
    _wire.copy(schema, response.schema)
    _wire.copy(snapshot, response.snapshot)
    if not snapshot.token or not snapshot.identity:
        raise ProtocolError("Statistics snapshot is empty")
    expected = [
        (
            out["alias"],
            out["field"],
            Aggregate.from_wire(_wire.nested("Query", "AggregateSamples", "outputs.aggregate", **out["aggregate"])),
        )
        for out in requested
    ]
    actual = [(value.alias, value.field, Aggregate.from_wire(value.aggregate)) for value in response.values]
    if actual != expected:
        raise ProtocolError("Statistics output order/identity differs from the request")
    values = tuple(
        Statistic(
            value.alias,
            value.field,
            Aggregate.from_wire(value.aggregate),
            statistic_value(value.value) if value.HasField("value") else None,
        )
        for value in response.values
    )
    quality = tuple(QualityCounts(q.field, q.finite, q.missing, q.null, q.non_finite) for q in response.quality_counts)
    return Statistics(schema, values, response.selected_rows, quality, snapshot, True)


def statistic_value(value: Any) -> int | float:
    member = _wire.variant(value)
    if member not in ("Count", "Number"):
        raise ProtocolError("Unknown statistic value kind")
    result = getattr(value, member).field_0
    if isinstance(result, float) and not math.isfinite(result):
        raise ProtocolError("Non-finite statistic result")
    return result


# === Private Functions === #

# === Entry Point === #
