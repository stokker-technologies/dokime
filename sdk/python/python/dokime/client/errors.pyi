# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

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

class DokimeError(Exception): ...
class ClientClosed(DokimeError): ...
class ProtocolError(DokimeError): ...
class BackendChanged(ProtocolError): ...

class RpcError(DokimeError):
    code: str
    message: str
    initial_metadata: tuple[tuple[str, str | bytes], ...]
    trailing_metadata: tuple[tuple[str, str | bytes], ...]
    debug_error_string: str | None

    def __init__(
        self,
        code: str,
        message: str,
        *,
        initial_metadata: tuple[tuple[str, str | bytes], ...] = (),
        trailing_metadata: tuple[tuple[str, str | bytes], ...] = (),
        debug_error_string: str | None = None,
    ) -> None: ...
    @property
    def submission_uncertain(self) -> bool: ...

class RunRejected(DokimeError):
    code: str
    message: str
    def __init__(self, code: str, message: str) -> None: ...

class ExecutionSubmissionUncertain(DokimeError):
    execution_id: str
    mode: Literal["automatic", "interactive"]

    def __init__(
        self,
        execution_id: str,
        mode: Literal["automatic", "interactive"],
    ) -> None: ...
