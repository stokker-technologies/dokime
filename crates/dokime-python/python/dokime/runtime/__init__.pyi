# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

from pathlib import Path

from dokime._native import (
    BearerGrant as BearerGrant,
    Role as Role,
    ServerConfig as ServerConfig,
    TlsIdentity as TlsIdentity,
)
from dokime.core.authoring.templates import TestSuite
from dokime.client import DokimeClient

class Runtime:
    def __init__(
        self,
        *,
        storage_root: str | Path | None = None,
        application_root: str | Path | None = None,
        plugins_root: str | Path | None = None,
    ) -> None: ...
    @property
    def client(self) -> DokimeClient: ...
    async def register(self, *suites: type[TestSuite]) -> None: ...
    async def start(
        self,
        *,
        server_config: ServerConfig | None = None,
        token: str | None = None,
        ca_certificate: str | Path | None = None,
    ) -> DokimeClient: ...
    async def close(self) -> None: ...
