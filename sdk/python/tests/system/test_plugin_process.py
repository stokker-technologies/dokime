# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Announce a Connectable Plugin Listener and Settle a Native Interrupt."""

# === Imports === #

# Standard Library Imports
import os
import signal
import socket
import subprocess
import sys
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from urllib.parse import urlsplit

# Third Party Imports
from dokime_tools.test import policy

# Local Imports

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class PluginProcessStory(unittest.TestCase):
    def test_managed_first_line_connects_and_interrupt_closes_without_traceback(self) -> None:
        self._run(managed=True)

    def test_manual_first_line_connects_and_interrupt_closes_without_traceback(self) -> None:
        self._run(managed=False)

    def _run(self, *, managed: bool) -> None:
        timeout = policy.load(Path(__file__).resolve().parents[4]).deadline("system")
        # Start the Installed Plugin With Closed Standard Input.
        flags = 0
        startup = None
        command = [sys.executable, "-m", "starter_project.plugin"]
        if os.name == "nt":
            flags = subprocess.CREATE_NEW_CONSOLE
            startup = subprocess.STARTUPINFO()
            startup.dwFlags = subprocess.STARTF_USESHOWWINDOW
            startup.wShowWindow = subprocess.SW_HIDE
            # Headless Launchers May Pass an Ignored Native Ctrl-C Flag.
            launch = """
import ctypes
import runpy

kernel = ctypes.WinDLL("kernel32", use_last_error=True)
if not kernel.SetConsoleCtrlHandler(None, False):
    raise ctypes.WinError(ctypes.get_last_error())
runpy.run_module("starter_project.plugin", run_name="__main__")
"""
            command = [sys.executable, "-c", launch]
        environment = dict(os.environ)
        environment.pop("HARMOS_ADDRESS", None)
        environment["DOKIME_PLUGIN_ADDRESS"] = "127.0.0.1:0"
        if managed:
            environment["HARMOS_ADDRESS"] = "127.0.0.1:0"
        process = subprocess.Popen(
            command,
            env=environment,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            creationflags=flags,
            startupinfo=startup,
        )
        reader = ThreadPoolExecutor(max_workers=1)
        try:
            # Read Exactly the First Line and Connect Without Readiness Polling.
            assert process.stdout is not None
            line = reader.submit(process.stdout.readline).result(timeout=timeout)
            if managed:
                self.assertTrue(line.startswith("harmos listening "), line)
            address = urlsplit(f"http://{line.rsplit(' ', 1)[-1].strip()}")
            self.assertEqual(address.scheme, "http")
            assert address.hostname is not None and address.port is not None
            connection = socket.create_connection((address.hostname, address.port), timeout=timeout)
            connection.close()
            self.assertIsNone(process.poll())

            # Deliver a Native Interrupt Only to the Owned Process's Console.
            if os.name == "nt":
                interrupt = """
import ctypes
import sys

kernel = ctypes.WinDLL("kernel32", use_last_error=True)
kernel.FreeConsole()
if not kernel.AttachConsole(int(sys.argv[1])):
    raise ctypes.WinError(ctypes.get_last_error())
if not kernel.SetConsoleCtrlHandler(None, True):
    raise ctypes.WinError(ctypes.get_last_error())
if not kernel.GenerateConsoleCtrlEvent(0, 0):
    raise ctypes.WinError(ctypes.get_last_error())
"""
                subprocess.run([sys.executable, "-c", interrupt, str(process.pid)], check=True, timeout=timeout)
            else:
                process.send_signal(signal.SIGINT)
            remaining, errors = process.communicate(timeout=timeout)
            self.assertEqual(process.returncode, 0, errors)
            self.assertEqual(remaining, "")
            self.assertEqual(errors, "")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=timeout)
            reader.shutdown(wait=True)
            if process.stdout is not None:
                process.stdout.close()
            if process.stderr is not None:
                process.stderr.close()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
