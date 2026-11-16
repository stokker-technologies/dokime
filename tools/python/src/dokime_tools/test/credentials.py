# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Own a Private Native Credential Store."

# === Imports === #

# Standard Library Imports
import os
import subprocess
import sys
from selectors import EVENT_READ, DefaultSelector

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.test.policy

from .. import temporary

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Store:
    """Own a Private Linux Secret Service for One Native Run."""

    def __init__(self, environment: dict[str, str] | None) -> None:
        self.environment = dict(os.environ if environment is None else environment)
        self._directory: temporary.Directory | None = None
        self._bus: subprocess.Popen[str] | None = None
        self._keyring: subprocess.Popen[bytes] | None = None
        self._timeout_s: float | None = None
        if sys.platform != "linux":
            return

        self._timeout_s = dokime_tools.test.policy.load(dokime_tools.test.policy.root()).deadline("integration")
        try:
            self._directory = temporary.Directory()
            home = self._directory.path / "credentials"
            home.mkdir(mode=0o700)
            self.environment.update({name: str(home) for name in ("XDG_DATA_HOME", "XDG_RUNTIME_DIR")})

            # Start the Private Session Bus.
            self._bus = subprocess.Popen(
                ["dbus-daemon", "--session", "--nofork", "--print-address=1"],
                env=self.environment,
                stdout=subprocess.PIPE,
                text=True,
            )
            if self._bus.stdout is None:
                raise RuntimeError("Start Story Credential Store: Private D-Bus Has No Output Pipe")
            ready = DefaultSelector()
            try:
                ready.register(self._bus.stdout, EVENT_READ)
                if not ready.select(self._timeout_s):
                    raise RuntimeError("Start Story Credential Store: Private D-Bus Did Not Publish Its Address")
            finally:
                ready.close()
            address = self._bus.stdout.readline().strip()
            if not address:
                raise RuntimeError(f"Start Story Credential Store: Private D-Bus Exited ({self._bus.wait()})")
            self.environment["DBUS_SESSION_BUS_ADDRESS"] = address

            # Unlock the Private Secret Service.
            self._keyring = subprocess.Popen(
                ["gnome-keyring-daemon", "--foreground", "--components=secrets", "--unlock", "--control-directory", str(home)],
                env=self.environment,
                stdin=subprocess.PIPE,
            )
            if self._keyring.stdin is None:
                raise RuntimeError("Start Story Credential Store: The Keyring Has No Input Pipe")
            self._keyring.stdin.write(b"dokime-story-keyring-password")
            self._keyring.stdin.close()
            subprocess.run(
                ["gdbus", "wait", "--session", "org.freedesktop.secrets"],
                env=self.environment,
                check=True,
                timeout=self._timeout_s,
            )
        except BaseException:
            self.close()
            raise

    def close(self) -> None:
        try:
            if self._keyring is not None:
                try:
                    if self._keyring.poll() is None:
                        self._keyring.terminate()
                    try:
                        self._keyring.wait(timeout=self._timeout_s)
                    except subprocess.TimeoutExpired:
                        self._keyring.kill()
                        self._keyring.wait()
                finally:
                    if self._keyring.stdin is not None:
                        self._keyring.stdin.close()
                    self._keyring.wait()
        finally:
            try:
                if self._bus is not None:
                    try:
                        if self._bus.poll() is None:
                            self._bus.terminate()
                        try:
                            self._bus.wait(timeout=self._timeout_s)
                        except subprocess.TimeoutExpired:
                            self._bus.kill()
                            self._bus.wait()
                    finally:
                        if self._bus.stdout is not None:
                            self._bus.stdout.close()
                        self._bus.wait()
            finally:
                if self._directory is not None:
                    self._directory.close()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
