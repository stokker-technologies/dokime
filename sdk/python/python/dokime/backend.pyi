# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from collections.abc import Sequence
from pathlib import Path
from types import TracebackType
from typing import Self

# Third Party Imports
# Local Imports
from . import _native
from .client.client import DokimeClient
from .client.errors import DokimeError

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

class BackendError(DokimeError): ...
class BackendClosed(BackendError): ...

class BackendRunning(BackendError):
    root: Path
    def __init__(self, message: str, root: Path) -> None: ...

class BackendAddressUnavailable(BackendError):
    address: str
    def __init__(self, message: str, address: str) -> None: ...

class StorageFormatMismatch(BackendError):
    root: Path
    writer_build: str
    running_build: str
    writer_format: str | None
    running_format: str
    def __init__(
        self, message: str, root: Path, writer_build: str, running_build: str, writer_format: str | None, running_format: str
    ) -> None: ...

class PluginIdConflict(BackendError):
    plugin_id: str
    def __init__(self, message: str, plugin_id: str) -> None: ...

class Backend:
    def __init__(
        self,
        *,
        port: int | None = None,
        root: str | Path | None = None,
        application_root: str | Path | None = None,
        plugins: Sequence[_native.EmbeddedProvider] = (),
    ) -> None: ...
    async def __aenter__(self) -> Self: ...
    async def __aexit__(self, exc_type: type[BaseException] | None, exc: BaseException | None, tb: TracebackType | None) -> None: ...
    @property
    def endpoint(self) -> str: ...
    @property
    def root(self) -> Path: ...
    def client(self) -> DokimeClient: ...
    def export_pairing(self, path: str | Path, *, label: str) -> None: ...
    async def close(self) -> None: ...

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
