# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Finite Query Client"""

# === Imports === #

# Standard Library Imports
import json
from typing import Any

# Third Party Imports
from google.protobuf import empty_pb2
from google.protobuf.message import DecodeError, Message
from google.protobuf.unknown_fields import UnknownFieldSet

# Local Imports
from ...operation import Operation
from .._transport import Transport
from ..errors import RpcError

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class QueryDependencyError(ImportError):
    """An Optional Dataframe Dependency Is Missing."""

    def __init__(self, extra: str):
        self.extra = extra
        super().__init__(f"Query conversion requires {extra}; install it with `uv add 'dokime-client[{extra}]'`")


class QueryConversionError(ValueError):
    """A Captured Field Cannot Be Represented Without Losing Its Type Or Value."""

    def __init__(self, field: str, message: str):
        self.field = field
        super().__init__(f"Cannot convert query field {field!r}: {message}; narrow fields or use typed iteration")


class QueryError(RpcError):
    """A Query Failure Retaining The Original RPC Status, Metadata And Cause."""

    def __init__(self, error: RpcError, details: dict[str, Any]):
        super().__init__(
            error.code,
            error.message,
            initial_metadata=error.initial_metadata,
            trailing_metadata=error.trailing_metadata,
            debug_error_string=error.debug_error_string,
        )
        self.details = details
        self.reason = details.get("reason")
        self.operation = details.get("operation")
        self.field = details.get("field")
        self.operator = details.get("operator")
        self.snapshot = details.get("snapshot")
        self.limit = details.get("limit")


class QueryLimitError(QueryError):
    """The Server Or Local Materializer Exceeded A Finite Budget."""


class QuerySnapshotError(QueryError):
    """The Requested Snapshot Is Expired Or Unavailable."""


# === Public Functions === #


async def query_call(transport: Transport, operation: Operation, request: Message) -> Message:
    try:
        return await transport.call(operation, request)
    except RpcError as error:
        details = _details(error)
        reason = details.get("reason")
        kind = QueryError
        if error.code == "RESOURCE_EXHAUSTED":
            kind = QueryLimitError
        elif reason in ("SnapshotExpired", "SnapshotUnavailable", "InvalidContinuation"):
            kind = QuerySnapshotError
        raise kind(error, details) from error


# === Private Functions === #


def _fields(payload: bytes, number: int) -> list[bytes]:
    # Inspect Standard Status/Any/ErrorInfo Fields Without Declaring Product Wire Types.
    module: Any = empty_pb2
    message = module.Empty.FromString(payload)
    return [bytes(field.data) for field in UnknownFieldSet(message) if field.field_number == number and field.wire_type == 2]


def _details(error: RpcError) -> dict[str, Any]:
    try:
        for key, payload in error.trailing_metadata:
            if key != "grpc-status-details-bin" or not isinstance(payload, bytes):
                continue
            for detail in _fields(payload, 3):
                if _fields(detail, 1) != [b"type.googleapis.com/google.rpc.ErrorInfo"]:
                    continue
                info = _fields(detail, 2)[0]
                if _fields(info, 2) != [b"dokime.query"]:
                    continue
                for entry in _fields(info, 3):
                    if _fields(entry, 1) == [b"query"]:
                        decoded = json.loads(_fields(entry, 2)[0])
                        if isinstance(decoded, dict):
                            return decoded
    except (DecodeError, ValueError, IndexError, TypeError):
        pass
    return {}


# === Entry Point === #
