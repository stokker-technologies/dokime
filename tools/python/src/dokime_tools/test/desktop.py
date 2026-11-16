# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"Own the Native Desktop Environment for One Run."

# === Imports === #

# Standard Library Imports
import subprocess
import sys
import time
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from ..ci import bootstrap

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Environment:
    """Prepare Desktop Prerequisites And Stop Only An Owned Display."""

    def __init__(self, directory: Path, environment: dict[str, str]) -> None:
        self.environment = environment
        self._display: subprocess.Popen[bytes] | None = None
        try:
            if sys.platform == "win32":
                bootstrap.prepare_windows_desktop(directory)
            else:
                bootstrap.install_gui_packages(runtime=True, desktop=True)
                bootstrap.prepare_linux_browser(directory)
                # WebKitGTK Takes WebDriver Screenshots From the Web Process, Which Cannot Snapshot an Idle Page Under
                # Accelerated Compositing on the Display's Software GL (WebKit Bug 206514), So the Capture Never Answers.
                self.environment.setdefault("WEBKIT_DISABLE_COMPOSITING_MODE", "1")
                if not self.environment.get("DISPLAY"):
                    number = next(number for number in range(90, 120) if not Path(f"/tmp/.X11-unix/X{number}").exists())
                    self._display = subprocess.Popen(
                        ["Xvfb", f":{number}", "-screen", "0", "1920x1080x24"],
                        stdin=subprocess.DEVNULL,
                        stdout=subprocess.DEVNULL,
                        stderr=sys.stderr,
                    )
                    deadline = time.monotonic() + 30
                    while not Path(f"/tmp/.X11-unix/X{number}").exists():
                        if self._display.poll() is not None or time.monotonic() > deadline:
                            raise ValueError("Run Desktop Bench Tests: desktop benchmark display did not start")
                        time.sleep(0.05)
                    self.environment["DISPLAY"] = f":{number}"
        except BaseException:
            self.close()
            raise

    def close(self) -> None:
        if self._display is not None:
            self._display.terminate()
            try:
                self._display.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self._display.kill()
                self._display.wait()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
