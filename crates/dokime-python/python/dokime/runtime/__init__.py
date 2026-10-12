# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Python authoring backed by the shared application and its gRPC client."""

# === Imports === #

# Standard Library Imports
from __future__ import annotations
import asyncio
from pathlib import Path

# Third Party Imports

# Local Imports
from dokime import _native
from dokime.client import DokimeClient

# Type Checking Imports

# === Public Exports === #

# === Constants === #

ServerConfig = _native.ServerConfig
TlsIdentity = _native.TlsIdentity
BearerGrant = _native.BearerGrant
Role = _native.Role

# === Types === #

# === Classes === #


class Runtime:
    """An inert local backend owner. Register, start, and close on one async loop."""

    def __init__(
        self,
        *,
        storage_root: str | Path | None = None,
        application_root: str | Path | None = None,
        plugins_root: str | Path | None = None,
    ) -> None:
        self._native = _native.Runtime(storage_root, application_root, plugins_root)
        self._client: DokimeClient | None = None
        self._starting = False
        self._closing: asyncio.Task[None] | None = None
        self._loop: asyncio.AbstractEventLoop | None = None

    @property
    def client(self) -> DokimeClient:
        if self._client is None:
            raise RuntimeError("start the runtime before accessing its client")
        return self._client

    async def register(self, *suites: type) -> None:
        """Replace this Python instance's complete suite set; reload if running."""
        self._check_loop()
        from dokime._provider import Provider

        await self._native.register(Provider(suites))

    async def start(
        self,
        *,
        server_config: ServerConfig | None = None,
        token: str | None = None,
        ca_certificate: str | Path | None = None,
    ) -> DokimeClient:
        self._check_loop()
        if self._closing is not None:
            raise RuntimeError("runtime is closed")
        if self._starting or self._client is not None:
            raise RuntimeError("runtime is already started or starting")

        self._starting = True
        try:
            endpoint = await self._native.start(server_config)
            self._client = DokimeClient(
                endpoint, token=token, ca_certificate=ca_certificate
            )
            await self._client.connect()
        except BaseException as failure:
            # Startup failure owns cleanup through completion. Further caller
            # cancellation must not replace the original failure or strand a
            # registered provider on the Python event loop.
            closing = self._close_task()
            while not closing.done():
                try:
                    await asyncio.shield(closing)
                except asyncio.CancelledError:
                    pass
                except BaseException as cleanup:
                    raise failure from cleanup
            if not closing.cancelled():
                try:
                    closing.result()
                except BaseException as cleanup:
                    raise failure from cleanup
            raise
        finally:
            self._starting = False

        return self._client

    async def close(self) -> None:
        self._check_loop()
        await asyncio.shield(self._close_task())

    def _close_task(self) -> asyncio.Task[None]:
        if self._closing is None or (
            self._closing.done()
            and (self._closing.cancelled() or self._closing.exception() is not None)
        ):
            self._closing = asyncio.create_task(self._close())
        return self._closing

    async def _close(self) -> None:
        try:
            if self._client is not None:
                await self._client.close()
                self._client = None
        finally:
            await self._native.close()

    def _check_loop(self) -> None:
        loop = asyncio.get_running_loop()
        if self._loop is not None and self._loop is not loop:
            raise RuntimeError(
                "a Runtime must be used on the asyncio loop that registered or started it"
            )
        self._loop = loop


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
