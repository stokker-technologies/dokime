# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Conversion at the private protobuf boundary."""

# === Imports === #

# Standard Library Imports
from datetime import UTC, datetime

# Third Party Imports
from google.protobuf.json_format import MessageToDict
from google.protobuf.message import Message

# Local Imports
from ._generated.dokime.v1 import common_pb2 as common
from ._generated.dokime.v1 import content_pb2 as content
from ._generated.dokime.v1 import evidence_pb2 as evidence
from ._generated.dokime.v1 import execution_pb2 as execution
from ._generated.dokime.v1 import telemetry_pb2 as telemetry
from ._generated.dokime.v1 import value_pb2 as value
from .errors import ProtocolError
from .models import (
    CommandAction,
    CommandPhase,
    ExecutionCommandReceipt,
    DatetimeValue,
    Record,
    ResultSummary,
    RunPhase,
    RunStatus,
    Sample,
    UnsignedValue,
    Value,
)

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def encode(item: Value) -> value.Value:
    if item is None:
        return value.Value(null_value=value.NULL_VALUE_NULL)
    if isinstance(item, bool):
        return value.Value(bool_value=item)
    if isinstance(item, UnsignedValue):
        return value.Value(unsigned_value=item)
    if isinstance(item, int):
        return (
            value.Value(unsigned_value=item)
            if item >= 2**63
            else value.Value(integer_value=item)
        )
    if isinstance(item, float):
        return value.Value(number_value=item)
    if isinstance(item, str):
        return value.Value(text_value=item)
    if isinstance(item, DatetimeValue) and item._wire_text is not None:
        return value.Value(datetime_value=item._wire_text)
    if isinstance(item, datetime):
        if item.tzinfo is None or item.utcoffset() is None:
            raise ValueError("parameter datetimes must include a timezone")
        return value.Value(datetime_value=item.astimezone(UTC).isoformat())
    if isinstance(item, list):
        return value.Value(
            array_value=value.ValueList(values=[encode(v) for v in item])
        )
    if isinstance(item, dict):
        return value.Value(
            object_value=value.ValueObject(
                fields={k: encode(v) for k, v in item.items()}
            )
        )
    raise TypeError(f"unsupported parameter value: {type(item).__name__}")


def decode(item: value.Value) -> Value:
    match item.WhichOneof("kind"):
        case "null_value":
            return None
        case "bool_value":
            return item.bool_value
        case "integer_value":
            return item.integer_value
        case "unsigned_value":
            return UnsignedValue(item.unsigned_value)
        case "number_value":
            return item.number_value
        case "text_value":
            return item.text_value
        case "datetime_value":
            return DatetimeValue.from_rfc3339(item.datetime_value)
        case "array_value":
            return [decode(v) for v in item.array_value.values]
        case "object_value":
            return fields(item.object_value)
        case _:
            raise ProtocolError("backend returned a Value without a kind")


def fields(item: value.ValueObject) -> dict[str, Value]:
    return {key: decode(v) for key, v in item.fields.items()}


def timestamp(item: Message, name: str) -> datetime | None:
    return getattr(item, name).ToDatetime(tzinfo=UTC) if item.HasField(name) else None


def scope(item: common.EvidenceScope | content.ArtifactScope) -> dict[str, str]:
    return {descriptor.name: str(v) for descriptor, v in item.ListFields()}


def status(item: execution.RunRequestReceipt) -> RunStatus:
    try:
        phase = RunPhase(
            execution.RunRequestPhase.Name(item.phase)
            .removeprefix("RUN_REQUEST_PHASE_")
            .lower()
        )
    except ValueError as error:
        raise ProtocolError(f"unknown run phase {item.phase}") from error
    return RunStatus(
        id=item.request_id,
        cycle_id=item.cycle_id,
        phase=phase,
        paused=item.paused,
        stop_requested=item.stop_requested,
        planned=item.planned_count,
        prepared=item.prepared_count,
        finished=item.finished_count,
        failed=item.failed_count,
        unstarted=item.unstarted_count,
        claimed=item.claimed_count,
        last_execution_id=item.last_claim.execution_id or None,
        accepted_at=timestamp(item, "accepted_at"),
        finished_at=timestamp(item, "finished_at"),
        interruption=item.interruption if item.HasField("interruption") else None,
    )


def command_receipt(item: execution.ExecutionCommandReceipt) -> ExecutionCommandReceipt:
    try:
        action = CommandAction(
            execution.ExecutionCommandAction.Name(item.action)
            .removeprefix("EXECUTION_COMMAND_ACTION_")
            .lower()
        )
        phase = CommandPhase(
            execution.ExecutionCommandPhase.Name(item.phase)
            .removeprefix("EXECUTION_COMMAND_PHASE_")
            .lower()
        )
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


def result(item: execution.ResultSummary) -> ResultSummary:
    return ResultSummary(
        status=execution.ResultStatus.Name(item.status)
        .removeprefix("RESULT_STATUS_")
        .lower(),
        passed=item.passed,
        failed=item.failed,
        blocked=item.blocked,
        skipped=item.skipped,
    )


def record(item: evidence.CapturedRecord) -> Record:
    return Record(
        id=item.record_id,
        type=item.record_type,
        sequence=item.sequence,
        schema_version=item.schema_version,
        source=item.source,
        severity=common.Severity.Name(item.severity).removeprefix("SEVERITY_").lower(),
        timestamp=timestamp(item, "timestamp"),
        message=item.message if item.HasField("message") else None,
        details=fields(item.details),
        scope=scope(item.scope),
        error=MessageToDict(item.error, preserving_proto_field_name=True)
        if item.HasField("error")
        else None,
        related_sample_id=item.related_sample_id
        if item.HasField("related_sample_id")
        else None,
        related_stream_id=item.related_stream_id
        if item.HasField("related_stream_id")
        else None,
        related_artifact_id=item.related_artifact_id
        if item.HasField("related_artifact_id")
        else None,
    )


def sample(item: telemetry.TelemetrySample) -> Sample:
    return Sample(
        id=item.sample_id,
        stream=item.stream,
        sequence=item.sequence,
        schema_version=item.schema_version,
        source=item.source,
        timestamp=timestamp(item, "timestamp"),
        received_at=timestamp(item, "received_at"),
        values=fields(item.values),
        summary=fields(item.summary),
        metadata=fields(item.metadata),
        scope=scope(item.scope),
    )


# === Private Functions === #

# === Entry Point === #
