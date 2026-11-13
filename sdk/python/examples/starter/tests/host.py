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
import platform
import sys
from pathlib import Path
from tempfile import TemporaryDirectory

# Third Party Imports
import dokime

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Host:
    def __init__(self):
        self.root = TemporaryDirectory(prefix="dokime-sdk-host-")
        self.children = []
        self.client = None
        self.process = None
        self.environment = dict(os.environ)
        self.flags = 0x08000000 if os.name == "nt" else 0

    @classmethod
    async def start(cls, story):
        host = cls()
        story.addAsyncCleanup(host.close)
        if sys.platform == "linux":
            # Start An Isolated Credential Store For This Host.
            home = Path(host.root.name) / "credentials"
            home.mkdir(mode=0o700)
            host.environment.update({key: str(home) for key in ("HOME", "XDG_DATA_HOME", "XDG_RUNTIME_DIR")})
            bus = await asyncio.create_subprocess_exec(
                "dbus-daemon",
                "--session",
                "--nofork",
                "--print-address=1",
                env=host.environment,
                stdout=asyncio.subprocess.PIPE,
            )
            host.children.append(bus)
            address = (await bus.stdout.readline()).decode().strip()
            if not address:
                raise RuntimeError("Start SDK Credential Store: dbus-daemon Exited Before Announcing Its Address")
            host.environment["DBUS_SESSION_BUS_ADDRESS"] = address
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
            host.children.append(keyring)
            keyring.stdin.write(b"sdk-story-keyring-password")
            await keyring.stdin.drain()
            keyring.stdin.close()
            while True:
                if bus.returncode is not None or keyring.returncode is not None:
                    raise RuntimeError("Start SDK Credential Store: An Owned Daemon Exited")
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
        repository = next((parent for parent in Path(__file__).resolve().parents if (parent / "Cargo.toml").is_file()), None)
        binary = os.environ.get("DOKIME_TEST_HOST")
        if binary is None and repository is not None:
            name = "dokime-host-next.exe" if os.name == "nt" else "dokime-host-next"
            if "TEST_PROFILE" in os.environ:
                system = {"Linux": "linux", "Windows": "windows", "Darwin": "macos"}[platform.system()]
                machine = platform.machine().lower()
                architecture = {"amd64": "x86_64", "arm64": "aarch64"}.get(machine, machine)
                binary = str(repository / "target/build" / os.environ["TEST_PROFILE"] / f"{system}-{architecture}" / "fixtures/host" / name)
            else:
                binary = str(repository / "target/debug" / name)
        if binary is None or not Path(binary).is_file():
            raise RuntimeError("Start SDK Host: Build Host Fixtures or Set DOKIME_TEST_HOST To the Prepared Host Binary")
        host.process = await asyncio.create_subprocess_exec(
            binary,
            "--endpoint",
            "127.0.0.1:0",
            "--storage",
            str(Path(host.root.name) / "evidence"),
            "--controlled-clock",
            env=host.environment,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            creationflags=host.flags,
        )
        url = (await host.process.stdout.readline()).decode().strip()
        if not url.startswith("http://127.0.0.1:"):
            raise RuntimeError("Start SDK Host: The Host Exited Before Announcing Its Listener")
        host.client = await dokime.connect(url)
        return host

    async def close(self):
        if self.client is not None:
            await self.client.close()
            self.client = None
        try:
            if self.process is not None:
                if self.process.stdin is not None:
                    self.process.stdin.close()
                result = await self.process.wait()
                self.process = None
                if result != 0:
                    raise RuntimeError(f"Close SDK Host: Host Exited With {result}")
        finally:
            for child in reversed(self.children):
                if child.returncode is None:
                    child.kill()
                await child.wait()
            self.root.cleanup()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
