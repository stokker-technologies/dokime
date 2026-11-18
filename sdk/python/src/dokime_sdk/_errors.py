# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Product Client Failures"""

# === Imports === #

# Standard Library Imports
from collections.abc import Awaitable, Mapping
from dataclasses import dataclass, field, replace
from types import MappingProxyType

# Third Party Imports
import harmos
from google.protobuf.message import DecodeError
from grpc import StatusCode
from grpc.aio import AioRpcError
from harmos._wire.model.v1.failure_pb2 import Classification

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


@dataclass(frozen=True)
class Failure:
    classification: str
    code: str
    message: str
    action: str
    operation: str = ""
    plugin: str = ""
    cause: "Failure | None" = None
    details: Mapping[str, str] = field(default_factory=lambda: MappingProxyType({}))
    diagnostics: str = ""

    def __post_init__(self) -> None:
        object.__setattr__(self, "details", MappingProxyType(dict(self.details)))


class SdkError(Exception):
    def __init__(self, failure: Failure, *, request_id: str | None = None) -> None:
        super().__init__(f"{failure.message}; {failure.action}")
        self.failure = replace(failure, details={**failure.details, "request_id": request_id}) if request_id is not None else failure
        self.request_id = request_id


class FailureError(SdkError):
    pass


# === Public Functions === #


def project_failure(value: harmos.FailureValue) -> Failure:
    classification = Classification.Name(value.classification)
    return Failure(
        classification.removeprefix("CLASSIFICATION_").replace("_", " ").title(),
        value.code,
        value.message,
        value.action,
        value.operation,
        value.plugin,
        project_failure(value.cause) if value.HasField("cause") else None,
        MappingProxyType(dict(value.details)),
        value.diagnostics,
    )


def refused(operation: str, message: str, action: str, *, request_id: str | None = None) -> SdkError:
    return SdkError(Failure("Refused", "host_capability_required", message, action, operation), request_id=request_id)


def invalid(operation: str, message: str) -> SdkError:
    return SdkError(Failure("Permanent", "invalid_response", message, "Read Again from the Intended Host", operation))


async def call[T](operation: str, pending: Awaitable[T], request_id: str | None = None) -> T:
    try:
        return await pending
    except SdkError:
        raise
    except AioRpcError as error:
        raise transport_error(operation, error, request_id) from error
    except ValueError as error:
        raise invalid(operation, str(error)) from error
    except RuntimeError as error:
        raise SdkError(
            Failure("Refused", "transport_unavailable", str(error), "Connect a New Client", operation), request_id=request_id
        ) from error


def transport_error(operation: str, error: AioRpcError, request_id: str | None = None) -> SdkError:
    for key, value in error.trailing_metadata() or ():
        if key != "grpc-status-details-bin" or not isinstance(value, bytes):
            continue
        try:
            remote = harmos.FailureValue.FromString(value)
        except DecodeError:
            continue
        if remote.classification:
            return SdkError(project_failure(remote), request_id=request_id)
    classification = {
        StatusCode.CANCELLED: "Cancelled",
        StatusCode.DEADLINE_EXCEEDED: "Timed Out",
        StatusCode.UNAVAILABLE: "Transient",
        StatusCode.ABORTED: "Refused",
        StatusCode.FAILED_PRECONDITION: "Refused",
        StatusCode.PERMISSION_DENIED: "Refused",
    }.get(error.code(), "Permanent")
    failure = Failure(
        classification,
        error.code().name,
        error.details() or operation,
        "Check the Host Failure and Retry the Unchanged Request When Appropriate",
        operation,
        diagnostics=error.debug_error_string() or "",
    )
    return SdkError(failure, request_id=request_id)


# === Private Functions === #

# === Entry Point === #
