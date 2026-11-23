# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Product Client Failures"""

# === Imports === #

# Standard Library Imports
from collections.abc import Awaitable, Sequence

# Third Party Imports
from google.protobuf.message import DecodeError
from grpc import StatusCode
from grpc.aio import AioRpcError
from harmos import OperationError
from harmos._wire.model.v1 import failure_pb2
from harmos._wire.model.v1.failure_pb2 import Failure

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


# === Public Functions === #


def chain(failures: Sequence[Failure]) -> Failure | None:
    cause: Failure | None = None
    for failure in reversed(failures):
        value = Failure()
        value.CopyFrom(failure)
        tail = value
        while tail.HasField("cause"):
            tail = tail.cause
        if cause is not None:
            tail.cause.CopyFrom(cause)
        cause = value
    return cause


def refused(operation: str, message: str, action: str, *, request_id: str | None = None) -> OperationError:
    failure = Failure(
        classification=failure_pb2.CLASSIFICATION_REFUSED,
        code="host_capability_required",
        message=message,
        action=action,
        operation=operation,
    )
    if request_id is not None:
        failure.details["request_id"] = request_id
    return OperationError(failure)


def invalid(operation: str, message: str, *, request_id: str | None = None) -> OperationError:
    failure = Failure(
        classification=failure_pb2.CLASSIFICATION_PERMANENT,
        code="invalid_response",
        message=message,
        action="Read Again from the Intended Host",
        operation=operation,
    )
    if request_id is not None:
        failure.details["request_id"] = request_id
    return OperationError(failure)


async def call[T](operation: str, pending: Awaitable[T], request_id: str | None = None) -> T:
    try:
        return await pending
    except AioRpcError as error:
        raise transport_error(operation, error, request_id) from error
    except ValueError as error:
        raise invalid(operation, str(error)) from error
    except RuntimeError as error:
        failure = refused(operation, str(error), "Connect a New Client", request_id=request_id)
        failure.value.code = "transport_unavailable"
        raise failure from error


def transport_error(operation: str, error: AioRpcError, request_id: str | None = None) -> OperationError:
    for key, value in error.trailing_metadata() or ():
        if key != "grpc-status-details-bin" or not isinstance(value, bytes):
            continue
        try:
            remote = Failure.FromString(value)
        except DecodeError:
            continue
        if remote.classification:
            if request_id is not None:
                remote.details["request_id"] = request_id
            return OperationError(remote)
    classification = {
        StatusCode.CANCELLED: failure_pb2.CLASSIFICATION_CANCELLED,
        StatusCode.DEADLINE_EXCEEDED: failure_pb2.CLASSIFICATION_TIMED_OUT,
        StatusCode.UNAVAILABLE: failure_pb2.CLASSIFICATION_TRANSIENT,
        StatusCode.ABORTED: failure_pb2.CLASSIFICATION_REFUSED,
        StatusCode.FAILED_PRECONDITION: failure_pb2.CLASSIFICATION_REFUSED,
        StatusCode.PERMISSION_DENIED: failure_pb2.CLASSIFICATION_REFUSED,
        StatusCode.UNAUTHENTICATED: failure_pb2.CLASSIFICATION_REFUSED,
    }.get(error.code(), failure_pb2.CLASSIFICATION_PERMANENT)
    message = error.details() or operation
    action = "Check the Host Failure and Retry the Unchanged Request When Appropriate"
    if error.code() == StatusCode.UNAUTHENTICATED:
        # Remote Access Refuses Revoked or Altered Keys; Only a New Pairing Restores Access.
        message, action = "The Host Refused the Access Key; Its Pairing Was Revoked or Altered", "Export a New Pairing From the Host"
    failure = Failure(
        classification=classification,
        code=error.code().name,
        message=message,
        action=action,
        operation=operation,
        diagnostics=error.debug_error_string() or "",
    )
    if request_id is not None:
        failure.details["request_id"] = request_id
    return OperationError(failure)


# === Private Functions === #

# === Entry Point === #
