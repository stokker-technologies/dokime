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

    def __init__(self, code: str, message: str) -> None:
        super().__init__(f"{code}: {message}")
        self.code = code
        self.message = message

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


class SubmissionUncertain(DokimeError):
    """Resolve the original Cycle.run(request_key=...) admission. Do not recreate."""

    def __init__(self, cycle_id: str, request_key: str) -> None:
        super().__init__(
            f"Run admission was not confirmed for cycle {cycle_id}. "
            "Resolve it with Cycle.run using this cycle_id and request_key."
        )
        self.cycle_id = cycle_id
        self.request_key = request_key


class ExecutionSubmissionUncertain(DokimeError):
    """Resolve admission with the same execution, request key and execution mode."""

    def __init__(
        self,
        execution_id: str,
        request_key: str,
        mode: Literal["automatic", "interactive"],
    ) -> None:
        super().__init__(
            f"Execution admission was not confirmed for {execution_id}. "
            "Resolve it with TestExecutionHandle.start using the same request_key and mode."
        )
        self.execution_id = execution_id
        self.request_key = request_key
        self.mode = mode


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
