# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Conversion at the private protobuf boundary."""

import re
from datetime import UTC, datetime, timedelta
from typing import Any

from google.protobuf.json_format import MessageToDict
from google.protobuf.message import Message

from ._contracts import copy, equal, nested, request, variant
from .errors import ProtocolError
from .models import (
    CommandAction,
    CommandPhase,
    DatetimeValue,
    ExecutionCommandReceipt,
    Record,
    ResultSummary,
    Sample,
    UnsignedValue,
    Value,
)

__all__ = ["copy", "equal", "nested", "request", "variant"]


def encode(item: Value, maximum_depth: int) -> dict:
    nodes = []

    def add(item: Value, depth: int) -> int:
        if depth > maximum_depth:
            raise ValueError("value exceeds the configured nesting depth")
        index = len(nodes)
        nodes.append({})
        if item is None:
            kind = {"Null": {}}
        elif isinstance(item, bool):
            kind = {"Bool": {"field_0": item}}
        elif isinstance(item, UnsignedValue):
            kind = {"Unsigned": {"field_0": int(item)}}
        elif isinstance(item, int):
            if not -(2**63) <= item < 2**64:
                raise ValueError("integer is outside the canonical 64-bit range")
            kind = {"Unsigned" if item >= 2**63 else "Integer": {"field_0": item}}
        elif isinstance(item, float):
            kind = {"Number": {"field_0": item}}
        elif isinstance(item, str):
            kind = {"Text": {"field_0": item}}
        elif isinstance(item, datetime):
            kind = {"Datetime": {"field_0": timestamp_fields(item)}}
        elif isinstance(item, list):
            kind = {"Array": {"field_0": {"items": [add(child, depth + 1) for child in item]}}}
        elif isinstance(item, dict):
            if not all(isinstance(key, str) for key in item):
                raise TypeError("object keys must be strings")
            kind = {"Object": {"field_0": {"fields": [{"key": key, "value": add(item[key], depth + 1)} for key in sorted(item)]}}}
        else:
            raise TypeError(f"unsupported parameter value: {type(item).__name__}")
        nodes[index] = {"kind": kind}
        return index

    add(item, 0)
    return {"nodes": nodes}


def decode(item: Message, maximum_depth: int) -> Value:
    if not item.nodes:
        raise ProtocolError("value has no root node")
    visited = set()

    def rebuild(index: int, parent: int, depth: int) -> Value:
        if depth > maximum_depth or not parent < index < len(item.nodes) or index in visited:
            raise ProtocolError("value graph is cyclic, shared, out of range, or too deep")
        visited.add(index)
        node = item.nodes[index]
        if not node.HasField("kind"):
            raise ProtocolError("value node has no kind")
        name = variant(node.kind)
        payload = getattr(node.kind, name)
        if name == "Null":
            return None
        if name in ("Bool", "Integer", "Number", "Text"):
            return payload.field_0
        if name == "Unsigned":
            return UnsignedValue(payload.field_0)
        if name == "Datetime":
            return datetime_value(payload.field_0)
        if name == "Array":
            return [rebuild(child, index, depth + 1) for child in payload.field_0.items]
        if name == "Object":
            result = {}
            for field in payload.field_0.fields:
                if field.key in result:
                    raise ProtocolError("value object repeats a key")
                result[field.key] = rebuild(field.value, index, depth + 1)
            return result
        raise ProtocolError("unknown value variant")

    result = rebuild(0, -1, 0)
    if len(visited) != len(item.nodes):
        raise ProtocolError("value graph contains unreachable nodes")
    return result


def fields(item: Message, maximum_depth: int) -> dict[str, Value]:
    result = decode(item, maximum_depth)
    if not isinstance(result, dict):
        raise ProtocolError("expected an object value")
    return result


def timestamp_fields(item: datetime) -> dict:
    if item.tzinfo is None or item.utcoffset() is None:
        raise ValueError("datetimes must include a timezone")
    utc = item.astimezone(UTC)
    delta = utc - datetime(1970, 1, 1, tzinfo=UTC)
    seconds = delta.days * 86400 + delta.seconds
    return {
        "seconds": seconds,
        "nanos": utc.microsecond * 1000 + getattr(item, "_nanosecond_remainder", 0),
    }


def datetime_value(item: Message) -> DatetimeValue:
    if not 0 <= item.nanos < 1_000_000_000:
        raise ProtocolError("timestamp nanoseconds are out of range")
    try:
        date = datetime(1970, 1, 1, tzinfo=UTC) + timedelta(seconds=item.seconds, microseconds=item.nanos // 1000)
    except (OverflowError, ValueError) as error:
        raise ProtocolError("timestamp is out of range") from error
    text = date.strftime("%Y-%m-%dT%H:%M:%S") + f".{item.nanos:09d}+00:00"
    return DatetimeValue.from_rfc3339(text)


def timestamp(item: Message, name: str) -> datetime | None:
    return datetime_value(getattr(item, name)) if item.HasField(name) else None


def enum_name(item: Message) -> str:
    name = variant(item)
    return re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()


def scope(item: Message) -> dict[str, Any]:
    return {
        descriptor.name: MessageToDict(value, preserving_proto_field_name=True) if isinstance(value, Message) else value
        for descriptor, value in item.ListFields()
    }


def command_receipt(item: Message) -> ExecutionCommandReceipt:
    try:
        action = CommandAction(enum_name(item.action))
        phase = CommandPhase(enum_name(item.phase))
    except ValueError as error:
        raise ProtocolError("unknown execution command action or phase") from error
    return ExecutionCommandReceipt(
        request_id=item.request_id,
        command_id=item.command_id,
        execution_id=item.execution_id,
        action=action,
        step_id=item.step_id if item.HasField("step_id") else None,
        phase=phase,
        attempt_id=item.attempt_id if item.HasField("attempt_id") else None,
        accepted_at=timestamp(item, "accepted_at"),
        updated_at=timestamp(item, "updated_at"),
        detail=item.detail if item.HasField("detail") else None,
    )


def result(item: Message) -> ResultSummary:
    return ResultSummary(
        status=enum_name(item.status),
        passed=item.passed,
        failed=item.failed,
        blocked=item.blocked,
        skipped=item.skipped,
    )


def record(item: Message, maximum_depth: int) -> Record:
    return Record(
        id=item.record_id,
        type=item.record_type,
        sequence=item.sequence,
        schema_version=item.schema_version,
        source=item.source,
        severity=enum_name(item.severity),
        timestamp=timestamp(item, "timestamp"),
        message=item.message if item.HasField("message") else None,
        details=fields(item.details, maximum_depth) if item.HasField("details") else {},
        scope=scope(item.scope),
        error=MessageToDict(item.error, preserving_proto_field_name=True) if item.HasField("error") else None,
        related_sample_id=item.related_sample_id if item.HasField("related_sample_id") else None,
        related_stream_id=item.related_stream_id if item.HasField("related_stream_id") else None,
        related_artifact_id=item.related_artifact_id if item.HasField("related_artifact_id") else None,
    )


def sample(item: Message, maximum_depth: int) -> Sample:
    return Sample(
        id=item.sample_id,
        stream=item.stream,
        sequence=item.sequence,
        schema_version=item.schema_version,
        source=item.source,
        timestamp=timestamp(item, "timestamp"),
        received_at=timestamp(item, "received_at"),
        values=fields(item.values, maximum_depth) if item.HasField("values") else {},
        summary=fields(item.summary, maximum_depth) if item.HasField("summary") else {},
        metadata=fields(item.metadata, maximum_depth) if item.HasField("metadata") else {},
        scope=scope(item.scope),
    )
