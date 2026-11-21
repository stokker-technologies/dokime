# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""An Owned Product Host For SDK And Starter Journeys."""

# === Imports === #
# Standard Library Imports
import asyncio
import os
import sys
import uuid
from asyncio.subprocess import Process
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest import IsolatedAsyncioTestCase

# isort: split

# Third Party Imports
from dokime_sdk import Client, connect

# isort: split

# Local Imports
from dokime_tools.build import paths

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Host:
    def __init__(self) -> None:
        self.root = TemporaryDirectory(prefix="dokime-sdk-host-", delete=False)
        # The Host Keeps Its Credentials in This Story's Own Namespace, Which Cleanup Removes.
        self.credentials = f"dokime-story-{uuid.uuid4().hex}"
        self.children: list[Process] = []
        self._client: Client | None = None
        self.process: Process | None = None
        self.binary: str | None = None
        self.endpoint = ""
        self.environment = dict(os.environ)
        self.flags = 0x08000000 if os.name == "nt" else 0

    @classmethod
    async def start(cls, story: IsolatedAsyncioTestCase, *, manual_clock: bool = False) -> "Host":
        host = cls()
        story.addAsyncCleanup(host.close)
        if sys.platform == "linux":
            # Start An Isolated Credential Store For This Host.
            home = Path(host.root.name) / "credentials"
            home.mkdir(mode=0o700)
            host.environment.update({key: str(home) for key in ("HOME", "XDG_DATA_HOME", "XDG_RUNTIME_DIR")})
            try:
                bus = await asyncio.create_subprocess_exec(
                    "dbus-daemon",
                    "--session",
                    "--nofork",
                    "--print-address=1",
                    env=host.environment,
                    stdout=asyncio.subprocess.PIPE,
                )
            except FileNotFoundError as error:
                raise RuntimeError("Start SDK Credential Store: Install the dbus Package") from error
            host.children.append(bus)
            if bus.stdout is None:
                raise RuntimeError("Start SDK Credential Store: dbus-daemon Started Without an Output Pipe")
            address = (await bus.stdout.readline()).decode().strip()
            if not address:
                raise RuntimeError("Start SDK Credential Store: dbus-daemon Exited Before Announcing Its Address")
            host.environment["DBUS_SESSION_BUS_ADDRESS"] = address
            try:
                keyring = await asyncio.create_subprocess_exec(
                    "gnome-keyring-daemon",
                    "--foreground",
                    "--components=secrets",
                    "--unlock",
                    "--control-directory",
                    str(home),
                    env=host.environment,
                    stdin=asyncio.subprocess.PIPE,
                    stdout=asyncio.subprocess.DEVNULL,
                )
            except FileNotFoundError as error:
                raise RuntimeError("Start SDK Credential Store: Install the gnome-keyring Package") from error
            host.children.append(keyring)
            if keyring.stdin is None:
                raise RuntimeError("Start SDK Credential Store: The Keyring Started Without an Input Pipe")
            keyring.stdin.write(b"sdk-story-keyring-password")
            await keyring.stdin.drain()
            keyring.stdin.close()
            while True:
                if bus.returncode is not None:
                    raise RuntimeError(f"Start SDK Credential Store: dbus-daemon Exited With {bus.returncode}; See Its Output Above")
                if keyring.returncode is not None:
                    raise RuntimeError(
                        f"Start SDK Credential Store: gnome-keyring-daemon Exited With {keyring.returncode}; See Its Output Above"
                    )
                check = await asyncio.create_subprocess_exec(
                    "dbus-send",
                    "--session",
                    "--print-reply",
                    "--dest=org.freedesktop.DBus",
                    "/org/freedesktop/DBus",
                    "org.freedesktop.DBus.NameHasOwner",
                    "string:org.freedesktop.secrets",
                    env=host.environment,
                    stdout=asyncio.subprocess.PIPE,
                )
                output, _ = await check.communicate()
                if check.returncode == 0 and b"boolean true" in output:
                    break
        # Launch Prepared Bytes And Await The Host's Ready Event.
        binary = str(cls.fixture("dokime-host"))
        host.binary = binary
        host.process = await asyncio.create_subprocess_exec(
            binary,
            "--credential-namespace",
            host.credentials,
            "--endpoint",
            "127.0.0.1:0",
            "--storage",
            str(Path(host.root.name) / "evidence"),
            "--manual-clock" if manual_clock else "--controlled-clock",
            env=host.environment,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            creationflags=host.flags,
        )
        if host.process.stdout is None:
            raise RuntimeError("Start SDK Host: The Host Started Without an Output Pipe")
        url = (await host.process.stdout.readline()).decode().strip()
        if not url.startswith("http://127.0.0.1:"):
            raise RuntimeError("Start SDK Host: The Host Exited Before Announcing Its Listener")
        host.endpoint = url
        host._client = await connect(url)
        return host

    @staticmethod
    def fixture(name: str) -> Path:
        variable = {"dokime-host": "DOKIME_TEST_HOST", "dokime": "DOKIME_TEST_CLI"}.get(
            name, f"DOKIME_TEST_{name.upper().replace('-', '_')}"
        )
        override = os.environ.get(variable)
        if override is not None:
            binary = Path(override)
        else:
            repository = next(parent for parent in Path(__file__).resolve().parents if (parent / "Cargo.toml").is_file())
            filename = f"{name}{'.exe' if os.name == 'nt' else ''}"
            if "TEST_PROFILE" in os.environ:
                home = "bin" if name == "dokime" else "fixtures/cli" if name == "rust" else "fixtures/host"
                binary = paths.output(repository, "build", os.environ["TEST_PROFILE"]) / home / filename
            else:
                binary = repository / "target/debug" / filename
        if not binary.is_file():
            raise RuntimeError(f"Locate SDK Fixture: Build {name} for TEST_PROFILE or Set {variable} to Its Prepared Binary")
        return binary

    @property
    def client(self) -> Client:
        if self._client is None:
            raise RuntimeError("Use SDK Host: Start the Host Before Calling Its Client")
        return self._client

    async def advance(self, at_ns: int) -> None:
        if self.process is None or self.process.stdin is None:
            raise RuntimeError("Advance SDK Host Clock: Start a Host With manual_clock=True")
        self.process.stdin.write(f"advance {at_ns}\n".encode())
        await self.process.stdin.drain()

    async def close(self) -> None:
        try:
            if self._client is not None:
                await self._client.close()
                self._client = None
        finally:
            try:
                if self.process is not None:
                    if self.process.stdin is not None:
                        self.process.stdin.close()
                    result = await self.process.wait()
                    self.process = None
                    if result != 0:
                        raise RuntimeError(f"Close SDK Host: Host Exited With {result}")
            finally:
                try:
                    if self.binary is not None:
                        cleanup = await asyncio.create_subprocess_exec(
                            self.binary,
                            "--credential-namespace",
                            self.credentials,
                            "--storage",
                            str(Path(self.root.name) / "evidence"),
                            "--remove-credentials",
                            env=self.environment,
                            creationflags=self.flags,
                        )
                        if await cleanup.wait() != 0:
                            raise RuntimeError(f"Remove SDK Host Credentials: Product Cleanup Failed; Retained Root: {self.root.name}")
                finally:
                    for child in reversed(self.children):
                        if child.returncode is None:
                            child.kill()
                        await child.wait()
                self.root.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
