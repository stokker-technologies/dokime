# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own A Native Backend For The Lifetime Of An Async Context"""

# === Imports === #

# Standard Library Imports
from collections.abc import Sequence
from pathlib import Path
from types import TracebackType
from typing import Self

# Third Party Imports
# Local Imports
from . import _native
from .authoring import Plugin
from .client.client import DokimeClient
from .client.errors import DokimeError

# Type Checking Imports


# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class BackendError(DokimeError):
    """Backend Startup, Pairing Or Shutdown Failed"""


class BackendClosed(BackendError):
    """The Backend Is Not Accepting Calls"""


class BackendRunning(BackendError):
    """Another Backend Owns The Requested Data Root"""

    def __init__(self, message: str, root: Path) -> None:
        super().__init__(message)
        self.root = root


class BackendAddressUnavailable(BackendError):
    """The Requested Listener Address Is Unavailable"""

    def __init__(self, message: str, address: str) -> None:
        super().__init__(f"{address}: {message}")
        self.address = address


class PluginIdConflict(BackendError):
    """An Embedded Plugin Uses An Installed Plugin's ID"""

    def __init__(self, message: str, plugin_id: str) -> None:
        super().__init__(message)
        self.plugin_id = plugin_id


class Backend:
    """Start The Shared Server; Port Zero Selects An Available Local Port"""

    def __init__(
        self,
        *,
        port: int | None = None,
        root: str | Path | None = None,
        application_root: str | Path | None = None,
        plugins: Sequence[Plugin | _native.EmbeddedProvider] = (),
    ) -> None:
        self._port = port
        self._root = Path(root) if root is not None else None
        self._application_root = Path(application_root) if application_root is not None else None
        self._plugins = tuple(plugins)
        self._inner: _native.Backend | None = None
        self._entered = False
        self._closed = False

    async def __aenter__(self) -> Self:
        if self._entered or self._closed:
            raise BackendClosed("Backend contexts can be entered once")
        self._entered = True
        plugins = tuple(plugin._bind() if isinstance(plugin, Plugin) else plugin for plugin in self._plugins)
        self._inner = await _native.Backend.start(self._port, self._root, self._application_root, plugins)
        return self

    async def __aexit__(self, exc_type: type[BaseException] | None, exc: BaseException | None, tb: TracebackType | None) -> None:
        try:
            await self.close()
        except BackendError as error:
            if exc is not None:
                raise error from exc
            raise

    @property
    def endpoint(self) -> str:
        return self._running().endpoint

    @property
    def root(self) -> Path:
        return self._running().root

    def client(self) -> DokimeClient:
        """Return A Client Context; Closing It Leaves This Backend Running"""
        return DokimeClient(self.endpoint)

    def export_pairing(self, path: str | Path, *, label: str) -> None:
        """Export GUI Pairing, Starting A Secure Loopback Listener If Disabled"""
        self._running().export_pairing(Path(path), label)

    async def close(self) -> None:
        self._closed = True
        if self._inner is not None:
            await self._inner.close()

    def _running(self) -> _native.Backend:
        if self._closed or self._inner is None:
            raise BackendClosed("Backend is not accepting calls")
        return self._inner


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
