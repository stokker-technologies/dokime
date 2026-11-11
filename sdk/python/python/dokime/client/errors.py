# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Explicit remote refusals, connection failures, and recovery information."""

# === Imports === #

# Standard Library Imports
from typing import Literal

# Third Party Imports

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class DokimeError(Exception):
    """Base class for client and remote domain errors."""


class ClientClosed(DokimeError):
    """The client has been explicitly closed."""


class ProtocolError(DokimeError):
    """The backend contract or response cannot be used safely."""


class BackendChanged(ProtocolError):
    """A live evidence cursor no longer belongs to this backend process."""


class RpcError(DokimeError):
    """A gRPC failure; the native transport error is available as __cause__."""

    def __init__(
        self,
        code: str,
        message: str,
        *,
        initial_metadata: tuple[tuple[str, str | bytes], ...] = (),
        trailing_metadata: tuple[tuple[str, str | bytes], ...] = (),
        debug_error_string: str | None = None,
    ) -> None:
        super().__init__(f"{code}: {message}")
        self.code = code
        self.message = message
        self.initial_metadata = initial_metadata
        self.trailing_metadata = trailing_metadata
        self.debug_error_string = debug_error_string

    @property
    def submission_uncertain(self) -> bool:
        return self.code in {
            "UNAVAILABLE",
            "DEADLINE_EXCEEDED",
            "UNKNOWN",
            "INTERNAL",
            "CANCELLED",
        }


class RunRejected(DokimeError):
    def __init__(self, code: str, message: str) -> None:
        super().__init__(f"{code}: {message}")
        self.code = code
        self.message = message


class ExecutionSubmissionUncertain(DokimeError):
    """Resolve Admission With The Same Execution Identity And Mode."""

    def __init__(
        self,
        execution_id: str,
        mode: Literal["automatic", "interactive"],
    ) -> None:
        super().__init__(
            f"Execution admission was not confirmed for {execution_id}. Resolve it with Execution.start using the same execution and mode."
        )
        self.execution_id = execution_id
        self.mode = mode


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
